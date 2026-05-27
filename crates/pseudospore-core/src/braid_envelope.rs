// SPDX-License-Identifier: AGPL-3.0-or-later

//! Braid envelope — `FermentBraid` wire types for pseudoSpore provenance.
//!
//! A `FermentTranscript` is the minimal provenance document that every pseudoSpore
//! carries in `provenance/ferment_transcript.json`. It identifies which spring
//! produced the computation, what dataset it belongs to, and links to the
//! ecosystem-wide braid/DAG/ledger identifiers.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Minimal provenance record from `provenance/ferment_transcript.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FermentTranscript {
    /// Dataset identifier this computation belongs to.
    #[serde(default)]
    pub dataset_id: String,
    /// Spring that produced the computation (e.g. `hotSpring`, `lithoSpore`).
    #[serde(default)]
    pub spring: String,
    #[serde(default)]
    pub spring_version: Option<String>,
    /// `FermentBraid` identifier linking to ecosystem lineage.
    #[serde(default)]
    pub braid_id: Option<String>,
    #[serde(default)]
    pub dag_session_id: Option<String>,
    /// Merkle root of the DAG session anchoring this run.
    #[serde(default)]
    pub dag_merkle_root: Option<String>,
    /// loamSpine session identifier when Tier 3 provenance is available.
    #[serde(default)]
    pub spine_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Free-form computation metadata (tool, version, parameters).
    #[serde(default)]
    pub computation: Option<serde_json::Value>,
}

impl FermentTranscript {
    /// Load from a `provenance/ferment_transcript.json` file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed as JSON.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse ferment_transcript.json: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_transcript() {
        let json = r#"{"dataset_id":"test_v1","spring":"hotSpring"}"#;
        let ft: FermentTranscript = serde_json::from_str(json).unwrap();
        assert_eq!(ft.dataset_id, "test_v1");
        assert_eq!(ft.spring, "hotSpring");
        assert!(ft.braid_id.is_none());
    }

    #[test]
    fn parse_full_transcript() {
        let json = r#"{
            "dataset_id":"cazyme_fel_v0.6.0",
            "spring":"hotSpring",
            "spring_version":"0.6.32",
            "braid_id":"braid-hotspring-cazyme-fel-20260524",
            "dag_session_id":"dag-hotspring-cazyme-001",
            "dag_merkle_root":"blake3:abc123",
            "spine_id":"spine-hotspring-cazyme-001",
            "timestamp":"2026-05-24T14:00:00Z",
            "computation":{"tool":"GROMACS 2026.0"}
        }"#;
        let ft: FermentTranscript = serde_json::from_str(json).unwrap();
        assert_eq!(
            ft.braid_id.as_deref(),
            Some("braid-hotspring-cazyme-fel-20260524")
        );
        assert!(ft.computation.is_some());
    }

    #[test]
    fn load_valid_transcript_from_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ferment_transcript.json");
        std::fs::write(
            &path,
            r#"{"dataset_id":"ds1","spring":"lithoSpore","braid_id":"braid-1"}"#,
        )
        .expect("write transcript");
        let ft = FermentTranscript::load(&path).expect("load transcript");
        assert_eq!(ft.dataset_id, "ds1");
        assert_eq!(ft.spring, "lithoSpore");
    }

    #[test]
    fn load_invalid_json_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("ferment_transcript.json");
        std::fs::write(&path, "NOT JSON {{{").expect("write bad json");
        let err = FermentTranscript::load(&path).unwrap_err();
        assert!(
            err.contains("Failed to parse"),
            "expected parse error, got: {err}"
        );
    }

    #[test]
    fn load_missing_file_fails() {
        let err = FermentTranscript::load(std::path::Path::new("/nonexistent/transcript.json"))
            .unwrap_err();
        assert!(
            err.contains("Failed to read"),
            "expected read error, got: {err}"
        );
    }
}
