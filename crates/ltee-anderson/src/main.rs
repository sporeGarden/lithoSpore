// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::Parser;
use litho_core::harness;

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
    let result = ltee_anderson::run_validation(&cli.data_dir, &cli.expected, cli.max_tier);
    harness::output_and_exit(&result, cli.json);
}
