// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho parity` — cross-tier numerical parity check.
//!
//! Runs both Tier 1 (Python) and Tier 2 (Rust) for all modules, then
//! compares results. If both tiers agree (same PASS/FAIL, same check
//! counts), the module is MATCH. Any disagreement is DIVERGENCE.

use crate::registry;

pub fn run(root: &str, json: bool) {
    let root_path = std::path::Path::new(root);

    let scope_name = registry::load_scope_name(root_path);
    let modules = registry::load_module_table(root_path);

    let mut parity_results: Vec<litho_core::ParityResult> = Vec::new();

    eprintln!("=== Cross-Tier Parity: Tier 1 (Python) vs Tier 2 (Rust) ===");
    eprintln!();

    for entry in &modules {
        let data_path = root_path.join(&entry.data_dir);
        let expected_path = root_path.join(&entry.expected);

        eprintln!("--- {} ---", entry.name);

        let tier2 = run_tier2(&entry.binary, &data_path, &expected_path);
        let tier1 = run_tier1(entry, root, root_path, &data_path, &expected_path);

        let parity = compute_parity(&tier1, &tier2);

        let symbol = match parity {
            litho_core::ParityStatus::Match => "MATCH",
            litho_core::ParityStatus::Divergence => "DIVERGENCE",
            litho_core::ParityStatus::Skipped => "SKIPPED",
        };
        eprintln!("  Tier 1: {:?} ({}/{})", tier1.status, tier1.checks_passed, tier1.checks);
        eprintln!("  Tier 2: {:?} ({}/{})", tier2.status, tier2.checks_passed, tier2.checks);
        eprintln!("  Parity: [{symbol}]");
        eprintln!();

        parity_results.push(litho_core::ParityResult {
            module: entry.name.clone(),
            tier1_status: tier1.status,
            tier2_status: tier2.status,
            tier1_checks: tier1.checks,
            tier2_checks: tier2.checks,
            tier1_passed: tier1.checks_passed,
            tier2_passed: tier2.checks_passed,
            parity,
        });
    }

    let all_match = parity_results.iter().all(|r| r.parity != litho_core::ParityStatus::Divergence);

    let report = litho_core::ParityReport {
        artifact: scope_name,
        version: env!("CARGO_PKG_VERSION").to_string(),
        modules: parity_results,
        parity_pass: all_match,
    };

    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(j) => println!("{j}"),
            Err(e) => {
                eprintln!("Error serializing parity report: {e}");
                std::process::exit(2);
            }
        }
    } else {
        println!("lithoSpore v{} — Cross-Tier Parity", env!("CARGO_PKG_VERSION"));
        println!();
        for r in &report.modules {
            let symbol = match r.parity {
                litho_core::ParityStatus::Match => "MATCH",
                litho_core::ParityStatus::Divergence => "DIVERGENCE",
                litho_core::ParityStatus::Skipped => "SKIPPED",
            };
            println!(
                "  {:<30} T1={:?}({}/{}) T2={:?}({}/{})  [{}]",
                r.module, r.tier1_status, r.tier1_passed, r.tier1_checks,
                r.tier2_status, r.tier2_passed, r.tier2_checks, symbol,
            );
        }
        println!();
        if all_match {
            println!("PARITY: PASS — math is stable between tiers");
        } else {
            println!("PARITY: FAIL — divergence detected between tiers");
        }
    }

    if all_match {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn run_tier2(
    binary: &str,
    data_path: &std::path::Path,
    expected_path: &std::path::Path,
) -> litho_core::ModuleResult {
    if !data_path.exists() || !expected_path.is_file() {
        return litho_core::ModuleResult {
            name: binary.to_string(),
            status: litho_core::ValidationStatus::Skip,
            tier: 2,
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some("data or expected file not found".into()),
        };
    }

    registry::dispatch_module(
        binary,
        data_path.to_str().unwrap_or(""),
        expected_path.to_str().unwrap_or(""),
        2,
    )
}

fn run_tier1(
    entry: &registry::ModuleEntry,
    root: &str,
    root_path: &std::path::Path,
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
            runtime_ms: 0,
            error: Some("data or expected not found".into()),
        };
    }

    let notebook = if !entry.tier1_notebook.is_empty() {
        entry.tier1_notebook.as_str()
    } else if let Some((_, nb)) = registry::LTEE_NOTEBOOKS.iter().find(|(n, _)| *n == name.as_str()) {
        *nb
    } else {
        return litho_core::ModuleResult {
            name: name.clone(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some("no Python baseline".into()),
        };
    };

    let nb_path = root_path.join(notebook);
    if !nb_path.exists() {
        return litho_core::ModuleResult {
            name: name.clone(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("notebook not found: {notebook}")),
        };
    }

    let python = find_python(root_path);
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
                error: if failed > 0 { Some(format!("{failed} check(s) failed")) } else { None },
            }
        }
        Err(e) => litho_core::ModuleResult {
            name: name.clone(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("python dispatch failed: {e}")),
        },
    }
}

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

fn compute_parity(
    t1: &litho_core::ModuleResult,
    t2: &litho_core::ModuleResult,
) -> litho_core::ParityStatus {
    if t1.status == litho_core::ValidationStatus::Skip
        || t2.status == litho_core::ValidationStatus::Skip
    {
        return litho_core::ParityStatus::Skipped;
    }

    if t1.status != t2.status {
        return litho_core::ParityStatus::Divergence;
    }

    if t1.status == litho_core::ValidationStatus::Pass
        && t2.status == litho_core::ValidationStatus::Pass
        && t1.checks_passed > 0
        && t2.checks_passed > 0
    {
        return litho_core::ParityStatus::Match;
    }

    if t1.status == t2.status {
        litho_core::ParityStatus::Match
    } else {
        litho_core::ParityStatus::Divergence
    }
}
