// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unified CLI entry point for lithoSpore.
//!
//! Subcommands (21): validate, parity, verify, fetch, assemble, grow, refresh,
//! status, spore, visualize, self-test, tier, chaos-test, deploy-test,
//! deploy-report, audit, promote, emit-pseudospore, ingest-pseudospore,
//! fetch-pseudospore, translate-config

mod assemble;
mod audit;
mod chaos;
mod deploy_test;
pub(crate) mod domain_profile;
mod emit_pseudospore;
mod fetch;
mod fetch_pseudospore;
mod grow;
mod ingest_pseudospore;
mod ops;
mod parity;
mod promote;
pub(crate) mod registry;
mod translate_config;
mod validate;
mod verify;
mod visualize;
mod viz;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "litho",
    about = "lithoSpore — guideStone verification chassis",
    version,
    long_about = "Self-contained, scope-driven validation artifact.\nCurrent instance: LTEE (Long-Term Evolution Experiment).\nSee https://github.com/sporeGarden/lithoSpore"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all science modules (scope-driven) and produce structured validation output
    Validate {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        #[arg(long)]
        json: bool,

        /// Max tier: 1 = Python only, 2 = Rust (default), 3 = Rust + NUCLEUS provenance.
        #[arg(long, default_value = "2")]
        max_tier: u8,

        /// Write provenance artifacts (results.json + provenance.toml) to this directory.
        /// Follows projectFOUNDATION Thread 10 conventions.
        #[arg(long)]
        provenance_dir: Option<String>,
    },

    /// Cross-tier parity check: run Tier 1 and Tier 2 side-by-side and compare results
    Parity {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        #[arg(long)]
        json: bool,
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

        /// Fetch full upstream data (SRA reads, complete archives) instead of summary stats.
        /// Requires SRA toolkit for genomic datasets. Can be 10s–100s of GB.
        #[arg(long)]
        full: bool,
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

    /// Grow: germinate the USB artifact into a full development environment
    Grow {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Target directory for the cloned source tree
        #[arg(long, default_value = ".")]
        target: String,

        /// Also provision a benchScale VM for isolated validation
        #[arg(long)]
        vm: bool,

        /// Deploy via Docker/Podman container (works on any OS)
        #[arg(long)]
        container: bool,

        /// Also clone the full ecoPrimals ecosystem
        #[arg(long)]
        ecosystem: bool,

        /// Skip building from source
        #[arg(long)]
        skip_build: bool,

        /// Skip fetching upstream datasets
        #[arg(long)]
        skip_fetch: bool,
    },

    /// Ingest a pseudoSpore: validate structure, import braids, register.
    ///
    /// Prefer `biomeos nucleus ingest` when NUCLEUS is available.
    /// This command is the offline/airgapped fallback path.
    ///
    /// Both paths validate via pseudospore-core; NUCLEUS adds provenance trio
    /// registration.
    IngestPseudospore {
        /// Path to the pseudoSpore directory
        path: String,

        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Verify BLAKE3 checksums after structural validation
        #[arg(long)]
        verify: bool,
    },

    /// Fetch a pseudoSpore from a remote URL (hosted gallery or direct tarball).
    ///
    /// Downloads, extracts, validates via pseudospore-core, and optionally
    /// chains into `ingest-pseudospore` for registry and braid import.
    FetchPseudospore {
        /// URL to download (tarball: .tar.gz)
        #[arg(long, alias = "from")]
        url: String,

        /// Output directory for extracted pseudoSpore
        #[arg(long, default_value = ".")]
        output: String,

        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// After fetch+validate, automatically ingest into lithoSpore registry
        #[arg(long)]
        ingest: bool,
    },

    /// Emit a pseudoSpore: assemble standard directory structure from module outputs.
    /// Works for any spring — driven by `domain_profile.toml` when provided.
    ///
    /// Use `--from-dir` to re-emit from an existing pseudoSpore directory (reads
    /// name/version/origin from its `scope.toml`). This is the delegation path
    /// used by nest-validate and other springs.
    EmitPseudospore {
        /// Artifact name (required unless --from-dir is set)
        #[arg(long, required_unless_present = "from_dir")]
        name: Option<String>,

        /// Artifact version (required unless --from-dir is set)
        #[arg(long, required_unless_present = "from_dir")]
        version: Option<String>,

        /// Origin spring/repo path (e.g., "ecoPrimals/springs/hotSpring")
        #[arg(long, default_value = "")]
        origin: String,

        /// Source spring name (e.g., "hotSpring", "groundSpring"). Auto-inferred from origin if omitted.
        #[arg(long)]
        spring: Option<String>,

        /// Output directory (pseudoSpore dir created inside)
        #[arg(long, default_value = ".")]
        output: String,

        /// Directory containing output files to include
        #[arg(long)]
        outputs: Option<String>,

        /// Directory containing config files to include
        #[arg(long)]
        configs: Option<String>,

        /// Directory containing braid JSON files to include
        #[arg(long)]
        braids: Option<String>,

        /// Directory containing raw data files (HILLS, topology) for zero-trust verification
        #[arg(long)]
        data: Option<String>,

        /// Path to a `domain_profile.toml` — drives domain-specific emit logic.
        /// Per `SPORE_OWNERSHIP_MATRIX.md`: each spring provides its own profile.
        #[arg(long, alias = "domain-profile")]
        profile: Option<String>,

        /// Re-emit from an existing pseudoSpore directory. Reads name, version,
        /// and origin from its `scope.toml`. Use for delegation from nest-validate.
        #[arg(long)]
        from_dir: Option<String>,
    },

    /// Pre-handoff audit: check config fidelity, translation, completeness, versioning
    Audit {
        /// Path to the pseudoSpore or lithoSpore proof/ directory
        #[arg(long, default_value = ".")]
        path: String,

        /// Show fix suggestions for each finding
        #[arg(long)]
        verbose: bool,

        /// Emit structured JSON report (guideStone validation format)
        #[arg(long)]
        json: bool,
    },

    /// Promote a pseudoSpore to a lithoSpore deployment chassis
    Promote {
        /// Path to the pseudoSpore directory
        #[arg(long)]
        pseudospore: String,

        /// Output directory (lithoSpore dir created inside)
        #[arg(long, default_value = ".")]
        output: String,

        /// Path to Tier 2 Rust crate to compile and include
        #[arg(long)]
        tier2_crate: Option<String>,

        /// Path to Tier 1 Python validation script to include
        #[arg(long)]
        tier1_script: Option<String>,

        /// Override the lithoSpore version (default: workspace package version)
        #[arg(long)]
        version: Option<String>,
    },

    /// Translate config file indices between domain and computation frames
    TranslateConfig {
        /// Path to `index_map.toml`
        #[arg(long)]
        index_map: String,

        /// Path to the config file to translate (e.g. plumed.dat)
        #[arg(long)]
        config: String,

        /// Target frame: 'domain' (PDB numbering) or 'computation' (runtime indices)
        #[arg(long, default_value = "domain")]
        frame: String,

        /// Output file path (prints to stdout if not specified)
        #[arg(long)]
        output: Option<String>,
    },
}

fn main() {
    // argv[0] symlink detection: if invoked as validate/verify/refresh/spore,
    // dispatch directly without requiring the subcommand name.
    // Strip .exe suffix for Windows compatibility.
    if let Some(invoked_as) = std::env::args().next().and_then(|a| {
        std::path::Path::new(&a).file_name().map(|f| {
            let name = f.to_string_lossy().to_string();
            name.strip_suffix(".exe").unwrap_or(&name).to_string()
        })
    }) {
        let root = ".".to_string();
        match invoked_as.as_str() {
            "validate" => {
                let args: Vec<String> = std::env::args().collect();
                let tier = if args.iter().any(|a| a == "--tier" || a == "--max-tier") {
                    args.windows(2)
                        .find(|w| w[0] == "--tier" || w[0] == "--max-tier")
                        .and_then(|w| w[1].parse::<u8>().ok())
                        .unwrap_or(2)
                } else {
                    2
                };
                let json_out = args.iter().any(|a| a == "--json");
                validate::run(&root, json_out, tier);
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
                if std::env::var(litho_core::env_vars::BIOMEOS_ORCHESTRATOR).is_ok() {
                    println!("lithoSpore: biomeOS orchestration detected");
                    println!("  Spore class: hypogeal-cotyledon");
                    println!("  Graph: biomeOS/graphs/lithoSpore_validation.toml");
                    return;
                }
                ops::cmd_spore(&root);
                return;
            }
            "parity" => {
                let args: Vec<String> = std::env::args().collect();
                let json_out = args.iter().any(|a| a == "--json");
                parity::run(&root, json_out);
                return;
            }
            "grow" => {
                let args: Vec<String> = std::env::args().collect();
                let container = args.iter().any(|a| a == "--container");
                let target = if let Some(w) = args.windows(2).find(|w| w[0] == "--target") {
                    w[1].clone()
                } else if container {
                    ".".to_string()
                } else {
                    eprintln!("ERROR: --target <DIR> is required for grow");
                    eprintln!("Usage: ./grow --target ~/Development/lithoSpore");
                    eprintln!("       ./grow --container   (Docker/Podman, any OS)");
                    std::process::exit(1);
                };
                let vm = args.iter().any(|a| a == "--vm");
                let ecosystem = args.iter().any(|a| a == "--ecosystem");
                let skip_build = args.iter().any(|a| a == "--skip-build");
                let skip_fetch = args.iter().any(|a| a == "--skip-fetch");
                grow::run(&grow::GrowOptions {
                    artifact_root: &root,
                    target: &target,
                    mode: grow::GrowModeFlags {
                        vm,
                        container,
                        ecosystem,
                    },
                    skip: grow::GrowSkipFlags {
                        build: skip_build,
                        fetch: skip_fetch,
                    },
                });
                return;
            }
            // Unrecognized symlink basename — fall through to full clap dispatch
            // (e.g. `litho` binary name or future subcommands not in this fast path).
            _ => {}
        }
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            artifact_root,
            json,
            max_tier,
            provenance_dir,
        } => {
            validate::run_with_provenance(
                &artifact_root,
                json,
                max_tier,
                provenance_dir.as_deref(),
            );
        }
        Commands::Parity {
            artifact_root,
            json,
        } => parity::run(&artifact_root, json),
        Commands::Refresh { artifact_root } => ops::cmd_refresh(&artifact_root),
        Commands::Status { artifact_root } => ops::cmd_status(&artifact_root),
        Commands::Spore { artifact_root } => ops::cmd_spore(&artifact_root),
        Commands::Verify {
            artifact_root,
            json,
        } => verify::run(&artifact_root, json),
        Commands::Visualize {
            artifact_root,
            format,
            output,
        } => visualize::run(&artifact_root, &format, &output),
        Commands::SelfTest { artifact_root } => ops::cmd_self_test(&artifact_root),
        Commands::Tier { artifact_root } => ops::cmd_tier(&artifact_root),
        Commands::Assemble {
            artifact_root,
            target,
            skip_python,
            skip_fetch,
            skip_build,
            dry_run,
        } => assemble::run(&assemble::AssembleOptions {
            root: &artifact_root,
            target: &target,
            skip: assemble::AssembleSkipFlags {
                python: skip_python,
                fetch: skip_fetch,
                build: skip_build,
            },
            dry_run,
        }),
        Commands::ChaosTest { artifact_root } => {
            if let Err(e) = chaos::run(&artifact_root) {
                eprintln!("ERROR: {e}");
                std::process::exit(1);
            }
        }
        Commands::DeployTest { artifact_root } => deploy_test::run(&artifact_root),
        Commands::Fetch {
            artifact_root,
            dataset,
            all,
            full,
        } => fetch::run(&artifact_root, dataset.as_deref(), all, full),
        Commands::DeployReport {
            artifact_root,
            pattern,
        } => ops::cmd_deploy_report(&artifact_root, &pattern),
        Commands::Grow {
            artifact_root,
            target,
            vm,
            container,
            ecosystem,
            skip_build,
            skip_fetch,
        } => grow::run(&grow::GrowOptions {
            artifact_root: &artifact_root,
            target: &target,
            mode: grow::GrowModeFlags {
                vm,
                container,
                ecosystem,
            },
            skip: grow::GrowSkipFlags {
                build: skip_build,
                fetch: skip_fetch,
            },
        }),
        Commands::IngestPseudospore {
            path,
            artifact_root,
            verify,
        } => ingest_pseudospore::run(&path, &artifact_root, verify),
        Commands::FetchPseudospore {
            url,
            output,
            artifact_root,
            ingest,
        } => fetch_pseudospore::run(&url, &output, &artifact_root, ingest),
        Commands::Audit {
            path,
            verbose,
            json,
        } => audit::run(&path, verbose, json),
        Commands::EmitPseudospore {
            name,
            version,
            origin,
            spring,
            output,
            outputs,
            configs,
            braids,
            data,
            profile,
            from_dir,
        } => {
            let (resolved_name, resolved_version, resolved_origin) = if let Some(dir) = &from_dir {
                resolve_emit_from_dir(dir, name.as_deref(), version.as_deref(), &origin)
            } else {
                (
                    name.unwrap_or_default(),
                    version.unwrap_or_default(),
                    origin.clone(),
                )
            };
            let effective_origin = if resolved_origin.is_empty() {
                spring
                    .as_deref()
                    .map(|s| format!("ecoPrimals/springs/{s}"))
                    .unwrap_or_default()
            } else {
                resolved_origin
            };
            let effective_outputs = outputs.as_deref().or(from_dir.as_deref());
            if let Err(e) = emit_pseudospore::run(&emit_pseudospore::EmitConfig {
                name: &resolved_name,
                version: &resolved_version,
                origin: &effective_origin,
                output_dir: &output,
                outputs_dir: effective_outputs,
                configs_dir: configs.as_deref(),
                braids_dir: braids.as_deref(),
                data_dir: data.as_deref(),
                profile_path: profile.as_deref(),
            }) {
                eprintln!("ERROR: {e}");
                std::process::exit(1);
            }
        }
        Commands::Promote {
            pseudospore,
            output,
            tier2_crate,
            tier1_script,
            version,
        } => {
            if let Err(e) = promote::run(
                &pseudospore,
                &output,
                tier2_crate.as_deref(),
                tier1_script.as_deref(),
                version.as_deref(),
            ) {
                eprintln!("ERROR: {e}");
                std::process::exit(1);
            }
        }
        Commands::TranslateConfig {
            index_map,
            config,
            frame,
            output,
        } => {
            if let Err(e) = translate_config::run(&index_map, &config, &frame, output.as_deref()) {
                eprintln!("ERROR: {e}");
                std::process::exit(1);
            }
        }
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

/// Read name, version, and origin from an existing pseudoSpore `scope.toml`.
///
/// CLI flags override scope values when provided.
fn resolve_emit_from_dir(
    dir: &str,
    name_override: Option<&str>,
    version_override: Option<&str>,
    origin_override: &str,
) -> (String, String, String) {
    let scope_path = std::path::Path::new(dir).join("scope.toml");
    match pseudospore_core::ScopeDoc::load(&scope_path) {
        Ok(scope) => {
            let name = name_override
                .map(String::from)
                .unwrap_or(scope.artifact.name);
            let version = version_override
                .map(String::from)
                .unwrap_or(scope.artifact.version);
            let origin = if origin_override.is_empty() {
                scope.artifact.origin
            } else {
                origin_override.to_string()
            };
            (name, version, origin)
        }
        Err(e) => {
            eprintln!(
                "ERROR: --from-dir: cannot read {}: {e}",
                scope_path.display()
            );
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_dir_reads_scope_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let scope = r#"
[artifact]
name = "hotSpring-exp115"
version = "1.7.0"
origin = "ecoPrimals/springs/hotSpring"
"#;
        std::fs::write(dir.path().join("scope.toml"), scope).expect("write");
        let (name, ver, origin) =
            resolve_emit_from_dir(dir.path().to_str().unwrap(), None, None, "");
        assert_eq!(name, "hotSpring-exp115");
        assert_eq!(ver, "1.7.0");
        assert_eq!(origin, "ecoPrimals/springs/hotSpring");
    }

    #[test]
    fn from_dir_cli_overrides_scope() {
        let dir = tempfile::tempdir().expect("tempdir");
        let scope = r#"
[artifact]
name = "hotSpring-exp115"
version = "1.7.0"
origin = "ecoPrimals/springs/hotSpring"
"#;
        std::fs::write(dir.path().join("scope.toml"), scope).expect("write");
        let (name, ver, origin) = resolve_emit_from_dir(
            dir.path().to_str().unwrap(),
            Some("custom-name"),
            Some("2.0.0"),
            "custom/origin",
        );
        assert_eq!(name, "custom-name");
        assert_eq!(ver, "2.0.0");
        assert_eq!(origin, "custom/origin");
    }
}
