// SPDX-License-Identifier: AGPL-3.0-or-later

//! BLAKE3 checksum integrity — verifies `receipts/checksums.blake3` matches on-disk files.

use std::fs;
use std::path::Path;

use super::{Finding, Severity};

/// Verify BLAKE3 checksums actually match file contents.
pub(super) fn check_blake3_integrity(root: &Path, findings: &mut Vec<Finding>) {
    let cksum_path = root.join("receipts/checksums.blake3");
    if !cksum_path.exists() {
        findings.push(Finding {
            id: "BLAKE3-MISSING".to_string(),
            severity: Severity::High,
            category: "Integrity",
            message: "receipts/checksums.blake3 not found".to_string(),
            fix: "Regenerate checksums with b3sum".to_string(),
        });
        return;
    }

    let content = match fs::read_to_string(&cksum_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut failures = 0;
    let mut checked = 0;

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        // Format: <hash>  <path>
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        if parts.len() != 2 {
            continue;
        }

        let expected_hash = parts[0].trim();
        let rel_path = parts[1].trim();
        let file_path = root.join(rel_path.trim_start_matches("./"));

        if !file_path.exists() {
            failures += 1;
            continue;
        }

        let file_bytes = if let Ok(b) = fs::read(&file_path) {
            b
        } else {
            failures += 1;
            continue;
        };

        let actual_hash = blake3::hash(&file_bytes).to_hex().to_string();
        if actual_hash != expected_hash {
            failures += 1;
            if failures <= 3 {
                findings.push(Finding {
                    id: format!("BLAKE3-MISMATCH-{}", rel_path.replace('/', "-")),
                    severity: Severity::High,
                    category: "Integrity",
                    message: format!(
                        "{rel_path}: checksum mismatch (file modified after sealing?)"
                    ),
                    fix: "Regenerate checksums or restore original file".to_string(),
                });
            }
        }
        checked += 1;
    }

    if failures > 3 {
        findings.push(Finding {
            id: "BLAKE3-MULTI-FAIL".to_string(),
            severity: Severity::High,
            category: "Integrity",
            message: format!(
                "{} of {} files have checksum mismatches",
                failures,
                checked + failures
            ),
            fix:
                "Regenerate all checksums: find . -type f | xargs b3sum > receipts/checksums.blake3"
                    .to_string(),
        });
    }
}
