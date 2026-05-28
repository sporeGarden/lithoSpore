// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho promote` — promote a pseudoSpore to a lithoSpore deployment chassis.
//!
//! Takes a validated pseudoSpore directory and wraps it in the lithoSpore chassis:
//! - Copies pseudoSpore verbatim into `proof/`
//! - Compiles Tier 2 binaries (if crate path provided)
//! - Copies litho CLI binary into `runtime/bin/`
//! - Snapshots Python requirements into `runtime/env/`
//! - Generates `expected/` from `proof/outputs/`
//! - Generates `tolerances.toml` from parity results
//! - Generates `guidestone.toml` from `proof/scope.toml`
//! - Writes automation scripts
//! - Computes final BLAKE3 seal

mod report;

use crate::domain_profile;
use report::{
    generate_chassis_readme, generate_expected_stub, generate_guidestone,
    generate_release_from_braids, generate_tolerances, write_scripts,
};
use std::fs;
use std::path::{Path, PathBuf};

const CHECKSUMS_FILE: &str = "CHECKSUMS.blake3";

pub(crate) fn run(
    pseudospore_path: &str,
    output_dir: &str,
    tier2_crate: Option<&str>,
    tier1_script: Option<&str>,
    version_override: Option<&str>,
) -> Result<(), String> {
    let ps_root = Path::new(pseudospore_path);
    let out = Path::new(output_dir);

    if !ps_root.exists() {
        eprintln!("ERROR: pseudoSpore not found at: {pseudospore_path}");
        std::process::exit(1);
    }

    // Load domain profile if present in pseudoSpore
    let profile = domain_profile::load_domain_profile(ps_root);

    // Load scope.toml from pseudoSpore to get metadata
    let scope_path = ps_root.join("scope.toml");
    let scope_content = fs::read_to_string(&scope_path).unwrap_or_else(|_| {
        eprintln!("ERROR: cannot read {}/scope.toml", ps_root.display());
        std::process::exit(1);
    });
    let scope: toml::Table = scope_content.parse().unwrap_or_else(|e| {
        eprintln!("ERROR: scope.toml parse failed: {e}");
        std::process::exit(1);
    });

    let artifact = scope
        .get("artifact")
        .and_then(|v| v.as_table())
        .unwrap_or_else(|| {
            eprintln!("ERROR: scope.toml missing [artifact]");
            std::process::exit(1);
        });

    let ps_name = artifact
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let ps_version = artifact
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0");
    let origin = artifact
        .get("origin")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let litho_version = version_override.unwrap_or(env!("CARGO_PKG_VERSION"));
    let litho_name = strip_spring_prefix(ps_name);
    let dir_name = format!("lithoSpore_{litho_name}_v{litho_version}");
    let root = out.join(&dir_name);

    println!("=== litho promote ===");
    println!("  pseudoSpore: {ps_name} v{ps_version}");
    println!("  lithoSpore:  {litho_name} v{litho_version}");
    println!("  Output:      {}", root.display());
    println!();

    // Create chassis structure
    fs::create_dir_all(root.join("proof")).map_err(|e| format!("create proof/: {e}"))?;
    fs::create_dir_all(root.join("runtime/bin"))
        .map_err(|e| format!("create runtime/bin/: {e}"))?;
    fs::create_dir_all(root.join("runtime/env"))
        .map_err(|e| format!("create runtime/env/: {e}"))?;
    fs::create_dir_all(root.join("runtime/scripts"))
        .map_err(|e| format!("create runtime/scripts/: {e}"))?;
    fs::create_dir_all(root.join("expected")).map_err(|e| format!("create expected/: {e}"))?;

    // 1. Copy pseudoSpore into proof/ verbatim
    print!("  [1/10] Copying pseudoSpore into proof/... ");
    copy_tree(ps_root, &root.join("proof"));
    println!("done");

    // 2. Copy litho CLI binary (stripped for size)
    print!("  [2/10] Installing litho CLI into runtime/bin/... ");
    let self_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("litho"));
    if self_exe.exists() {
        fs::copy(&self_exe, root.join("runtime/bin/litho")).ok();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            fs::set_permissions(root.join("runtime/bin/litho"), perms.clone()).ok();
            // Strip debug symbols to reduce size (~87MB → ~5MB)
            let strip_result = std::process::Command::new("strip")
                .arg(root.join("runtime/bin/litho"))
                .output();
            match strip_result {
                Ok(o) if o.status.success() => {
                    let size = fs::metadata(root.join("runtime/bin/litho"))
                        .map(|m| m.len() / 1024 / 1024)
                        .unwrap_or(0);
                    println!("done (stripped, {size}MB)");
                }
                _ => println!("done (unstripped)"),
            }
        }
        #[cfg(not(unix))]
        println!("done");
    } else {
        println!("skipped (binary not found)");
    }

    // 3. Compile Tier 2 binary (if crate provided)
    if let Some(crate_path) = tier2_crate {
        print!("  [3/10] Compiling Tier 2 binary... ");
        let crate_dir = Path::new(crate_path);
        if crate_dir.exists() {
            let output = std::process::Command::new("cargo")
                .args(["build", "--release"])
                .current_dir(crate_dir)
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    // Find the binary in target/release/
                    let crate_name = crate_dir
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let bin_name = crate_name.replace('-', "_");
                    let target_dir = crate_dir.join("target/release");
                    let bin_path = target_dir.join(&crate_name);
                    let bin_path_alt = target_dir.join(&bin_name);

                    let src_bin = if bin_path.exists() {
                        Some(bin_path)
                    } else if bin_path_alt.exists() {
                        Some(bin_path_alt)
                    } else {
                        None
                    };

                    if let Some(src) = src_bin {
                        let dest = root.join(format!("runtime/bin/{crate_name}"));
                        fs::copy(&src, &dest).ok();
                        // Strip debug symbols
                        std::process::Command::new("strip").arg(&dest).output().ok();
                        let size = fs::metadata(&dest).map(|m| m.len() / 1024).unwrap_or(0);
                        println!("done ({crate_name}, {size}KB)");
                    } else {
                        println!("built, but binary not found in target/release/");
                    }
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    println!("FAILED");
                    eprintln!(
                        "    cargo build stderr: {}",
                        &stderr[..stderr.len().min(200)]
                    );
                }
                Err(e) => println!("FAILED ({e})"),
            }
        } else {
            println!("skipped (crate path not found: {crate_path})");
        }
    } else {
        println!("  [3/10] Tier 2 binary: skipped (no --tier2-crate)");
    }

    // 4. Snapshot Python environment
    print!("  [4/10] Capturing Python environment... ");
    if let Some(script) = tier1_script {
        let script_path = Path::new(script);
        if script_path.exists() {
            fs::copy(script_path, root.join("runtime/env/tier1_validator.py")).ok();
        }
    }
    let pip_output = std::process::Command::new("pip")
        .args(["freeze", "--local"])
        .output();
    match pip_output {
        Ok(o) if o.status.success() => {
            let reqs = String::from_utf8_lossy(&o.stdout);
            fs::write(root.join("runtime/env/requirements.txt"), reqs.as_ref()).ok();
            println!("done ({} packages)", reqs.lines().count());
        }
        _ => {
            fs::write(
                root.join("runtime/env/requirements.txt"),
                "# pip freeze not available at promote time\nnumpy\nscipy\n",
            )
            .ok();
            println!("fallback (pip not available)");
        }
    }

    // 5. Copy forcefields if present in pseudoSpore configs
    let ff_src = ps_root.join("configs/forcefields");
    if ff_src.exists() && ff_src.is_dir() {
        print!("  [5/10] Copying forcefields... ");
        let ff_dst = root.join("runtime/forcefields");
        copy_tree(&ff_src, &ff_dst);
        println!("done");
    } else {
        println!("  [5/10] Forcefields: skipped (none in pseudoSpore)");
    }

    // 6. Generate expected/ from proof/outputs/
    print!("  [6/10] Generating expected values... ");
    let outputs_dir = root.join("proof/outputs");
    let mut expected_count = 0;
    if outputs_dir.exists()
        && let Ok(modules) = fs::read_dir(&outputs_dir)
    {
        for module in modules.flatten() {
            if module.path().is_dir() {
                let mod_name = module.file_name().to_string_lossy().to_string();
                let expected_dir = root.join(format!("expected/{mod_name}"));
                fs::create_dir_all(&expected_dir).ok();
                let expected_json = generate_expected_stub(&mod_name, profile.as_ref());
                fs::write(expected_dir.join("expected.json"), &expected_json).ok();
                expected_count += 1;
            }
        }
    }
    println!("done ({expected_count} modules)");

    // 7. Generate tolerances.toml (profile-driven when available)
    print!("  [7/10] Writing tolerances.toml... ");
    let tolerances = generate_tolerances(&scope, profile.as_ref());
    fs::write(root.join("tolerances.toml"), &tolerances).ok();
    println!("done");

    // 8. Generate guidestone.toml and automation scripts
    print!("  [8/10] Writing guidestone.toml and scripts... ");
    let guidestone = generate_guidestone(&litho_name, litho_version, ps_version, origin);
    fs::write(root.join("guidestone.toml"), &guidestone).ok();
    write_scripts(&root, profile.as_ref());
    println!("done");

    // 9. Auto-generate RELEASE.md from braid supersedes chain
    print!("  [9/10] Generating RELEASE.md from provenance... ");
    let release = generate_release_from_braids(
        &root.join("proof/provenance"),
        &litho_name,
        litho_version,
        ps_version,
    );
    fs::write(root.join("RELEASE.md"), &release).ok();
    println!("done");

    // Generate README before seal so it is included in CHECKSUMS.blake3
    let readme = generate_chassis_readme(&litho_name, litho_version, ps_version, profile.as_ref());
    fs::write(root.join("README.md"), &readme).ok();

    // 10. Compute final BLAKE3 seal of the entire chassis
    print!("  [10/10] Computing final BLAKE3 seal... ");
    let seal = compute_chassis_seal(&root);
    fs::write(root.join(CHECKSUMS_FILE), &seal).ok();
    println!("done");

    println!();
    println!("=== lithoSpore promoted ===");
    println!("  Output: {}", root.display());
    println!();
    println!("Verify:");
    println!("  cd {}", root.display());
    println!("  ./runtime/bin/litho verify --artifact-root proof/");
    println!("  ./runtime/scripts/validate.sh");
    println!("  ./runtime/scripts/translate.sh --frame domain");
    Ok(())
}

/// Compute BLAKE3 checksums for every file under the chassis root.
///
/// Returns manifest text in `<hash>  <relative-path>` format (two spaces),
/// sorted by path. Skips `CHECKSUMS.blake3` so the manifest is not self-referential.
fn compute_chassis_seal(root: &Path) -> String {
    let mut entries: Vec<(String, String)> = Vec::new();
    collect_chassis_hashes(root, root, &mut entries);
    entries.sort_by(|a, b| a.1.cmp(&b.1));
    entries
        .into_iter()
        .map(|(hash, path)| format!("{hash}  {path}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_chassis_hashes(root: &Path, dir: &Path, entries: &mut Vec<(String, String)>) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = read_dir.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            collect_chassis_hashes(root, &path, entries);
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if rel == CHECKSUMS_FILE {
                continue;
            }
            if let Ok(data) = fs::read(&path) {
                let hash = blake3::hash(&data).to_hex().to_string();
                entries.push((hash, rel));
            }
        }
    }
}

/// Strip a leading `<spring>-` prefix from a pseudoSpore artifact name.
///
/// Recognises the canonical spring naming convention: a camelCase name ending
/// with "Spring" followed by a hyphen (e.g. `hotSpring-`, `groundSpring-`).
fn strip_spring_prefix(name: &str) -> String {
    if let Some(idx) = name.find("Spring-") {
        name[idx + "Spring-".len()..].to_string()
    } else {
        name.to_string()
    }
}

fn copy_tree(src: &Path, dst: &Path) {
    if !src.is_dir() {
        if src.is_file() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::copy(src, dst).ok();
        }
        return;
    }
    fs::create_dir_all(dst).ok();
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let dest = dst.join(entry.file_name());
            if path.is_dir() {
                copy_tree(&path, &dest);
            } else {
                fs::copy(&path, &dest).ok();
            }
        }
    }
}
