// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance chain: tracks data lineage for every computation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub dataset_id: String,
    pub binary_version: String,
    pub tolerance_name: String,
    pub blake3_input: String,
    pub blake3_output: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    pub entries: Vec<ProvenanceEntry>,
}

impl ProvenanceChain {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, entry: ProvenanceEntry) {
        self.entries.push(entry);
    }
}

impl Default for ProvenanceChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_append_and_default() {
        let mut chain = ProvenanceChain::default();
        assert!(chain.entries.is_empty());

        chain.append(ProvenanceEntry {
            dataset_id: "wiser_2013".into(),
            binary_version: "0.1.0".into(),
            tolerance_name: "fitness_relative_error".into(),
            blake3_input: "abc123".into(),
            blake3_output: "def456".into(),
            timestamp: "2026-05-12T00:00:00Z".into(),
        });
        assert_eq!(chain.entries.len(), 1);
        assert_eq!(chain.entries[0].dataset_id, "wiser_2013");
    }

    #[test]
    fn provenance_json_roundtrip() {
        let entry = ProvenanceEntry {
            dataset_id: "test".into(),
            binary_version: "0.1.0".into(),
            tolerance_name: "tol".into(),
            blake3_input: "aaa".into(),
            blake3_output: "bbb".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: ProvenanceEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.dataset_id, "test");
    }
}
