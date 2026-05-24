// SPDX-License-Identifier: AGPL-3.0-or-later

//! pseudoSpore: lightweight braid-first deployment artifact.
//!
//! A pseudoSpore proves a computation happened, what it produced, and how to
//! reproduce it — without carrying the runtime or raw inputs. This module
//! provides parsing, validation, and checksum verification for the pseudoSpore
//! standard (see `specs/PSEUDOSPORE_STANDARD.md`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// --- Scope types (pseudoSpore-specific header) ---

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

// --- Validation result types ---

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationSummary {
    #[serde(default)]
    pub modules_total: u32,
    #[serde(default)]
    pub modules_pass: u32,
    #[serde(default)]
    pub modules_in_flight: u32,
}

// --- Environment receipt ---

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvironmentReceipt {
    #[serde(default)]
    pub hardware: Option<BTreeMap<String, toml::Value>>,
    #[serde(default)]
    pub software: Option<BTreeMap<String, toml::Value>>,
    #[serde(default)]
    pub timestamps: Option<BTreeMap<String, toml::Value>>,
}

// --- Ferment transcript (minimal fields for validation) ---

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FermentTranscript {
    #[serde(default)]
    pub dataset_id: String,
    #[serde(default)]
    pub spring: String,
    #[serde(default)]
    pub spring_version: Option<String>,
    #[serde(default)]
    pub braid_id: Option<String>,
    #[serde(default)]
    pub dag_session_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

// --- Checksum entry ---

#[derive(Debug, Clone)]
pub struct ChecksumEntry {
    pub hash: String,
    pub path: String,
}

// --- Verification status ---

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

// --- Composite manifest (loaded pseudoSpore) ---

#[derive(Debug, Clone)]
pub struct PseudoSporeManifest {
    pub root: PathBuf,
    pub scope: PseudoSporeScope,
    pub validation: ValidationDoc,
    pub environment: EnvironmentReceipt,
    pub ferment: FermentTranscript,
    pub checksums: Vec<ChecksumEntry>,
    pub status: SporeStatus,
    pub errors: Vec<String>,
}

// --- Parsing and validation ---

/// Parse checksums.blake3 file content into entries.
pub fn parse_checksums(content: &str) -> Vec<ChecksumEntry> {
    content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let mut parts = line.splitn(2, "  ");
            let hash = parts.next()?.trim().to_string();
            let path = parts.next()?.trim().to_string();
            if hash.is_empty() || path.is_empty() {
                return None;
            }
            Some(ChecksumEntry { hash, path })
        })
        .collect()
}

/// Validate a pseudoSpore directory structure. Returns a manifest with status.
pub fn load_pseudospore(root: &Path) -> PseudoSporeManifest {
    let mut errors = Vec::new();

    // 1. Parse scope.toml
    let scope_path = root.join("scope.toml");
    let scope: PseudoSporeScope = match std::fs::read_to_string(&scope_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("scope.toml parse error: {e}"));
                return invalid_manifest(root, errors);
            }
        },
        Err(_) => {
            errors.push("scope.toml not found".to_string());
            return invalid_manifest(root, errors);
        }
    };

    if scope.artifact.artifact_type != "pseudoSpore" && scope.artifact.artifact_type != "pseudo-lithoSpore" {
        errors.push(format!(
            "scope.toml type is '{}', expected 'pseudoSpore'",
            scope.artifact.artifact_type
        ));
    }

    // 2. Parse validation.json
    let val_path = root.join("validation.json");
    let validation: ValidationDoc = match std::fs::read_to_string(&val_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("validation.json parse error: {e}"));
                return invalid_manifest(root, errors);
            }
        },
        Err(_) => {
            errors.push("validation.json not found".to_string());
            return invalid_manifest(root, errors);
        }
    };

    if validation.modules.is_empty() {
        errors.push("validation.json has no modules".to_string());
    }

    // 3. Parse receipts/environment.toml
    let env_path = root.join("receipts/environment.toml");
    let environment: EnvironmentReceipt = match std::fs::read_to_string(&env_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("receipts/environment.toml parse error: {e}"));
                return invalid_manifest(root, errors);
            }
        },
        Err(_) => {
            errors.push("receipts/environment.toml not found".to_string());
            return invalid_manifest(root, errors);
        }
    };

    if environment.hardware.is_none() {
        errors.push("receipts/environment.toml missing [hardware]".to_string());
    }
    if environment.software.is_none() {
        errors.push("receipts/environment.toml missing [software]".to_string());
    }

    // 4. Parse receipts/checksums.blake3
    let cksum_path = root.join("receipts/checksums.blake3");
    let checksums: Vec<ChecksumEntry> = match std::fs::read_to_string(&cksum_path) {
        Ok(content) => parse_checksums(&content),
        Err(_) => {
            errors.push("receipts/checksums.blake3 not found".to_string());
            Vec::new()
        }
    };

    // 5. Parse provenance/ferment_transcript.json
    let ferment_path = root.join("provenance/ferment_transcript.json");
    let ferment: FermentTranscript = match std::fs::read_to_string(&ferment_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(f) => f,
            Err(e) => {
                errors.push(format!("provenance/ferment_transcript.json parse error: {e}"));
                return invalid_manifest(root, errors);
            }
        },
        Err(_) => {
            errors.push("provenance/ferment_transcript.json not found".to_string());
            return invalid_manifest(root, errors);
        }
    };

    if ferment.dataset_id.is_empty() {
        errors.push("ferment_transcript.json missing dataset_id".to_string());
    }
    if ferment.spring.is_empty() {
        errors.push("ferment_transcript.json missing spring".to_string());
    }

    // 6. Check README.md
    let readme_path = root.join("README.md");
    if !readme_path.exists() {
        errors.push("README.md not found".to_string());
    } else if std::fs::metadata(&readme_path).map(|m| m.len()).unwrap_or(0) == 0 {
        errors.push("README.md is empty".to_string());
    }

    // Determine base status
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
        match std::fs::read(&file_path) {
            Ok(data) => {
                let computed = blake3::hash(&data).to_hex().to_string();
                if computed != entry.hash {
                    manifest.errors.push(format!(
                        "Checksum mismatch: {} (expected {}, got {})",
                        entry.path,
                        &entry.hash[..12],
                        &computed[..12]
                    ));
                    all_ok = false;
                }
            }
            Err(_) => {
                manifest.errors.push(format!("Missing file: {}", entry.path));
                all_ok = false;
            }
        }
    }

    if all_ok {
        manifest.status = SporeStatus::Verified;
    }
    all_ok
}

/// Check completeness (all modules PASS or SKIP, none IN_FLIGHT).
pub fn check_completeness(manifest: &mut PseudoSporeManifest) -> bool {
    if manifest.status == SporeStatus::Invalid {
        return false;
    }

    let all_done = manifest.validation.modules.iter().all(|m| {
        let s = m.status.to_uppercase();
        s == "PASS" || s == "SKIP"
    });

    if all_done && (manifest.status == SporeStatus::Verified || manifest.status == SporeStatus::Valid) {
        manifest.status = SporeStatus::Complete;
    }
    all_done
}

// --- Emission helpers ---

/// Compute BLAKE3 checksums for all files under a directory, relative to root.
pub fn compute_checksums(root: &Path, dirs: &[&str]) -> Vec<ChecksumEntry> {
    let mut entries = Vec::new();
    for dir_name in dirs {
        let dir = root.join(dir_name);
        if !dir.exists() {
            continue;
        }
        if let Ok(walker) = walk_dir(&dir) {
            for file_path in walker {
                if let Ok(data) = std::fs::read(&file_path) {
                    let hash = blake3::hash(&data).to_hex().to_string();
                    let rel = file_path
                        .strip_prefix(root)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .to_string();
                    entries.push(ChecksumEntry { hash, path: rel });
                }
            }
        }
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries
}

/// Format checksum entries into the checksums.blake3 file content.
pub fn format_checksums(entries: &[ChecksumEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{}  {}", e.hash, e.path))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Simple recursive directory walker (no external deps).
fn walk_dir(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk_dir(&path)?);
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
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
        validation: ValidationDoc {
            artifact: String::new(),
            version: String::new(),
            date: String::new(),
            modules: Vec::new(),
            summary: None,
        },
        environment: EnvironmentReceipt {
            hardware: None,
            software: None,
            timestamps: None,
        },
        ferment: FermentTranscript {
            dataset_id: String::new(),
            spring: String::new(),
            spring_version: None,
            braid_id: None,
            dag_session_id: None,
            timestamp: None,
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
            ChecksumEntry { hash: "aaa".to_string(), path: "a.txt".to_string() },
            ChecksumEntry { hash: "bbb".to_string(), path: "b.txt".to_string() },
        ];
        let formatted = format_checksums(&entries);
        assert_eq!(formatted, "aaa  a.txt\nbbb  b.txt");
    }
}
