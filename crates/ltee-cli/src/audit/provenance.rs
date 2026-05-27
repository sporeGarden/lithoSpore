// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance and version sync — ferment transcripts, braid URNs, and doc/JSON version alignment.

use std::fs;
use std::path::Path;

use super::{Finding, Severity};

pub(super) fn check_version_consistency(root: &Path, findings: &mut Vec<Finding>) {
    let scope_path = root.join("scope.toml");
    let scope_version = fs::read_to_string(&scope_path)
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("version"))
                .and_then(|l| l.split('"').nth(1))
                .map(std::string::ToString::to_string)
        })
        .unwrap_or_default();

    if scope_version.is_empty() {
        return;
    }

    // Check docs reference current version
    let doc_files = ["ABG_HANDOFF.md", "RELEASE.md", "README.md"];
    for doc in &doc_files {
        let doc_path = root.join(doc);
        if let Ok(content) = fs::read_to_string(&doc_path) {
            let first_10_lines: String = content.lines().take(10).collect::<Vec<_>>().join("\n");
            if first_10_lines.contains("v0.")
                && !first_10_lines.contains(&format!("v{scope_version}"))
            {
                findings.push(Finding {
                    id: format!("VERSION-STALE-{doc}"),
                    severity: Severity::Medium,
                    category: "Documentation",
                    message: format!(
                        "{doc} references an older version in header (scope.toml says v{scope_version})"
                    ),
                    fix: format!("Update {doc} to reference v{scope_version}"),
                });
            }
        }
    }

    // Check JSON files with "version" fields match scope.toml
    let json_files = ["validation.json", "validation_matrix.json"];
    for jf in &json_files {
        let jpath = root.join(jf);
        if let Ok(content) = fs::read_to_string(&jpath)
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(jv) = v.get("version").and_then(|v| v.as_str())
            && jv != scope_version
        {
            findings.push(Finding {
                id: format!("VERSION-JSON-{jf}"),
                severity: Severity::Low,
                category: "Version Sync",
                message: format!(
                    "{jf} says version \"{jv}\" but scope.toml says \"{scope_version}\""
                ),
                fix: format!("Update \"version\" field in {jf} to \"{scope_version}\""),
            });
        }
    }

    // Check environment.toml total_production_ns matches actual module sum from scope.toml
    let env_path = root.join("receipts/environment.toml");
    if let Ok(env_content) = fs::read_to_string(&env_path) {
        let claimed_ns: Option<u64> = env_content
            .lines()
            .find(|l| l.starts_with("total_production_ns"))
            .and_then(|l| l.split('=').nth(1))
            .and_then(|v| v.trim().parse().ok());

        // Sum simulation_time_ns from scope.toml modules
        let scope_content = fs::read_to_string(&scope_path).unwrap_or_default();
        let actual_ns: u64 = scope_content
            .lines()
            .filter(|l| l.starts_with("simulation_time_ns"))
            .filter_map(|l| l.split('=').nth(1))
            .filter_map(|v| v.trim().parse::<u64>().ok())
            .sum();

        if let Some(claimed) = claimed_ns
            && actual_ns > 0
            && claimed != actual_ns
        {
            findings.push(Finding {
                    id: "ENV-PRODUCTION-NS".to_string(),
                    severity: Severity::Low,
                    category: "Version Sync",
                    message: format!(
                        "environment.toml claims {claimed} ns total but scope.toml modules sum to {actual_ns} ns"
                    ),
                    fix: format!("Update total_production_ns to {actual_ns}"),
                });
        }
    }
}

pub(super) fn check_provenance(root: &Path, findings: &mut Vec<Finding>) {
    let ferment_path = root.join("provenance/ferment_transcript.json");
    if let Ok(content) = fs::read_to_string(&ferment_path)
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&content)
    {
        let empty_fields: Vec<&str> = [
            "dataset_id",
            "spring",
            "dag_session_id",
            "braid_id",
            "timestamp",
        ]
        .iter()
        .filter(|&&field| {
            v.get(field)
                .is_none_or(|val| val.as_str().is_none_or(str::is_empty))
        })
        .copied()
        .collect();

        if !empty_fields.is_empty() {
            findings.push(Finding {
                id: "PROVENANCE-GAPS".to_string(),
                severity: Severity::Medium,
                category: "Provenance",
                message: format!(
                    "ferment_transcript.json has empty fields: {}",
                    empty_fields.join(", ")
                ),
                fix: "Populate all provenance fields before handoff".to_string(),
            });
        }

        // Check for placeholder merkle root
        if let Some(merkle) = v.get("dag_merkle_root").and_then(|v| v.as_str())
            && (merkle.contains("pending") || merkle.contains("placeholder") || merkle.is_empty())
        {
            findings.push(Finding {
                        id: "PROVENANCE-MERKLE-PLACEHOLDER".to_string(),
                        severity: Severity::Medium,
                        category: "Provenance",
                        message: format!("dag_merkle_root is placeholder: \"{merkle}\""),
                        fix: "Compute actual BLAKE3 merkle root: b3sum outputs/*/fes_*.dat data/*/HILLS* | b3sum".to_string(),
                    });
        }
    }

    // Check braid JSONs for frozen/stale URNs
    let provenance_dir = root.join("provenance");
    if !provenance_dir.exists() {
        return;
    }

    let scope_version = fs::read_to_string(root.join("scope.toml"))
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("version"))
                .and_then(|l| l.split('"').nth(1))
                .map(std::string::ToString::to_string)
        })
        .unwrap_or_default();

    if let Ok(entries) = fs::read_dir(&provenance_dir) {
        let mut braid_ids: Vec<(String, String)> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("cazyme_fel_v"))
            {
                continue;
            }
            if path.extension().is_some_and(|e| e == "json")
                && let Ok(content) = fs::read_to_string(&path)
                && let Ok(v) = serde_json::from_str::<serde_json::Value>(&content)
            {
                let fname = path.file_name().unwrap().to_string_lossy().to_string();
                if let Some(bid) = v.get("braid_id").and_then(|v| v.as_str()) {
                    braid_ids.push((fname.clone(), bid.to_string()));
                }
            }
        }

        // Detect frozen URNs: if all braids share the same braid_id despite different versions
        if braid_ids.len() > 1 {
            let first_id = &braid_ids[0].1;
            let all_same = braid_ids.iter().all(|(_, id)| id == first_id);
            if all_same {
                findings.push(Finding {
                    id: "PROVENANCE-FROZEN-URN".to_string(),
                    severity: Severity::Low,
                    category: "Provenance",
                    message: format!(
                        "All {} braid JSONs share identical braid_id \"{}\" — should be unique per version",
                        braid_ids.len(), first_id
                    ),
                    fix: "Each braid version should have its own unique braid_id URN".to_string(),
                });
            }
        }

        // Check that the latest braid's URN references the current version
        if !scope_version.is_empty() {
            let latest_braid = format!("cazyme_fel_v{scope_version}.json");
            let latest_path = provenance_dir.join(&latest_braid);
            if let Ok(content) = fs::read_to_string(&latest_path)
                && let Ok(v) = serde_json::from_str::<serde_json::Value>(&content)
                && let Some(bid) = v.get("braid_id").and_then(|v| v.as_str())
                && !bid.contains(&scope_version.replace('.', "-"))
                && !bid.contains(&scope_version)
            {
                findings.push(Finding {
                                id: "PROVENANCE-URN-VERSION-MISMATCH".to_string(),
                                severity: Severity::Low,
                                category: "Provenance",
                                message: format!(
                                    "{latest_braid}: braid_id \"{bid}\" doesn't reference current version {scope_version}"
                                ),
                                fix: format!("Update braid_id to include version identifier (e.g., urn:braid:...-v{scope_version})"),
                            });
            }
        }
    }
}
