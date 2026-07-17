// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho unpack-pseudospore` — extract a pseudoSpore `.tar.gz` tarball
//! and optionally validate the envelope.

use std::path::Path;

pub fn run(tarball: &str, output: &str, validate: bool) {
    let tarball_path = Path::new(tarball);
    if !tarball_path.is_file() {
        eprintln!("ERROR: {tarball} not found or not a file");
        std::process::exit(1);
    }

    let output_path = Path::new(output);

    println!("unpack-pseudospore: {tarball}");
    println!("  target: {}", output_path.display());

    match pseudospore_core::extract_tarball(tarball_path, output_path) {
        Ok(extracted_root) => {
            println!("  extracted: {}", extracted_root.display());

            if validate {
                println!("  validating envelope...");
                match pseudospore_core::PseudoSporeEnvelope::load(&extracted_root) {
                    Ok(envelope) => {
                        let result = envelope.validate();
                        if result.valid {
                            println!("  VALID: {} checksums verified", result.checksums_verified);
                        } else {
                            println!("  INVALID:");
                            for err in &result.errors {
                                println!("    ERROR: {err}");
                            }
                        }
                        for warn in &result.warnings {
                            println!("    WARNING: {warn}");
                        }
                    }
                    Err(e) => {
                        eprintln!("  VALIDATION FAILED: {e}");
                        std::process::exit(1);
                    }
                }
            }

            println!("  UNPACKED: {}", extracted_root.display());
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        }
    }
}
