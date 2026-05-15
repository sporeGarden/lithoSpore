// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::Parser;
use litho_core::harness;

#[derive(Parser)]
#[command(name = "ltee-breseq", about = "264-genome breseq comparison validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/tenaillon_2016")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module6_breseq.json")]
    expected: String,

    #[arg(long, default_value = "2")]
    max_tier: u8,

    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let result = ltee_breseq::run_validation(&cli.data_dir, &cli.expected, cli.max_tier);
    harness::output_and_exit(&result, cli.json);
}
