// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho pack-pseudospore` — create a distributable `.tar.gz` from a
//! pseudoSpore directory, with `[present]`/`[external]` integrity split.

use std::path::Path;

pub fn run(path: &str, output: Option<&str>, external: &[String]) {
    let root = Path::new(path);
    if !root.is_dir() {
        eprintln!("ERROR: {path} is not a directory");
        std::process::exit(1);
    }

    let scope_path = root.join("scope.toml");
    if !scope_path.exists() {
        eprintln!("ERROR: {path}/scope.toml not found — is this a pseudoSpore directory?");
        std::process::exit(1);
    }

    let dir_name = root.file_name().map_or_else(
        || "pseudospore".to_string(),
        |n| n.to_string_lossy().into_owned(),
    );

    let tarball_name = format!("{dir_name}.tar.gz");
    let output_path = output.map_or_else(
        || {
            root.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&tarball_name)
        },
        std::path::PathBuf::from,
    );

    let patterns: Vec<&str> = if external.is_empty() {
        pseudospore_core::tarball::DEFAULT_EXTERNAL_PATTERNS.to_vec()
    } else {
        external.iter().map(String::as_str).collect()
    };

    let (present, ext) = pseudospore_core::split_present_external(root, &patterns);

    println!("pack-pseudospore: {path}");
    println!("  present: {} files", present.len());
    println!("  external: {} files (excluded from tarball)", ext.len());
    println!("  output: {}", output_path.display());

    if !ext.is_empty() {
        let manifest_path = root.join("data.toml");
        if !manifest_path.exists() {
            println!("  writing integrity manifest: data.toml");
            match std::fs::File::create(&manifest_path) {
                Ok(mut f) => {
                    if let Err(e) =
                        pseudospore_core::write_integrity_manifest(root, &present, &ext, &mut f)
                    {
                        eprintln!("  WARNING: failed to write data.toml: {e}");
                    }
                }
                Err(e) => eprintln!("  WARNING: failed to create data.toml: {e}"),
            }
        }
    }

    match pseudospore_core::create_tarball(root, &output_path, &patterns) {
        Ok(hash) => {
            println!("  BLAKE3: {hash}");
            println!("  PACKED: {}", output_path.display());
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        }
    }
}
