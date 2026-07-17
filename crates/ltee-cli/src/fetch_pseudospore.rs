// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho fetch-pseudospore` — download and verify a pseudoSpore from a remote URL.
//!
//! Supports HTTP(S) tarball downloads. After fetch, validates via
//! `pseudospore-core` and optionally chains into ingest.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use pseudospore_core::PseudoSporeEnvelope;

pub fn run(url: &str, output_dir: &str, artifact_root: &str, ingest: bool) {
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path).unwrap_or_else(|e| {
        eprintln!("ERROR: cannot create output directory: {e}");
        std::process::exit(1);
    });

    println!("=== litho fetch-pseudospore ===");
    println!("  URL: {url}");
    println!();

    let tarball_path = download(url, output_path);

    println!("  Extracting...");
    let extracted_dir = extract_tarball(&tarball_path, output_path);

    // Remove tarball after extraction
    fs::remove_file(&tarball_path).ok();

    println!("  Validating...");
    let envelope = match PseudoSporeEnvelope::load(&extracted_dir) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("INVALID pseudoSpore — {err}");
            eprintln!("  Fetched content at: {}", extracted_dir.display());
            std::process::exit(1);
        }
    };

    let result = envelope.validate();
    if !result.valid {
        eprintln!("INVALID pseudoSpore — validation errors:");
        for err in &result.errors {
            eprintln!("  - {err}");
        }
        eprintln!("  Fetched content at: {}", extracted_dir.display());
        std::process::exit(1);
    }

    if let Some(scope) = &envelope.scope {
        println!(
            "  Valid: {} v{} ({}/{} modules pass)",
            scope.artifact.name,
            scope.artifact.version,
            result
                .checksums_verified
                .saturating_sub(result.checksums_failed),
            envelope.checksums.len()
        );
    }

    if !result.warnings.is_empty() {
        for w in &result.warnings {
            println!("  [WARN] {w}");
        }
    }
    println!();

    if ingest {
        println!("  Ingesting into lithoSpore...");
        super::ingest_pseudospore::run(extracted_dir.to_str().unwrap_or("."), artifact_root, true);
    } else {
        println!("  Fetched to: {}", extracted_dir.display());
        println!();
        println!("To ingest:");
        println!(
            "  litho ingest-pseudospore {} --verify",
            extracted_dir.display()
        );
    }
}

/// Download a URL to a local file. Returns the path to the downloaded file.
fn download(url: &str, output_dir: &Path) -> PathBuf {
    let filename = url
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty() && s.contains('.'))
        .unwrap_or("pseudospore.tar.gz");

    let dest = output_dir.join(filename);

    print!("  Downloading... ");
    std::io::stdout().flush().ok();

    let mut response = ureq::get(url)
        .header(
            "User-Agent",
            &format!(
                "lithoSpore/{} (pseudospore-fetch)",
                env!("CARGO_PKG_VERSION")
            ),
        )
        .call()
        .unwrap_or_else(|e| {
            eprintln!("\nERROR: fetch failed: {e}");
            std::process::exit(1);
        });

    let status = response.status();
    if status != 200 {
        eprintln!("\nERROR: server returned HTTP {status}");
        std::process::exit(1);
    }

    let mut file = fs::File::create(&dest).unwrap_or_else(|e| {
        eprintln!("\nERROR: cannot create file: {e}");
        std::process::exit(1);
    });

    let body = response.body_mut().read_to_vec().unwrap_or_else(|e| {
        eprintln!("\nERROR: download failed: {e}");
        std::process::exit(1);
    });

    file.write_all(&body).unwrap_or_else(|e| {
        eprintln!("\nERROR: write failed: {e}");
        std::process::exit(1);
    });

    println!("{} bytes", body.len());
    dest
}

/// Extract a tarball (.tar.gz) to the output directory.
/// Returns the path to the top-level extracted directory.
fn extract_tarball(tarball_path: &Path, output_dir: &Path) -> PathBuf {
    let file = fs::File::open(tarball_path).unwrap_or_else(|e| {
        eprintln!("ERROR: cannot open tarball: {e}");
        std::process::exit(1);
    });

    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    archive.unpack(output_dir).unwrap_or_else(|e| {
        eprintln!("ERROR: extraction failed: {e}");
        std::process::exit(1);
    });

    // Find the extracted top-level directory (first entry with scope.toml)
    find_pseudospore_root(output_dir).unwrap_or_else(|| {
        // Fallback: if tarball extracts flat, the output_dir itself is the root
        if output_dir.join("scope.toml").exists() {
            output_dir.to_path_buf()
        } else {
            eprintln!("ERROR: no scope.toml found in extracted content");
            std::process::exit(1);
        }
    })
}

/// Walk the output directory to find a subdirectory containing `scope.toml`.
fn find_pseudospore_root(dir: &Path) -> Option<PathBuf> {
    if dir.join("scope.toml").exists() {
        return Some(dir.to_path_buf());
    }

    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.join("scope.toml").exists() {
                return Some(path);
            }
            // One level deeper (tar may have a single top-level dir)
            if let Some(found) = find_pseudospore_root(&path) {
                return Some(found);
            }
        }
    }
    None
}
