// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 6: 264-genome breseq comparison
//!
//! Reproduces Tenaillon et al. 2016 (B7) — tempo and mode of genome evolution.
//! Springs: wetSpring (sovereign 16S/genomics pipeline), groundSpring (epistasis quantification).
//!
//! Upstream gaps:
//! - wetSpring B7: download 264 genomes from NCBI `BioProject`, mutation accumulation curves
//! - groundSpring B7: epistasis quantification, parallel evolution significance tests

use clap::Parser;
use litho_core::{ModuleResult, ValidationStatus};

#[derive(Parser)]
#[command(name = "ltee-breseq", about = "264-genome breseq comparison validation")]
struct Cli {
    #[arg(long, default_value = "data/tenaillon_2016")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module6_breseq.json")]
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
            "Module 6 (breseq): {} — {}/{} checks",
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
        name: "breseq_264_genomes".to_string(),
        status: ValidationStatus::Skip,
        tier: 2,
        checks: 0,
        checks_passed: 0,
        runtime_ms: 0,
        error: Some("Awaiting upstream spring reproductions (wetSpring B7, groundSpring B7)".to_string()),
    }
}
