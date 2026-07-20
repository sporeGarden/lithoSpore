// SPDX-License-Identifier: AGPL-3.0-or-later

//! scope.toml parsing — domain-agnostic artifact identity and module listing.

use serde::{Deserialize, Serialize};

/// Top-level scope.toml document — artifact identity, modules, and provenance links.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeDoc {
    /// `[artifact]` (or legacy `[guidestone]`) header: name, version, and type.
    #[serde(alias = "guidestone")]
    pub artifact: ArtifactHeader,
    /// Optional bibliographic target this artifact reproduces or validates.
    #[serde(default)]
    pub target: Option<TargetRef>,
    /// Declared validation modules and their last-known status.
    #[serde(default, alias = "modules")]
    pub module: Vec<ModuleEntry>,
    /// Evolution tier labels (Tier 0–3) for maturity tracking.
    #[serde(default)]
    pub evolution: Option<EvolutionTiers>,
    /// Source repository metadata when the artifact is built from git.
    #[serde(default)]
    pub source: Option<SourceRef>,
    /// Links to parent braid, plumed build, and DAG merkle root.
    #[serde(default)]
    pub provenance: Option<ProvenanceRef>,
}

/// `[artifact]` section — human-readable identity for a pseudoSpore or guideStone.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArtifactHeader {
    pub name: String,
    pub version: String,
    /// Artifact kind (e.g. `pseudoSpore`, `guideStone`).
    #[serde(rename = "type", default)]
    pub artifact_type: String,
    #[serde(default)]
    pub date: String,
    /// Producing spring or garden (e.g. `lithoSpore`, `hotSpring`).
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub license: String,
}

/// Bibliographic reference for the scientific target this artifact claims.
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

/// A single `[[module]]` entry in scope.toml.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModuleEntry {
    pub name: String,
    /// Last validation status (e.g. `pass`, `fail`, `in_flight`).
    #[serde(default)]
    pub status: String,
    /// Number of checks recorded for this module, if known.
    #[serde(default)]
    pub checks: Option<u32>,
    #[serde(default)]
    pub description: String,
    /// Entity group this module validates (maps to data subdirectories).
    #[serde(default)]
    pub entity_group: Option<String>,
    /// Computation keys within this module (e.g. sub-analyses or pipelines).
    #[serde(default)]
    pub computation: Vec<String>,
}

/// Evolution tier labels mapping maturity stages to version strings.
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

/// `[source]` section — git coordinates for reproducible builds.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceRef {
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub branch: String,
}

/// `[provenance]` section — ecosystem braid and DAG identifiers.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProvenanceRef {
    /// Parent `FermentBraid` identifier for lineage.
    #[serde(default)]
    pub parent_braid: String,
    #[serde(default)]
    pub plumed_version: String,
    /// Merkle root of the DAG session that produced this artifact.
    #[serde(default)]
    pub dag_merkle_root: String,
}

impl ScopeDoc {
    /// Load scope.toml from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &std::path::Path) -> Result<Self, crate::SporeError> {
        let content = std::fs::read_to_string(path).map_err(|source| crate::SporeError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&content).map_err(|e| crate::SporeError::Parse {
            path: path.to_path_buf(),
            detail: e.to_string(),
        })
    }

    /// Read a single field from a section (utility for backward compatibility).
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const VALID_SCOPE: &str = r#"
[artifact]
name = "test-spore"
version = "0.1.0"
type = "pseudoSpore"
date = "2026-05-27"
origin = "lithoSpore"

[[module]]
name = "ltee-fitness"
status = "pass"
checks = 12
description = "Fitness module"

[provenance]
parent_braid = "braid-test"
"#;

    #[test]
    fn load_valid_scope() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("scope.toml");
        fs::write(&path, VALID_SCOPE).expect("write scope");
        let doc = ScopeDoc::load(&path).expect("load scope");
        assert_eq!(doc.artifact.name, "test-spore");
        assert_eq!(doc.artifact.version, "0.1.0");
        assert_eq!(doc.module.len(), 1);
        assert_eq!(doc.module[0].name, "ltee-fitness");
        assert_eq!(doc.module[0].checks, Some(12));
    }

    #[test]
    fn guidestone_alias_parses_as_artifact() {
        let content = r#"
[guidestone]
name = "alias-artifact"
version = "2.0.0"
"#;
        let doc: ScopeDoc = toml::from_str(content).expect("parse guidestone alias");
        assert_eq!(doc.artifact.name, "alias-artifact");
    }

    #[test]
    fn missing_required_name_fails() {
        let content = r#"
[artifact]
version = "0.1.0"
"#;
        let err = toml::from_str::<ScopeDoc>(content).unwrap_err();
        assert!(
            err.to_string().contains("name"),
            "expected missing name error, got: {err}"
        );
    }

    #[test]
    fn field_accessor_returns_artifact_values() {
        let doc: ScopeDoc = toml::from_str(VALID_SCOPE).expect("parse");
        assert_eq!(doc.field("artifact", "name").as_deref(), Some("test-spore"));
        assert_eq!(
            doc.field("provenance", "parent_braid").as_deref(),
            Some("braid-test")
        );
        assert!(doc.field("unknown", "key").is_none());
    }

    #[test]
    fn modules_alias_parses_spring_scope() {
        let content = r#"
[artifact]
name = "airSpring-Agricultural-Meteorology"
version = "1.0.0"
type = "pseudoSpore"
date = "2026-07-18"
origin = "ecoPrimals/springs/airSpring"
license = "AGPL-3.0-or-later"

[[modules]]
name = "et0_reference"
entity_group = "et0_reference"
computation = ["module1_et0"]

[[modules]]
name = "soil_physics"
entity_group = "soil_physics"
computation = ["module3_soil_physics"]
"#;
        let doc: ScopeDoc = toml::from_str(content).expect("parse spring scope");
        assert_eq!(doc.artifact.name, "airSpring-Agricultural-Meteorology");
        assert_eq!(doc.module.len(), 2);
        assert_eq!(doc.module[0].name, "et0_reference");
        assert_eq!(doc.module[0].entity_group.as_deref(), Some("et0_reference"));
        assert_eq!(doc.module[0].computation, vec!["module1_et0"]);
        assert_eq!(doc.module[1].name, "soil_physics");
    }
}
