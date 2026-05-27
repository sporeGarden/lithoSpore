// SPDX-License-Identifier: AGPL-3.0-or-later

//! validation.json types — machine-readable per-module results.

use serde::{Deserialize, Serialize};

/// Top-level validation.json document — machine-readable per-module results.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationDoc {
    /// Artifact name echoed from scope.toml.
    #[serde(default)]
    pub artifact: String,
    #[serde(default)]
    pub version: String,
    /// ISO date of the validation run.
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub modules: Vec<ValidationModule>,
    /// Aggregate pass/fail counts across all modules.
    #[serde(default)]
    pub summary: Option<ValidationSummary>,
}

/// Per-module validation result.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationModule {
    pub name: String,
    /// Outcome status (e.g. `pass`, `fail`, `skip`).
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub checks_total: Option<u32>,
    #[serde(default)]
    pub checks_passed: Option<u32>,
    /// Individual check records (structure varies by module).
    #[serde(default)]
    pub checks: Vec<serde_json::Value>,
    /// Known discrepancies or caveats for this module.
    #[serde(default)]
    pub errata: Vec<serde_json::Value>,
}

/// Overall summary counts for a validation.json run.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationSummary {
    #[serde(default)]
    pub modules_total: u32,
    #[serde(default)]
    pub modules_pass: u32,
    /// Modules still running or awaiting upstream data.
    #[serde(default)]
    pub modules_in_flight: u32,
}

impl ValidationDoc {
    /// Load from a validation.json file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse validation.json: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_validation_with_modules() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("validation.json");
        let content = r#"{
            "artifact": "test-artifact",
            "version": "1.0.0",
            "date": "2026-05-27",
            "modules": [
                {"name": "mod-a", "status": "pass", "checks_total": 5, "checks_passed": 5}
            ],
            "summary": {"modules_total": 1, "modules_pass": 1, "modules_in_flight": 0}
        }"#;
        fs::write(&path, content).expect("write validation");
        let doc = ValidationDoc::load(&path).expect("load validation");
        assert_eq!(doc.artifact, "test-artifact");
        assert_eq!(doc.modules.len(), 1);
        assert_eq!(doc.modules[0].name, "mod-a");
        let summary = doc.summary.expect("summary present");
        assert_eq!(summary.modules_pass, 1);
    }

    #[test]
    fn empty_validation_json_uses_defaults() {
        let doc: ValidationDoc = serde_json::from_str("{}").expect("parse empty");
        assert!(doc.artifact.is_empty(), "artifact defaults empty");
        assert!(doc.modules.is_empty(), "modules defaults empty");
        assert!(doc.summary.is_none(), "summary defaults none");
    }

    #[test]
    fn load_missing_file_fails() {
        let err =
            ValidationDoc::load(std::path::Path::new("/nonexistent/validation.json")).unwrap_err();
        assert!(
            err.contains("Failed to read"),
            "expected read error, got: {err}"
        );
    }
}
