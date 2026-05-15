// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::Parser;
use litho_core::harness;

#[derive(Parser)]
#[command(name = "ltee-mutations", about = "Mutation accumulation curve validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/barrick_2009")]
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
    let result = ltee_mutations::run_validation(&cli.data_dir, &cli.expected, cli.max_tier);
    harness::output_and_exit(&result, cli.json);
}
