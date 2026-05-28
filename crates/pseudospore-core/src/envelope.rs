// SPDX-License-Identifier: AGPL-3.0-or-later

//! `PseudoSpore` envelope — unified validation API for consumers.
//!
//! [`PseudoSporeEnvelope`] loads and validates a pseudoSpore directory
//! by checking all required components: scope, data manifest, checksums,
//! liveSpore schema, and optional provenance.
//!
//! This is the primary API for external consumers (e.g. biomeOS
//! `nucleus ingest`) — call [`PseudoSporeEnvelope::load`] then
//! [`PseudoSporeEnvelope::validate`].

use std::path::{Path, PathBuf};

use crate::braid_envelope::FermentTranscript;
use crate::receipts::{ChecksumEntry, EnvironmentReceipt, parse_checksums};
use crate::{Blake3Manifest, LiveSporeDoc, ScopeDoc, SporeError, ValidationDoc};

/// Validation result from [`PseudoSporeEnvelope::validate`].
#[derive(Debug, Clone)]
pub struct EnvelopeValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub scope: Option<ScopeDoc>,
    pub checksums_verified: usize,
    pub checksums_failed: usize,
}

/// A loaded pseudoSpore envelope — all components parsed from a directory.
#[derive(Debug)]
pub struct PseudoSporeEnvelope {
    pub root: PathBuf,
    pub scope: Option<ScopeDoc>,
    pub data_manifest: Option<Blake3Manifest>,
    pub validation: Option<ValidationDoc>,
    pub livespore: Option<LiveSporeDoc>,
    pub environment: Option<EnvironmentReceipt>,
    pub ferment: Option<FermentTranscript>,
    pub checksums: Vec<ChecksumEntry>,
    /// Warnings accumulated during load (unparseable optional components, etc.)
    pub load_warnings: Vec<String>,
}

impl PseudoSporeEnvelope {
    /// Load all components from a pseudoSpore directory.
    ///
    /// `scope.toml` is required; a missing or unparseable scope returns an error.
    /// Optional components that fail to load are omitted and logged as warnings.
    ///
    /// # Errors
    ///
    /// Returns an error if `scope.toml` is missing or cannot be parsed.
    pub fn load(root: &Path) -> Result<Self, SporeError> {
        let root = root.to_path_buf();

        let scope_path = root.join("scope.toml");
        if !scope_path.exists() {
            return Err(SporeError::NotFound(scope_path));
        }
        let scope = ScopeDoc::load(&scope_path)?;

        let mut envelope = Self {
            root,
            scope: Some(scope),
            data_manifest: None,
            validation: None,
            livespore: None,
            environment: None,
            ferment: None,
            checksums: Vec::new(),
            load_warnings: Vec::new(),
        };

        let data_path = envelope.root.join("data.toml");
        if data_path.exists() {
            match Blake3Manifest::load(&data_path) {
                Ok(m) => envelope.data_manifest = Some(m),
                Err(e) => envelope.load_warnings.push(e.to_string()),
            }
        }

        let validation_path = envelope.root.join("validation.json");
        if validation_path.exists() {
            match ValidationDoc::load(&validation_path) {
                Ok(v) => envelope.validation = Some(v),
                Err(e) => envelope.load_warnings.push(e.to_string()),
            }
        }

        let livespore_path = envelope.root.join("liveSpore.json");
        if livespore_path.exists() {
            match LiveSporeDoc::load(&livespore_path) {
                Ok(d) => envelope.livespore = Some(d),
                Err(e) => envelope.load_warnings.push(e.to_string()),
            }
        }

        let env_path = envelope.root.join("receipts/environment.toml");
        if env_path.exists() {
            match EnvironmentReceipt::load(&env_path) {
                Ok(r) => envelope.environment = Some(r),
                Err(e) => envelope.load_warnings.push(e.to_string()),
            }
        }

        let checksums_path = envelope.root.join("receipts/checksums.blake3");
        if checksums_path.exists() {
            match std::fs::read_to_string(&checksums_path) {
                Ok(content) => envelope.checksums = parse_checksums(&content),
                Err(e) => envelope
                    .load_warnings
                    .push(format!("failed to read {}: {e}", checksums_path.display())),
            }
        }

        let ferment_path = envelope.root.join("provenance/ferment_transcript.json");
        if ferment_path.exists() {
            match FermentTranscript::load(&ferment_path) {
                Ok(f) => envelope.ferment = Some(f),
                Err(e) => envelope.load_warnings.push(e.to_string()),
            }
        }

        Ok(envelope)
    }

    /// Run structural validation checks on the loaded envelope.
    #[must_use]
    pub fn validate(&self) -> EnvelopeValidation {
        let mut errors = Vec::new();
        let mut warnings = self.load_warnings.clone();
        let mut checksums_verified = 0usize;
        let mut checksums_failed = 0usize;

        let scope = if let Some(s) = &self.scope {
            if s.artifact.name.trim().is_empty() {
                errors.push("scope.toml: artifact.name is empty".to_string());
            }
            if s.artifact.version.trim().is_empty() {
                errors.push("scope.toml: artifact.version is empty".to_string());
            }
            Some(s.clone())
        } else {
            errors.push("scope.toml missing or not loaded".to_string());
            None
        };

        if self.data_manifest.is_none() {
            warnings.push("data.toml not present (recommended for integrity)".to_string());
        }

        if let Some(manifest) = &self.data_manifest {
            let manifest_errors = manifest.verify_present(&self.root);
            let total = manifest.present.len();
            checksums_failed = manifest_errors.len();
            checksums_verified = total.saturating_sub(checksums_failed);
            for err in manifest_errors {
                match err {
                    crate::blake3_manifest::ManifestError::Missing(path) => {
                        errors.push(format!("data.toml: missing file {path}"));
                    }
                    crate::blake3_manifest::ManifestError::HashMismatch {
                        path,
                        expected,
                        actual,
                    } => {
                        errors.push(format!(
                            "data.toml: checksum mismatch for {path} (expected {expected}, got {actual})"
                        ));
                    }
                }
            }
        }

        let livespore_path = self.root.join("liveSpore.json");
        if livespore_path.exists() {
            match check_livespore_unified(&livespore_path) {
                Ok(()) => {}
                Err(e) => errors.push(e.to_string()),
            }
        }

        let has_outputs = self.root.join("outputs").is_dir();
        let has_provenance = self.root.join("provenance").is_dir();
        let has_configs = self.root.join("configs").is_dir();
        if !has_outputs && !has_provenance && !has_configs {
            warnings.push("no outputs/, provenance/, or configs/ directory found".to_string());
        }

        let tolerances_path = self.root.join("tolerances.toml");
        let has_tolerances = tolerances_path.is_file();
        if !has_tolerances {
            warnings.push("tolerances.toml not found (GUIDESTONE-GRADE item 11)".to_string());
        }

        let calibration_path = self.root.join("derivations/threshold_calibration.toml");
        let has_calibration = calibration_path.is_file();
        if !has_calibration {
            warnings.push(
                "derivations/threshold_calibration.toml not found (GUIDESTONE-GRADE item 12)"
                    .to_string(),
            );
        }

        if has_tolerances && let Ok(content) = std::fs::read_to_string(&tolerances_path) {
            check_tolerance_derivation_fields(&content, &mut warnings);
        }

        let valid = errors.is_empty();
        EnvelopeValidation {
            valid,
            errors,
            warnings,
            scope,
            checksums_verified,
            checksums_failed,
        }
    }
}

/// GUIDESTONE items 13-14: check that `[[tolerance]]` entries have `derivation`
/// fields and none carry `_anchoring = "NEEDS_CALIBRATION"`.
fn check_tolerance_derivation_fields(content: &str, warnings: &mut Vec<String>) {
    let table: toml::Table = if let Ok(t) = content.parse() {
        t
    } else {
        warnings.push("tolerances.toml: failed to parse as TOML".to_string());
        return;
    };
    let Some(tolerances) = table.get("tolerance").and_then(|v| v.as_array()) else {
        return;
    };
    let mut missing_derivation = 0u32;
    let mut needs_calibration = 0u32;
    for tol in tolerances {
        let tol = match tol.as_table() {
            Some(t) => t,
            None => continue,
        };
        let name = tol
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("<unnamed>");
        if tol
            .get("derivation")
            .and_then(|v| v.as_str())
            .is_none_or(|s| s.trim().is_empty())
        {
            missing_derivation += 1;
        }
        if tol
            .get("_anchoring")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v == "NEEDS_CALIBRATION")
        {
            needs_calibration += 1;
            warnings.push(format!(
                "tolerance '{name}' has _anchoring = \"NEEDS_CALIBRATION\" (GUIDESTONE item 14)"
            ));
        }
    }
    if missing_derivation > 0 {
        warnings.push(format!(
            "{missing_derivation} tolerance(s) missing 'derivation' field (GUIDESTONE item 13)"
        ));
    }
    if needs_calibration > 0 {
        warnings.push(format!(
            "{needs_calibration} tolerance(s) still NEEDS_CALIBRATION (GUIDESTONE item 14)"
        ));
    }
}

fn check_livespore_unified(path: &Path) -> Result<(), crate::SporeError> {
    let content = std::fs::read_to_string(path).map_err(|e| crate::SporeError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let raw: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| crate::SporeError::Parse {
            path: path.to_path_buf(),
            detail: e.to_string(),
        })?;

    if raw.is_array() {
        return Err(crate::SporeError::Validation(
            "liveSpore.json uses legacy bare-array schema; unified {envelope, validations} required"
                .into(),
        ));
    }

    if raw.get("envelope").is_none() || raw.get("validations").is_none() {
        return Err(crate::SporeError::Validation(
            "liveSpore.json must contain top-level envelope and validations keys".into(),
        ));
    }

    Ok(())
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
"#;

    #[test]
    fn test_load_valid_envelope() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("scope.toml"), VALID_SCOPE).expect("write scope");
        let envelope = PseudoSporeEnvelope::load(dir.path()).expect("load envelope");
        assert!(envelope.scope.is_some());
        assert_eq!(envelope.scope.as_ref().unwrap().artifact.name, "test-spore");
    }

    #[test]
    fn test_validate_missing_scope() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = PseudoSporeEnvelope::load(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("scope.toml"),
            "expected scope error, got: {err}"
        );

        let envelope = PseudoSporeEnvelope {
            root: dir.path().to_path_buf(),
            scope: None,
            data_manifest: None,
            validation: None,
            livespore: None,
            environment: None,
            ferment: None,
            checksums: Vec::new(),
            load_warnings: Vec::new(),
        };
        let result = envelope.validate();
        assert!(!result.valid);
        assert!(
            result.errors.iter().any(|e| e.contains("scope.toml")),
            "validate reports missing scope"
        );
    }

    #[test]
    fn test_validate_checksum_mismatch() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("scope.toml"), VALID_SCOPE).expect("scope");
        fs::write(dir.path().join("payload.txt"), b"tampered").expect("payload");

        let wrong_hash = blake3::hash(b"original").to_hex().to_string();
        let data_toml = format!(
            r#"
[present]
"payload.txt" = "{wrong_hash}"
"#
        );
        fs::write(dir.path().join("data.toml"), data_toml).expect("data.toml");

        let envelope = PseudoSporeEnvelope::load(dir.path()).expect("load");
        let result = envelope.validate();
        assert!(!result.valid);
        assert!(result.checksums_failed > 0);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("checksum mismatch")),
            "checksum mismatch reported"
        );
    }

    #[test]
    fn test_validate_unified_livespore() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("scope.toml"), VALID_SCOPE).expect("scope");
        fs::create_dir_all(dir.path().join("outputs")).expect("outputs dir");
        let unified = r#"{"envelope":{"artifact":"test"},"validations":[]}"#;
        fs::write(dir.path().join("liveSpore.json"), unified).expect("liveSpore");

        let envelope = PseudoSporeEnvelope::load(dir.path()).expect("load");
        let result = envelope.validate();
        assert!(result.valid, "errors: {:?}", result.errors);

        let legacy = r#"[{"timestamp":"2026-01-01T00:00:00Z","hostname_hash":"abc"}]"#;
        fs::write(dir.path().join("liveSpore.json"), legacy).expect("legacy liveSpore");
        let envelope = PseudoSporeEnvelope::load(dir.path()).expect("reload");
        let result = envelope.validate();
        assert!(!result.valid);
        assert!(
            result.errors.iter().any(|e| e.contains("bare-array")),
            "legacy array rejected"
        );
    }

    #[test]
    fn test_golden_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();

        fs::write(root.join("scope.toml"), VALID_SCOPE).expect("scope");
        fs::create_dir_all(root.join("outputs")).expect("outputs");
        fs::create_dir_all(root.join("configs")).expect("configs");

        let payload = b"golden-payload-data";
        fs::write(root.join("outputs/results.csv"), payload).expect("payload");

        let hash = blake3::hash(payload).to_hex().to_string();
        let data_toml = format!("[present]\n\"outputs/results.csv\" = \"{hash}\"\n");
        fs::write(root.join("data.toml"), &data_toml).expect("data.toml");

        let livespore =
            r#"{"envelope":{"artifact":"test-artifact","version":"1.0.0"},"validations":[]}"#;
        fs::write(root.join("liveSpore.json"), livespore).expect("liveSpore");

        let envelope = PseudoSporeEnvelope::load(root).expect("load golden");
        let scope = envelope.scope.as_ref().expect("scope present");
        assert_eq!(scope.artifact.name, "test-spore");
        assert_eq!(scope.artifact.version, "0.1.0");

        let result = envelope.validate();
        assert!(
            result.valid,
            "golden round-trip failed: {:?}",
            result.errors
        );
        assert_eq!(result.checksums_verified, 1);
        assert_eq!(result.checksums_failed, 0);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("tolerances.toml")),
            "should warn about missing tolerances"
        );
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("threshold_calibration")),
            "should warn about missing derivation anchoring"
        );
    }

    #[test]
    fn guidestone_grade_tolerance_checks() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();

        fs::write(root.join("scope.toml"), VALID_SCOPE).expect("scope");
        fs::create_dir_all(root.join("outputs")).expect("outputs");
        fs::create_dir_all(root.join("derivations")).expect("derivations");
        fs::write(
            root.join("derivations/threshold_calibration.toml"),
            "[metadata]\nstandard = \"v1.0\"\n",
        )
        .expect("calibration");

        let tol = "[[tolerance]]\nname = \"rmsd\"\nvalue = 2.0\n_anchoring = \"NEEDS_CALIBRATION\"\n\n\
                   [[tolerance]]\nname = \"checksum\"\nvalue = 0\nderivation = \"BLAKE3\"\n";
        fs::write(root.join("tolerances.toml"), tol).expect("tolerances");

        let envelope = PseudoSporeEnvelope::load(root).expect("load");
        let result = envelope.validate();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("NEEDS_CALIBRATION")),
            "should flag NEEDS_CALIBRATION: {:?}",
            result.warnings
        );
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("missing 'derivation'")),
            "should flag missing derivation: {:?}",
            result.warnings
        );
        assert!(
            !result
                .warnings
                .iter()
                .any(|w| w.contains("threshold_calibration.toml not found")),
            "should not warn about calibration when present"
        );
    }
}
