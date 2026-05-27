// SPDX-License-Identifier: AGPL-3.0-or-later

//! BLAKE3 manifest (data.toml) read/write/verify.
//!
//! The data.toml manifest tracks files in two sections:
//! - `[present]`: files included in the tarball (checksummed on pack)
//! - `[external]`: files too large for tarball, verified when available

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A parsed data.toml manifest.
#[derive(Debug, Clone, Default)]
pub struct Blake3Manifest {
    pub present: BTreeMap<String, String>,
    pub external: BTreeMap<String, String>,
}

impl Blake3Manifest {
    /// Load from a data.toml file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        Self::parse(&content)
    }

    /// Parse data.toml content.
    pub fn parse(content: &str) -> Result<Self, String> {
        let table: toml::Table = content
            .parse()
            .map_err(|e| format!("Failed to parse data.toml: {e}"))?;

        let present = extract_section(&table, "present");
        let external = extract_section(&table, "external");

        Ok(Self { present, external })
    }

    /// Verify all `[present]` entries against actual files relative to root.
    pub fn verify_present(&self, root: &Path) -> Vec<ManifestError> {
        let mut errors = Vec::new();
        for (rel_path, expected_hash) in &self.present {
            let file_path = root.join(rel_path);
            match std::fs::read(&file_path) {
                Ok(data) => {
                    let actual = blake3::hash(&data).to_hex().to_string();
                    if actual != *expected_hash {
                        errors.push(ManifestError::HashMismatch {
                            path: rel_path.clone(),
                            expected: expected_hash.clone(),
                            actual,
                        });
                    }
                }
                Err(_) => {
                    errors.push(ManifestError::Missing(rel_path.clone()));
                }
            }
        }
        errors
    }

    /// Compute BLAKE3 hashes for all files under given directories, relative to root.
    pub fn compute(root: &Path, dirs: &[&str]) -> BTreeMap<String, String> {
        let mut hashes = BTreeMap::new();
        for dir_name in dirs {
            let dir = root.join(dir_name);
            if !dir.exists() {
                continue;
            }
            for path in walk_files(&dir) {
                if let Ok(data) = std::fs::read(&path) {
                    let hash = blake3::hash(&data).to_hex().to_string();
                    let rel = path
                        .strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    hashes.insert(rel, hash);
                }
            }
        }
        hashes
    }

    /// Serialize to data.toml format with [present] and [external] sections.
    pub fn to_toml(&self) -> String {
        let mut output = String::new();
        if !self.present.is_empty() {
            output.push_str("[present]\n");
            for (path, hash) in &self.present {
                output.push_str(&format!("\"{}\" = \"{}\"\n", path, hash));
            }
        }
        if !self.external.is_empty() {
            output.push_str("\n[external]\n");
            for (path, hash) in &self.external {
                output.push_str(&format!("\"{}\" = \"{}\"\n", path, hash));
            }
        }
        output
    }
}

#[derive(Debug, Clone)]
pub enum ManifestError {
    Missing(String),
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },
}

fn extract_section(table: &toml::Table, key: &str) -> BTreeMap<String, String> {
    table
        .get(key)
        .and_then(|v| v.as_table())
        .map(|t| {
            t.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                files.extend(walk_files(&path));
            } else {
                files.push(path);
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest() {
        let content = r#"
[present]
"scope.toml" = "abc123"
"README.md" = "def456"

[external]
"data/big_trajectory.xtc" = "789012"
"#;
        let m = Blake3Manifest::parse(content).unwrap();
        assert_eq!(m.present.len(), 2);
        assert_eq!(m.external.len(), 1);
        assert_eq!(m.present["scope.toml"], "abc123");
    }

    #[test]
    fn roundtrip_toml() {
        let mut m = Blake3Manifest::default();
        m.present.insert("a.txt".to_string(), "hash_a".to_string());
        m.external
            .insert("b.xtc".to_string(), "hash_b".to_string());
        let serialized = m.to_toml();
        let parsed = Blake3Manifest::parse(&serialized).unwrap();
        assert_eq!(parsed.present["a.txt"], "hash_a");
        assert_eq!(parsed.external["b.xtc"], "hash_b");
    }
}
