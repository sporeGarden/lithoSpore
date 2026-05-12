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
use litho_core::{ModuleResult, ValidationStatus};

#[derive(Parser)]
#[command(name = "ltee-alleles", about = "Allele frequency trajectory validation")]
struct Cli {
    #[arg(long, default_value = "data/good_2017")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module3_alleles.json")]
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
            "Module 3 (alleles): {} — {}/{} checks",
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
    let _ = (&cli.data_dir, &cli.expected);

    ModuleResult {
        name: "allele_trajectories".to_string(),
        status: ValidationStatus::Skip,
        tier: 2,
        checks: 0,
        checks_passed: 0,
        runtime_ms: 0,
        error: Some("Awaiting upstream spring reproductions (neuralSpring B3, groundSpring B3)".to_string()),
    }
}
