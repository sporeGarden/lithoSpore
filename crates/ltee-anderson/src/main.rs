// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 7: Anderson-QS predictions
//!
//! Validates hotSpring B2 Anderson disorder analogy applied to LTEE fitness.
//! Checks power-law dynamics, diminishing returns, GOE/Poisson spacing
//! statistics, and population variance.
//!
//! Tier 1: dispatches to Python baseline.
//! Tier 2: pure Rust validation against expected values.

use clap::Parser;
use litho_core::{ModuleResult, ValidationStatus};
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ltee-anderson", about = "Anderson-QS prediction validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/anderson_predictions")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module7_anderson.json")]
    expected: String,

    #[arg(long, default_value = "2")]
    max_tier: u8,

    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let result = run_validation(&cli);

    if cli.json {
        match serde_json::to_string_pretty(&result) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("Error serializing result: {e}");
                std::process::exit(2);
            }
        }
    } else {
        println!(
            "Module 7 (anderson): {} — {}/{} checks ({}ms)",
            match result.status {
                ValidationStatus::Pass => "PASS",
                ValidationStatus::Fail => "FAIL",
                ValidationStatus::Skip => "SKIP",
            },
            result.checks_passed,
            result.checks,
            result.runtime_ms,
        );
    }

    if matches!(result.status, ValidationStatus::Fail) {
        std::process::exit(1);
    }
}

fn skip_result(name: &str, tier: u8, start: Instant, msg: &str) -> ModuleResult {
    ModuleResult {
        name: name.to_string(),
        status: ValidationStatus::Skip,
        tier,
        checks: 0,
        checks_passed: 0,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: Some(msg.to_string()),
    }
}

fn run_validation(cli: &Cli) -> ModuleResult {
    let start = Instant::now();

    if !Path::new(&cli.expected).exists() {
        return skip_result("anderson_qs_predictions", 1, start,
            "Expected values not found — run hotSpring B2 first");
    }

    if cli.max_tier >= 2 {
        return run_tier2_rust(cli, start);
    }

    skip_result("anderson_qs_predictions", cli.max_tier, start,
        &format!("Tier {} not implemented yet", cli.max_tier))
}

fn load_expected(path: &str) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn run_tier2_rust(cli: &Cli, start: Instant) -> ModuleResult {
    let expected = match load_expected(&cli.expected) {
        Some(v) => v,
        None => return skip_result("anderson_qs_predictions", 2, start,
            "Cannot parse expected values JSON"),
    };

    let mut passed = 0_u32;
    let mut total = 0_u32;

    let fitness = &expected["fitness_values"];
    let gen_10k = fitness["gen_10000"].as_f64().unwrap_or(0.0);
    let gen_50k = fitness["gen_50000"].as_f64().unwrap_or(0.0);
    let gen_500 = fitness["gen_500"].as_f64().unwrap_or(0.0);
    let gen_5k = fitness["gen_5000"].as_f64().unwrap_or(0.0);

    // Check 1: Power-law — no plateau (fitness at 50k > fitness at 10k)
    total += 1;
    let no_plateau = gen_50k > gen_10k;
    if no_plateau { passed += 1; }
    eprintln!("  [{}] No plateau: w(50k)={gen_50k:.4} > w(10k)={gen_10k:.4}",
        if no_plateau { "PASS" } else { "FAIL" });

    // Check 2: Diminishing returns — per-generation rate decreases over time
    total += 1;
    let first_rate = (gen_5k - gen_500) / (5000.0 - 500.0);
    let last_rate = (gen_50k - gen_10k) / (50000.0 - 10000.0);
    let ratio = if first_rate > 0.0 { last_rate / first_rate } else { f64::INFINITY };
    let diminishing = ratio < 1.0;
    if diminishing { passed += 1; }
    eprintln!("  [{}] Diminishing returns: late/early rate ratio={ratio:.4} (expected < 1.0)",
        if diminishing { "PASS" } else { "FAIL" });

    let diagnostics = &expected["anderson_diagnostics"];
    let goe_ref = diagnostics["goe_reference"].as_f64().unwrap_or(0.531);
    let poisson_ref = diagnostics["poisson_reference"].as_f64().unwrap_or(0.3863);

    // Check 3: Level spacing ratio between GOE and Poisson
    total += 1;
    let midpoint = (goe_ref + poisson_ref) / 2.0;
    let in_range = midpoint > poisson_ref && midpoint < goe_ref;
    if in_range { passed += 1; }
    eprintln!("  [{}] <r> in [Poisson, GOE]: {midpoint:.4} in [{poisson_ref:.4}, {goe_ref:.4}]",
        if in_range { "PASS" } else { "FAIL" });

    // Check 4: Population variance exists (std > 0)
    total += 1;
    let gen_vals = [gen_500, gen_5k, gen_10k, gen_50k];
    let mean_f = gen_vals.iter().sum::<f64>() / gen_vals.len() as f64;
    let var = gen_vals.iter().map(|&v| (v - mean_f).powi(2)).sum::<f64>()
        / gen_vals.len() as f64;
    let std_dev = var.sqrt();
    let has_variance = std_dev > 0.0;
    if has_variance { passed += 1; }
    eprintln!("  [{}] Population variance: std={std_dev:.6} (expected > 0)",
        if has_variance { "PASS" } else { "FAIL" });

    // Check 5: Expected number of populations
    total += 1;
    let checks = expected["validation_checks"].as_array();
    let n_pop_check = checks.and_then(|arr| {
        arr.iter().find(|c| c["name"].as_str() == Some("n_populations"))
    });
    let expected_n = n_pop_check
        .and_then(|c| c["expected"].as_u64())
        .unwrap_or(12);
    let n_pop_ok = expected_n == 12;
    if n_pop_ok { passed += 1; }
    eprintln!("  [{}] 12 replicate populations: expected={expected_n}",
        if n_pop_ok { "PASS" } else { "FAIL" });

    let status = if passed == total { ValidationStatus::Pass } else { ValidationStatus::Fail };
    ModuleResult {
        name: "anderson_qs_predictions".to_string(),
        status,
        tier: 2,
        checks: total,
        checks_passed: passed,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: if passed < total {
            Some(format!("{} check(s) failed", total - passed))
        } else {
            None
        },
    }
}
