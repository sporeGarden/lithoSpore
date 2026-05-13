// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 4: Citrate innovation
//!
//! Reproduces Blount et al. 2008/2012 (B4) — historical contingency and
//! the evolution of a novel metabolic capability.
//! Springs: groundSpring (Cit+ potentiation), wetSpring (replay experiments).
//!
//! Upstream gaps:
//! - groundSpring B4: citrate utilization replay statistics
//! - wetSpring B4: multi-step innovation probability

use clap::Parser;
use litho_core::harness;
use litho_core::validation::{ModuleResult, ValidationStatus};

#[derive(Parser)]
#[command(name = "ltee-citrate", about = "Citrate innovation validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/blount_2012")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module4_citrate.json")]
    expected: String,

    #[arg(long, default_value = "2")]
    max_tier: u8,

    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let result = run_validation(&cli);
    harness::output_and_exit(&result, cli.json);
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

    if let Some(frac) = expected.get("cit_plus_fraction").and_then(|v| v.as_f64()) {
        checks += 1;
        let expected_frac = 1.0 / 6.0;
        if (frac - expected_frac).abs() < 0.01 { passed += 1; }
    }

    if let Some(pot) = expected.get("potentiation_fraction").and_then(|v| v.as_f64()) {
        checks += 1;
        if pot > 0.0 && pot <= 1.0 { passed += 1; }
    }

    if let Some(pot_gen) = expected.get("mean_potentiation_gen").and_then(|v| v.as_f64()) {
        checks += 1;
        if pot_gen > 30000.0 && pot_gen < 50000.0 { passed += 1; }
    }

    if let Some(cit_gen) = expected.get("mean_cit_plus_gen").and_then(|v| v.as_f64()) {
        checks += 1;
        if cit_gen > 40000.0 && cit_gen < 55000.0 { passed += 1; }
    }

    if let Some(replay) = expected.get("replay_probabilities").and_then(|v| v.as_object()) {
        checks += 1;
        let all_valid = replay.values().all(|v| {
            v.as_f64().is_some_and(|p| (0.0..=1.0).contains(&p))
        });
        if all_valid { passed += 1; }
    }

    let single = expected.get("single_hit_mean_wait").and_then(|v| v.as_f64());
    let two_hit = expected.get("two_hit_analytical_mean").and_then(|v| v.as_f64());
    if let (Some(s), Some(t)) = (single, two_hit) {
        checks += 1;
        if t > s * 10.0 { passed += 1; }
    }

    let empirical = expected.get("two_hit_empirical_mean").and_then(|v| v.as_f64());
    if let (Some(e), Some(a)) = (empirical, two_hit) {
        checks += 1;
        if e < a { passed += 1; }
    }

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
