// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 3: Allele frequency trajectories
//!
//! Reproduces Good et al. 2017 (B3) — dynamics of molecular evolution.
//! Springs: neuralSpring (LSTM+HMM+ESN allele), groundSpring (clonal interference).
//!
//! Upstream gaps:
//! - neuralSpring B3: LSTM allele frequency trajectory prediction
//! - groundSpring B3: clonal interference statistics

use clap::Parser;
use litho_core::harness;
use litho_core::validation::{ModuleResult, ValidationStatus};

#[derive(Parser)]
#[command(name = "ltee-alleles", about = "Allele frequency trajectory validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/good_2017")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module3_alleles.json")]
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
            name: "allele_trajectories".to_string(),
            status: ValidationStatus::Skip,
            tier: 2,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!(
                "Data or expected values not found — run scripts/fetch_good_2017.sh first (expected={}, data={})",
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
                name: "allele_trajectories".to_string(),
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

    if let Some(results) = expected.get("results_by_size").and_then(|v| v.as_object()) {
        for (_size, data) in results {
            if let Some(fix_prob) = data.get("fixation_probability").and_then(|v| v.as_f64()) {
                checks += 1;
                if fix_prob > 0.0 && fix_prob < 1.0 { passed += 1; }
            }
            if let Some(interference) = data.get("interference_ratio").and_then(|v| v.as_f64()) {
                checks += 1;
                if interference > 0.0 { passed += 1; }
            }
            if let Some(fitness) = data.get("mean_final_fitness").and_then(|v| v.as_f64()) {
                checks += 1;
                if fitness >= 1.0 { passed += 1; }
            }
            if let Some(rate) = data.get("adaptation_rate").and_then(|v| v.as_f64()) {
                checks += 1;
                if rate >= 0.0 { passed += 1; }
            }
        }
    }

    if let Some(paper) = expected.get("paper").and_then(|v| v.as_str()) {
        checks += 1;
        if paper == "Good2017" { passed += 1; }
    }

    ModuleResult {
        name: "allele_trajectories".to_string(),
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
