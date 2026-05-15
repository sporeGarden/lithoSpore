// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::Parser;
use litho_core::harness;

#[derive(Parser)]
#[command(name = "ltee-fitness", about = "Power-law fitness trajectory validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/wiser_2013")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module1_fitness.json")]
    expected: String,

    #[arg(long, default_value = "2")]
    max_tier: u8,

    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let result = ltee_fitness::run_validation(&cli.data_dir, &cli.expected, cli.max_tier);
    harness::output_and_exit(&result, cli.json);
}
