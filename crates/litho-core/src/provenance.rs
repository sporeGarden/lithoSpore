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
