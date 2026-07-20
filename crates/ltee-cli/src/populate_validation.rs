// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho populate-validation` — populate a pseudoSpore's validation.json
//! from spring team module results.
//!
//! Accepts either a results JSON file (array of module results) or inline
//! module declarations, merges them into the existing validation.json, and
//! updates the summary counts.

use std::path::Path;

use pseudospore_core::validation::{ValidationDoc, ValidationModule, ValidationSummary};

/// Run the populate-validation command.
///
/// # Errors
///
/// Returns an error string if the operation fails.
pub fn run(
    spore_path: &str,
    results_path: Option<&str>,
    modules: &[(String, String)],
) -> Result<(), String> {
    let spore_dir = Path::new(spore_path);
    let validation_path = spore_dir.join("validation.json");

    if !validation_path.exists() {
        return Err(format!(
            "No validation.json found at {}. Run `litho emit-pseudospore` first.",
            validation_path.display()
        ));
    }

    let mut doc = ValidationDoc::load(&validation_path)
        .map_err(|e| format!("Failed to load validation.json: {e}"))?;

    if let Some(rp) = results_path {
        let content = std::fs::read_to_string(rp)
            .map_err(|e| format!("Cannot read results file {rp}: {e}"))?;

        let new_modules: Vec<ValidationModule> = serde_json::from_str(&content)
            .map_err(|e| format!("Cannot parse results JSON: {e}"))?;

        merge_modules(&mut doc, &new_modules);
    }

    for (name, status) in modules {
        let vm = ValidationModule {
            name: name.clone(),
            status: status.clone(),
            checks_total: None,
            checks_passed: None,
            checks: Vec::new(),
            errata: Vec::new(),
        };
        merge_modules(&mut doc, &[vm]);
    }

    update_summary(&mut doc);

    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize validation.json: {e}"))?;
    std::fs::write(&validation_path, &json)
        .map_err(|e| format!("Failed to write validation.json: {e}"))?;

    let summary = doc.summary.as_ref();
    let pass = summary.map_or(0, |s| s.modules_pass);
    let total = summary.map_or(0, |s| s.modules_total);

    println!("=== populate-validation ===");
    println!("  Target: {}", validation_path.display());
    println!("  Modules: {total} total, {pass} pass");
    println!(
        "  Status: {}",
        if pass == total && total > 0 {
            "COMPLETE"
        } else {
            "PARTIAL"
        }
    );

    Ok(())
}

fn merge_modules(doc: &mut ValidationDoc, new_modules: &[ValidationModule]) {
    for new in new_modules {
        if let Some(existing) = doc.modules.iter_mut().find(|m| m.name == new.name) {
            *existing = new.clone();
        } else {
            doc.modules.push(new.clone());
        }
    }
}

fn update_summary(doc: &mut ValidationDoc) {
    let total = doc.modules.len() as u32;
    let pass = doc
        .modules
        .iter()
        .filter(|m| m.status.eq_ignore_ascii_case("pass"))
        .count() as u32;
    let in_flight = doc
        .modules
        .iter()
        .filter(|m| {
            let s = m.status.to_lowercase();
            s == "pending" || s == "in_flight" || s == "running"
        })
        .count() as u32;

    doc.summary = Some(ValidationSummary {
        modules_total: total,
        modules_pass: pass,
        modules_in_flight: in_flight,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn populate_from_results_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let spore = dir.path();

        let initial = serde_json::json!({
            "artifact": "test-spore",
            "version": "1.0.0",
            "date": "2026-07-20",
            "modules": [],
            "summary": {"modules_total": 0, "modules_pass": 0, "modules_in_flight": 0}
        });
        fs::write(
            spore.join("validation.json"),
            serde_json::to_string_pretty(&initial).expect("serialize"),
        )
        .expect("write validation.json");

        let results = serde_json::json!([
            {"name": "module_a", "status": "PASS", "checks_total": 10, "checks_passed": 10},
            {"name": "module_b", "status": "FAIL", "checks_total": 5, "checks_passed": 3}
        ]);
        let results_path = spore.join("results.json");
        fs::write(
            &results_path,
            serde_json::to_string(&results).expect("serialize"),
        )
        .expect("write results.json");

        run(
            spore.to_str().expect("path"),
            Some(results_path.to_str().expect("path")),
            &[],
        )
        .expect("populate should succeed");

        let doc = ValidationDoc::load(&spore.join("validation.json")).expect("reload");
        assert_eq!(doc.modules.len(), 2);
        assert_eq!(doc.summary.as_ref().expect("summary").modules_pass, 1);
        assert_eq!(doc.summary.as_ref().expect("summary").modules_total, 2);
    }

    #[test]
    fn populate_with_inline_modules() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let spore = dir.path();

        let initial = serde_json::json!({
            "artifact": "test-spore",
            "version": "1.0.0",
            "date": "2026-07-20",
            "modules": [],
            "summary": {"modules_total": 0, "modules_pass": 0, "modules_in_flight": 0}
        });
        fs::write(
            spore.join("validation.json"),
            serde_json::to_string_pretty(&initial).expect("serialize"),
        )
        .expect("write validation.json");

        let modules = vec![
            ("et0_reference".to_string(), "PASS".to_string()),
            ("soil_physics".to_string(), "PASS".to_string()),
        ];

        run(spore.to_str().expect("path"), None, &modules).expect("populate should succeed");

        let doc = ValidationDoc::load(&spore.join("validation.json")).expect("reload");
        assert_eq!(doc.modules.len(), 2);
        assert_eq!(doc.summary.as_ref().expect("summary").modules_pass, 2);
    }

    #[test]
    fn populate_merges_existing_modules() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let spore = dir.path();

        let initial = serde_json::json!({
            "artifact": "test-spore",
            "version": "1.0.0",
            "date": "2026-07-20",
            "modules": [
                {"name": "existing_mod", "status": "PENDING"}
            ],
            "summary": {"modules_total": 1, "modules_pass": 0, "modules_in_flight": 0}
        });
        fs::write(
            spore.join("validation.json"),
            serde_json::to_string_pretty(&initial).expect("serialize"),
        )
        .expect("write validation.json");

        let modules = vec![("existing_mod".to_string(), "PASS".to_string())];

        run(spore.to_str().expect("path"), None, &modules).expect("populate should succeed");

        let doc = ValidationDoc::load(&spore.join("validation.json")).expect("reload");
        assert_eq!(doc.modules.len(), 1);
        assert_eq!(doc.modules[0].status, "PASS");
        assert_eq!(doc.summary.as_ref().expect("summary").modules_pass, 1);
    }
}
