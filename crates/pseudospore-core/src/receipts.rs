// SPDX-License-Identifier: AGPL-3.0-or-later

//! Receipts — environment capture and BLAKE3 checksum management.
//!
//! Handles `receipts/environment.toml` parsing and `receipts/checksums.blake3`
//! read/write/verify. Checksum entries use the `<hash>  <path>` format
//! compatible with `b3sum --no-names`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Parsed `receipts/environment.toml` — hardware, software, and timing capture.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnvironmentReceipt {
    /// CPU, memory, and other host facts at emit/validate time.
    #[serde(default)]
    pub hardware: Option<BTreeMap<String, toml::Value>>,
    /// Tool versions (GROMACS, plumed, Rust toolchain, etc.).
    #[serde(default)]
    pub software: Option<BTreeMap<String, toml::Value>>,
    /// Key timestamps (emit, pack, validate) for audit trails.
    #[serde(default)]
    pub timestamps: Option<BTreeMap<String, toml::Value>>,
}

impl EnvironmentReceipt {
    /// Load from a `receipts/environment.toml` file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse environment.toml: {e}"))
    }
}

/// A single `<blake3_hash>  <relative_path>` entry from `checksums.blake3`.
#[derive(Debug, Clone)]
pub struct ChecksumEntry {
    /// BLAKE3 hex digest of the file contents.
    pub hash: String,
    /// Path relative to the pseudoSpore root.
    pub path: String,
}

/// Parse `checksums.blake3` file content into entries.
#[must_use]
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

/// Compute BLAKE3 checksums for all files under given directories, relative to root.
#[must_use]
pub fn compute_checksums(root: &Path, dirs: &[&str]) -> Vec<ChecksumEntry> {
    let mut entries = Vec::new();
    for dir_name in dirs {
        let dir = root.join(dir_name);
        if !dir.exists() {
            continue;
        }
        for file_path in walk_dir(&dir) {
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
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries
}

/// Format checksum entries into `checksums.blake3` file content.
#[must_use]
pub fn format_checksums(entries: &[ChecksumEntry]) -> String {
    entries
        .iter()
        .map(|e| format!("{}  {}", e.hash, e.path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn walk_dir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                files.extend(walk_dir(&path));
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
            ChecksumEntry {
                hash: "aaa".to_string(),
                path: "a.txt".to_string(),
            },
            ChecksumEntry {
                hash: "bbb".to_string(),
                path: "b.txt".to_string(),
            },
        ];
        let formatted = format_checksums(&entries);
        let reparsed = parse_checksums(&formatted);
        assert_eq!(reparsed.len(), 2);
        assert_eq!(reparsed[0].hash, "aaa");
        assert_eq!(reparsed[1].path, "b.txt");
    }

    #[test]
    fn parse_checksums_empty_input() {
        assert!(parse_checksums("").is_empty(), "empty string → no entries");
        assert!(
            parse_checksums("\n\n").is_empty(),
            "blank lines only → no entries"
        );
    }

    #[test]
    fn parse_checksums_malformed_lines_skipped() {
        let input = "noseparator\nonlyhash\n  \nvalidhash  valid/path\n";
        let entries = parse_checksums(input);
        assert_eq!(entries.len(), 1, "only well-formed line parses");
        assert_eq!(entries[0].path, "valid/path");
    }

    #[test]
    fn parse_checksums_empty_hash_or_path_skipped() {
        let input = "  still/path\nhashonly\n  trailing/path\n";
        let entries = parse_checksums(input);
        assert!(entries.is_empty(), "malformed entries must be skipped");
    }

    #[test]
    fn compute_checksums_known_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        let outputs = root.join("outputs");
        std::fs::create_dir_all(&outputs).expect("outputs dir");
        std::fs::write(outputs.join("alpha.dat"), b"alpha-content").expect("alpha");
        std::fs::write(outputs.join("beta.dat"), b"beta-content").expect("beta");

        let entries = compute_checksums(root, &["outputs"]);
        assert_eq!(entries.len(), 2, "two files under outputs/");
        assert!(
            entries.iter().any(|e| e.path.ends_with("alpha.dat")),
            "alpha.dat present"
        );
        let alpha_hash = blake3::hash(b"alpha-content").to_hex().to_string();
        let alpha = entries
            .iter()
            .find(|e| e.path.ends_with("alpha.dat"))
            .expect("alpha entry");
        assert_eq!(alpha.hash, alpha_hash, "hash matches file content");
    }

    #[test]
    fn environment_receipt_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("environment.toml");
        std::fs::write(
            &path,
            r#"
[hardware]
cpu = "test-cpu"

[software]
gromacs = "2026.0"
"#,
        )
        .expect("write environment");
        let receipt = EnvironmentReceipt::load(&path).expect("load environment");
        assert!(receipt.hardware.is_some());
        assert!(receipt.software.is_some());
    }
}
