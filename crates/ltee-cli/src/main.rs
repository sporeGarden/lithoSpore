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
    let root_path = std::path::Path::new(root);

    let live_modules: &[(&str, &str, &str, &str)] = &[
        ("power_law_fitness", "ltee-fitness", "artifact/data/wiser_2013", "validation/expected/module1_fitness.json"),
        ("mutation_accumulation", "ltee-mutations", "artifact/data/barrick_2009", "validation/expected/module2_mutations.json"),
        ("allele_trajectories", "ltee-alleles", "artifact/data/good_2017", "validation/expected/module3_alleles.json"),
        ("citrate_innovation", "ltee-citrate", "artifact/data/blount_2012", "validation/expected/module4_citrate.json"),
        ("breseq_264_genomes", "ltee-breseq", "artifact/data/tenaillon_2016", "validation/expected/module6_breseq.json"),
        ("anderson_qs_predictions", "ltee-anderson", "artifact/data/anderson_predictions", "validation/expected/module7_anderson.json"),
    ];

    for (name, binary, data_dir, expected) in live_modules {
        let data_path = root_path.join(data_dir);
        let expected_path = root_path.join(expected);
        let binary_path = root_path.join(format!("target/release/{binary}"));

        if binary_path.exists() && data_path.exists() && expected_path.exists() {
            let start = std::time::Instant::now();
            let output = std::process::Command::new(&binary_path)
                .arg("--data-dir").arg(&data_path)
                .arg("--expected").arg(&expected_path)
                .arg("--max-tier").arg(max_tier.to_string())
                .arg("--json")
                .output();

            match output {
                Ok(out) => {
                    if let Ok(result) = serde_json::from_slice::<litho_core::ModuleResult>(&out.stdout) {
                        report.add_module(result);
                    } else {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let passed = stdout.matches("[PASS]").count() as u32;
                        let failed = stdout.matches("[FAIL]").count() as u32;
                        report.add_module(litho_core::ModuleResult {
                            name: (*name).to_string(),
                            status: if failed == 0 && passed > 0 {
                                litho_core::ValidationStatus::Pass
                            } else if failed > 0 {
                                litho_core::ValidationStatus::Fail
                            } else {
                                litho_core::ValidationStatus::Skip
                            },
                            tier: max_tier.min(2),
                            checks: passed + failed,
                            checks_passed: passed,
                            runtime_ms: start.elapsed().as_millis() as u64,
                            error: if failed > 0 { Some(format!("{failed} check(s) failed")) } else { None },
                        });
                    }
                }
                Err(e) => {
                    report.add_module(litho_core::ModuleResult {
                        name: (*name).to_string(),
                        status: litho_core::ValidationStatus::Skip,
                        tier: 1,
                        checks: 0,
                        checks_passed: 0,
                        runtime_ms: start.elapsed().as_millis() as u64,
                        error: Some(format!("Binary dispatch failed: {e}")),
                    });
                }
            }
        } else {
            report.add_module(dispatch_python_tier1(name, root, &data_path, &expected_path));
        }
    }

    let scaffold_modules = [
        "biobrick_burden",
    ];

    for name in &scaffold_modules {
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
            match serde_json::to_string_pretty(&report) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Error serializing report: {e}");
                    std::process::exit(2);
                }
            }
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

    write_livespore(root, &report);
    std::process::exit(report.exit_code());
}

fn write_livespore(root: &str, report: &litho_core::ValidationReport) {
    let spore_path = std::path::Path::new(root).join("artifact/liveSpore.json");

    let mut entries: Vec<litho_core::LiveSporeEntry> = spore_path
        .exists()
        .then(|| std::fs::read_to_string(&spore_path).ok())
        .flatten()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();

    entries.push(litho_core::LiveSporeEntry::from_report(report));

    match serde_json::to_string_pretty(&entries) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&spore_path, json) {
                eprintln!("Warning: could not write liveSpore.json: {e}");
            } else {
                eprintln!("liveSpore: recorded validation run ({} entries)", entries.len());
            }
        }
        Err(e) => eprintln!("Warning: could not serialize liveSpore: {e}"),
    }
}

fn dispatch_python_tier1(
    name: &str,
    root: &str,
    data_path: &std::path::Path,
    expected_path: &std::path::Path,
) -> litho_core::ModuleResult {
    let start = std::time::Instant::now();

    if !data_path.exists() || !expected_path.exists() {
        return litho_core::ModuleResult {
            name: name.to_string(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some("Data or expected values not found — run fetch scripts first".to_string()),
        };
    }

    let notebook = match name {
        "power_law_fitness" => "notebooks/module1_fitness/power_law_fitness.py",
        "mutation_accumulation" => "notebooks/module2_mutations/mutation_accumulation.py",
        "allele_trajectories" => "notebooks/module3_alleles/allele_trajectories.py",
        "citrate_innovation" => "notebooks/module4_citrate/citrate_innovation.py",
        _ => return litho_core::ModuleResult {
            name: name.to_string(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some("No Python baseline available".to_string()),
        },
    };

    let root_path = std::path::Path::new(root);
    let nb_path = root_path.join(notebook);
    if !nb_path.exists() {
        return litho_core::ModuleResult {
            name: name.to_string(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Python baseline not found: {notebook}")),
        };
    }

    let output = std::process::Command::new("python3")
        .arg(&nb_path)
        .current_dir(root)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let passed = stdout.matches("[PASS]").count() as u32;
            let failed = stdout.matches("[FAIL]").count() as u32;
            let total = passed + failed;

            litho_core::ModuleResult {
                name: name.to_string(),
                status: if out.status.code() == Some(0) && failed == 0 {
                    litho_core::ValidationStatus::Pass
                } else if out.status.code() == Some(2) {
                    litho_core::ValidationStatus::Skip
                } else {
                    litho_core::ValidationStatus::Fail
                },
                tier: 1,
                checks: total,
                checks_passed: passed,
                runtime_ms: start.elapsed().as_millis() as u64,
                error: if failed > 0 { Some(format!("{failed} check(s) failed")) } else { None },
            }
        }
        Err(e) => litho_core::ModuleResult {
            name: name.to_string(),
            status: litho_core::ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: start.elapsed().as_millis() as u64,
            error: Some(format!("Python dispatch failed: {e}")),
        },
    }
}

fn cmd_refresh(root: &str) {
    println!("litho refresh: re-fetching datasets from source URIs...");
    println!("  artifact root: {root}");

    let root_path = std::path::Path::new(root);
    let data_toml = root_path.join("artifact/data.toml");

    let toml_content = match std::fs::read_to_string(&data_toml) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ERROR: Cannot read {}: {e}", data_toml.display());
            std::process::exit(1);
        }
    };

    let manifest: toml::Value = match toml::from_str(&toml_content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("  ERROR: Failed to parse data.toml: {e}");
            std::process::exit(1);
        }
    };

    let datasets = if let Some(arr) = manifest.get("dataset").and_then(|v| v.as_array()) { arr } else {
        println!("  No [[dataset]] entries found in data.toml");
        return;
    };

    let mut fetched = 0u32;
    let mut skipped = 0u32;
    let mut failed = 0u32;

    for ds in datasets {
        let id = ds.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let refresh_cmd = ds.get("refresh_command").and_then(|v| v.as_str()).unwrap_or("");

        if refresh_cmd.is_empty() {
            println!("  [{id}] no refresh_command — skip");
            skipped += 1;
            continue;
        }

        let script_path = root_path.join(refresh_cmd);
        if !script_path.exists() {
            println!("  [{id}] script not found: {refresh_cmd} — skip");
            skipped += 1;
            continue;
        }

        println!("  [{id}] running {refresh_cmd}...");
        let result = std::process::Command::new("bash")
            .arg(&script_path)
            .current_dir(root)
            .status();

        match result {
            Ok(s) if s.success() => {
                println!("  [{id}] OK");
                fetched += 1;
            }
            Ok(s) => {
                eprintln!("  [{id}] FAILED (exit {})", s.code().unwrap_or(-1));
                failed += 1;
            }
            Err(e) => {
                eprintln!("  [{id}] FAILED ({e})");
                failed += 1;
            }
        }
    }

    println!();
    println!("  Refresh complete: {fetched} fetched, {skipped} skipped, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

fn cmd_status(root: &str) {
    let root_path = std::path::Path::new(root);

    let modules: &[(&str, &str, &str)] = &[
        ("1 (fitness)", "validation/expected/module1_fitness.json", "artifact/data/wiser_2013"),
        ("2 (mutations)", "validation/expected/module2_mutations.json", "artifact/data/barrick_2009"),
        ("3 (alleles)", "validation/expected/module3_alleles.json", "artifact/data/good_2017"),
        ("4 (citrate)", "validation/expected/module4_citrate.json", "artifact/data/blount_2012"),
        ("5 (biobricks)", "validation/expected/module5_biobricks.json", "artifact/data/biobricks_2024"),
        ("6 (breseq)", "validation/expected/module6_breseq.json", "artifact/data/tenaillon_2016"),
        ("7 (anderson)", "validation/expected/module7_anderson.json", "artifact/data/anderson_predictions"),
    ];

    let mut live = 0_u32;
    println!("lithoSpore v{} — LTEE Targeted GuideStone", env!("CARGO_PKG_VERSION"));
    println!("  Artifact root: {root}");

    for &(name, expected_path, data_path) in modules {
        let has_expected = root_path.join(expected_path).exists();
        let has_data = root_path.join(data_path).exists();
        if has_expected { live += 1; }
        println!("  Module {name:<14} expected={has_expected} data={has_data}");
    }

    println!("  Modules: 7 ({live} live, {} scaffold)", 7 - live);
    println!("  Tier support: 1 (Python) + 2 (Rust) + 3 (Primal/NUCLEUS)");
}

fn cmd_spore(root: &str) {
    let spore_path = format!("{root}/artifact/liveSpore.json");
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
