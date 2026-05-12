// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 2: Mutation accumulation curves
//!
//! Reproduces Barrick et al. 2009 (B1) — genome evolution and adaptation.
//! Springs: groundSpring (drift vs selection), neuralSpring (LSTM prediction).

use clap::Parser;
use litho_core::{ModuleResult, ValidationStatus};
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ltee-mutations", about = "Mutation accumulation curve validation")]
struct Cli {
    #[arg(long, default_value = "data/barrick_2009")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module2_mutations.json")]
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
        println!("{}", serde_json::to_string_pretty(&result).expect("JSON serialization"));
    } else {
        println!(
            "Module 2 (mutations): {} — {}/{} checks ({}ms)",
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

fn run_validation(cli: &Cli) -> ModuleResult {
    let start = Instant::now();

    if !Path::new(&cli.expected).exists() {
        return ModuleResult {
            name: "mutation_accumulation".to_string(),
            status: ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some("Expected values not found — run groundSpring B1 first".to_string()),
        };
    }

    if !Path::new(&cli.data_dir).exists() {
        return ModuleResult {
            name: "mutation_accumulation".to_string(),
            status: ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some("Data not fetched — run scripts/fetch_barrick_2009.sh".to_string()),
        };
    }

    if cli.max_tier >= 1 {
        return run_tier1_python(cli, start);
    }

    ModuleResult {
        name: "mutation_accumulation".to_string(),
        status: ValidationStatus::Skip,
        tier: cli.max_tier,
        checks: 0,
        checks_passed: 0,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: Some(format!("Tier {} not implemented yet", cli.max_tier)),
    }
}

fn run_tier1_python(cli: &Cli, start: Instant) -> ModuleResult {
    let notebook_path = Path::new("notebooks/module2_mutations/mutation_accumulation.py");
    if !notebook_path.exists() {
        return ModuleResult {
            name: "mutation_accumulation".to_string(),
            status: ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some("Python baseline not found".to_string()),
        };
    }

    let output = std::process::Command::new("python3")
        .arg(notebook_path)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            eprintln!("{stdout}");
            if !stderr.is_empty() {
                eprintln!("{stderr}");
            }

            let passed = stdout.matches("[PASS]").count() as u32;
            let failed = stdout.matches("[FAIL]").count() as u32;
            let total = passed + failed;

            let status = if out.status.code() == Some(0) && failed == 0 {
                ValidationStatus::Pass
            } else if out.status.code() == Some(2) {
                ValidationStatus::Skip
            } else {
                ValidationStatus::Fail
            };

            ModuleResult {
                name: "mutation_accumulation".to_string(),
                status,
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
        Err(e) => ModuleResult {
            name: "mutation_accumulation".to_string(),
            status: ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Python dispatch failed: {e}")),
        },
    }
}
