// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho fetch` — download and verify datasets from data.toml source URIs.
//!
//! Pure Rust replacement for 7 `scripts/fetch_*.sh` bash scripts.
//! Strategy per dataset: try HTTP download → fall back to expected-JSON
//! synthesis → compute BLAKE3 hash → report.
//!
//! Spring validation fallback: set `LITHO_SPRINGS_ROOT` to the directory that
//! contains per-spring folders (e.g. `groundSpring/validation/`). When unset,
//! defaults to `{workspace}/../springs` (two parents above the artifact root).

use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize)]
struct DataManifest {
    dataset: Option<Vec<DatasetEntry>>,
}

#[derive(Debug, serde::Deserialize)]
struct DatasetEntry {
    id: String,
    #[serde(default)]
    source_uri: String,
    #[serde(default)]
    local_path: String,
    #[serde(default)]
    module: String,
    #[serde(default)]
    blake3: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    sra_accession: String,
    #[serde(default)]
    data_tier: String,
    #[serde(default)]
    full_data_size: String,
    #[serde(default)]
    full_data_tool: String,
    #[serde(default)]
    full_data_checks: String,
}

pub fn run(root: &str, dataset_filter: Option<&str>, all: bool, full: bool) {
    let root_path = Path::new(root);
    let data_toml = root_path.join("artifact/data.toml");

    let content = match std::fs::read_to_string(&data_toml) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Cannot read {}: {e}", data_toml.display());
            std::process::exit(1);
        }
    };

    let manifest: DataManifest = match toml::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("ERROR: Failed to parse data.toml: {e}");
            std::process::exit(1);
        }
    };

    let datasets = manifest.dataset.unwrap_or_default();
    if datasets.is_empty() {
        println!("No [[dataset]] entries in data.toml");
        return;
    }

    if full {
        eprintln!("=== Full data mode: pulling upstream datasets ===");
        eprintln!("  This may download 10s–100s of GB of sequencing data.");
        eprintln!();
    }

    let mut fetched = 0u32;
    let mut failed = 0u32;

    for ds in &datasets {
        if let Some(filter) = dataset_filter {
            if ds.id != filter && ds.module != filter {
                continue;
            }
        } else if !all {
            eprintln!("Use --all to fetch all datasets, or --dataset <ID> for one");
            std::process::exit(1);
        }

        if matches!(ds.status.as_str(), "internal" | "derived") && dataset_filter.is_none() && !full
        {
            println!(
                "[{id}] SKIP ({status}): data is generated from expected values, not fetched",
                id = ds.id,
                status = ds.status
            );
            fetched += 1;
            continue;
        }

        // In full mode, report the data tier and what we're pulling
        if full {
            let tier = if ds.data_tier.is_empty() {
                "unknown"
            } else {
                &ds.data_tier
            };
            println!("[{id}] Data tier: {tier}", id = ds.id);
            if !ds.full_data_size.is_empty() {
                println!(
                    "[{id}]   Full data: {} (via {})",
                    ds.full_data_size,
                    if ds.full_data_tool.is_empty() {
                        "fetch"
                    } else {
                        &ds.full_data_tool
                    },
                    id = ds.id
                );
            }
            if !ds.full_data_checks.is_empty() {
                println!(
                    "[{id}]   Additional checks: {}",
                    ds.full_data_checks,
                    id = ds.id
                );
            }
            if ds.data_tier == "complete" {
                println!(
                    "[{id}]   Already complete — no additional download needed",
                    id = ds.id
                );
                fetched += 1;
                continue;
            }
        }

        let target_dir = root_path.join(&ds.local_path);
        std::fs::create_dir_all(&target_dir).ok();

        println!(
            "[{id}] Fetching: {uri}",
            id = ds.id,
            uri = if ds.source_uri.is_empty() {
                "(generated)"
            } else {
                &ds.source_uri
            }
        );

        match fetch_dataset(root_path, ds, full) {
            Ok(hash) => {
                println!("[{id}]   BLAKE3: {hash}", id = ds.id);
                if !ds.blake3.is_empty() && ds.blake3 != hash {
                    eprintln!(
                        "[{id}]   WARN: hash mismatch (expected: {})",
                        ds.blake3,
                        id = ds.id
                    );
                }
                fetched += 1;
            }
            Err(e) => {
                eprintln!("[{id}]   FAILED: {e}", id = ds.id);
                failed += 1;
            }
        }
    }

    if dataset_filter.is_none() || all {
        let total = fetched + failed;
        println!();
        println!("Fetch complete: {fetched}/{total} fetched, {failed} failed");
    }

    if failed > 0 {
        std::process::exit(1);
    }
}

fn fetch_dataset(root: &Path, ds: &DatasetEntry, full: bool) -> Result<String, String> {
    let target_dir = root.join(&ds.local_path);

    // In full mode, prioritize SRA download for genomic datasets
    if full && !ds.sra_accession.is_empty() {
        match try_sra_download(&ds.sra_accession, &target_dir, &ds.id) {
            Ok(()) => return hash_directory(&target_dir),
            Err(e) => eprintln!(
                "[{}]   SRA download failed: {e} — trying other strategies",
                ds.id
            ),
        }
    }

    // Strategy 1: SRA toolkit for genomic datasets with accession numbers (non-full mode skips BioProject)
    if !full && !ds.sra_accession.is_empty() {
        match try_sra_download(&ds.sra_accession, &target_dir, &ds.id) {
            Ok(()) => return hash_directory(&target_dir),
            Err(e) => eprintln!("[{}]   SRA download skipped: {e}", ds.id),
        }
    }

    // Strategy 2: try HTTP download if source URI is present and not a BioProject landing page
    if !ds.source_uri.is_empty() && ds.source_uri.starts_with("http") {
        if !full && is_landing_page_uri(&ds.source_uri) {
            eprintln!(
                "[{}]   URI is a landing page — skipping HTTP download",
                ds.id
            );
            eprintln!(
                "[{}]   Use --full to pull via SRA toolkit: prefetch {} && fastq-dump",
                ds.id,
                if ds.sra_accession.is_empty() {
                    "<accession>"
                } else {
                    &ds.sra_accession
                }
            );
        } else {
            match try_http_download(&ds.source_uri, &target_dir, &ds.id) {
                Ok(()) => return hash_directory(&target_dir),
                Err(e) => eprintln!("[{}]   HTTP download failed: {e} — trying fallback", ds.id),
            }
        }
    }

    // Strategy 3: generate from expected JSON
    let expected_dir = root.join("validation/expected");
    match generate_from_expected(root, &target_dir, &ds.module, &expected_dir) {
        Ok(()) => return hash_directory(&target_dir),
        Err(e) => eprintln!("[{}]   Fallback generation failed: {e}", ds.id),
    }

    // Strategy 4: check if data already exists
    if target_dir.exists() && std::fs::read_dir(&target_dir).is_ok_and(|mut d| d.next().is_some()) {
        eprintln!(
            "[{}]   Using existing data in {}",
            ds.id,
            target_dir.display()
        );
        return hash_directory(&target_dir);
    }

    Err("No download source, no expected values, and no existing data".to_string())
}

fn is_landing_page_uri(uri: &str) -> bool {
    uri.contains("ncbi.nlm.nih.gov/bioproject")
        || uri.contains("ncbi.nlm.nih.gov/sra")
        || uri.contains("ncbi.nlm.nih.gov/Traces")
        || uri.contains("datadryad.org/stash/dataset")
        || uri.contains("datadryad.org/dataset")
}

fn try_sra_download(accession: &str, target_dir: &Path, id: &str) -> Result<(), String> {
    let has_prefetch = std::process::Command::new("which")
        .arg("prefetch")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success());

    if !has_prefetch {
        return Err("SRA toolkit not installed (prefetch not found). \
                    Install: https://github.com/ncbi/sra-tools/wiki"
            .to_string());
    }

    eprintln!("[{id}]   Fetching via SRA toolkit: prefetch {accession}");
    let status = std::process::Command::new("prefetch")
        .arg(accession)
        .arg("--output-directory")
        .arg(target_dir)
        .status()
        .map_err(|e| format!("prefetch failed: {e}"))?;

    if !status.success() {
        return Err(format!("prefetch exited with {status}"));
    }

    eprintln!("[{id}]   SRA prefetch complete for {accession}");
    Ok(())
}

fn try_http_download(uri: &str, target_dir: &Path, id: &str) -> Result<(), String> {
    eprintln!("[{id}]   Downloading from {uri}...");

    let uri_path = PathBuf::from(uri);
    let uri_lower = uri.to_ascii_lowercase();
    let ext_matches = |ext: &str| {
        uri_path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
    };
    let (suffix, label) = if uri_lower.ends_with(".tar.gz") || ext_matches("tgz") {
        (".tar.gz", "tarball")
    } else if ext_matches("zip") {
        ("_raw.zip", "archive")
    } else if ext_matches("json") {
        (".json", "JSON")
    } else if ext_matches("csv") {
        (".csv", "CSV")
    } else {
        ("_data", "data")
    };

    let dest = target_dir.join(format!("{id}{suffix}"));
    let output = std::process::Command::new("curl")
        .args([
            "-fSL",
            "--max-time",
            "300",
            "-A",
            &format!(
                "lithoSpore/{} (science-validation)",
                env!("CARGO_PKG_VERSION")
            ),
            "-o",
            &dest.to_string_lossy(),
            uri,
        ])
        .output()
        .map_err(|e| format!("curl not found or failed to start: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl failed (exit {}): {stderr}", output.status));
    }

    let body = std::fs::read(&dest).map_err(|e| format!("read downloaded file: {e}"))?;

    if body.len() < 256 {
        let preview = String::from_utf8_lossy(&body[..body.len().min(128)]);
        if preview.contains("<html") || preview.contains("<!DOCTYPE") || preview.contains("<HTML") {
            std::fs::remove_file(&dest).ok();
            return Err("Response body appears to be HTML. \
                 Likely a redirect to a login/landing page."
                .to_string());
        }
    }

    eprintln!(
        "[{id}]   Saved {:.1} KB {label}",
        body.len() as f64 / 1024.0
    );

    if suffix == ".tar.gz" {
        unpack_tar_gz(&dest, target_dir, id);
    } else if suffix == "_raw.zip" {
        unpack_zip(&dest, target_dir, id);
    }

    Ok(())
}

fn unpack_tar_gz(archive: &Path, target_dir: &Path, id: &str) {
    let file = match std::fs::File::open(archive) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[{id}]   Cannot open archive: {e}");
            return;
        }
    };
    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);
    match tar.unpack(target_dir) {
        Ok(()) => eprintln!("[{id}]   Unpacked tar.gz to {}", target_dir.display()),
        Err(e) => eprintln!("[{id}]   Unpack failed: {e}"),
    }
}

fn unpack_zip(archive: &Path, target_dir: &Path, id: &str) {
    let file = match std::fs::File::open(archive) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[{id}]   Cannot open archive: {e}");
            return;
        }
    };
    let mut zip = match zip::ZipArchive::new(file) {
        Ok(z) => z,
        Err(e) => {
            eprintln!("[{id}]   Invalid zip: {e}");
            return;
        }
    };
    match zip.extract(target_dir) {
        Ok(()) => eprintln!(
            "[{id}]   Unpacked zip ({} entries) to {}",
            zip.len(),
            target_dir.display()
        ),
        Err(e) => eprintln!("[{id}]   Zip extraction failed: {e}"),
    }
}

fn generate_from_expected(
    root: &Path,
    target_dir: &Path,
    module: &str,
    expected_dir: &Path,
) -> Result<(), String> {
    let expected_path = find_expected_for_module(expected_dir, module)
        .ok_or_else(|| format!("No expected JSON found for module: {module}"))?;

    if !expected_path.exists() {
        if let Some(filename) = expected_path.file_name() {
            let Some(spring_name) = spring_for_module(root, module) else {
                eprintln!(
                    "  WARN: no [[spring]] mapping for {module} in scope.toml; skipping spring-side fetch"
                );
                return Err(format!("No expected JSON found for module: {module}"));
            };
            let spring_expected = springs_root(root)
                .join(spring_name)
                .join("validation")
                .join(filename);

            if spring_expected.exists() {
                std::fs::copy(&spring_expected, target_dir.join("expected_values.json"))
                    .map_err(|e| format!("copy from spring: {e}"))?;
                return Ok(());
            }
        }

        return Err(format!(
            "Expected values not found: {}",
            expected_path.display()
        ));
    }

    let content = std::fs::read_to_string(&expected_path).map_err(|e| format!("read: {e}"))?;

    let expected: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("parse: {e}"))?;

    // Delegate synthesis to domain crates; generic fallback copies expected values
    match module {
        "ltee-fitness" => ltee_fitness::synthesize_from_expected(target_dir, &expected),
        "ltee-mutations" => ltee_mutations::synthesize_from_expected(target_dir, &expected),
        _ => {
            let dest = target_dir.join("expected_values.json");
            std::fs::write(&dest, &content).map_err(|e| format!("write: {e}"))?;
            Ok(())
        }
    }
}

/// Root directory containing per-spring folders (`{spring}/validation/`, etc.).
fn springs_root(artifact_root: &Path) -> PathBuf {
    if let Ok(env_root) = std::env::var(litho_core::env_vars::LITHO_SPRINGS_ROOT)
        && !env_root.is_empty()
    {
        return PathBuf::from(env_root);
    }
    artifact_root
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(|| PathBuf::from("springs"), |eco| eco.join("springs"))
}

/// Resolve which spring owns a module binary from `scope.toml` `[[spring]]` entries.
fn spring_for_module(artifact_root: &Path, module: &str) -> Option<String> {
    let scope_path = artifact_root.join("scope.toml");
    let content = std::fs::read_to_string(scope_path).ok()?;
    let scope: toml::Table = content.parse().ok()?;
    let springs = scope.get("spring")?.as_array()?;
    for spring in springs {
        let table = spring.as_table()?;
        let name = table.get("name")?.as_str()?;
        let modules = table.get("modules")?.as_array()?;
        if modules.iter().any(|m| m.as_str() == Some(module)) {
            return Some(name.to_string());
        }
    }
    None
}

/// Find the expected JSON file for a module by scanning the expected directory
/// for any `.json` file whose name contains the module's key (e.g., "`ltee_fitness`"
/// matches "`module1_fitness.json`"). Domain-agnostic: no hardcoded module list.
fn find_expected_for_module(expected_dir: &Path, module: &str) -> Option<std::path::PathBuf> {
    let suffix = module.replace('-', "_");
    let short = suffix.strip_prefix("ltee_").unwrap_or(&suffix);

    let entries = std::fs::read_dir(expected_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".json") && (name_str.contains(&suffix) || name_str.contains(short)) {
            return Some(entry.path());
        }
    }
    None
}

fn hash_directory(dir: &Path) -> Result<String, String> {
    let mut files: Vec<_> = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.file_name().to_str().is_some_and(|n| n.starts_with('.')))
        .collect();

    files.sort_by(|a, b| a.path().cmp(b.path()));

    let mut hasher = blake3::Hasher::new();
    for entry in &files {
        let content = std::fs::read(entry.path())
            .map_err(|e| format!("read {}: {e}", entry.path().display()))?;
        hasher.update(&content);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_directory_deterministic() {
        let dir = std::env::temp_dir().join("litho_fetch_test_hash");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("a.txt"), "hello").unwrap();
        std::fs::write(dir.join("b.txt"), "world").unwrap();

        let h1 = hash_directory(&dir).unwrap();
        let h2 = hash_directory(&dir).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fitness_synthesis_delegated_to_domain_crate() {
        let dir = std::env::temp_dir().join("litho_fetch_test_fitness_delegate");
        let _ = std::fs::create_dir_all(&dir);

        let expected = serde_json::json!({
            "generations": [500, 5000, 10000],
            "mean_fitness": [1.05, 1.15, 1.22],
        });

        let result = ltee_fitness::synthesize_from_expected(&dir, &expected);
        assert!(result.is_ok());
        let csv = std::fs::read_to_string(dir.join("fitness_data.csv")).unwrap();
        assert!(csv.contains("generation,mean_fitness"));
        assert!(csv.contains("500,"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
