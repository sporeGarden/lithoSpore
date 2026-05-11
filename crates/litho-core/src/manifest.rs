// SPDX-License-Identifier: AGPL-3.0-or-later

//! Data manifest: TOML-driven inventory of every dataset in the artifact.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataManifest {
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub source_uri: String,
    pub license: String,
    pub local_path: String,
    pub blake3: String,
    pub retrieved: String,
    pub refresh_command: String,
}

impl DataManifest {
    /// Load from a TOML file path.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let manifest: Self = toml::from_str(&contents)?;
        Ok(manifest)
    }

    /// Verify all datasets have non-empty BLAKE3 hashes.
    #[must_use]
    pub fn verify_hashes(&self) -> Vec<&Dataset> {
        self.datasets
            .iter()
            .filter(|d| d.blake3.is_empty())
            .collect()
    }
}
