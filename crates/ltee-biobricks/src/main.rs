// SPDX-License-Identifier: AGPL-3.0-or-later

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
    let result = ltee_biobricks::run_validation(&cli.data_dir, &cli.expected, cli.max_tier);
    harness::output_and_exit(&result, cli.json);
}
