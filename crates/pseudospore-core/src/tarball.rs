// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tarball creation with [present]/[external] split.
//!
//! pseudoSpore tarballs include only [present] files (scope, receipts, outputs,
//! configs, figures, provenance). [external] files (large trajectories, raw data
//! that can be re-fetched) are listed in data.toml but excluded from the tarball.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Determine which files should be included in the tarball ([present]) vs
/// excluded ([external]). Returns (present_files, external_files) as relative paths.
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

        let is_external = external_set.iter().any(|pattern| {
            rel_str.starts_with(pattern) || rel_str.contains(pattern)
        });

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
pub const DEFAULT_EXTERNAL_PATTERNS: &[&str] = &[
    "data/",
    "structures/",
    "topologies/",
];

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
