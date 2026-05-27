// SPDX-License-Identifier: AGPL-3.0-or-later

//! scope.toml parsing — domain-agnostic artifact identity and module listing.

use serde::{Deserialize, Serialize};

/// Top-level scope.toml document.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeDoc {
    #[serde(alias = "guidestone")]
    pub artifact: ArtifactHeader,
    #[serde(default)]
    pub target: Option<TargetRef>,
    #[serde(default)]
    pub module: Vec<ModuleEntry>,
    #[serde(default)]
    pub evolution: Option<EvolutionTiers>,
    #[serde(default)]
    pub source: Option<SourceRef>,
    #[serde(default)]
    pub provenance: Option<ProvenanceRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArtifactHeader {
    pub name: String,
    pub version: String,
    #[serde(rename = "type", default)]
    pub artifact_type: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TargetRef {
    #[serde(default)]
    pub paper_doi: String,
    #[serde(default)]
    pub paper_title: String,
    #[serde(default)]
    pub paper_authors: String,
    #[serde(default)]
    pub paper_year: Option<u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModuleEntry {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub checks: Option<u32>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvolutionTiers {
    #[serde(default)]
    pub tier_0: Option<String>,
    #[serde(default)]
    pub tier_1: Option<String>,
    #[serde(default)]
    pub tier_2: Option<String>,
    #[serde(default)]
    pub tier_3: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceRef {
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub branch: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProvenanceRef {
    #[serde(default)]
    pub parent_braid: String,
    #[serde(default)]
    pub plumed_version: String,
    #[serde(default)]
    pub dag_merkle_root: String,
}

impl ScopeDoc {
    /// Load scope.toml from a file path.
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse scope.toml: {e}"))
    }

    /// Read a single field from a section (utility for backward compatibility).
    pub fn field(&self, section: &str, key: &str) -> Option<String> {
        match section {
            "artifact" | "guidestone" => match key {
                "name" => Some(self.artifact.name.clone()),
                "version" => Some(self.artifact.version.clone()),
                "type" => Some(self.artifact.artifact_type.clone()),
                "date" => Some(self.artifact.date.clone()),
                "origin" => Some(self.artifact.origin.clone()),
                _ => None,
            },
            "provenance" => self.provenance.as_ref().and_then(|p| match key {
                "parent_braid" => Some(p.parent_braid.clone()),
                "plumed_version" => Some(p.plumed_version.clone()),
                "dag_merkle_root" => Some(p.dag_merkle_root.clone()),
                _ => None,
            }),
            _ => None,
        }
    }
}
