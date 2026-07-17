// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unified CLI entry point for lithoSpore.
//!
//! Subcommands (23): validate, parity, verify, fetch, assemble, grow, refresh,
//! status, spore, visualize, self-test, tier, chaos-test, deploy-test,
//! deploy-report, audit, promote, emit-pseudospore, ingest-pseudospore,
//! fetch-pseudospore, pack-pseudospore, unpack-pseudospore, translate-config

mod assemble;
mod audit;
mod chaos;
mod deploy_test;
mod dispatch;
pub(crate) mod domain_profile;
mod emit_pseudospore;
#[cfg(feature = "fetch")]
mod fetch;
#[cfg(feature = "fetch")]
mod fetch_pseudospore;
mod grow;
mod ingest_pseudospore;
mod ops;
mod pack_pseudospore;
mod parity;
mod promote;
pub(crate) mod registry;
mod translate_config;
mod unpack_pseudospore;
mod validate;
mod verify;
mod visualize;
mod viz;

mod commands;

use clap::Parser;
use commands::{Cli, Commands};

fn main() {
    if dispatch::try_symlink_dispatch() {
        return;
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
        } => {
            #[cfg(feature = "fetch")]
            fetch::run(&artifact_root, dataset.as_deref(), all, full);
            #[cfg(not(feature = "fetch"))]
            {
                let _ = (&artifact_root, &dataset, all, full);
                eprintln!("ERROR: litho was compiled without the 'fetch' feature (no TLS/ring).");
                eprintln!("Rebuild with: cargo build --features fetch");
                std::process::exit(1);
            }
        }
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
        } => {
            #[cfg(feature = "fetch")]
            fetch_pseudospore::run(&url, &output, &artifact_root, ingest);
            #[cfg(not(feature = "fetch"))]
            {
                let _ = (&url, &output, &artifact_root, ingest);
                eprintln!("ERROR: litho was compiled without the 'fetch' feature (no TLS/ring).");
                eprintln!("Rebuild with: cargo build --features fetch");
                std::process::exit(1);
            }
        }
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
                    origin,
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
        Commands::PackPseudospore {
            path,
            output,
            external,
        } => pack_pseudospore::run(&path, output.as_deref(), &external),
        Commands::UnpackPseudospore {
            tarball,
            output,
            validate,
        } => unpack_pseudospore::run(&tarball, &output, validate),
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
