// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tarball creation with [present]/[external] split.
//!
//! pseudoSpore tarballs include only [present] files (scope, receipts, outputs,
//! configs, figures, provenance). [external] files (large trajectories, raw data
//! that can be re-fetched) are listed in data.toml but excluded from the tarball.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Determine which files should be included in the tarball ([present]) vs
/// excluded ([external]). Returns (`present_files`, `external_files`) as relative paths.
#[must_use]
pub fn split_present_external(
    root: &Path,
    external_patterns: &[&str],
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut present = Vec::new();
    let mut external = Vec::new();

    let external_set: BTreeSet<&str> = external_patterns.iter().copied().collect();

    for path in walk_all_files(root) {
        let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let rel_str = rel.to_string_lossy();

        let is_external = external_set
            .iter()
            .any(|pattern| rel_str.starts_with(pattern) || rel_str.contains(pattern));

        if is_external {
            external.push(rel);
        } else {
            present.push(rel);
        }
    }

    present.sort();
    external.sort();
    (present, external)
}

/// Default external patterns for computational chemistry pseudoSpores.
/// Large trajectory and raw simulation data that can be re-derived.
pub const DEFAULT_EXTERNAL_PATTERNS: &[&str] = &["data/", "structures/", "topologies/"];

fn walk_all_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                files.extend(walk_all_files(&path));
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
    use std::fs;

    #[test]
    fn split_present_external_categorizes_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::create_dir_all(root.join("outputs/module_a")).expect("outputs dir");
        fs::create_dir_all(root.join("data/big")).expect("data dir");
        fs::create_dir_all(root.join("scope")).expect("scope dir");
        fs::write(root.join("outputs/module_a/result.dat"), b"out").expect("output file");
        fs::write(root.join("data/big/trajectory.xtc"), b"xtc").expect("data file");
        fs::write(root.join("scope.toml"), b"scope").expect("scope file");

        let (present, external) = split_present_external(root, DEFAULT_EXTERNAL_PATTERNS);

        let present_str: Vec<String> = present.iter().map(|p| p.to_string_lossy().into()).collect();
        let external_str: Vec<String> = external
            .iter()
            .map(|p| p.to_string_lossy().into())
            .collect();

        assert!(
            present_str.iter().any(|p| p.contains("outputs/")),
            "outputs should be present: {present_str:?}"
        );
        assert!(
            present_str.iter().any(|p| p.ends_with("scope.toml")),
            "scope.toml should be present: {present_str:?}"
        );
        assert!(
            external_str.iter().any(|p| p.contains("data/")),
            "data/ should be external: {external_str:?}"
        );
        assert!(
            !present_str.iter().any(|p| p.contains("data/")),
            "data/ must not appear in present"
        );
    }

    #[test]
    fn custom_external_pattern_matches_substring() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::create_dir_all(root.join("archive")).expect("archive dir");
        fs::write(root.join("archive/large.bin"), b"big").expect("archive file");
        fs::write(root.join("readme.txt"), b"small").expect("readme");

        let (present, external) = split_present_external(root, &["archive/"]);
        assert_eq!(external.len(), 1, "archive file should be external");
        assert_eq!(present.len(), 1, "readme should be present");
    }
}
