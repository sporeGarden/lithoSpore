// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho ingest-pseudospore <path>` — validate and import a pseudoSpore artifact.
//!
//! Validates the pseudoSpore structure, verifies checksums, imports braids into
//! `provenance/braids/`, and registers it in `pseudospores/registry.toml`.

use litho_core::pseudospore::{
    self, ChecksumEntry, PseudoSporeManifest, SporeStatus,
};
use std::path::Path;

pub fn run(pseudospore_path: &str, artifact_root: &str, verify: bool) {
    let ps_root = Path::new(pseudospore_path);
    let litho_root = Path::new(artifact_root);

    if !ps_root.exists() {
        eprintln!("ERROR: pseudoSpore path does not exist: {pseudospore_path}");
        std::process::exit(1);
    }

    println!("=== litho ingest-pseudospore ===");
    println!("  Source: {pseudospore_path}");
    println!();

    // 1. Load and validate structure
    let mut manifest = pseudospore::load_pseudospore(ps_root);

    if manifest.status == SporeStatus::Invalid {
        eprintln!("INVALID pseudoSpore — structural errors:");
        for err in &manifest.errors {
            eprintln!("  - {err}");
        }
        std::process::exit(1);
    }

    println!("  Artifact: {} v{}", manifest.scope.artifact.name, manifest.scope.artifact.version);
    println!("  Origin:   {}", manifest.scope.artifact.origin);
    println!("  Date:     {}", manifest.scope.artifact.date);
    println!("  Modules:  {}", manifest.scope.module.len());
    println!();

    // 2. Optionally verify checksums
    if verify {
        print!("  Verifying checksums... ");
        if pseudospore::verify_checksums(&mut manifest) {
            println!("OK ({} files)", manifest.checksums.len());
        } else {
            println!("FAILED");
            for err in &manifest.errors {
                if err.contains("Checksum") || err.contains("Missing file") {
                    eprintln!("    {err}");
                }
            }
        }
    }

    // 3. Check completeness
    let complete = pseudospore::check_completeness(&mut manifest);
    println!("  Status: {}", manifest.status);
    if !complete {
        let in_flight: Vec<_> = manifest.validation.modules.iter()
            .filter(|m| m.status.to_uppercase() == "IN_FLIGHT")
            .map(|m| m.name.as_str())
            .collect();
        if !in_flight.is_empty() {
            println!("  In-flight modules: {}", in_flight.join(", "));
        }
    }
    println!();

    // 3b. Validate index_map.toml if present
    let index_map_path = ps_root.join("index_map.toml");
    if index_map_path.exists() {
        print!("  Validating index_map.toml... ");
        match validate_index_map(&index_map_path) {
            Ok(system_count) => println!("OK ({system_count} systems mapped)"),
            Err(e) => println!("WARNING: {e}"),
        }
    } else {
        println!("  index_map.toml: not present (optional, recommended for domain translation)");
    }

    // 3c. Check data/ directory for zero-trust verification
    let data_dir = ps_root.join("data");
    if data_dir.exists() {
        let data_modules = std::fs::read_dir(&data_dir)
            .map(|entries| entries.flatten().filter(|e| e.path().is_dir()).count())
            .unwrap_or(0);
        println!("  data/: present ({data_modules} modules — zero-trust derivation enabled)");
    } else {
        println!("  data/: not present (trust-required mode)");
    }
    println!();

    // 4. Import braids
    let braids_src = ps_root.join("provenance/braids");
    let braids_dst = litho_root.join("provenance/braids");
    let mut braids_imported = 0;

    if braids_src.exists() {
        std::fs::create_dir_all(&braids_dst).ok();
        if let Ok(entries) = std::fs::read_dir(&braids_src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    let dest = braids_dst.join(entry.file_name());
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if std::fs::write(&dest, &content).is_ok() {
                            braids_imported += 1;
                        }
                    }
                }
            }
        }
    }

    // Also import ferment_transcript as a braid
    let ferment_src = ps_root.join("provenance/ferment_transcript.json");
    if ferment_src.exists() {
        std::fs::create_dir_all(&braids_dst).ok();
        let ferment_name = format!(
            "{}_ferment.json",
            manifest.scope.artifact.name.replace(' ', "_").to_lowercase()
        );
        let ferment_dst = braids_dst.join(&ferment_name);
        if !ferment_dst.exists() {
            if let Ok(content) = std::fs::read_to_string(&ferment_src) {
                if std::fs::write(&ferment_dst, &content).is_ok() {
                    braids_imported += 1;
                }
            }
        }
    }

    println!("  Braids imported: {braids_imported}");

    // 5. Register in pseudospores/registry.toml
    let registry_dir = litho_root.join("pseudospores");
    std::fs::create_dir_all(&registry_dir).ok();
    let registry_path = registry_dir.join("registry.toml");

    let entry = format_registry_entry(&manifest);
    let mut registry_content = std::fs::read_to_string(&registry_path).unwrap_or_default();
    if !registry_content.contains(&format!("name = \"{}\"", manifest.scope.artifact.name)) {
        registry_content.push_str(&entry);
        std::fs::write(&registry_path, &registry_content).ok();
        println!("  Registered in pseudospores/registry.toml");
    } else {
        println!("  Already registered (skipped)");
    }

    println!();
    println!("Done. pseudoSpore ingested as {} ({})",
        manifest.scope.artifact.name, manifest.status);
}

fn format_registry_entry(manifest: &PseudoSporeManifest) -> String {
    let modules_pass = manifest.validation.modules.iter()
        .filter(|m| m.status.to_uppercase() == "PASS")
        .count();
    let modules_total = manifest.validation.modules.len();

    format!(
        r#"
[[pseudospore]]
name = "{name}"
version = "{version}"
origin = "{origin}"
date = "{date}"
spring = "{spring}"
status = "{status}"
modules_pass = {modules_pass}
modules_total = {modules_total}
"#,
        name = manifest.scope.artifact.name,
        version = manifest.scope.artifact.version,
        origin = manifest.scope.artifact.origin,
        date = manifest.scope.artifact.date,
        spring = manifest.ferment.spring,
        status = manifest.status,
        modules_pass = modules_pass,
        modules_total = modules_total,
    )
}

/// Validate an index_map.toml file: parseable TOML with [meta] and [systems.*].
fn validate_index_map(path: &Path) -> Result<usize, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read: {e}"))?;
    let table: toml::Table = content.parse()
        .map_err(|e| format!("parse error: {e}"))?;

    if !table.contains_key("meta") {
        return Err("[meta] section missing".to_string());
    }

    let systems = table.get("systems")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "[systems] section missing or not a table".to_string())?;

    let mut count = 0;
    for (name, val) in systems {
        if let Some(sys) = val.as_table() {
            if sys.contains_key("ring") || sys.contains_key("atoms") {
                count += 1;
            } else {
                return Err(format!("systems.{name} has no ring/atoms mapping"));
            }
        }
    }

    if count == 0 {
        return Err("no systems with atom mappings found".to_string());
    }

    Ok(count)
}

/// Print a structured summary of checksum entries (for --verbose).
#[allow(dead_code)]
fn print_checksums(checksums: &[ChecksumEntry]) {
    for entry in checksums {
        println!("    {}  {}", &entry.hash[..12], entry.path);
    }
}
