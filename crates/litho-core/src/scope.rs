// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scope graph: the guideStone "birth certificate" that declares which
//! springs, primals, foundation threads, and modules compose this artifact.
//!
//! By reading `scope.toml` at runtime, the lithoSpore chassis becomes
//! domain-agnostic — the same validate/fetch/assemble pipeline works for
//! any guideStone instance.

use serde::Deserialize;
use std::path::Path;

/// Parsed scope.toml — the guideStone birth certificate for chassis dispatch.
#[derive(Debug, Clone, Deserialize)]
pub struct ScopeManifest {
    pub guidestone: GuideStoneIdentity,
    /// Contributing springs and their module binaries.
    #[serde(default)]
    pub spring: Vec<SpringEntry>,
    /// NUCLEUS primals required for Tier 3 provenance.
    #[serde(default)]
    pub primal: Vec<PrimalEntry>,
    /// Foundation threads grouping datasets by scientific narrative.
    #[serde(default)]
    pub foundation_thread: Vec<FoundationThread>,
    #[serde(default)]
    pub module: Vec<ScopeModule>,
    #[serde(default)]
    pub source: Option<SourceMetadata>,
}

/// Per-module metadata — the domain-agnostic contract that replaces
/// compiled `LTEE_MODULES` / `LTEE_NOTEBOOKS` constants.
///
/// When present in scope.toml, the chassis uses these fields for dispatch,
/// parity, assembly, and Tier-1 notebook resolution.
#[derive(Debug, Clone, Deserialize)]
pub struct ScopeModule {
    pub name: String,
    pub binary: String,
    #[serde(default)]
    pub data_dir: String,
    #[serde(default)]
    pub expected: String,
    #[serde(default)]
    pub tier1_notebook: String,
}

/// `[source]` metadata for reproducible builds and container images.
#[derive(Debug, Clone, Deserialize)]
pub struct SourceMetadata {
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub repo_https: String,
    #[serde(default)]
    pub branch: String,
    /// Parent ecosystem monorepo (e.g. ecoPrimals).
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

/// `[guidestone]` identity — name, version, and paths to targets/graph files.
#[derive(Debug, Clone, Deserialize)]
pub struct GuideStoneIdentity {
    pub name: String,
    pub version: String,
    /// Human-readable description of the scientific target.
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub created: String,
    /// Compliance standard (e.g. `TARGETED_GUIDESTONE_STANDARD`).
    #[serde(default)]
    pub standard: String,
    /// Relative path to targets.toml.
    #[serde(default)]
    pub targets_file: String,
    /// Relative path to the module dependency graph.
    #[serde(default)]
    pub graph_file: String,
}

/// A contributing spring and the modules it supplies to this guideStone.
#[derive(Debug, Clone, Deserialize)]
pub struct SpringEntry {
    pub name: String,
    /// Capability tags this spring contributes (e.g. `mutations`, `fitness`).
    #[serde(default)]
    pub contributes: Vec<String>,
    /// Module binary names owned by this spring.
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub papers: Vec<String>,
}

/// A NUCLEUS primal dependency — required for higher validation tiers.
#[derive(Debug, Clone, Deserialize)]
pub struct PrimalEntry {
    pub name: String,
    /// Minimum tier at which this primal must be reachable.
    #[serde(default)]
    pub tier: u8,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub purpose: String,
}

/// A foundation thread — groups related datasets under one scientific narrative.
#[derive(Debug, Clone, Deserialize)]
pub struct FoundationThread {
    pub id: String,
    pub name: String,
    /// Dataset paths or IDs belonging to this thread.
    #[serde(default)]
    pub datasets: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

impl ScopeManifest {
    /// Load from a TOML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, crate::LithoError> {
        let contents = std::fs::read_to_string(path).map_err(|e| crate::LithoError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let scope: Self = toml::from_str(&contents).map_err(|e| crate::LithoError::Parse {
            path: path.to_path_buf(),
            detail: e.to_string(),
        })?;
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
targets_file = "data/targets/test_targets.toml"
graph_file = "graphs/test_graph.toml"

[[spring]]
name = "testSpring"
contributes = ["thing"]
modules = ["mod-a", "mod-b"]
papers = ["B1"]

[[spring]]
name = "otherSpring"
modules = ["mod-b", "mod-c"]

[[module]]
name = "widget_analysis"
binary = "mod-a"
data_dir = "artifact/data/widgets"
expected = "validation/expected/widgets.json"
tier1_notebook = "notebooks/widgets/analyze.py"

[[module]]
name = "gadget_validation"
binary = "mod-b"
data_dir = "artifact/data/gadgets"
expected = "validation/expected/gadgets.json"

[[foundation_thread]]
id = "01"
name = "Test Thread"
datasets = ["data_a"]
"#;
        let scope: ScopeManifest = toml::from_str(toml).unwrap();
        assert_eq!(scope.guidestone.name, "test-guidestone");
        assert_eq!(
            scope.guidestone.targets_file,
            "data/targets/test_targets.toml"
        );
        assert_eq!(scope.guidestone.graph_file, "graphs/test_graph.toml");
        assert_eq!(scope.spring.len(), 2);
        assert_eq!(scope.module_binaries(), vec!["mod-a", "mod-b", "mod-c"]);
        assert_eq!(scope.module.len(), 2);
        assert_eq!(scope.module[0].name, "widget_analysis");
        assert_eq!(scope.module[0].binary, "mod-a");
        assert_eq!(scope.module[1].tier1_notebook, "");
        assert_eq!(scope.foundation_thread.len(), 1);
    }
}
