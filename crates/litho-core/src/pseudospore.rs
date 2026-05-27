// SPDX-License-Identifier: AGPL-3.0-or-later

//! DEPRECATED: Use `pseudospore_core` directly.
//!
//! This module re-exports pseudoSpore types from `pseudospore-core` for backward
//! compatibility. New code should depend on `pseudospore-core` directly.
//! See `SPORE_OWNERSHIP_MATRIX.md` for the canonical crate.

#[deprecated(note = "use pseudospore_core::ChecksumEntry directly")]
pub type ChecksumEntry = pseudospore_core::ChecksumEntry;

#[deprecated(note = "use pseudospore_core::EnvironmentReceipt directly")]
pub type EnvironmentReceipt = pseudospore_core::EnvironmentReceipt;

#[deprecated(note = "use pseudospore_core::FermentTranscript directly")]
pub type FermentTranscript = pseudospore_core::FermentTranscript;

pub use pseudospore_core::receipts::{compute_checksums, format_checksums, parse_checksums};
pub use pseudospore_core::validation::{ValidationDoc, ValidationModule, ValidationSummary};

// --- Types preserved for backward compatibility ---
// These were originally defined here; now canonical versions live in pseudospore-core.
// Re-exported without deprecation warnings to avoid churn in existing consumers.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PseudoSporeScope {
    pub artifact: ArtifactIdentity,
    #[serde(default)]
    pub target: Option<TargetPaper>,
    #[serde(default)]
    pub module: Vec<PseudoModule>,
    #[serde(default)]
    pub evolution: Option<EvolutionTiers>,
    #[serde(default)]
    pub source: Option<SourceRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArtifactIdentity {
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub origin: String,
    #[serde(default)]
    pub experiment: Option<u32>,
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TargetPaper {
    #[serde(default)]
    pub paper_doi: String,
    #[serde(default)]
    pub paper_title: String,
    #[serde(default)]
    pub paper_authors: String,
    #[serde(default)]
    pub paper_year: Option<u16>,
    #[serde(default)]
    pub paper_pdb: Option<String>,
    #[serde(default)]
    pub paper_system: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PseudoModule {
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub checks: Option<u32>,
    #[serde(default)]
    pub checks_total: Option<u32>,
    #[serde(default)]
    pub checks_passed: Option<u32>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cv: Option<String>,
    #[serde(default)]
    pub force_field: Option<String>,
    #[serde(default)]
    pub errata: Option<String>,
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
    #[serde(default)]
    pub acceptance_test: Option<String>,
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

/// Verification status of a loaded pseudoSpore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SporeStatus {
    Valid,
    Verified,
    Complete,
    Invalid,
}

impl std::fmt::Display for SporeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "VALID"),
            Self::Verified => write!(f, "VERIFIED"),
            Self::Complete => write!(f, "COMPLETE"),
            Self::Invalid => write!(f, "INVALID"),
        }
    }
}

/// Composite manifest of a loaded pseudoSpore directory.
#[derive(Debug, Clone)]
pub struct PseudoSporeManifest {
    pub root: PathBuf,
    pub scope: PseudoSporeScope,
    pub validation: pseudospore_core::ValidationDoc,
    pub environment: pseudospore_core::EnvironmentReceipt,
    pub ferment: pseudospore_core::FermentTranscript,
    pub checksums: Vec<pseudospore_core::ChecksumEntry>,
    pub status: SporeStatus,
    pub errors: Vec<String>,
}

/// Validate a pseudoSpore directory structure. Returns a manifest with status.
#[must_use]
pub fn load_pseudospore(root: &Path) -> PseudoSporeManifest {
    let mut errors = Vec::new();

    let scope_path = root.join("scope.toml");
    let scope: PseudoSporeScope = if let Ok(content) = std::fs::read_to_string(&scope_path) {
        match toml::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("scope.toml parse error: {e}"));
                return invalid_manifest(root, errors);
            }
        }
    } else {
        errors.push("scope.toml not found".to_string());
        return invalid_manifest(root, errors);
    };

    if scope.artifact.artifact_type != "pseudoSpore"
        && scope.artifact.artifact_type != "pseudo-lithoSpore"
    {
        errors.push(format!(
            "scope.toml type is '{}', expected 'pseudoSpore'",
            scope.artifact.artifact_type
        ));
    }

    let val_path = root.join("validation.json");
    let validation: pseudospore_core::ValidationDoc =
        if let Ok(content) = std::fs::read_to_string(&val_path) {
            match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    errors.push(format!("validation.json parse error: {e}"));
                    return invalid_manifest(root, errors);
                }
            }
        } else {
            errors.push("validation.json not found".to_string());
            return invalid_manifest(root, errors);
        };

    if validation.modules.is_empty() {
        errors.push("validation.json has no modules".to_string());
    }

    let env_path = root.join("receipts/environment.toml");
    let environment: pseudospore_core::EnvironmentReceipt =
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            match toml::from_str(&content) {
                Ok(e) => e,
                Err(e) => {
                    errors.push(format!("receipts/environment.toml parse error: {e}"));
                    return invalid_manifest(root, errors);
                }
            }
        } else {
            errors.push("receipts/environment.toml not found".to_string());
            return invalid_manifest(root, errors);
        };

    if environment.hardware.is_none() {
        errors.push("receipts/environment.toml missing [hardware]".to_string());
    }
    if environment.software.is_none() {
        errors.push("receipts/environment.toml missing [software]".to_string());
    }

    let cksum_path = root.join("receipts/checksums.blake3");
    let checksums: Vec<pseudospore_core::ChecksumEntry> =
        if let Ok(content) = std::fs::read_to_string(&cksum_path) {
            parse_checksums(&content)
        } else {
            errors.push("receipts/checksums.blake3 not found".to_string());
            Vec::new()
        };

    let ferment_path = root.join("provenance/ferment_transcript.json");
    let ferment: pseudospore_core::FermentTranscript =
        if let Ok(content) = std::fs::read_to_string(&ferment_path) {
            match serde_json::from_str(&content) {
                Ok(f) => f,
                Err(e) => {
                    errors.push(format!(
                        "provenance/ferment_transcript.json parse error: {e}"
                    ));
                    return invalid_manifest(root, errors);
                }
            }
        } else {
            errors.push("provenance/ferment_transcript.json not found".to_string());
            return invalid_manifest(root, errors);
        };

    if ferment.dataset_id.is_empty() {
        errors.push("ferment_transcript.json missing dataset_id".to_string());
    }
    if ferment.spring.is_empty() {
        errors.push("ferment_transcript.json missing spring".to_string());
    }

    let readme_path = root.join("README.md");
    if !readme_path.exists() {
        errors.push("README.md not found".to_string());
    } else if std::fs::metadata(&readme_path)
        .map(|m| m.len())
        .unwrap_or(0)
        == 0
    {
        errors.push("README.md is empty".to_string());
    }

    let status = if errors.is_empty() {
        SporeStatus::Valid
    } else {
        SporeStatus::Invalid
    };

    PseudoSporeManifest {
        root: root.to_path_buf(),
        scope,
        validation,
        environment,
        ferment,
        checksums,
        status,
        errors,
    }
}

/// Verify BLAKE3 checksums against actual files. Upgrades status to Verified.
pub fn verify_checksums(manifest: &mut PseudoSporeManifest) -> bool {
    if manifest.status == SporeStatus::Invalid {
        return false;
    }

    if manifest.checksums.is_empty() {
        manifest.errors.push("No checksums to verify".to_string());
        return false;
    }

    let mut all_ok = true;
    for entry in &manifest.checksums {
        let file_path = manifest.root.join(&entry.path);
        if let Ok(data) = std::fs::read(&file_path) {
            let computed = blake3::hash(&data).to_hex().to_string();
            if computed != entry.hash {
                manifest.errors.push(format!(
                    "Checksum mismatch: {} (expected {}, got {})",
                    entry.path,
                    &entry.hash[..12.min(entry.hash.len())],
                    &computed[..12]
                ));
                all_ok = false;
            }
        } else {
            manifest
                .errors
                .push(format!("Missing file: {}", entry.path));
            all_ok = false;
        }
    }

    if all_ok {
        manifest.status = SporeStatus::Verified;
    }
    all_ok
}

/// Check completeness (all modules PASS or SKIP, none `IN_FLIGHT`).
pub fn check_completeness(manifest: &mut PseudoSporeManifest) -> bool {
    if manifest.status == SporeStatus::Invalid {
        return false;
    }

    let all_done = manifest.validation.modules.iter().all(|m| {
        let s = m.status.to_uppercase();
        s == "PASS" || s == "SKIP"
    });

    if all_done
        && (manifest.status == SporeStatus::Verified || manifest.status == SporeStatus::Valid)
    {
        manifest.status = SporeStatus::Complete;
    }
    all_done
}

fn invalid_manifest(root: &Path, errors: Vec<String>) -> PseudoSporeManifest {
    PseudoSporeManifest {
        root: root.to_path_buf(),
        scope: PseudoSporeScope {
            artifact: ArtifactIdentity {
                name: String::new(),
                version: String::new(),
                artifact_type: String::new(),
                date: String::new(),
                origin: String::new(),
                experiment: None,
                license: String::new(),
            },
            target: None,
            module: Vec::new(),
            evolution: None,
            source: None,
        },
        validation: pseudospore_core::ValidationDoc {
            artifact: String::new(),
            version: String::new(),
            date: String::new(),
            modules: Vec::new(),
            summary: None,
        },
        environment: pseudospore_core::EnvironmentReceipt {
            hardware: None,
            software: None,
            timestamps: None,
        },
        ferment: pseudospore_core::FermentTranscript {
            dataset_id: String::new(),
            spring: String::new(),
            spring_version: None,
            braid_id: None,
            dag_session_id: None,
            dag_merkle_root: None,
            spine_id: None,
            timestamp: None,
            computation: None,
        },
        checksums: Vec::new(),
        status: SporeStatus::Invalid,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_checksums_basic() {
        let input = "abc123def456  outputs/foo.dat\n789012345678  provenance/bar.json\n";
        let entries = parse_checksums(input);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "abc123def456");
        assert_eq!(entries[0].path, "outputs/foo.dat");
        assert_eq!(entries[1].path, "provenance/bar.json");
    }

    #[test]
    fn parse_checksums_skips_comments_and_empty() {
        let input = "# header comment\nabc123  file.dat\n\n# another comment\n";
        let entries = parse_checksums(input);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn format_checksums_roundtrip() {
        let entries = vec![
            pseudospore_core::ChecksumEntry {
                hash: "aaa".to_string(),
                path: "a.txt".to_string(),
            },
            pseudospore_core::ChecksumEntry {
                hash: "bbb".to_string(),
                path: "b.txt".to_string(),
            },
        ];
        let formatted = format_checksums(&entries);
        assert_eq!(formatted, "aaa  a.txt\nbbb  b.txt");
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_reexports_match_pseudospore_core() {
        let core_entry = pseudospore_core::ChecksumEntry {
            hash: "h".to_string(),
            path: "p".to_string(),
        };
        let wrapped: ChecksumEntry = core_entry.clone();
        assert_eq!(wrapped.hash, core_entry.hash);
        assert_eq!(wrapped.path, core_entry.path);

        let input = "abc  file.txt\n";
        assert_eq!(
            parse_checksums(input).len(),
            pseudospore_core::parse_checksums(input).len()
        );
        let entries = vec![ChecksumEntry {
            hash: "x".to_string(),
            path: "y.txt".to_string(),
        }];
        assert_eq!(
            format_checksums(&entries),
            pseudospore_core::format_checksums(&entries)
        );
    }

    #[test]
    fn validation_doc_reexport_roundtrip() {
        let doc = ValidationDoc {
            artifact: "a".to_string(),
            version: "1".to_string(),
            date: String::new(),
            modules: vec![ValidationModule {
                name: "m".to_string(),
                status: "pass".to_string(),
                checks_total: Some(1),
                checks_passed: Some(1),
                checks: vec![],
                errata: vec![],
            }],
            summary: Some(ValidationSummary {
                modules_total: 1,
                modules_pass: 1,
                modules_in_flight: 0,
            }),
        };
        let json = serde_json::to_string(&doc).expect("serialize");
        let parsed: ValidationDoc = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.artifact, "a");
        assert_eq!(parsed.modules[0].name, "m");
    }
}
