// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unified CLI entry point for lithoSpore.
//!
//! Subcommands: validate, refresh, status, spore

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "litho",
    about = "lithoSpore — LTEE Targeted GuideStone",
    version,
    long_about = "Self-contained validation artifact for the Long-Term Evolution Experiment.\nSee https://github.com/sporeGarden/lithoSpore"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all 7 LTEE modules and produce structured validation output
    Validate {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        #[arg(long)]
        json: bool,

        /// Only run Tier 1 (Python) or Tier 2 (Rust) checks
        #[arg(long, default_value = "2")]
        max_tier: u8,
    },

    /// Re-fetch datasets from source URIs and re-validate
    Refresh {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Show artifact status: version, tier support, module availability
    Status {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Show liveSpore deployment history
    Spore {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            artifact_root,
            json,
            max_tier,
        } => cmd_validate(&artifact_root, json, max_tier),
        Commands::Refresh { artifact_root } => cmd_refresh(&artifact_root),
        Commands::Status { artifact_root } => cmd_status(&artifact_root),
        Commands::Spore { artifact_root } => cmd_spore(&artifact_root),
    }
}

fn cmd_validate(root: &str, json: bool, max_tier: u8) {
    let mut report = litho_core::ValidationReport::new("ltee-guidestone", env!("CARGO_PKG_VERSION"));

    // TODO: Dispatch to each module binary or run inline when implementations land.
    // For now, report scaffold status.
    let modules = [
        "power_law_fitness",
        "mutation_accumulation",
        "allele_trajectories",
        "citrate_innovation",
        "biobrick_burden",
        "breseq_264_genomes",
        "anderson_qs_predictions",
    ];

    for name in &modules {
        report.add_module(litho_core::ModuleResult {
            name: (*name).to_string(),
            status: litho_core::ValidationStatus::Skip,
            tier: max_tier.min(2),
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some("Module scaffold — awaiting upstream spring reproductions".to_string()),
        });
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("JSON serialization")
        );
    } else {
        println!("lithoSpore v{} — LTEE GuideStone", env!("CARGO_PKG_VERSION"));
        println!("Tier reached: {}", report.tier_reached);
        println!("Modules: {}", report.modules.len());
        for m in &report.modules {
            let status = match m.status {
                litho_core::ValidationStatus::Pass => "PASS",
                litho_core::ValidationStatus::Fail => "FAIL",
                litho_core::ValidationStatus::Skip => "SKIP",
            };
            println!("  {} — {} ({}/{})", m.name, status, m.checks_passed, m.checks);
        }
    }

    let _ = root;
    std::process::exit(report.exit_code());
}

fn cmd_refresh(root: &str) {
    println!("litho refresh: re-fetching datasets from source URIs...");
    println!("  artifact root: {root}");
    println!("  TODO: implement data manifest refresh when data.toml is populated");
}

fn cmd_status(root: &str) {
    println!("lithoSpore v{} — LTEE Targeted GuideStone", env!("CARGO_PKG_VERSION"));
    println!("  Artifact root: {root}");
    println!("  Modules: 7 (all scaffold — awaiting spring reproductions)");
    println!("  Tier support: 1 (Python) + 2 (Rust) + 3 (Primal/NUCLEUS)");
    println!("  Data manifest: data.toml (template)");
    println!("  Tolerances: tolerances.toml (template)");
}

fn cmd_spore(root: &str) {
    let spore_path = format!("{root}/liveSpore.json");
    match std::fs::read_to_string(&spore_path) {
        Ok(contents) => {
            let entries: Vec<litho_core::LiveSporeEntry> =
                serde_json::from_str(&contents).unwrap_or_default();
            println!("liveSpore: {} validation runs recorded", entries.len());
            for e in &entries {
                println!(
                    "  {} — {} {} tier {} ({}/{} modules, {}ms)",
                    e.timestamp, e.os, e.arch, e.tier_reached, e.modules_passed, e.modules_total, e.runtime_ms
                );
            }
        }
        Err(_) => println!("No liveSpore.json found at {spore_path} — no validation runs recorded yet"),
    }
}
