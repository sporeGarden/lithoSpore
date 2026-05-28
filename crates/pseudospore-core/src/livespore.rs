// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unified liveSpore.json schema.
//!
//! Combines emit-time metadata (envelope) with append-only validation journal.
//! Handles migration from legacy schemas (bare array, hotSpring flat object).

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Unified liveSpore.json document — emit-time envelope plus validation journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSporeDoc {
    /// Emit-time metadata (artifact, version, software, provenance chain).
    #[serde(default)]
    pub envelope: serde_json::Value,
    /// Append-only log of validation runs on deployed copies.
    #[serde(default)]
    pub validations: Vec<ValidationEntry>,
}

/// A single validation run entry in the append-only journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationEntry {
    /// RFC 3339 timestamp of the validation run.
    pub timestamp: String,
    /// BLAKE3 hash of the host name (no raw hostname stored).
    pub hostname_hash: String,
    #[serde(default)]
    pub arch: String,
    #[serde(default)]
    pub os: String,
    /// Highest validation tier achieved in this run (0–3).
    #[serde(default)]
    pub tier_reached: u8,
    #[serde(default)]
    pub modules_passed: u32,
    #[serde(default)]
    pub modules_total: u32,
    /// Wall-clock runtime of the validation harness in milliseconds.
    #[serde(default)]
    pub runtime_ms: u64,
}

impl LiveSporeDoc {
    /// Create a new liveSpore document with envelope metadata and empty validations.
    #[must_use]
    pub fn new(envelope: serde_json::Value) -> Self {
        Self {
            envelope,
            validations: Vec::new(),
        }
    }

    /// Load from a liveSpore.json file, handling all three legacy formats:
    /// 1. Unified: `{"envelope": {...}, "validations": [...]}`
    /// 2. Legacy lithoSpore: bare array `[...]`
    /// 3. Legacy hotSpring: `{"liveSpore": {...}, "software": {...}, "provenance_chain": {...}}`
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed as JSON.
    pub fn load(path: &Path) -> Result<Self, crate::SporeError> {
        let content = std::fs::read_to_string(path).map_err(|source| crate::SporeError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let raw: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| crate::SporeError::Parse {
                path: path.to_path_buf(),
                detail: e.to_string(),
            })?;

        Ok(Self::from_value(raw))
    }

    /// Convert any of the three JSON shapes into the unified schema.
    #[must_use]
    pub fn from_value(raw: serde_json::Value) -> Self {
        // Unified schema
        if raw.get("envelope").is_some() && raw.get("validations").is_some() {
            return serde_json::from_value(raw).unwrap_or_else(|_| Self {
                envelope: serde_json::json!({}),
                validations: Vec::new(),
            });
        }

        // Legacy lithoSpore: bare array
        if raw.is_array() {
            let entries: Vec<ValidationEntry> = serde_json::from_value(raw).unwrap_or_default();
            return Self {
                envelope: serde_json::json!({}),
                validations: entries,
            };
        }

        // Legacy hotSpring: {"liveSpore": {...}, "software": {...}, ...}
        if raw.get("liveSpore").is_some() {
            let mut envelope = raw
                .get("liveSpore")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            if let Some(chain) = raw.get("provenance_chain") {
                envelope["provenance_chain"] = chain.clone();
            }
            if let Some(sw) = raw.get("software") {
                envelope["software"] = sw.clone();
            }
            return Self {
                envelope,
                validations: Vec::new(),
            };
        }

        // Unknown shape — treat whole thing as envelope
        Self {
            envelope: raw,
            validations: Vec::new(),
        }
    }

    /// Append a validation entry.
    pub fn append_validation(&mut self, entry: ValidationEntry) {
        self.validations.push(entry);
    }

    /// Save to a file in the unified format.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or writing fails.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize liveSpore: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("Failed to write {}: {e}", path.display()))
    }
}

/// Create a standard validation entry from the current environment.
#[must_use]
pub fn make_validation_entry(
    tier_reached: u8,
    modules_passed: u32,
    modules_total: u32,
    runtime_ms: u64,
) -> ValidationEntry {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "unknown".into());
    let hostname_hash = blake3::hash(hostname.as_bytes()).to_hex().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    ValidationEntry {
        timestamp,
        hostname_hash,
        arch: std::env::consts::ARCH.to_string(),
        os: std::env::consts::OS.to_string(),
        tier_reached,
        modules_passed,
        modules_total,
        runtime_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_roundtrip() {
        let doc = LiveSporeDoc::new(serde_json::json!({
            "artifact": "test",
            "version": "1.0.0"
        }));
        let json = serde_json::to_string(&doc).unwrap();
        let parsed: LiveSporeDoc = serde_json::from_str(&json).unwrap();
        assert!(parsed.validations.is_empty());
        assert_eq!(parsed.envelope["artifact"], "test");
    }

    #[test]
    fn legacy_array_migration() {
        let raw = serde_json::json!([
            {"timestamp": "2026-01-01T00:00:00Z", "hostname_hash": "abc", "tier_reached": 2}
        ]);
        let doc = LiveSporeDoc::from_value(raw);
        assert_eq!(doc.validations.len(), 1);
        assert_eq!(doc.validations[0].tier_reached, 2);
    }

    #[test]
    fn legacy_hotspring_migration() {
        let raw = serde_json::json!({
            "liveSpore": {"artifact": "test", "version": "1.0"},
            "software": {"gromacs": "2026.0"},
            "provenance_chain": {"parent": "v0.9"}
        });
        let doc = LiveSporeDoc::from_value(raw);
        assert_eq!(doc.envelope["artifact"], "test");
        assert_eq!(doc.envelope["software"]["gromacs"], "2026.0");
        assert_eq!(doc.envelope["provenance_chain"]["parent"], "v0.9");
        assert!(doc.validations.is_empty());
    }

    #[test]
    fn unified_schema_load_and_save_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("liveSpore.json");
        let mut doc = LiveSporeDoc::new(serde_json::json!({"artifact": "roundtrip"}));
        doc.append_validation(ValidationEntry {
            timestamp: "2026-05-27T00:00:00Z".to_string(),
            hostname_hash: "deadbeef".to_string(),
            arch: "x86_64".to_string(),
            os: "linux".to_string(),
            tier_reached: 2,
            modules_passed: 3,
            modules_total: 5,
            runtime_ms: 42,
        });
        doc.save(&path).expect("save liveSpore");
        let loaded = LiveSporeDoc::load(&path).expect("load liveSpore");
        assert_eq!(loaded.envelope["artifact"], "roundtrip");
        assert_eq!(loaded.validations.len(), 1);
        assert_eq!(loaded.validations[0].tier_reached, 2);
    }

    #[test]
    fn legacy_array_load_from_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("liveSpore.json");
        std::fs::write(
            &path,
            r#"[{"timestamp":"2026-01-01T00:00:00Z","hostname_hash":"abc","tier_reached":1}]"#,
        )
        .expect("write legacy array");
        let doc = LiveSporeDoc::load(&path).expect("load legacy array");
        assert_eq!(doc.validations.len(), 1);
        assert!(doc.envelope.as_object().unwrap().is_empty());
    }

    #[test]
    fn unknown_shape_treats_whole_value_as_envelope() {
        let raw = serde_json::json!({"custom_field": 42, "nested": {"a": 1}});
        let doc = LiveSporeDoc::from_value(raw);
        assert_eq!(doc.envelope["custom_field"], 42);
        assert!(doc.validations.is_empty());
    }

    #[test]
    fn serde_deserialize_unified_directly() {
        let json = r#"{"envelope":{"v":1},"validations":[]}"#;
        let doc: LiveSporeDoc = serde_json::from_str(json).expect("deserialize");
        assert_eq!(doc.envelope["v"], 1);
    }
}
