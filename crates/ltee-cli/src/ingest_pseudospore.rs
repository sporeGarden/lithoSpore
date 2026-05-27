// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho ingest-pseudospore <path>` — validate and import a pseudoSpore artifact.
//!
//! Validates the pseudoSpore structure, verifies checksums, imports braids into
//! `provenance/braids/`, and registers it in `pseudospores/registry.toml`.

use pseudospore_core::{ChecksumEntry, PseudoSporeEnvelope, ScopeDoc};
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

    // 1. Load envelope via pseudospore-core canonical API
    let envelope = match PseudoSporeEnvelope::load(ps_root) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("INVALID pseudoSpore — {err}");
            std::process::exit(1);
        }
    };

    let result = envelope.validate();
    if !result.valid {
        eprintln!("INVALID pseudoSpore — validation errors:");
        for err in &result.errors {
            eprintln!("  - {err}");
        }
        std::process::exit(1);
    }

    let scope = envelope
        .scope
        .as_ref()
        .expect("scope guaranteed present after successful load");

    println!(
        "  Artifact: {} v{}",
        scope.artifact.name, scope.artifact.version
    );
    println!("  Origin:   {}", scope.artifact.origin);
    println!("  Date:     {}", scope.artifact.date);
    println!("  Modules:  {}", scope.module.len());
    if !result.warnings.is_empty() {
        for w in &result.warnings {
            println!("  [WARN] {w}");
        }
    }
    println!();

    // 2. Optionally verify checksums (already checked by validate, report details)
    if verify {
        print!("  Checksums: ");
        if result.checksums_failed == 0 {
            println!(
                "OK ({} verified, {} in receipts)",
                result.checksums_verified,
                envelope.checksums.len()
            );
            print_checksums(&envelope.checksums);
        } else {
            println!(
                "FAILED ({} mismatches out of {})",
                result.checksums_failed,
                result.checksums_verified + result.checksums_failed
            );
            for err in &result.errors {
                if err.contains("checksum") || err.contains("missing file") {
                    eprintln!("    {err}");
                }
            }
        }
    }

    // 3. Load validation.json for completeness reporting
    let validation = &envelope.validation;
    if let Some(val) = validation {
        let in_flight: Vec<_> = val
            .modules
            .iter()
            .filter(|m| m.status.eq_ignore_ascii_case("in_flight"))
            .map(|m| m.name.as_str())
            .collect();
        let pass_count = val
            .modules
            .iter()
            .filter(|m| m.status.eq_ignore_ascii_case("pass"))
            .count();
        println!(
            "  Validation: {}/{} modules pass",
            pass_count,
            val.modules.len()
        );
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
                if path.extension().is_some_and(|e| e == "json") {
                    let dest = braids_dst.join(entry.file_name());
                    if let Ok(content) = std::fs::read_to_string(&path)
                        && std::fs::write(&dest, &content).is_ok()
                    {
                        braids_imported += 1;
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
            scope.artifact.name.replace(' ', "_").to_lowercase()
        );
        let ferment_dst = braids_dst.join(&ferment_name);
        if !ferment_dst.exists()
            && let Ok(content) = std::fs::read_to_string(&ferment_src)
            && std::fs::write(&ferment_dst, &content).is_ok()
        {
            braids_imported += 1;
        }
    }

    println!("  Braids imported: {braids_imported}");

    // 5. Register in pseudospores/registry.toml
    let registry_dir = litho_root.join("pseudospores");
    std::fs::create_dir_all(&registry_dir).ok();
    let registry_path = registry_dir.join("registry.toml");

    let entry = format_registry_entry(scope, &envelope);
    let mut registry_content = std::fs::read_to_string(&registry_path).unwrap_or_default();
    if registry_content.contains(&format!("name = \"{}\"", scope.artifact.name)) {
        println!("  Already registered (skipped)");
    } else {
        registry_content.push_str(&entry);
        std::fs::write(&registry_path, &registry_content).ok();
        println!("  Registered in pseudospores/registry.toml");
    }

    println!();
    println!("Done. pseudoSpore ingested: {}", scope.artifact.name);
}

fn format_registry_entry(scope: &ScopeDoc, envelope: &PseudoSporeEnvelope) -> String {
    let (modules_pass, modules_total) = envelope.validation.as_ref().map_or((0, 0), |v| {
        let pass = v
            .modules
            .iter()
            .filter(|m| m.status.eq_ignore_ascii_case("pass"))
            .count();
        (pass, v.modules.len())
    });

    let spring = envelope
        .ferment
        .as_ref()
        .map_or("unknown", |f| f.spring.as_str());

    format!(
        r#"
[[pseudospore]]
name = "{name}"
version = "{version}"
origin = "{origin}"
date = "{date}"
spring = "{spring}"
modules_pass = {modules_pass}
modules_total = {modules_total}
"#,
        name = scope.artifact.name,
        version = scope.artifact.version,
        origin = scope.artifact.origin,
        date = scope.artifact.date,
    )
}

/// Validate an `index_map.toml` file: parseable TOML with [meta] and [systems.*].
fn validate_index_map(path: &Path) -> Result<usize, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("cannot read: {e}"))?;
    let table: toml::Table = content.parse().map_err(|e| format!("parse error: {e}"))?;

    if !table.contains_key("meta") {
        return Err("[meta] section missing".to_string());
    }

    let systems = table
        .get("systems")
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

/// Print a structured summary of checksum entries after verification.
fn print_checksums(checksums: &[ChecksumEntry]) {
    for entry in checksums {
        println!("    {}  {}", &entry.hash[..12], entry.path);
    }
}
