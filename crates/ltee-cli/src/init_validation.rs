// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho init-validation` — generate a validation.json stub from a scope.toml.
//!
//! Reads the `scope.toml` in the given pseudoSpore directory, extracts module
//! names, and writes a `validation.json` with all modules set to PENDING.
//! Existing validation.json is backed up if present.

use std::path::Path;

use pseudospore_core::scope::ScopeDoc;
use pseudospore_core::validation::{ValidationDoc, ValidationModule, ValidationSummary};

/// Generate a validation.json stub from scope.toml modules.
///
/// # Errors
///
/// Returns an error string if scope.toml cannot be read/parsed.
pub fn run(spore_path: &str, force: bool) -> Result<(), String> {
    let spore_dir = Path::new(spore_path);
    let scope_path = spore_dir.join("scope.toml");
    let validation_path = spore_dir.join("validation.json");

    if !scope_path.exists() {
        return Err(format!(
            "No scope.toml found at {}. This directory may not be a pseudoSpore.",
            scope_path.display()
        ));
    }

    if validation_path.exists() && !force {
        return Err(format!(
            "validation.json already exists at {}. Use --force to overwrite.",
            validation_path.display()
        ));
    }

    let scope = ScopeDoc::load(&scope_path).map_err(|e| format!("Cannot parse scope.toml: {e}"))?;

    let name = &scope.artifact.name;
    let version = &scope.artifact.version;
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let modules: Vec<ValidationModule> = scope
        .module
        .iter()
        .map(|m| ValidationModule {
            name: m.name.clone(),
            status: "PENDING".to_string(),
            checks_total: None,
            checks_passed: None,
            checks: Vec::new(),
            errata: Vec::new(),
        })
        .collect();

    let total = modules.len() as u32;

    let doc = ValidationDoc {
        artifact: name.clone(),
        version: version.clone(),
        date,
        modules,
        summary: Some(ValidationSummary {
            modules_total: total,
            modules_pass: 0,
            modules_in_flight: total,
        }),
    };

    if validation_path.exists() {
        let backup = spore_dir.join("validation.json.bak");
        std::fs::copy(&validation_path, &backup)
            .map_err(|e| format!("Cannot backup validation.json: {e}"))?;
        println!("  Backed up existing validation.json → validation.json.bak");
    }

    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Cannot serialize validation.json: {e}"))?;
    std::fs::write(&validation_path, &json)
        .map_err(|e| format!("Cannot write validation.json: {e}"))?;

    println!("=== init-validation ===");
    println!("  Spore: {name} v{version}");
    println!("  Modules: {total} (all PENDING)");
    for m in &scope.module {
        println!("    - {}", m.name);
    }
    println!("  Output: {}", validation_path.display());
    println!();
    println!("  Next: run spring validators, then:");
    println!("    litho populate-validation {spore_path} --results <results.json>");
    println!("    litho promote-spore {spore_path} --artifact-root <root>");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn init_from_spring_scope() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spore = dir.path();

        let scope = r#"
[artifact]
name = "test-Spring-Artifact"
version = "1.0.0"
type = "pseudoSpore"
date = "2026-07-20"
origin = "testSpring"

[[modules]]
name = "module_alpha"
entity_group = "alpha"
computation = ["compute_a"]

[[modules]]
name = "module_beta"
entity_group = "beta"
computation = ["compute_b", "compute_c"]
"#;
        fs::write(spore.join("scope.toml"), scope).expect("write scope");

        run(spore.to_str().expect("path"), false).expect("init should succeed");

        let doc = ValidationDoc::load(&spore.join("validation.json")).expect("load");
        assert_eq!(doc.artifact, "test-Spring-Artifact");
        assert_eq!(doc.modules.len(), 2);
        assert_eq!(doc.modules[0].name, "module_alpha");
        assert_eq!(doc.modules[0].status, "PENDING");
        assert_eq!(doc.modules[1].name, "module_beta");
        assert_eq!(doc.summary.as_ref().expect("summary").modules_total, 2);
        assert_eq!(doc.summary.as_ref().expect("summary").modules_pass, 0);
        assert_eq!(doc.summary.as_ref().expect("summary").modules_in_flight, 2);
    }

    #[test]
    fn init_refuses_overwrite_without_force() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spore = dir.path();

        let scope = r#"
[artifact]
name = "test"
version = "1.0.0"

[[modules]]
name = "mod_a"
"#;
        fs::write(spore.join("scope.toml"), scope).expect("write scope");
        fs::write(spore.join("validation.json"), "{}").expect("write existing");

        let result = run(spore.to_str().expect("path"), false);
        assert!(result.is_err());
        assert!(result.expect_err("should fail").contains("--force"));
    }

    #[test]
    fn init_overwrites_with_force_and_backs_up() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spore = dir.path();

        let scope = r#"
[artifact]
name = "test"
version = "1.0.0"

[[modules]]
name = "mod_a"
"#;
        fs::write(spore.join("scope.toml"), scope).expect("write scope");
        fs::write(spore.join("validation.json"), r#"{"old": true}"#).expect("write existing");

        run(spore.to_str().expect("path"), true).expect("force init should succeed");

        assert!(spore.join("validation.json.bak").exists());
        let backup = fs::read_to_string(spore.join("validation.json.bak")).expect("read backup");
        assert!(backup.contains("old"));

        let doc = ValidationDoc::load(&spore.join("validation.json")).expect("load");
        assert_eq!(doc.modules.len(), 1);
    }
}
