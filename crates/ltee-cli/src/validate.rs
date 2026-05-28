// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho validate` — run science modules in-process and produce structured output.
//!
//! The module table is loaded from `scope.toml` + `data.toml` when available,
//! making the validation pipeline domain-agnostic. The compiled `LTEE_MODULES`
//! constant serves as the fallback for the LTEE instance (first lithoSpore).

use crate::registry::{self, ModuleEntry};
use crate::resolve_livespore;

pub fn run(root: &str, json: bool, max_tier: u8) {
    run_with_provenance(root, json, max_tier, None);
}

pub(crate) fn run_with_provenance(
    root: &str,
    json: bool,
    max_tier: u8,
    provenance_dir: Option<&str>,
) {
    let root_path = std::path::Path::new(root);

    let scope_name = litho_core::ScopeManifest::load(&root_path.join("artifact/scope.toml"))
        .map_or_else(
            |_| "ltee-guidestone".to_string(),
            |s| s.guidestone.name.clone(),
        );

    let mut report = litho_core::ValidationReport::new(&scope_name, env!("CARGO_PKG_VERSION"));
    let modules = registry::load_module_table(root_path);

    if max_tier == 1 {
        eprintln!("=== Tier 1 (Python) — chain escalation baseline ===");
        for entry in &modules {
            let data_path = root_path.join(&entry.data_dir);
            let expected_path = root_path.join(&entry.expected);
            report.add_module(dispatch_python_tier1(
                entry,
                root,
                &data_path,
                &expected_path,
            ));
        }
    } else {
        eprintln!("=== Tier 2 (Rust) — compiled validation ===");
        for entry in &modules {
            let data_path = root_path.join(&entry.data_dir);
            let expected_path = root_path.join(&entry.expected);

            if !entry.expected.is_empty() && data_path.exists() && expected_path.is_file() {
                let result = run_module_in_process(
                    &entry.binary,
                    data_path.to_str().unwrap_or(&entry.data_dir),
                    expected_path.to_str().unwrap_or(&entry.expected),
                    max_tier,
                );
                report.add_module(result);
            } else {
                report.add_module(dispatch_python_tier1(
                    entry,
                    root,
                    &data_path,
                    &expected_path,
                ));
            }
        }

        // Tier 3: attempt provenance recording via NUCLEUS primals
        if max_tier >= 3 {
            eprintln!();
            eprintln!("=== Tier 3 (Primal) — NUCLEUS composition ===");

            // Announce ourselves to biomeOS (non-fatal)
            if litho_core::discovery::announce_self().is_some() {
                eprintln!("  Announced to biomeOS");
            }

            let (mode, _) = litho_core::discovery::probe_operating_mode();
            eprintln!("  Discovery mode: {mode}");

            match litho_core::provenance::try_record_tier3(&report) {
                Ok(session) => {
                    eprintln!("  [PASS] DAG session: {}", session.dag_session_id);
                    eprintln!("  [PASS] Merkle root: {}", session.dag_merkle_root);
                    eprintln!("  [PASS] Spine ID:    {}", session.spine_id);
                    eprintln!("  [PASS] Braid ID:    {}", session.braid_id);
                    eprintln!("  Primals: {}", session.primals_reached.join(", "));
                    report.tier_reached = 3;
                    report.tier3 = Some(session);
                }
                Err(e) => {
                    eprintln!("  Tier 3 unavailable: {e}");
                    eprintln!(
                        "  Remaining at Tier {} (science validation complete)",
                        report.tier_reached
                    );
                }
            }
        }
    }

    wire_target_coverage(root_path, &mut report);

    // Upstream ferment transcript braids
    let braids_dir = root_path.join("provenance/braids");
    let braids = litho_core::braid::load_braids(&braids_dir);
    let accessions_from_data = load_sra_accessions(root_path);
    let expected_accessions: Vec<(&str, &str)> = accessions_from_data
        .iter()
        .map(|(id, acc)| (id.as_str(), acc.as_str()))
        .collect();
    let braid_checks = litho_core::braid::validate_braids(&braids, &expected_accessions);

    // Write provenance artifacts for projectFOUNDATION Thread 10
    if let Some(dir) = provenance_dir {
        write_provenance_dir(dir, &report);
    }

    if json {
        println!(
            "{}",
            match serde_json::to_string_pretty(&report) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Error serializing report: {e}");
                    std::process::exit(2);
                }
            }
        );
    } else {
        println!("lithoSpore v{} — {scope_name}", env!("CARGO_PKG_VERSION"));
        println!("Tier reached: {}", report.tier_reached);
        println!("Modules: {}", report.modules.len());
        for m in &report.modules {
            let status = match m.status {
                litho_core::ValidationStatus::Pass => "PASS",
                litho_core::ValidationStatus::Fail => "FAIL",
                litho_core::ValidationStatus::Skip => "SKIP",
            };
            println!(
                "  {} — {} ({}/{})",
                m.name, status, m.checks_passed, m.checks
            );
        }
        if let Some(ref t3) = report.tier3 {
            println!("\nTier 3 Provenance:");
            println!("  DAG:   {}", t3.dag_session_id);
            println!("  Spine: {}", t3.spine_id);
            println!("  Braid: {}", t3.braid_id);
        }

        // Display upstream braids
        if !braids.is_empty() {
            println!("\nUpstream Braids ({}):", braids.len());
            println!("{}", litho_core::braid::format_braid_summary(&braids));
            for check in &braid_checks {
                if !check.found_accession.is_empty() {
                    if check.accession_ok {
                        println!(
                            "  [PASS] {} accession: {}",
                            check.braid_id, check.found_accession
                        );
                    } else {
                        println!(
                            "  [FAIL] {} accession mismatch: got {}, expected {}",
                            check.braid_id, check.found_accession, check.expected_accession
                        );
                    }
                }
            }
        }

        if !report.target_coverage.is_empty() {
            println!("\nTarget Coverage (T01–T14):");
            for tc in &report.target_coverage {
                println!("  {} — {} [{}]: {}", tc.id, tc.status, tc.module, tc.claim);
            }
        }
    }

    write_livespore(root, &report);
    std::process::exit(report.exit_code());
}

/// Write provenance artifacts to a dated directory for projectFOUNDATION Thread 10.
/// Pattern: `<dir>/results.json` + `<dir>/provenance.toml`
fn write_provenance_dir(dir: &str, report: &litho_core::ValidationReport) {
    let dir_path = std::path::Path::new(dir);
    if let Err(e) = std::fs::create_dir_all(dir_path) {
        eprintln!("WARNING: Could not create provenance dir {dir}: {e}");
        return;
    }

    // results.json — full validation report
    let results_path = dir_path.join("results.json");
    match serde_json::to_string_pretty(report) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&results_path, &json) {
                eprintln!("WARNING: Could not write {}: {e}", results_path.display());
            } else {
                eprintln!("  Provenance: {}", results_path.display());
            }
        }
        Err(e) => eprintln!("WARNING: Could not serialize results: {e}"),
    }

    // provenance.toml — summary metadata
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let passed = report
        .modules
        .iter()
        .filter(|m| m.status == litho_core::ValidationStatus::Pass)
        .count();
    let toml_content = format!(
        "# lithoSpore provenance artifact — projectFOUNDATION Thread 10\n\
         [meta]\n\
         artifact = \"{}\"\n\
         version = \"{}\"\n\
         timestamp = \"{timestamp}\"\n\
         tier_reached = {}\n\
         modules_passed = {passed}\n\
         modules_total = {}\n",
        report.artifact,
        report.version,
        report.tier_reached,
        report.modules.len(),
    );
    let toml_path = dir_path.join("provenance.toml");
    if let Err(e) = std::fs::write(&toml_path, &toml_content) {
        eprintln!("WARNING: Could not write {}: {e}", toml_path.display());
    } else {
        eprintln!("  Provenance: {}", toml_path.display());
    }
}

/// Resolve a module binary, checking USB layout (`bin/`) first, then dev layout.
#[cfg(test)]
pub(crate) fn resolve_binary(root: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let usb = root.join(format!("bin/{name}"));
    if usb.exists() {
        return Some(usb);
    }
    let dev = root.join(format!("target/release/{name}"));
    if dev.exists() {
        return Some(dev);
    }
    None
}

fn run_module_in_process(
    binary_name: &str,
    data_dir: &str,
    expected: &str,
    max_tier: u8,
) -> litho_core::ModuleResult {
    registry::dispatch_module(binary_name, data_dir, expected, max_tier)
}

fn wire_target_coverage(root_path: &std::path::Path, report: &mut litho_core::ValidationReport) {
    let targets_path = registry::load_scope(root_path)
        .and_then(|s| {
            let f = &s.guidestone.targets_file;
            if f.is_empty() {
                None
            } else {
                Some(root_path.join(f))
            }
        })
        .unwrap_or_else(|| root_path.join("data/targets/ltee_validation_targets.toml"));

    if !targets_path.exists() {
        return;
    }
    let Ok(content) = std::fs::read_to_string(&targets_path) else {
        return;
    };
    let Ok(targets_toml) = content.parse::<toml::Value>() else {
        return;
    };
    let Some(targets_arr) = targets_toml.get("targets").and_then(|v| v.as_array()) else {
        return;
    };

    let modules = registry::load_module_table(root_path);

    for target in targets_arr {
        let id = target
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let module = target
            .get("module")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let claim = target
            .get("claim")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let target_status = target
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("active");

        if target_status == "pending_upstream" {
            report.target_coverage.push(litho_core::TargetCoverage {
                id,
                module,
                claim,
                status: "PENDING".to_string(),
            });
            continue;
        }

        let module_result = report
            .modules
            .iter()
            .find(|m| registry::module_name_matches(&modules, &m.name, &module));

        let status = match module_result {
            Some(m) if m.status == litho_core::ValidationStatus::Pass => "PASS",
            Some(m) if m.status == litho_core::ValidationStatus::Fail => "FAIL",
            Some(_) => "SKIP",
            None => "NOT_RUN",
        };

        report.target_coverage.push(litho_core::TargetCoverage {
            id,
            module,
            claim,
            status: status.to_string(),
        });
    }
}

fn write_livespore(root: &str, report: &litho_core::ValidationReport) {
    let spore_path = resolve_livespore(std::path::Path::new(root));

    let mut entries: Vec<litho_core::LiveSporeEntry> = Vec::new();

    if spore_path.exists() {
        match std::fs::read_to_string(&spore_path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<litho_core::LiveSporeEntry>>(&content) {
                    Ok(parsed) => entries = parsed,
                    Err(e) => {
                        let backup = spore_path.with_extension("json.bak");
                        if let Err(be) = std::fs::copy(&spore_path, &backup) {
                            eprintln!(
                                "Warning: liveSpore.json is corrupt ({e}) and backup failed ({be})"
                            );
                        } else {
                            eprintln!(
                                "Warning: liveSpore.json is corrupt ({e}), backed up to {}",
                                backup.display()
                            );
                        }
                    }
                }
            }
            Err(e) => eprintln!("Warning: could not read liveSpore.json: {e}"),
        }
    }

    entries.push(litho_core::LiveSporeEntry::from_report(report));

    match serde_json::to_string_pretty(&entries) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&spore_path, json) {
                eprintln!("Warning: could not write liveSpore.json: {e}");
            } else {
                eprintln!(
                    "liveSpore: recorded validation run ({} entries)",
                    entries.len()
                );
            }
        }
        Err(e) => eprintln!("Warning: could not serialize liveSpore: {e}"),
    }
}

fn dispatch_python_tier1(
    entry: &ModuleEntry,
    root: &str,
    data_path: &std::path::Path,
    expected_path: &std::path::Path,
) -> litho_core::ModuleResult {
    let start = std::time::Instant::now();
    let name = &entry.name;

    if !data_path.exists() && !expected_path.exists() {
        return litho_core::ModuleResult {
            name: name.clone(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some("Data or expected values not found — run fetch scripts first".to_string()),
        };
    }

    let notebook = if !entry.tier1_notebook.is_empty() {
        entry.tier1_notebook.as_str()
    } else if let Some((_, nb)) = registry::LTEE_NOTEBOOKS
        .iter()
        .find(|(n, _)| *n == name.as_str())
    {
        *nb
    } else {
        return litho_core::ModuleResult {
            name: name.clone(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some("No Python baseline available".to_string()),
        };
    };

    let root_path = std::path::Path::new(root);
    let nb_path = root_path.join(notebook);
    if !nb_path.exists() {
        return litho_core::ModuleResult {
            name: name.clone(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Python baseline not found: {notebook}")),
        };
    }

    let python = find_python(root_path);
    eprintln!("  Python: {python}");
    let output = std::process::Command::new(&python)
        .arg(&nb_path)
        .current_dir(root)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let passed = stdout.matches("[PASS]").count() as u32;
            let failed = stdout.matches("[FAIL]").count() as u32;
            let total = passed + failed;

            litho_core::ModuleResult {
                name: name.clone(),
                status: if out.status.code() == Some(0) && failed == 0 {
                    litho_core::ValidationStatus::Pass
                } else if out.status.code() == Some(2) {
                    litho_core::ValidationStatus::Skip
                } else {
                    litho_core::ValidationStatus::Fail
                },
                tier: 1,
                checks: total,
                checks_passed: passed,
                runtime_ms: start.elapsed().as_millis() as u64,
                error: if failed > 0 {
                    Some(format!("{failed} check(s) failed"))
                } else {
                    None
                },
            }
        }
        Err(e) => litho_core::ModuleResult {
            name: name.clone(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Python dispatch failed: {e}")),
        },
    }
}

/// Find the best Python interpreter: bundled first, then system.
fn find_python(root: &std::path::Path) -> String {
    for candidate in [
        root.join("python/bin/python3.13"),
        root.join("python/bin/python3"),
    ] {
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }
    "python3".to_string()
}

/// Derive expected SRA accessions from `data.toml` dataset entries.
/// Returns (`dataset_id_fragment`, `sra_accession`) pairs, extracting the
/// dataset name prefix (e.g. "`barrick_2009`" from "`barrick_2009_mutations`")
/// for braid matching.
fn load_sra_accessions(root: &std::path::Path) -> Vec<(String, String)> {
    let data_path = root.join("artifact/data.toml");
    let content = match std::fs::read_to_string(&data_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let data_toml: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut accessions = Vec::new();
    if let Some(datasets) = data_toml.get("dataset").and_then(|v| v.as_array()) {
        for ds in datasets {
            let id = ds.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let acc = ds
                .get("sra_accession")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !acc.is_empty() {
                accessions.push((id.to_string(), acc.to_string()));
                // Also add a trimmed variant (e.g. "barrick_2009" from "barrick_2009_mutations")
                // so braid files that reference partial dataset IDs can match.
                if let Some(pos) = id.rfind('_') {
                    let prefix = &id[..pos];
                    if prefix != id {
                        accessions.push((prefix.to_string(), acc.to_string()));
                    }
                }
            }
        }
    }
    accessions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_binary_returns_none_for_nonexistent() {
        let root = std::path::Path::new("/nonexistent");
        assert!(resolve_binary(root, "ltee-fitness").is_none());
    }
}
