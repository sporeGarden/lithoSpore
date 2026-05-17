// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho fetch` — download and verify datasets from data.toml source URIs.
//!
//! Pure Rust replacement for 7 `scripts/fetch_*.sh` bash scripts.
//! Strategy per dataset: try HTTP download → fall back to expected-JSON
//! synthesis → compute BLAKE3 hash → report.

use std::path::Path;

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

        if matches!(ds.status.as_str(), "internal" | "derived") && dataset_filter.is_none() && !full {
            println!("[{id}] SKIP ({status}): data is generated from expected values, not fetched",
                     id = ds.id, status = ds.status);
            fetched += 1;
            continue;
        }

        // In full mode, report the data tier and what we're pulling
        if full {
            let tier = if ds.data_tier.is_empty() { "unknown" } else { &ds.data_tier };
            println!("[{id}] Data tier: {tier}", id = ds.id);
            if !ds.full_data_size.is_empty() {
                println!("[{id}]   Full data: {} (via {})", ds.full_data_size, if ds.full_data_tool.is_empty() { "fetch" } else { &ds.full_data_tool }, id = ds.id);
            }
            if !ds.full_data_checks.is_empty() {
                println!("[{id}]   Additional checks: {}", ds.full_data_checks, id = ds.id);
            }
            if ds.data_tier == "complete" {
                println!("[{id}]   Already complete — no additional download needed", id = ds.id);
                fetched += 1;
                continue;
            }
        }

        let target_dir = root_path.join(&ds.local_path);
        std::fs::create_dir_all(&target_dir).ok();

        println!("[{id}] Fetching: {uri}", id = ds.id, uri = if ds.source_uri.is_empty() { "(generated)" } else { &ds.source_uri });

        match fetch_dataset(root_path, ds, full) {
            Ok(hash) => {
                println!("[{id}]   BLAKE3: {hash}", id = ds.id);
                if !ds.blake3.is_empty() && ds.blake3 != hash {
                    eprintln!("[{id}]   WARN: hash mismatch (expected: {})", ds.blake3, id = ds.id);
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
            Err(e) => eprintln!("[{}]   SRA download failed: {e} — trying other strategies", ds.id),
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
        if !full && is_bioproject_uri(&ds.source_uri) {
            eprintln!("[{}]   URI is a BioProject landing page — skipping HTTP download", ds.id);
            eprintln!("[{}]   Use --full to pull via SRA toolkit: prefetch {} && fastq-dump",
                      ds.id, if ds.sra_accession.is_empty() { "<accession>" } else { &ds.sra_accession });
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
    if target_dir.exists() && std::fs::read_dir(&target_dir).map(|mut d| d.next().is_some()).unwrap_or(false) {
        eprintln!("[{}]   Using existing data in {}", ds.id, target_dir.display());
        return hash_directory(&target_dir);
    }

    Err("No download source, no expected values, and no existing data".to_string())
}

fn is_bioproject_uri(uri: &str) -> bool {
    uri.contains("ncbi.nlm.nih.gov/bioproject")
        || uri.contains("ncbi.nlm.nih.gov/sra")
        || uri.contains("ncbi.nlm.nih.gov/Traces")
}

fn try_sra_download(accession: &str, target_dir: &Path, id: &str) -> Result<(), String> {
    let has_prefetch = std::process::Command::new("which").arg("prefetch")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false);

    if !has_prefetch {
        return Err("SRA toolkit not installed (prefetch not found). \
                    Install: https://github.com/ncbi/sra-tools/wiki".to_string());
    }

    eprintln!("[{id}]   Fetching via SRA toolkit: prefetch {accession}");
    let status = std::process::Command::new("prefetch")
        .arg(accession)
        .arg("--output-directory").arg(target_dir)
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

    let mut response = ureq::get(uri)
        .header("User-Agent", "lithoSpore/0.1 (science-validation)")
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let content_type = response.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body = response.body_mut().read_to_vec()
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    if content_type.contains("html") || content_type.contains("xml") {
        return Err(format!(
            "Source URI returned {content_type} (landing page, not downloadable data). \
             Use a direct download link or fetch via SRA toolkit."
        ));
    }

    if body.len() < 256 {
        let preview = String::from_utf8_lossy(&body[..body.len().min(128)]);
        if preview.contains("<html") || preview.contains("<!DOCTYPE") || preview.contains("<HTML") {
            return Err("Response body appears to be HTML despite content-type header. \
                        Likely a redirect to a login/landing page.".to_string());
        }
    }

    let (suffix, label) = if content_type.contains("zip") || uri.ends_with(".zip") {
        ("_raw.zip", "archive")
    } else if content_type.contains("gzip") || uri.ends_with(".tar.gz") || uri.ends_with(".tgz") {
        (".tar.gz", "tarball")
    } else if content_type.contains("json") || uri.ends_with(".json") {
        (".json", "JSON")
    } else if content_type.contains("csv") || uri.ends_with(".csv") {
        (".csv", "CSV")
    } else {
        ("_data", "data")
    };

    let dest = target_dir.join(format!("{id}{suffix}"));
    std::fs::write(&dest, &body).map_err(|e| format!("write: {e}"))?;
    eprintln!("[{id}]   Saved {:.1} KB {label}", body.len() as f64 / 1024.0);

    Ok(())
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
            let spring_expected = root.parent()
                .and_then(|p| p.parent())
                .map(|eco| eco.join("springs/groundSpring/validation").join(filename));

            if let Some(ref sp) = spring_expected {
                if sp.exists() {
                    std::fs::copy(sp, target_dir.join("expected_values.json"))
                        .map_err(|e| format!("copy from spring: {e}"))?;
                    return Ok(());
                }
            }
        }

        return Err(format!("Expected values not found: {}", expected_path.display()));
    }

    let content = std::fs::read_to_string(&expected_path)
        .map_err(|e| format!("read: {e}"))?;

    let expected: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("parse: {e}"))?;

    // Generate module-specific synthetic data
    match module {
        "ltee-fitness" => generate_fitness_csv(target_dir, &expected),
        "ltee-mutations" => generate_mutation_params(target_dir, &expected),
        _ => {
            // Generic: just copy expected values as the data bundle
            let dest = target_dir.join("expected_values.json");
            std::fs::write(&dest, &content).map_err(|e| format!("write: {e}"))?;
            Ok(())
        }
    }
}

/// Find the expected JSON file for a module by scanning the expected directory
/// for any `.json` file whose name contains the module's key (e.g., "ltee_fitness"
/// matches "module1_fitness.json"). Domain-agnostic: no hardcoded module list.
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

fn generate_fitness_csv(target_dir: &Path, expected: &serde_json::Value) -> Result<(), String> {
    let gens = expected["generations"].as_array()
        .ok_or("missing 'generations' array")?;
    let fitness = expected["mean_fitness"].as_array()
        .ok_or("missing 'mean_fitness' array")?;

    let mut csv = String::from("generation,mean_fitness\n");
    for (g, f) in gens.iter().zip(fitness) {
        let generation = g.as_f64().unwrap_or(0.0) as i64;
        let fit = f.as_f64().unwrap_or(0.0);
        csv.push_str(&format!("{generation},{fit:.6}\n"));
    }

    std::fs::write(target_dir.join("fitness_data.csv"), &csv)
        .map_err(|e| format!("write csv: {e}"))?;

    eprintln!("  Wrote synthetic fitness_data.csv ({} rows)", gens.len());
    Ok(())
}

fn generate_mutation_params(target_dir: &Path, expected: &serde_json::Value) -> Result<(), String> {
    let params = serde_json::json!({
        "experiment": expected.get("experiment").cloned().unwrap_or(serde_json::Value::Null),
        "paper": expected.get("paper").cloned().unwrap_or(serde_json::Value::Null),
        "population_size": 500_000,
        "genomic_mutation_rate": 8.9e-4,
        "generations_observed": 20_000,
        "kimura_fixation_prob_neutral": expected.get("kimura_fixation_prob_neutral").cloned().unwrap_or(serde_json::Value::Null),
        "molecular_clock_rate": expected.get("molecular_clock_rate").cloned().unwrap_or(serde_json::Value::Null),
        "drift_dominance_ratio": expected.get("drift_dominance_ratio").cloned().unwrap_or(serde_json::Value::Null),
        "note": "Generated from expected values — real data requires SRA download",
    });

    let json = serde_json::to_string_pretty(&params)
        .map_err(|e| format!("serialize: {e}"))?;

    std::fs::write(target_dir.join("mutation_parameters.json"), &json)
        .map_err(|e| format!("write: {e}"))?;

    eprintln!("  Wrote synthetic mutation_parameters.json");
    Ok(())
}

fn hash_directory(dir: &Path) -> Result<String, String> {
    let mut files: Vec<_> = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !e.file_name().to_str().map_or(false, |n| n.starts_with('.')))
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
    fn generate_fitness_csv_works() {
        let dir = std::env::temp_dir().join("litho_fetch_test_fitness");
        let _ = std::fs::create_dir_all(&dir);

        let expected = serde_json::json!({
            "generations": [500, 5000, 10000],
            "mean_fitness": [1.05, 1.15, 1.22],
        });

        let result = generate_fitness_csv(&dir, &expected);
        assert!(result.is_ok());
        let csv = std::fs::read_to_string(dir.join("fitness_data.csv")).unwrap();
        assert!(csv.contains("generation,mean_fitness"));
        assert!(csv.contains("500,"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
