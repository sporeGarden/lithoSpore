// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tarball creation and extraction with present/external split.
//!
//! pseudoSpore tarballs include only present files (scope, receipts, outputs,
//! configs, figures, provenance). External files (large trajectories, raw data
//! that can be re-fetched) are listed in `data.toml` but excluded from the tarball.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::SporeError;

/// Determine which files should be included in the tarball (present) vs
/// excluded (external). Returns (present\_files, external\_files) as relative paths.
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

/// Create a `.tar.gz` archive from a pseudoSpore directory.
///
/// Only present files (per `external_patterns`) are included in the archive.
/// The archive uses the pseudoSpore directory name as the top-level prefix so
/// extraction produces a single named directory.
///
/// Returns the BLAKE3 hash of the tarball.
///
/// # Errors
///
/// Returns an error if the directory cannot be read, the output file cannot be
/// created, or any file cannot be added to the archive.
pub fn create_tarball(
    root: &Path,
    output: &Path,
    external_patterns: &[&str],
) -> Result<String, SporeError> {
    let (present, _external) = split_present_external(root, external_patterns);

    let dir_name = root.file_name().map_or_else(
        || "pseudospore".to_string(),
        |n| n.to_string_lossy().into_owned(),
    );

    let file = std::fs::File::create(output).map_err(|e| SporeError::Io {
        path: output.to_path_buf(),
        source: e,
    })?;
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut archive = tar::Builder::new(encoder);

    for rel_path in &present {
        let full_path = root.join(rel_path);
        let archive_path = Path::new(&dir_name).join(rel_path);
        archive
            .append_path_with_name(&full_path, &archive_path)
            .map_err(|e| SporeError::Io {
                path: full_path.clone(),
                source: e,
            })?;
    }

    let encoder = archive.into_inner().map_err(|e| SporeError::Io {
        path: output.to_path_buf(),
        source: e,
    })?;
    encoder.finish().map_err(|e| SporeError::Io {
        path: output.to_path_buf(),
        source: e,
    })?;

    let tarball_bytes = std::fs::read(output).map_err(|e| SporeError::Io {
        path: output.to_path_buf(),
        source: e,
    })?;
    let hash = blake3::hash(&tarball_bytes).to_hex().to_string();

    Ok(hash)
}

/// Extract a `.tar.gz` pseudoSpore tarball to a target directory.
///
/// Returns the path to the extracted pseudoSpore root directory (the single
/// top-level directory inside the archive).
///
/// # Errors
///
/// Returns an error if the tarball cannot be read, is malformed, or extraction
/// fails.
pub fn extract_tarball(tarball: &Path, target: &Path) -> Result<PathBuf, SporeError> {
    let file = std::fs::File::open(tarball).map_err(|e| SporeError::Io {
        path: tarball.to_path_buf(),
        source: e,
    })?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    std::fs::create_dir_all(target).map_err(|e| SporeError::Io {
        path: target.to_path_buf(),
        source: e,
    })?;

    let mut top_dir: Option<PathBuf> = None;

    for entry_result in archive.entries().map_err(|e| SporeError::Io {
        path: tarball.to_path_buf(),
        source: e,
    })? {
        let mut entry = entry_result.map_err(|e| SporeError::Io {
            path: tarball.to_path_buf(),
            source: e,
        })?;

        let entry_path = entry
            .path()
            .map_err(|e| SporeError::Io {
                path: tarball.to_path_buf(),
                source: e,
            })?
            .into_owned();

        if top_dir.is_none()
            && let Some(first_component) = entry_path.components().next()
        {
            top_dir = Some(PathBuf::from(first_component.as_os_str()));
        }

        entry.unpack_in(target).map_err(|e| SporeError::Io {
            path: target.join(&entry_path),
            source: e,
        })?;
    }

    let root = top_dir.map_or_else(|| target.to_path_buf(), |d| target.join(d));

    Ok(root)
}

/// Write a present/external BLAKE3 manifest to the given writer.
///
/// Computes BLAKE3 hashes for all present and external files relative to `root`.
///
/// # Errors
///
/// Returns an error if any file cannot be read.
pub fn write_integrity_manifest(
    root: &Path,
    present: &[PathBuf],
    external: &[PathBuf],
    writer: &mut dyn Write,
) -> Result<(), SporeError> {
    writeln!(writer, "[present]").map_err(|e| SporeError::Other(e.to_string()))?;
    for rel in present {
        let full = root.join(rel);
        if full.is_file() {
            let data = std::fs::read(&full).map_err(|e| SporeError::Io {
                path: full.clone(),
                source: e,
            })?;
            let hash = blake3::hash(&data).to_hex().to_string();
            writeln!(writer, "\"{}\" = \"{hash}\"", rel.display())
                .map_err(|e| SporeError::Other(e.to_string()))?;
        }
    }

    if !external.is_empty() {
        writeln!(writer, "\n[external]").map_err(|e| SporeError::Other(e.to_string()))?;
        for rel in external {
            let full = root.join(rel);
            if full.is_file() {
                let data = std::fs::read(&full).map_err(|e| SporeError::Io {
                    path: full.clone(),
                    source: e,
                })?;
                let hash = blake3::hash(&data).to_hex().to_string();
                writeln!(writer, "\"{}\" = \"{hash}\"", rel.display())
                    .map_err(|e| SporeError::Other(e.to_string()))?;
            }
        }
    }

    Ok(())
}

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

    #[test]
    fn create_and_extract_tarball_round_trip() {
        let src = tempfile::tempdir().expect("source dir");
        let root = src.path().join("test-spore_v1.0.0");
        fs::create_dir_all(root.join("outputs")).expect("outputs dir");
        fs::create_dir_all(root.join("receipts")).expect("receipts dir");
        fs::create_dir_all(root.join("data/raw")).expect("data dir");

        let scope = "[artifact]\nname = \"test\"\nversion = \"1.0.0\"\n";
        fs::write(root.join("scope.toml"), scope).expect("scope");
        fs::write(root.join("outputs/result.csv"), b"x,y\n1,2\n").expect("result");
        fs::write(
            root.join("receipts/environment.toml"),
            b"[emit_host]\nos = \"linux\"\n",
        )
        .expect("env");
        fs::write(root.join("data/raw/trajectory.xtc"), b"big-data-here").expect("trajectory");

        let out_dir = tempfile::tempdir().expect("output dir");
        let tarball_path = out_dir.path().join("test-spore.tar.gz");
        let hash = create_tarball(&root, &tarball_path, DEFAULT_EXTERNAL_PATTERNS).expect("create");
        assert!(!hash.is_empty(), "hash should be non-empty");
        assert!(tarball_path.exists(), "tarball should exist");

        let extract_dir = tempfile::tempdir().expect("extract dir");
        let extracted = extract_tarball(&tarball_path, extract_dir.path()).expect("extract");

        assert!(extracted.join("scope.toml").exists(), "scope extracted");
        assert!(
            extracted.join("outputs/result.csv").exists(),
            "outputs extracted"
        );
        assert!(
            extracted.join("receipts/environment.toml").exists(),
            "receipts extracted"
        );
        assert!(
            !extracted.join("data/raw/trajectory.xtc").exists(),
            "external data excluded from tarball"
        );
    }

    #[test]
    fn write_integrity_manifest_produces_valid_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::write(root.join("scope.toml"), b"[artifact]\nname = \"x\"\n").expect("scope");
        fs::create_dir_all(root.join("data")).expect("data");
        fs::write(root.join("data/big.dat"), b"external-data").expect("external");

        let present = vec![PathBuf::from("scope.toml")];
        let external = vec![PathBuf::from("data/big.dat")];
        let mut buf = Vec::new();
        write_integrity_manifest(root, &present, &external, &mut buf).expect("write manifest");

        let content = String::from_utf8(buf).expect("utf8");
        assert!(content.contains("[present]"), "has present section");
        assert!(content.contains("[external]"), "has external section");
        assert!(
            content.contains("scope.toml"),
            "present file listed: {content}"
        );
        assert!(
            content.contains("data/big.dat"),
            "external file listed: {content}"
        );

        let parsed: toml::Table = content.parse().expect("valid TOML");
        assert!(parsed.contains_key("present"), "present key");
        assert!(parsed.contains_key("external"), "external key");
    }
}
