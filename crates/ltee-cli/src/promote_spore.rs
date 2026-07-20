// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho promote-spore` — promote a pseudoSpore from PENDING to COMPLETE.
//!
//! Validates that the spore's `validation.json` has all modules passing,
//! then updates the registry entry status to COMPLETE.

use std::path::Path;

use pseudospore_core::validation::ValidationDoc;

/// Promote a pseudoSpore from PENDING to COMPLETE in the registry.
///
/// Checks that all modules in `validation.json` have status "PASS" before
/// allowing promotion. Returns an error if validation is incomplete.
///
/// # Errors
///
/// Returns an error string if any pre-promotion check fails.
pub fn run(spore_path: &str, artifact_root: &str) -> Result<(), String> {
    let spore_dir = Path::new(spore_path);
    let validation_path = spore_dir.join("validation.json");

    let doc = ValidationDoc::load(&validation_path)
        .map_err(|e| format!("Cannot load validation.json: {e}"))?;

    let scope_path = spore_dir.join("scope.toml");
    let scope_name = if scope_path.exists() {
        let content = std::fs::read_to_string(&scope_path)
            .map_err(|e| format!("Cannot read scope.toml: {e}"))?;
        let parsed: toml::Value =
            toml::from_str(&content).map_err(|e| format!("Cannot parse scope.toml: {e}"))?;
        parsed
            .get("artifact")
            .and_then(|a| a.get("name"))
            .and_then(toml::Value::as_str)
            .unwrap_or(&doc.artifact)
            .to_string()
    } else {
        doc.artifact.clone()
    };

    if doc.modules.is_empty() {
        return Err("validation.json has no modules — run populate-validation first".into());
    }

    let failing: Vec<&str> = doc
        .modules
        .iter()
        .filter(|m| !m.status.eq_ignore_ascii_case("pass"))
        .map(|m| m.name.as_str())
        .collect();

    if !failing.is_empty() {
        return Err(format!(
            "Cannot promote: {} module(s) not passing: {}",
            failing.len(),
            failing.join(", ")
        ));
    }

    let registry_path = Path::new(artifact_root).join("pseudospores/registry.toml");
    if !registry_path.exists() {
        return Err(format!("Registry not found at {}", registry_path.display()));
    }

    let content = std::fs::read_to_string(&registry_path)
        .map_err(|e| format!("Cannot read registry: {e}"))?;
    let mut registry: toml::Value =
        toml::from_str(&content).map_err(|e| format!("Cannot parse registry: {e}"))?;

    let entries = registry
        .get_mut("pseudospore")
        .and_then(toml::Value::as_array_mut)
        .ok_or("Registry has no [[pseudospore]] entries")?;

    let found = entries
        .iter_mut()
        .find(|e| e.get("name").and_then(toml::Value::as_str) == Some(&scope_name));

    let Some(entry) = found else {
        return Err(format!("No registry entry found for '{scope_name}'"));
    };

    let current_status = entry
        .get("status")
        .and_then(toml::Value::as_str)
        .unwrap_or("UNKNOWN");
    if current_status == "COMPLETE" {
        println!("=== promote-spore ===");
        println!("  {scope_name} is already COMPLETE — no change needed");
        return Ok(());
    }

    let pass_count = i64::try_from(doc.modules.len()).unwrap_or(i64::MAX);
    if let Some(table) = entry.as_table_mut() {
        table.insert("status".into(), toml::Value::String("COMPLETE".into()));
        table.insert("modules_pass".into(), toml::Value::Integer(pass_count));
        table.insert("modules_total".into(), toml::Value::Integer(pass_count));
    }

    let out =
        toml::to_string_pretty(&registry).map_err(|e| format!("Cannot serialize registry: {e}"))?;
    std::fs::write(&registry_path, &out).map_err(|e| format!("Cannot write registry: {e}"))?;

    println!("=== promote-spore ===");
    println!("  Spore: {scope_name}");
    println!("  Modules: {pass_count}/{pass_count} PASS");
    println!("  Registry: {}", registry_path.display());
    println!("  Status: PENDING → COMPLETE");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn promote_succeeds_with_all_pass() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let spore = dir.path().join("spore");
        fs::create_dir_all(&spore).expect("mkdir spore");

        let scope = toml::toml! {
            [artifact]
            name = "test-spore"
            version = "1.0.0"
        };
        fs::write(
            spore.join("scope.toml"),
            toml::to_string(&scope).expect("ser"),
        )
        .expect("write scope");

        let validation = serde_json::json!({
            "artifact": "test-spore",
            "version": "1.0.0",
            "date": "2026-07-20",
            "modules": [
                {"name": "mod_a", "status": "PASS", "checks_total": 5, "checks_passed": 5},
                {"name": "mod_b", "status": "PASS", "checks_total": 3, "checks_passed": 3}
            ]
        });
        fs::write(
            spore.join("validation.json"),
            serde_json::to_string_pretty(&validation).expect("ser"),
        )
        .expect("write validation");

        let root = dir.path().join("root");
        fs::create_dir_all(root.join("pseudospores")).expect("mkdir");
        let registry_content = r#"
[meta]
total_ingested = 1

[[pseudospore]]
name = "test-spore"
version = "1.0.0"
spring = "testSpring"
status = "PENDING"
modules_pass = 0
modules_total = 0
date = "2026-07-20"
"#;
        fs::write(root.join("pseudospores/registry.toml"), registry_content)
            .expect("write registry");

        run(spore.to_str().expect("path"), root.to_str().expect("path"))
            .expect("promote should succeed");

        let reg = fs::read_to_string(root.join("pseudospores/registry.toml")).expect("read");
        assert!(reg.contains("COMPLETE"), "Should be promoted: {reg}");
    }

    #[test]
    fn promote_fails_with_failing_module() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let spore = dir.path().join("spore");
        fs::create_dir_all(&spore).expect("mkdir");

        let validation = serde_json::json!({
            "artifact": "test-spore",
            "version": "1.0.0",
            "date": "2026-07-20",
            "modules": [
                {"name": "mod_a", "status": "PASS"},
                {"name": "mod_b", "status": "FAIL"}
            ]
        });
        fs::write(
            spore.join("validation.json"),
            serde_json::to_string_pretty(&validation).expect("ser"),
        )
        .expect("write validation");

        let err_msg = run(spore.to_str().expect("path"), "/dev/null").expect_err("should fail");
        assert!(err_msg.contains("mod_b"));
    }

    #[test]
    fn promote_fails_with_no_modules() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let spore = dir.path().join("spore");
        fs::create_dir_all(&spore).expect("mkdir");

        let validation = serde_json::json!({
            "artifact": "test-spore",
            "version": "1.0.0",
            "date": "2026-07-20",
            "modules": []
        });
        fs::write(
            spore.join("validation.json"),
            serde_json::to_string_pretty(&validation).expect("ser"),
        )
        .expect("write validation");

        let err_msg = run(spore.to_str().expect("path"), "/dev/null").expect_err("should fail");
        assert!(err_msg.contains("no modules"));
    }
}
