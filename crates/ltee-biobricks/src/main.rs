// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 5: BioBrick metabolic burden
//!
//! Reproduces burden measurements from 2024 BioBrick fitness assays.
//! Springs: healthSpring (burden quantification), airSpring (FLS2 immunity).
//!
//! Upstream gaps:
//! - healthSpring B5: BioBrick burden quantification pipeline
//! - airSpring: FLS2 plant immunity cross-validation

use clap::Parser;
use litho_core::harness;

#[derive(Parser)]
#[command(name = "ltee-biobricks", about = "BioBrick metabolic burden validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/biobricks_2024")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module5_biobricks.json")]
    expected: String,

    #[arg(long, default_value = "2")]
    max_tier: u8,

    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let start = std::time::Instant::now();
    let result = harness::skip(
        "biobrick_burden", cli.max_tier, start,
        "Awaiting upstream spring reproductions (healthSpring B5, airSpring FLS2)",
    );
    harness::output_and_exit(&result, cli.json);
}
