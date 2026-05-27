// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho emit-pseudospore` — assemble a pseudoSpore directory from module outputs.
//!
//! Generates the standard directory structure, computes BLAKE3 checksums,
//! captures environment metadata, and creates a README from scope metadata.

mod environment;
mod figures;
mod index_map;
mod manifest;
mod scope;
mod scripts;

use crate::domain_profile::{self, DomainProfile};
use pseudospore_core::receipts;
use std::fs;
use std::path::Path;

pub(crate) const LITHOSPORE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct EmitConfig<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub origin: &'a str,
    pub output_dir: &'a str,
    pub outputs_dir: Option<&'a str>,
    pub configs_dir: Option<&'a str>,
    pub braids_dir: Option<&'a str>,
    pub data_dir: Option<&'a str>,
    pub profile_path: Option<&'a str>,
}

pub fn run(config: &EmitConfig<'_>) {
    let EmitConfig {
        name,
        version,
        origin,
        output_dir,
        outputs_dir,
        configs_dir,
        braids_dir,
        data_dir,
        profile_path,
    } = config;

    let spring_name = origin.split('/').next_back().unwrap_or("unknown");

    let out = Path::new(output_dir);
    let dir_name = format!("pseudoSpore_{name}_v{version}");
    let root = out.join(&dir_name);

    println!("=== litho emit-pseudospore ===");
    println!("  Name:    {name}");
    println!("  Version: {version}");
    println!("  Origin:  {origin}");
    println!("  Output:  {}", root.display());

    // Load domain profile if provided
    let profile = profile_path.map(Path::new).and_then(|pp| {
        if pp.exists() {
            domain_profile::load_from_file(pp)
        } else {
            eprintln!("  WARN: profile path not found: {}", pp.display());
            None
        }
    });

    if let Some(ref p) = profile {
        println!("  Profile: {} v{}", p.id, p.version);
    } else {
        println!("  Profile: (none — generic skeleton only)");
    }
    println!();

    // Create directory structure
    std::fs::create_dir_all(root.join("receipts")).expect("Failed to create receipts/");
    std::fs::create_dir_all(root.join("provenance/braids"))
        .expect("Failed to create provenance/braids/");
    std::fs::create_dir_all(root.join("outputs")).expect("Failed to create outputs/");
    std::fs::create_dir_all(root.join("configs")).expect("Failed to create configs/");

    // 1. Generate scope.toml (profile-aware: auto-populates modules from data/)
    let scope_content = scope::generate_scope(
        name,
        version,
        origin,
        profile.as_ref(),
        data_dir.map(Path::new),
        configs_dir.map(Path::new),
    );
    std::fs::write(root.join("scope.toml"), &scope_content).expect("Failed to write scope.toml");
    println!("  [+] scope.toml");

    // 2. Generate stub validation.json
    let validation_content = scripts::generate_validation_stub(name, version);
    std::fs::write(root.join("validation.json"), &validation_content)
        .expect("Failed to write validation.json");
    println!("  [+] validation.json (stub — populate with results)");

    // 3. Capture environment (profile-aware: probes tool versions, computes total production)
    let env_content =
        environment::capture_environment(profile.as_ref(), configs_dir.map(Path::new));
    std::fs::write(root.join("receipts/environment.toml"), &env_content)
        .expect("Failed to write receipts/environment.toml");
    println!("  [+] receipts/environment.toml");

    // 4. Copy outputs if provided
    if let Some(src) = outputs_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("outputs"));
            println!("  [+] outputs/ (copied from {src})");
        }
    }

    // 5. Copy configs if provided
    if let Some(src) = configs_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("configs"));
            println!("  [+] configs/ (copied from {src})");
        }
    }

    // 6. Copy braids if provided
    if let Some(src) = braids_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("provenance/braids"));
            println!("  [+] provenance/braids/ (copied from {src})");
        }
    }

    // 7. Copy data if provided
    if let Some(src) = data_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            std::fs::create_dir_all(root.join("data")).expect("Failed to create data/");
            copy_tree(src_path, &root.join("data"));
            println!("  [+] data/ (copied from {src})");
        }
    }

    // 8. Copy domain_profile.toml into the pseudoSpore if provided
    if let Some(pp) = profile_path {
        let pp = Path::new(pp);
        if pp.exists() {
            fs::copy(pp, root.join("domain_profile.toml")).ok();
            println!("  [+] domain_profile.toml (copied from {})", pp.display());
        }
    }

    // 9. Auto-generate index_map.toml from topology files in data/ (profile-conditional)
    let do_translation = profile
        .as_ref()
        .is_none_or(DomainProfile::translation_enabled);
    let data_root = root.join("data");
    if do_translation && data_root.exists() {
        let entity_groups = profile.as_ref().and_then(|p| p.translation_entity_groups());
        if let Some(index_map) = index_map::auto_generate_index_map(&data_root, entity_groups) {
            std::fs::write(root.join("index_map.toml"), &index_map)
                .expect("Failed to write index_map.toml");
            println!("  [+] index_map.toml (auto-generated from topology files)");
        }
    } else if !do_translation {
        println!("  [~] index_map.toml skipped (translation disabled in profile)");
    }

    // 10. Generate ferment transcript stub
    let ferment_content = scripts::generate_ferment_stub(name, version, origin);
    std::fs::write(
        root.join("provenance/ferment_transcript.json"),
        &ferment_content,
    )
    .expect("Failed to write provenance/ferment_transcript.json");
    println!("  [+] provenance/ferment_transcript.json (stub)");

    // 11. Compute checksums for outputs/, provenance/, and data/
    let checksums =
        receipts::compute_checksums(&root, &["outputs", "provenance", "data", "configs"]);
    let cksum_content = receipts::format_checksums(&checksums);
    std::fs::write(root.join("receipts/checksums.blake3"), &cksum_content)
        .expect("Failed to write receipts/checksums.blake3");
    println!(
        "  [+] receipts/checksums.blake3 ({} entries)",
        checksums.len()
    );

    // 12. Generate README (profile-aware, domain-expert-facing)
    let readme = scripts::generate_readme(
        name,
        version,
        origin,
        profile.as_ref(),
        data_dir.map(Path::new),
        configs_dir.map(Path::new),
    );
    std::fs::write(root.join("README.md"), &readme).expect("Failed to write README.md");
    println!("  [+] README.md");

    // 13. Generate TRANSLATE.md stub (only if translation enabled)
    if do_translation {
        let translate = scripts::generate_translate_stub();
        std::fs::write(root.join("TRANSLATE.md"), &translate)
            .expect("Failed to write TRANSLATE.md");
        println!("  [+] TRANSLATE.md (stub — populate with derivation commands)");
    }

    // 14. Generate data.toml — data manifest (guideStone data component)
    if data_root.exists() {
        let data_manifest =
            manifest::generate_data_manifest(&data_root, name, version, spring_name);
        fs::write(root.join("data.toml"), &data_manifest).expect("Failed to write data.toml");
        println!("  [+] data.toml (data manifest)");
    }

    // 15. Generate tolerances.toml with scientific justification
    let tolerances = manifest::generate_tolerances_justified(profile.as_ref());
    fs::write(root.join("tolerances.toml"), &tolerances).expect("Failed to write tolerances.toml");
    println!("  [+] tolerances.toml (named tolerances with justification)");

    // 16. Initialize liveSpore.json (unified schema: envelope + validations)
    let livespore_initial = serde_json::json!({
        "envelope": {
            "artifact": name,
            "version": version,
            "emit_timestamp": chrono::Utc::now().to_rfc3339(),
            "emit_host": run_cmd("hostname", &[]).unwrap_or_else(|| "unknown".to_string()),
            "tool": "litho emit-pseudospore",
            "tool_version": LITHOSPORE_VERSION,
            "integrity": "BLAKE3 (receipts/checksums.blake3)",
        },
        "validations": []
    });
    fs::write(
        root.join("liveSpore.json"),
        serde_json::to_string_pretty(&livespore_initial).unwrap_or_else(|_| "{}".to_string()),
    )
    .expect("Failed to write liveSpore.json");
    println!("  [+] liveSpore.json (unified schema: envelope + empty validations)");

    // 17. Generate validate + refresh entry point scripts
    scripts::generate_entry_scripts(&root, name, version);
    println!("  [+] validate (entry point script)");
    println!("  [+] refresh (data freshness script)");

    // 18. Auto-generate figures (profile-conditional)
    let do_figures = profile.as_ref().is_none_or(DomainProfile::figures_enabled);
    if do_figures {
        figures::try_generate_figures(&root);
    } else {
        println!("  [~] figures/ skipped (disabled in profile)");
    }

    // 19. Re-seal checksums (include figures/ if generated)
    let final_checksums = receipts::compute_checksums(
        &root,
        &["outputs", "provenance", "data", "configs", "figures"],
    );
    let final_cksum_content = receipts::format_checksums(&final_checksums);
    std::fs::write(root.join("receipts/checksums.blake3"), &final_cksum_content)
        .expect("Failed to write final receipts/checksums.blake3");
    if final_checksums.len() > checksums.len() {
        println!(
            "  [+] receipts/checksums.blake3 re-sealed ({} entries, +{} from figures)",
            final_checksums.len(),
            final_checksums.len() - checksums.len()
        );
    }

    println!();
    println!("Done. pseudoSpore emitted to: {}", root.display());
    println!();
    println!("Verify:");
    println!("  cd {}", root.display());
    println!("  ./validate                  # airgapped validation + liveSpore append");
    println!("  ./refresh                   # re-fetch datasets from source_uri");
    println!("  litho audit --path . --json # structured JSON report");
}

pub(crate) fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn copy_tree(src: &Path, dst: &Path) {
    if !src.is_dir() {
        return;
    }
    std::fs::create_dir_all(dst).ok();
    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let dest = dst.join(entry.file_name());
            if path.is_dir() {
                copy_tree(&path, &dest);
            } else {
                std::fs::copy(&path, &dest).ok();
            }
        }
    }
}
