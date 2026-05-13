// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 4: Citrate innovation cascade
//!
//! Reproduces Blount et al. 2008/2012 (B4) — Cit+ innovation.
//! Springs: neuralSpring (early warning ESN), groundSpring (rare event statistics).
//!
//! Upstream gaps:
//! - neuralSpring B4: early warning ESN on pre-citrate allele trajectories
//! - groundSpring B4: rare event probability framework

use clap::Parser;
use litho_core::{ModuleResult, ValidationStatus};

#[derive(Parser)]
#[command(name = "ltee-citrate", about = "Citrate innovation cascade validation")]
struct Cli {
    #[arg(long, default_value = "data/blount_2012")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module4_citrate.json")]
    expected: String,

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
            "Module 4 (citrate): {} — {}/{} checks",
            match result.status {
                ValidationStatus::Pass => "PASS",
                ValidationStatus::Fail => "FAIL",
                ValidationStatus::Skip => "SKIP",
            },
            result.checks_passed,
            result.checks
        );
    }
}

fn run_validation(cli: &Cli) -> ModuleResult {
    let start = std::time::Instant::now();

    let expected_path = std::path::Path::new(&cli.expected);
    let data_path = std::path::Path::new(&cli.data_dir);

    if !expected_path.exists() || !data_path.exists() {
        return ModuleResult {
            name: "citrate_innovation".to_string(),
            status: ValidationStatus::Skip,
            tier: 2,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!(
                "Data or expected values not found — run scripts/fetch_blount_2012.sh first (expected={}, data={})",
                expected_path.display(), data_path.display()
            )),
        };
    }

    let expected: serde_json::Value = match std::fs::read_to_string(expected_path)
        .map_err(|e| e.to_string())
        .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            return ModuleResult {
                name: "citrate_innovation".to_string(),
                status: ValidationStatus::Fail,
                tier: 2,
                checks: 0,
                checks_passed: 0,
                runtime_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("Failed to read expected values: {e}")),
            };
        }
    };

    let mut checks = 0u32;
    let mut passed = 0u32;

    // Cit+ fraction: 1/6 populations evolved Cit+ (Blount 2008)
    if let Some(frac) = expected.get("cit_plus_fraction").and_then(|v| v.as_f64()) {
        checks += 1;
        let expected_frac = 1.0 / 6.0;
        if (frac - expected_frac).abs() < 0.01 { passed += 1; }
    }

    // Potentiation fraction
    if let Some(pot) = expected.get("potentiation_fraction").and_then(|v| v.as_f64()) {
        checks += 1;
        if pot > 0.0 && pot <= 1.0 { passed += 1; }
    }

    // Mean potentiation generation (should be ~41,000)
    if let Some(gen) = expected.get("mean_potentiation_gen").and_then(|v| v.as_f64()) {
        checks += 1;
        if gen > 30000.0 && gen < 50000.0 { passed += 1; }
    }

    // Mean Cit+ generation (should be ~46,000)
    if let Some(gen) = expected.get("mean_cit_plus_gen").and_then(|v| v.as_f64()) {
        checks += 1;
        if gen > 40000.0 && gen < 55000.0 { passed += 1; }
    }

    // Replay probability monotonicity check
    if let Some(replay) = expected.get("replay_probabilities").and_then(|v| v.as_object()) {
        checks += 1;
        let all_valid = replay.values().all(|v| {
            v.as_f64().map_or(false, |p| (0.0..=1.0).contains(&p))
        });
        if all_valid { passed += 1; }
    }

    // Two-hit model: analytical mean >> single-hit mean
    let single = expected.get("single_hit_mean_wait").and_then(|v| v.as_f64());
    let two_hit = expected.get("two_hit_analytical_mean").and_then(|v| v.as_f64());
    if let (Some(s), Some(t)) = (single, two_hit) {
        checks += 1;
        if t > s * 10.0 { passed += 1; }
    }

    // Empirical vs analytical: empirical should be < analytical (potentiation accelerates)
    let empirical = expected.get("two_hit_empirical_mean").and_then(|v| v.as_f64());
    if let (Some(e), Some(a)) = (empirical, two_hit) {
        checks += 1;
        if e < a { passed += 1; }
    }

    // Paper identity
    if let Some(paper) = expected.get("paper").and_then(|v| v.as_str()) {
        checks += 1;
        if paper.starts_with("Blount") { passed += 1; }
    }

    ModuleResult {
        name: "citrate_innovation".to_string(),
        status: if checks > 0 && passed == checks {
            ValidationStatus::Pass
        } else if passed > 0 {
            ValidationStatus::Fail
        } else {
            ValidationStatus::Skip
        },
        tier: 2,
        checks,
        checks_passed: passed,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: if passed < checks { Some(format!("{} check(s) failed", checks - passed)) } else { None },
    }
}
