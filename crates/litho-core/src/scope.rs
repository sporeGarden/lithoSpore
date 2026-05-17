// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scope graph: the guideStone "birth certificate" that declares which
//! springs, primals, foundation threads, and modules compose this artifact.
//!
//! By reading `scope.toml` at runtime, the lithoSpore chassis becomes
//! domain-agnostic — the same validate/fetch/assemble pipeline works for
//! any guideStone instance.

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ScopeManifest {
    pub guidestone: GuideStoneIdentity,
    #[serde(default)]
    pub spring: Vec<SpringEntry>,
    #[serde(default)]
    pub primal: Vec<PrimalEntry>,
    #[serde(default)]
    pub foundation_thread: Vec<FoundationThread>,
    #[serde(default)]
    pub source: Option<SourceMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceMetadata {
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub repo_https: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub ecosystem_repo: String,
    #[serde(default)]
    pub ecosystem_repo_https: String,
    #[serde(default)]
    pub rust_toolchain: String,
    #[serde(default)]
    pub rust_target: String,
    #[serde(default)]
    pub containerfile: String,
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GuideStoneIdentity {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub standard: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpringEntry {
    pub name: String,
    #[serde(default)]
    pub contributes: Vec<String>,
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub papers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrimalEntry {
    pub name: String,
    #[serde(default)]
    pub tier: u8,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub purpose: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FoundationThread {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub datasets: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

impl ScopeManifest {
    /// Load from a TOML file.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let scope: Self = toml::from_str(&contents)?;
        Ok(scope)
    }

    /// Collect the deduplicated set of module binary names declared across
    /// all springs. Order is preserved (first occurrence wins).
    #[must_use]
    pub fn module_binaries(&self) -> Vec<&str> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for spring in &self.spring {
            for m in &spring.modules {
                if seen.insert(m.as_str()) {
                    out.push(m.as_str());
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scope_toml() {
        let toml = r#"
[guidestone]
name = "test-guidestone"
version = "0.1.0"
target = "Test Target"
created = "2026-05-16"
standard = "TARGETED_GUIDESTONE_STANDARD v1.0"

[[spring]]
name = "testSpring"
contributes = ["thing"]
modules = ["mod-a", "mod-b"]
papers = ["B1"]

[[spring]]
name = "otherSpring"
modules = ["mod-b", "mod-c"]

[[foundation_thread]]
id = "01"
name = "Test Thread"
datasets = ["data_a"]
"#;
        let scope: ScopeManifest = toml::from_str(toml).unwrap();
        assert_eq!(scope.guidestone.name, "test-guidestone");
        assert_eq!(scope.spring.len(), 2);
        assert_eq!(scope.module_binaries(), vec!["mod-a", "mod-b", "mod-c"]);
        assert_eq!(scope.foundation_thread.len(), 1);
    }
}
