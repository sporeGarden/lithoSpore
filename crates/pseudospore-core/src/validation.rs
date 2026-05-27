// SPDX-License-Identifier: AGPL-3.0-or-later

//! validation.json types — machine-readable per-module results.

use serde::{Deserialize, Serialize};

/// Top-level validation.json document.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationDoc {
    #[serde(default)]
    pub artifact: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub modules: Vec<ValidationModule>,
    #[serde(default)]
    pub summary: Option<ValidationSummary>,
}

/// Per-module validation result.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationModule {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub checks_total: Option<u32>,
    #[serde(default)]
    pub checks_passed: Option<u32>,
    #[serde(default)]
    pub checks: Vec<serde_json::Value>,
    #[serde(default)]
    pub errata: Vec<serde_json::Value>,
}

/// Overall summary counts.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationSummary {
    #[serde(default)]
    pub modules_total: u32,
    #[serde(default)]
    pub modules_pass: u32,
    #[serde(default)]
    pub modules_in_flight: u32,
}

impl ValidationDoc {
    /// Load from a validation.json file.
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse validation.json: {e}"))
    }
}
