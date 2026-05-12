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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dataset(id: &str, hash: &str) -> Dataset {
        Dataset {
            id: id.into(),
            source_uri: "https://example.com".into(),
            license: "CC0".into(),
            local_path: format!("data/{id}/"),
            blake3: hash.into(),
            retrieved: "2026-05-12".into(),
            refresh_command: String::new(),
        }
    }

    #[test]
    fn verify_hashes_finds_empty() {
        let m = DataManifest {
            datasets: vec![
                sample_dataset("a", "abc123"),
                sample_dataset("b", ""),
                sample_dataset("c", "def456"),
            ],
        };
        let unhashed = m.verify_hashes();
        assert_eq!(unhashed.len(), 1);
        assert_eq!(unhashed[0].id, "b");
    }

    #[test]
    fn verify_hashes_all_present() {
        let m = DataManifest {
            datasets: vec![
                sample_dataset("a", "abc"),
                sample_dataset("b", "def"),
            ],
        };
        assert!(m.verify_hashes().is_empty());
    }
}
