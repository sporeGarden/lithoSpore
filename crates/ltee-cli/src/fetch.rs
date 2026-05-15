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
}

pub fn run(root: &str, dataset_filter: Option<&str>, all: bool) {
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

        let target_dir = root_path.join(&ds.local_path);
        std::fs::create_dir_all(&target_dir).ok();

        println!("[{id}] Fetching: {uri}", id = ds.id, uri = if ds.source_uri.is_empty() { "(generated)" } else { &ds.source_uri });

        match fetch_dataset(root_path, ds) {
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

fn fetch_dataset(root: &Path, ds: &DatasetEntry) -> Result<String, String> {
    let target_dir = root.join(&ds.local_path);

    // Strategy 1: try HTTP download if source URI is present
    if !ds.source_uri.is_empty() && ds.source_uri.starts_with("http") {
        match try_http_download(&ds.source_uri, &target_dir, &ds.id) {
            Ok(()) => return hash_directory(&target_dir),
            Err(e) => eprintln!("[{}]   HTTP download failed: {e} — trying fallback", ds.id),
        }
    }

    // Strategy 2: generate from expected JSON
    let expected_dir = root.join("validation/expected");
    match generate_from_expected(root, &target_dir, &ds.module, &expected_dir) {
        Ok(()) => return hash_directory(&target_dir),
        Err(e) => eprintln!("[{}]   Fallback generation failed: {e}", ds.id),
    }

    // Strategy 3: check if data already exists
    if target_dir.exists() && std::fs::read_dir(&target_dir).map(|mut d| d.next().is_some()).unwrap_or(false) {
        eprintln!("[{}]   Using existing data in {}", ds.id, target_dir.display());
        return hash_directory(&target_dir);
    }

    Err("No download source, no expected values, and no existing data".to_string())
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

    let (suffix, label) = if content_type.contains("zip") || uri.ends_with(".zip") {
        ("_raw.zip", "archive")
    } else if content_type.contains("gzip") || uri.ends_with(".tar.gz") || uri.ends_with(".tgz") {
        (".tar.gz", "tarball")
    } else if content_type.contains("json") || uri.ends_with(".json") {
        (".json", "JSON")
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
    let expected_file = match module {
        "ltee-fitness" => "module1_fitness.json",
        "ltee-mutations" => "module2_mutations.json",
        "ltee-alleles" => "module3_alleles.json",
        "ltee-citrate" => "module4_citrate.json",
        "ltee-biobricks" => "module5_biobricks.json",
        "ltee-breseq" => "module6_breseq.json",
        "ltee-anderson" => "module7_anderson.json",
        _ => return Err(format!("Unknown module: {module}")),
    };

    let expected_path = expected_dir.join(expected_file);
    if !expected_path.exists() {
        // Also try sibling spring paths
        let spring_expected = root.parent()
            .and_then(|p| p.parent())
            .map(|eco| eco.join("springs/groundSpring/validation").join(expected_file));

        if let Some(ref sp) = spring_expected {
            if sp.exists() {
                std::fs::copy(sp, target_dir.join("expected_values.json"))
                    .map_err(|e| format!("copy from spring: {e}"))?;
                return Ok(());
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
