// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 1: Power-law fitness trajectories
//!
//! Reproduces Wiser et al. 2013 (B2) — long-term fitness dynamics.
//! Springs: groundSpring (jackknife + AIC/BIC), wetSpring (diversity metrics).
//!
//! Upstream gaps (to be filled by spring reproductions):
//! - groundSpring B2: jackknife + AIC/BIC model selection
//! - wetSpring B2: Anderson-QS predictions for LTEE biofilm experiments

use clap::Parser;
use litho_core::{ModuleResult, ValidationStatus};

#[derive(Parser)]
#[command(name = "ltee-fitness", about = "Power-law fitness trajectory validation")]
struct Cli {
    #[arg(long, default_value = "data/wiser_2013")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module1_fitness.json")]
    expected: String,

    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = run_validation(&cli);

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&result).expect("JSON serialization"));
    } else {
        println!(
            "Module 1 (fitness): {} — {}/{} checks",
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
    // TODO: Implement when groundSpring B2 + wetSpring B2 reproductions land.
    //
    // Expected flow:
    // 1. Load Wiser 2013 fitness data from cli.data_dir
    // 2. Fit power-law, hyperbolic, logarithmic models
    // 3. Compare via AIC/BIC model selection (groundSpring)
    // 4. Compute Anderson disorder analogy (hotSpring B2)
    // 5. Compare against expected values from cli.expected
    let _ = (&cli.data_dir, &cli.expected);

    ModuleResult {
        name: "power_law_fitness".to_string(),
        status: ValidationStatus::Skip,
        tier: 2,
        checks: 0,
        checks_passed: 0,
        runtime_ms: 0,
        error: Some("Awaiting upstream spring reproductions (groundSpring B2, wetSpring B2)".to_string()),
    }
}
