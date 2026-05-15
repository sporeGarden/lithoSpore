// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unified CLI entry point for lithoSpore.
//!
//! Subcommands: validate, refresh, status, spore, verify, visualize, self-test, tier

mod assemble;
mod chaos;
mod deploy_test;
mod fetch;
mod ops;
mod validate;
mod verify;
mod visualize;

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

    /// Verify data integrity: rehash local files against manifest, and
    /// optionally probe upstream source URIs for changes when online
    Verify {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate scientific visualizations for all modules
    Visualize {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Output format: svg, json, dashboard, baselines (Barrick Lab baseline validation)
        #[arg(long, default_value = "json")]
        format: String,

        /// Output directory for generated figures (--format svg)
        #[arg(long, default_value = "figures")]
        output: String,
    },

    /// Validate artifact integrity: expected JSONs, data dirs, binaries, papers
    SelfTest {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Report which validation tier is achievable on this machine
    Tier {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Assemble the USB artifact directory (replaces scripts/assemble-usb.sh)
    Assemble {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Target directory for the assembled artifact
        #[arg(long, default_value = "usb-staging")]
        target: String,

        #[arg(long)]
        skip_python: bool,

        #[arg(long)]
        skip_fetch: bool,

        #[arg(long)]
        skip_build: bool,

        #[arg(long)]
        dry_run: bool,
    },

    /// Fetch datasets from source URIs (replaces scripts/fetch_*.sh)
    Fetch {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Fetch a specific dataset by ID or module name
        #[arg(long)]
        dataset: Option<String>,

        /// Fetch all datasets
        #[arg(long)]
        all: bool,
    },

    /// Run fault injection tests against the artifact (replaces scripts/chaos-test.sh)
    ChaosTest {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Simulate local deployment: assemble, verify, validate (replaces scripts/deploy-test-local.sh)
    DeployTest {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Generate a TOML deployment report combining self-test, validate, verify
    DeployReport {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Deployment pattern label (e.g. container-airgap, vps-spore, usb-local)
        #[arg(long, default_value = "local")]
        pattern: String,
    },
}

fn main() {
    // argv[0] symlink detection: if invoked as validate/verify/refresh/spore,
    // dispatch directly without requiring the subcommand name.
    if let Some(invoked_as) = std::env::args().next().and_then(|a| {
        std::path::Path::new(&a).file_name().map(|f| f.to_string_lossy().to_string())
    }) {
        let root = ".".to_string();
        match invoked_as.as_str() {
            "validate" => {
                validate::run(&root, false, 2);
                return;
            }
            "verify" => {
                verify::run(&root, false);
                return;
            }
            "refresh" => {
                ops::cmd_refresh(&root);
                return;
            }
            "spore" | "spore.sh" => {
                if std::env::var("BIOMEOS_ORCHESTRATOR").is_ok() {
                    println!("lithoSpore: biomeOS orchestration detected");
                    println!("  Spore class: hypogeal-cotyledon");
                    println!("  Graph: biomeOS/graphs/lithoSpore_validation.toml");
                    return;
                }
                ops::cmd_spore(&root);
                return;
            }
            "ltee" => {
                // Legacy entry point — re-parse remaining args as subcommands
            }
            _ => {}
        }
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { artifact_root, json, max_tier } => validate::run(&artifact_root, json, max_tier),
        Commands::Refresh { artifact_root } => ops::cmd_refresh(&artifact_root),
        Commands::Status { artifact_root } => ops::cmd_status(&artifact_root),
        Commands::Spore { artifact_root } => ops::cmd_spore(&artifact_root),
        Commands::Verify { artifact_root, json } => verify::run(&artifact_root, json),
        Commands::Visualize { artifact_root, format, output } => visualize::run(&artifact_root, &format, &output),
        Commands::SelfTest { artifact_root } => ops::cmd_self_test(&artifact_root),
        Commands::Tier { artifact_root } => ops::cmd_tier(&artifact_root),
        Commands::Assemble { artifact_root, target, skip_python, skip_fetch, skip_build, dry_run } =>
            assemble::run(&artifact_root, &target, skip_python, skip_fetch, skip_build, dry_run),
        Commands::ChaosTest { artifact_root } => chaos::run(&artifact_root),
        Commands::DeployTest { artifact_root } => deploy_test::run(&artifact_root),
        Commands::Fetch { artifact_root, dataset, all } => fetch::run(&artifact_root, dataset.as_deref(), all),
        Commands::DeployReport { artifact_root, pattern } => ops::cmd_deploy_report(&artifact_root, &pattern),
    }
}

/// Resolve liveSpore.json path — root-level (USB) takes precedence over
/// `artifact/liveSpore.json` (dev).
fn resolve_livespore(root: &std::path::Path) -> std::path::PathBuf {
    let usb = root.join("liveSpore.json");
    if usb.exists() || root.join(".biomeos-spore").exists() {
        return usb;
    }
    root.join("artifact/liveSpore.json")
}
