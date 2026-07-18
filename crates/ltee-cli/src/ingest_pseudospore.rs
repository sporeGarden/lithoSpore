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

    let Some(scope) = &envelope.scope else {
        eprintln!("ERROR: scope missing after successful load");
        std::process::exit(1);
    };

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
        let data_modules = std::fs::read_dir(&data_dir).map_or(0, |entries| {
            entries.flatten().filter(|e| e.path().is_dir()).count()
        });
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

    // 5. Register in pseudospores/registry.toml (structured TOML upsert)
    let registry_dir = litho_root.join("pseudospores");
    std::fs::create_dir_all(&registry_dir).ok();
    let registry_path = registry_dir.join("registry.toml");

    match upsert_registry(&registry_path, scope, &envelope) {
        Ok(RegistryAction::Inserted) => {
            println!("  Registered in pseudospores/registry.toml");
        }
        Ok(RegistryAction::Updated) => {
            println!("  Updated existing entry in pseudospores/registry.toml");
        }
        Err(e) => {
            eprintln!("  WARNING: registry update failed: {e}");
        }
    }

    println!();
    println!("Done. pseudoSpore ingested: {}", scope.artifact.name);
}

enum RegistryAction {
    Inserted,
    Updated,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Registry {
    meta: RegistryMeta,
    #[serde(default)]
    pseudospore: Vec<RegistryEntry>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RegistryMeta {
    last_updated: String,
    total_ingested: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RegistryEntry {
    name: String,
    version: String,
    origin: String,
    date: String,
    spring: String,
    status: String,
    modules_pass: usize,
    modules_total: usize,
}

fn upsert_registry(
    registry_path: &Path,
    scope: &ScopeDoc,
    envelope: &PseudoSporeEnvelope,
) -> Result<RegistryAction, String> {
    let mut registry: Registry = std::fs::read_to_string(registry_path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or(Registry {
            meta: RegistryMeta {
                last_updated: String::new(),
                total_ingested: 0,
            },
            pseudospore: Vec::new(),
        });

    let (modules_pass, modules_total) = envelope.validation.as_ref().map_or((0, 0), |v| {
        let pass = v
            .modules
            .iter()
            .filter(|m| m.status.eq_ignore_ascii_case("pass"))
            .count();
        (pass, v.modules.len())
    });

    let spring = envelope.ferment.as_ref().map_or_else(
        || {
            scope
                .artifact
                .origin
                .split('/')
                .next_back()
                .unwrap_or("unknown")
                .to_string()
        },
        |f| f.spring.clone(),
    );

    let status = if modules_total > 0 && modules_pass == modules_total {
        "COMPLETE".to_string()
    } else if modules_pass > 0 {
        "PARTIAL".to_string()
    } else {
        "PENDING".to_string()
    };

    let new_entry = RegistryEntry {
        name: scope.artifact.name.clone(),
        version: scope.artifact.version.clone(),
        origin: scope.artifact.origin.clone(),
        date: scope.artifact.date.clone(),
        spring,
        status,
        modules_pass,
        modules_total,
    };

    let action = if let Some(existing) = registry
        .pseudospore
        .iter_mut()
        .find(|e| e.name == new_entry.name && e.version == new_entry.version)
    {
        *existing = new_entry;
        RegistryAction::Updated
    } else {
        registry.pseudospore.push(new_entry);
        RegistryAction::Inserted
    };

    registry.meta.last_updated = chrono::Utc::now().format("%Y-%m-%d").to_string();
    registry.meta.total_ingested = registry.pseudospore.len();

    let header = "# pseudoSpore Registry\n\
                  #\n\
                  # Tracks ingested pseudoSpore artifacts. Each entry represents a lightweight\n\
                  # braid-first deployment that has been validated and registered with lithoSpore.\n\
                  #\n\
                  # Added automatically by `litho ingest-pseudospore <path>`.\n\
                  # See specs/PSEUDOSPORE_STANDARD.md for the pseudoSpore format.\n\n";

    let body = toml::to_string_pretty(&registry).map_err(|e| format!("serialize registry: {e}"))?;

    let content = format!("{header}{body}");
    std::fs::write(registry_path, content).map_err(|e| format!("write registry: {e}"))?;

    Ok(action)
}

/// Validate an `index_map.toml` file: parseable TOML with `[meta]` and `[systems.*]`.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scope(name: &str, version: &str) -> ScopeDoc {
        let dir = tempfile::tempdir().unwrap();
        let scope_path = dir.path().join("scope.toml");
        let toml_str = format!(
            "[artifact]\nname = \"{name}\"\nversion = \"{version}\"\norigin = \"eco/springs/test\"\ndate = \"2026-05-30\"\n"
        );
        std::fs::write(&scope_path, &toml_str).unwrap();
        let scope = ScopeDoc::load(&scope_path).unwrap();
        std::mem::forget(dir);
        scope
    }

    fn make_envelope(scope: &ScopeDoc) -> PseudoSporeEnvelope {
        PseudoSporeEnvelope {
            root: std::path::PathBuf::new(),
            scope: Some(scope.clone()),
            data_manifest: None,
            validation: None,
            livespore: None,
            environment: None,
            ferment: None,
            checksums: vec![],
            load_warnings: vec![],
        }
    }

    #[test]
    fn registry_insert_creates_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.toml");

        let scope = make_scope("test-spore", "1.0.0");
        let envelope = make_envelope(&scope);
        let result = upsert_registry(&path, &scope, &envelope).unwrap();
        assert!(matches!(result, RegistryAction::Inserted));

        let reg: Registry = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(reg.meta.total_ingested, 1);
        assert_eq!(reg.pseudospore.len(), 1);
        assert_eq!(reg.pseudospore[0].name, "test-spore");
        assert_eq!(reg.pseudospore[0].status, "PENDING");
    }

    #[test]
    fn registry_upsert_replaces_same_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.toml");

        let scope = make_scope("test-spore", "1.0.0");
        let envelope = make_envelope(&scope);
        upsert_registry(&path, &scope, &envelope).unwrap();

        let result = upsert_registry(&path, &scope, &envelope).unwrap();
        assert!(matches!(result, RegistryAction::Updated));

        let reg: Registry = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(reg.meta.total_ingested, 1);
    }

    #[test]
    fn registry_insert_new_version_appends() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.toml");

        let s1 = make_scope("test-spore", "1.0.0");
        upsert_registry(&path, &s1, &make_envelope(&s1)).unwrap();

        let s2 = make_scope("test-spore", "2.0.0");
        let result = upsert_registry(&path, &s2, &make_envelope(&s2)).unwrap();
        assert!(matches!(result, RegistryAction::Inserted));

        let reg: Registry = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(reg.meta.total_ingested, 2);
        assert_eq!(reg.pseudospore.len(), 2);
    }

    #[test]
    fn existing_registry_toml_parses() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pseudospores/registry.toml");
        let content = std::fs::read_to_string(&root).unwrap();
        let reg: Registry = toml::from_str(&content).unwrap();
        assert!(
            reg.meta.total_ingested >= 1,
            "registry should have at least 1 ingested pseudoSpore"
        );
        assert_eq!(reg.pseudospore.len(), reg.meta.total_ingested as usize);
        let hotspring = reg
            .pseudospore
            .iter()
            .find(|p| p.name == "hotSpring-CompChem-GuideStone");
        assert!(hotspring.is_some(), "hotSpring entry must be present");
        assert_eq!(hotspring.expect("checked above").status, "COMPLETE");
    }
}
