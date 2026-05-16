// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho assemble` — build the USB artifact directory.
//!
//! Pure Rust replacement for `scripts/assemble-usb.sh`.
//! Creates the directory tree, stages binaries, data, papers, notebooks,
//! figures, documentation, and generates data_manifest.toml with BLAKE3 hashes.

use std::path::Path;

pub fn run(root: &str, target: &str, skip_python: bool, skip_fetch: bool, skip_build: bool, dry_run: bool) {
    let root_path = Path::new(root);
    let target_path = Path::new(target);

    if dry_run {
        print_dry_run(root, target, skip_python, skip_fetch, skip_build);
        return;
    }

    println!("=== lithoSpore USB Assembly — Hypogeal Cotyledon ===");
    println!("  Source:  {root}");
    println!("  Target:  {target}");

    // 1. Create directory tree
    step("1. Creating directory tree");
    let dirs = [
        "bin", "artifact/data", "validation/expected",
        "projectFOUNDATION/targets", "notebooks", "figures",
        "biomeOS/graphs", "papers",
    ];
    for dir in &dirs {
        std::fs::create_dir_all(target_path.join(dir)).ok();
    }
    if !skip_python {
        std::fs::create_dir_all(target_path.join("python")).ok();
    }
    println!("  Directory tree created");

    // 2. Stage root files
    step("2. Staging root files");
    copy_if_exists(root_path, "artifact/usb-root/.biomeos-spore", target_path, ".biomeos-spore");
    touch(target_path, ".family.seed");

    // Create symlinks instead of copying shell shims
    for shim in ["validate", "verify", "refresh", "spore"] {
        let link = target_path.join(shim);
        let _ = std::fs::remove_file(&link);
        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink("bin/litho", &link);
        }
        #[cfg(not(unix))]
        {
            let litho_src = target_path.join("bin/litho.exe");
            if litho_src.exists() {
                let _ = std::fs::copy(&litho_src, &link.with_extension("exe"));
            }
        }
    }

    let livespore = target_path.join("liveSpore.json");
    if !livespore.exists() {
        std::fs::write(&livespore, "[]").ok();
        println!("  liveSpore.json initialized");
    }
    println!("  Root files staged (symlinks → bin/litho)");

    // 3. Stage biomeOS files
    step("3. Staging biomeOS files");
    copy_if_exists(root_path, "artifact/usb-root/biomeOS/tower.toml", target_path, "biomeOS/tower.toml");
    copy_if_exists(root_path, "artifact/usb-root/biomeOS/graphs/lithoSpore_validation.toml",
        target_path, "biomeOS/graphs/lithoSpore_validation.toml");
    println!("  biomeOS files staged");

    // 4. Build and stage binaries
    step("4. Staging binaries");
    if skip_build {
        println!("  SKIP: --skip-build");
    } else {
        let scope_bins = load_binary_list(root_path);
        let mut staged = 0u32;
        for bin in &scope_bins {
            let bin = bin.as_str();
            let src = root_path.join(format!("target/release/{bin}"));
            if src.exists() {
                let dest = target_path.join(format!("bin/{bin}"));
                if let Err(e) = std::fs::copy(&src, &dest) {
                    eprintln!("  WARNING: copy {bin}: {e}");
                } else {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755)).ok();
                    }
                    staged += 1;
                }
            }
        }
        println!("  {staged} binaries staged");
    }

    // 5. Fetch and stage data
    step("5. Staging data bundles");
    if !skip_fetch {
        crate::fetch::run(root, None, true);
    }
    let data_src = root_path.join("artifact/data");
    if data_src.exists() {
        copy_dir_recursive(&data_src, &target_path.join("artifact/data"));
    }
    for toml_file in ["scope.toml", "data.toml", "tolerances.toml"] {
        copy_if_exists(root_path, &format!("artifact/{toml_file}"), target_path, &format!("artifact/{toml_file}"));
    }
    println!("  Data bundles staged");

    // 6. Stage papers and docs
    step("6. Staging papers and documentation");
    let papers_src = root_path.join("papers");
    if papers_src.exists() {
        copy_dir_recursive(&papers_src, &target_path.join("papers"));
    }
    for doc in ["GETTING_STARTED.md", "SCIENCE.md"] {
        copy_if_exists(root_path, doc, target_path, doc);
    }
    println!("  Documentation staged");

    // 7. Stage expected values
    step("7. Staging expected values");
    let expected_src = root_path.join("validation/expected");
    if expected_src.exists() {
        copy_dir_recursive(&expected_src, &target_path.join("validation/expected"));
    }

    // 8. Stage figures
    step("8. Staging figures");
    let fig_src = root_path.join("figures");
    if fig_src.exists() {
        copy_dir_recursive(&fig_src, &target_path.join("figures"));
    }

    // 9. Stage notebooks
    step("9. Staging notebooks");
    let nb_src = root_path.join("notebooks");
    if nb_src.exists() {
        copy_dir_recursive(&nb_src, &target_path.join("notebooks"));
    }

    // 10. Generate data_manifest.toml
    step("10. Generating data_manifest.toml");
    let artifact_name = litho_core::ScopeManifest::load(&root_path.join("artifact/scope.toml"))
        .map_or_else(|_| "ltee-guidestone".to_string(), |s| s.guidestone.name.clone());
    generate_manifest(target_path, &artifact_name);

    // Summary
    step("Assembly Complete");
    let bin_count = count_files(&target_path.join("bin"));
    let data_count = count_subdirs(&target_path.join("artifact/data"));
    let fig_count = count_files_with_ext(&target_path.join("figures"), "svg");
    println!();
    println!("  Target:    {target}");
    println!("  Binaries:  {bin_count} ecoBin modules");
    println!("  Data:      {data_count} datasets");
    println!("  Figures:   {fig_count} SVG figures");
    println!();
    println!("  To validate: cd {target} && ./validate");
}

/// Build the binary staging list from scope.toml if available, otherwise
/// fall back to the compiled LTEE binary set. Always includes `litho` itself.
fn load_binary_list(root: &Path) -> Vec<String> {
    let scope_path = root.join("artifact/scope.toml");
    if let Ok(scope) = litho_core::ScopeManifest::load(&scope_path) {
        let mut bins: Vec<String> = vec!["litho".to_string()];
        for b in scope.module_binaries() {
            let name = b.to_string();
            if !bins.contains(&name) {
                bins.push(name);
            }
        }
        return bins;
    }
    ["litho", "ltee-fitness", "ltee-mutations", "ltee-alleles",
     "ltee-citrate", "ltee-biobricks", "ltee-breseq", "ltee-anderson"]
        .iter().map(|s| s.to_string()).collect()
}

fn step(msg: &str) {
    println!();
    println!("=== {msg} ===");
}

fn touch(dir: &Path, name: &str) {
    let path = dir.join(name);
    if !path.exists() {
        std::fs::write(&path, "").ok();
    }
}

fn copy_if_exists(src_root: &Path, src_rel: &str, dst_root: &Path, dst_rel: &str) {
    let src = src_root.join(src_rel);
    if src.exists() {
        let dst = dst_root.join(dst_rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::copy(&src, &dst).ok();
    }
}

pub(crate) fn copy_dir_recursive_pub(src: &Path, dst: &Path) {
    copy_dir_recursive(src, dst);
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    for entry in walkdir::WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(src).unwrap_or(entry.path());
        let dest = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest).ok();
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::copy(entry.path(), &dest).ok();
        }
    }
}

fn generate_manifest(target: &Path, artifact_name: &str) {
    let manifest_path = target.join("data_manifest.toml");
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut output = format!(
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n\
         #\n\
         # Data manifest — BLAKE3 inventory of all bundled data.\n\
         # Generated by litho assemble on {timestamp}\n\n\
         [meta]\n\
         artifact = \"{artifact_name}\"\n\
         generated = \"{timestamp}\"\n\
         arch = \"{arch}\"\n\n",
        arch = std::env::consts::ARCH,
    );

    let mut count = 0u32;

    for subdir in ["artifact/data", "figures"] {
        let dir = target.join(subdir);
        if !dir.exists() {
            continue;
        }

        let mut files: Vec<_> = walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| !e.file_name().to_str().map_or(false, |n| n.starts_with('.')))
            .collect();
        files.sort_by(|a, b| a.path().cmp(b.path()));

        for entry in files {
            if let Ok(content) = std::fs::read(entry.path()) {
                let hash = blake3::hash(&content);
                let rel = entry.path().strip_prefix(target).unwrap_or(entry.path());
                output.push_str(&format!(
                    "[[file]]\npath = \"{}\"\nblake3 = \"{}\"\n\n",
                    rel.display(), hash.to_hex()
                ));
                count += 1;
            }
        }
    }

    match std::fs::write(&manifest_path, &output) {
        Ok(()) => println!("  data_manifest.toml generated ({count} files hashed)"),
        Err(e) => eprintln!("  WARNING: could not write manifest: {e}"),
    }
}

fn count_files(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false)).count())
        .unwrap_or(0)
}

fn count_subdirs(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false)).count())
        .unwrap_or(0)
}

fn count_files_with_ext(dir: &Path, ext: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == ext).unwrap_or(false))
            .count())
        .unwrap_or(0)
}

fn print_dry_run(root: &str, target: &str, skip_python: bool, skip_fetch: bool, skip_build: bool) {
    println!("DRY RUN — showing what would be assembled");
    println!("  Source:  {root}");
    println!("  Target:  {target}");
    println!("  Python:  {}", if skip_python { "SKIP" } else { "EMBED" });
    println!("  Fetch:   {}", if skip_fetch { "SKIP" } else { "FETCH" });
    println!("  Build:   {}", if skip_build { "SKIP" } else { "BUILD" });
    println!();
    println!("Directory tree that would be created:");
    println!("  {target}/");
    println!("  ├── .biomeos-spore");
    println!("  ├── validate → bin/litho (symlink)");
    println!("  ├── verify → bin/litho (symlink)");
    println!("  ├── refresh → bin/litho (symlink)");
    println!("  ├── spore → bin/litho (symlink)");
    println!("  ├── bin/litho (unified binary)");
    println!("  ├── artifact/data/ (7 datasets)");
    println!("  ├── validation/expected/ (7 JSONs)");
    println!("  ├── papers/ (registry + reading order)");
    println!("  ├── figures/ (SVGs)");
    println!("  └── data_manifest.toml (BLAKE3)");
}
