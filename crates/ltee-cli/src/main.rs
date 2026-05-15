// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unified CLI entry point for lithoSpore.
//!
//! Subcommands: validate, refresh, status, spore, verify, visualize

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
        Commands::Verify { artifact_root, json } => cmd_verify(&artifact_root, json),
        Commands::Visualize { artifact_root, format, output } => cmd_visualize(&artifact_root, &format, &output),
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
        ("biobrick_burden", "ltee-biobricks", "artifact/data/biobricks_2024", "validation/expected/module5_biobricks.json"),
        ("breseq_264_genomes", "ltee-breseq", "artifact/data/tenaillon_2016", "validation/expected/module6_breseq.json"),
        ("anderson_qs_predictions", "ltee-anderson", "artifact/data/anderson_predictions", "validation/expected/module7_anderson.json"),
    ];

    for (name, binary, data_dir, expected) in live_modules {
        let data_path = root_path.join(data_dir);
        let expected_path = root_path.join(expected);
        let binary_path = resolve_binary(root_path, binary);

        if let Some(binary_path) = binary_path.filter(|_| data_path.exists() && expected_path.exists()) {
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

/// Resolve a module binary, checking USB layout (`bin/`) first, then dev layout
/// (`target/release/`).
fn resolve_binary(root: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let usb = root.join(format!("bin/{name}"));
    if usb.exists() {
        return Some(usb);
    }
    let dev = root.join(format!("target/release/{name}"));
    if dev.exists() {
        return Some(dev);
    }
    None
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

fn write_livespore(root: &str, report: &litho_core::ValidationReport) {
    let spore_path = resolve_livespore(std::path::Path::new(root));

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
        "biobrick_burden" => "notebooks/module5_biobricks/biobrick_burden.py",
        "breseq_264_genomes" => "notebooks/module6_breseq/breseq_comparison.py",
        "anderson_qs_predictions" => "notebooks/module7_anderson/anderson_predictions.py",
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

fn cmd_visualize(root: &str, format: &str, output_dir: &str) {
    let root_path = std::path::Path::new(root);

    let modules: &[(&str, &str)] = &[
        ("power_law_fitness", "validation/expected/module1_fitness.json"),
        ("mutation_accumulation", "validation/expected/module2_mutations.json"),
        ("allele_trajectories", "validation/expected/module3_alleles.json"),
        ("citrate_innovation", "validation/expected/module4_citrate.json"),
        ("biobrick_burden", "validation/expected/module5_biobricks.json"),
        ("breseq_264_genomes", "validation/expected/module6_breseq.json"),
        ("anderson_qs_predictions", "validation/expected/module7_anderson.json"),
    ];

    match format {
        "json" => {
            let mut all_modules: Vec<(&str, serde_json::Value)> = Vec::new();
            for (name, expected_path) in modules {
                let path = root_path.join(expected_path);
                if !path.exists() {
                    eprintln!("  SKIP {name}: expected values not found");
                    continue;
                }
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => { eprintln!("  SKIP {name}: {e}"); continue; }
                };
                let expected: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(v) => v,
                    Err(e) => { eprintln!("  SKIP {name}: parse error: {e}"); continue; }
                };
                all_modules.push((name, expected));
            }

            let refs: Vec<(&str, &serde_json::Value)> = all_modules
                .iter()
                .map(|(n, v)| (*n, v))
                .collect();

            let dashboard = litho_core::viz::build_dashboard(&refs);
            println!("{}", serde_json::to_string_pretty(&dashboard).unwrap_or_default());
        }
        "svg" => {
            eprintln!("litho visualize --format svg: generating static figures via Python baselines");

            let notebooks: &[(&str, &str)] = &[
                ("module1_fitness", "notebooks/module1_fitness/power_law_fitness.py"),
                ("module2_mutations", "notebooks/module2_mutations/mutation_accumulation.py"),
                ("module3_alleles", "notebooks/module3_alleles/allele_trajectories.py"),
                ("module4_citrate", "notebooks/module4_citrate/citrate_innovation.py"),
                ("module5_biobricks", "notebooks/module5_biobricks/biobrick_burden.py"),
                ("module6_breseq", "notebooks/module6_breseq/breseq_comparison.py"),
                ("module7_anderson", "notebooks/module7_anderson/anderson_predictions.py"),
            ];

            let out_path = root_path.join(output_dir);
            std::fs::create_dir_all(&out_path).ok();

            let mut generated = 0u32;
            for (mod_name, notebook) in notebooks {
                let nb_path = root_path.join(notebook);
                if !nb_path.exists() {
                    eprintln!("  SKIP {mod_name}: notebook not found");
                    continue;
                }

                eprintln!("  {mod_name}...");
                let status = std::process::Command::new("python3")
                    .arg(&nb_path)
                    .current_dir(root)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();

                match status {
                    Ok(s) if s.success() => generated += 1,
                    Ok(s) => eprintln!("    WARNING: exit {}", s.code().unwrap_or(-1)),
                    Err(e) => eprintln!("    WARNING: {e}"),
                }
            }

            eprintln!("  {generated} modules processed, figures in {}", out_path.display());

            let svgs: Vec<String> = std::fs::read_dir(&out_path)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "svg").unwrap_or(false))
                .map(|e| e.path().display().to_string())
                .collect();

            for svg in &svgs {
                println!("  {svg}");
            }
            eprintln!("  {} SVG figures generated", svgs.len());
        }
        "dashboard" => {
            let socket_path = discover_petaltongue_socket();
            if socket_path.is_empty() {
                eprintln!("ERROR: petalTongue socket not found");
                eprintln!("  Set PETALTONGUE_SOCKET or start petalTongue first");
                eprintln!("  Falling back to JSON output:");
                cmd_visualize(root, "json", output_dir);
                return;
            }

            eprintln!("litho visualize --format dashboard");
            eprintln!("  petalTongue socket: {socket_path}");

            let mut all_modules: Vec<(&str, serde_json::Value)> = Vec::new();
            for (name, expected_path) in modules {
                let path = root_path.join(expected_path);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(expected) = serde_json::from_str(&content) {
                        all_modules.push((name, expected));
                    }
                }
            }

            let refs: Vec<(&str, &serde_json::Value)> = all_modules
                .iter()
                .map(|(n, v)| (*n, v))
                .collect();

            let dashboard = litho_core::viz::build_dashboard(&refs);

            let rpc_request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "visualization.render",
                "params": dashboard,
                "id": 1,
            });

            let payload = serde_json::to_vec(&rpc_request).unwrap_or_default();

            match send_uds(&socket_path, &payload) {
                Ok(response) => {
                    eprintln!("  Dashboard pushed to petalTongue");
                    if !response.is_empty() {
                        println!("{response}");
                    }
                }
                Err(e) => {
                    eprintln!("  WARNING: petalTongue push failed: {e}");
                    eprintln!("  Falling back to JSON output:");
                    println!("{}", serde_json::to_string_pretty(&dashboard).unwrap_or_default());
                }
            }
        }
        "baselines" => {
            let baselines_dir = root_path.join("baselines");
            if !baselines_dir.exists() {
                eprintln!("ERROR: baselines/ directory not found at {}", baselines_dir.display());
                std::process::exit(1);
            }

            let baseline_tools: &[&str] = &[
                "breseq", "plannotate", "ostir", "cryptkeeper",
                "efm", "marker_divergence", "rna_mi",
            ];

            let mut all_tools: Vec<(&str, serde_json::Value)> = Vec::new();
            for tool in baseline_tools {
                let ref_path = baselines_dir.join(tool).join("reference_data.json");
                if !ref_path.exists() {
                    eprintln!("  SKIP {tool}: reference_data.json not found");
                    continue;
                }
                let content = match std::fs::read_to_string(&ref_path) {
                    Ok(c) => c,
                    Err(e) => { eprintln!("  SKIP {tool}: {e}"); continue; }
                };
                let data: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(v) => v,
                    Err(e) => { eprintln!("  SKIP {tool}: parse error: {e}"); continue; }
                };
                eprintln!("  Loaded baseline: {tool}");
                all_tools.push((tool, data));
            }

            let refs: Vec<(&str, &serde_json::Value)> = all_tools
                .iter()
                .map(|(n, v)| (*n, v))
                .collect();

            let dashboard = litho_core::viz::build_baseline_dashboard(&refs);
            let bindings_count = dashboard["bindings"].as_array().map(|a| a.len()).unwrap_or(0);
            eprintln!("  {bindings_count} DataBindings from {} tools", all_tools.len());

            let socket_path = discover_petaltongue_socket();
            if socket_path.is_empty() {
                eprintln!("  petalTongue not found — outputting JSON");
                println!("{}", serde_json::to_string_pretty(&dashboard).unwrap_or_default());
                return;
            }

            eprintln!("  Pushing to petalTongue: {socket_path}");

            let rpc_request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "visualization.render",
                "params": dashboard,
                "id": 1,
            });

            let payload = serde_json::to_vec(&rpc_request).unwrap_or_default();
            match send_uds(&socket_path, &payload) {
                Ok(response) => {
                    eprintln!("  Baselines pushed to petalTongue");
                    if !response.is_empty() {
                        println!("{response}");
                    }
                }
                Err(e) => {
                    eprintln!("  WARNING: petalTongue push failed: {e}");
                    println!("{}", serde_json::to_string_pretty(&dashboard).unwrap_or_default());
                }
            }
        }
        _ => {
            eprintln!("ERROR: unknown format '{format}' (use: svg, json, dashboard, baselines)");
            std::process::exit(1);
        }
    }
}

fn discover_petaltongue_socket() -> String {
    if let Ok(path) = std::env::var("PETALTONGUE_SOCKET") {
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }

    let xdg_runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        // Portable UID detection via `id -u`
        let uid = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "1000".to_string());
        format!("/run/user/{uid}")
    });

    let candidates = [
        // biomeOS canonical path (matches petalTongue socket_path.rs)
        format!("{xdg_runtime}/biomeos/petaltongue.sock"),
        // Family-scoped default
        format!("{xdg_runtime}/biomeos/petaltongue-nat0.sock"),
        // Fallback
        "/tmp/biomeos/petaltongue.sock".into(),
        // Legacy paths (backward compat)
        format!("{xdg_runtime}/petalTongue/petal-tongue.sock"),
        format!("{xdg_runtime}/petal-tongue.sock"),
        "/tmp/petal-tongue.sock".into(),
    ];

    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return candidate.clone();
        }
    }

    String::new()
}

fn send_uds(socket_path: &str, payload: &[u8]) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("connect: {e}"))?;

    stream.set_write_timeout(Some(std::time::Duration::from_secs(5))).ok();
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10))).ok();

    stream.write_all(payload).map_err(|e| format!("write: {e}"))?;
    stream.flush().map_err(|e| format!("flush: {e}"))?;

    stream.shutdown(std::net::Shutdown::Write).ok();

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| format!("read: {e}"))?;

    Ok(response)
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
    let spore_path = resolve_livespore(std::path::Path::new(root));
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
        Err(_) => println!("No liveSpore.json found at {} — no validation runs recorded yet", spore_path.display()),
    }
}

fn cmd_verify(root: &str, json_output: bool) {
    let root_path = std::path::Path::new(root);
    let manifest_path = root_path.join("data_manifest.toml");
    let data_toml_path = root_path.join("artifact/data.toml");

    let mut results = VerifyResults::default();

    // Phase 1: verify local files against data_manifest.toml (BLAKE3)
    if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path).unwrap_or_default();
        let manifest: toml::Value = toml::from_str(&content).unwrap_or(toml::Value::Table(Default::default()));

        if let Some(files) = manifest.get("file").and_then(|v| v.as_array()) {
            if !json_output { println!("=== Local integrity check (BLAKE3) ==="); }

            for entry in files {
                let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let expected_hash = entry.get("blake3").and_then(|v| v.as_str()).unwrap_or("");

                if path.is_empty() || expected_hash.is_empty() { continue; }

                let full_path = root_path.join(path);
                let check = if full_path.exists() {
                    match hash_file(&full_path) {
                        Ok(actual) => {
                            if actual == expected_hash {
                                FileCheck { path: path.into(), status: "ok".into(), expected: expected_hash.into(), actual, detail: None }
                            } else {
                                FileCheck { path: path.into(), status: "DRIFT".into(), expected: expected_hash.into(), actual, detail: Some("local file hash does not match manifest".into()) }
                            }
                        }
                        Err(e) => FileCheck { path: path.into(), status: "ERROR".into(), expected: expected_hash.into(), actual: String::new(), detail: Some(format!("hash error: {e}")) },
                    }
                } else {
                    FileCheck { path: path.into(), status: "MISSING".into(), expected: expected_hash.into(), actual: String::new(), detail: Some("file not found on disk".into()) }
                };

                if !json_output && check.status != "ok" {
                    println!("  [{:>7}] {}{}", check.status, check.path, check.detail.as_deref().map(|d| format!(" — {d}")).unwrap_or_default());
                }
                results.local_checks.push(check);
            }

            let ok_count = results.local_checks.iter().filter(|c| c.status == "ok").count();
            let total = results.local_checks.len();
            if !json_output { println!("  {ok_count}/{total} files verified\n"); }
        }
    } else if !json_output {
        println!("  No data_manifest.toml found — cannot verify local integrity\n");
    }

    // Phase 2: check connectivity and probe upstream sources
    let online = check_connectivity();
    results.online = online;

    if !json_output {
        println!("=== Upstream source check ===");
        println!("  Connectivity: {}", if online { "ONLINE" } else { "OFFLINE (airgapped) — skipping upstream checks" });
    }

    if online && data_toml_path.exists() {
        let content = std::fs::read_to_string(&data_toml_path).unwrap_or_default();
        let data_toml: toml::Value = toml::from_str(&content).unwrap_or(toml::Value::Table(Default::default()));

        if let Some(datasets) = data_toml.get("dataset").and_then(|v| v.as_array()) {
            for ds in datasets {
                let id = ds.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
                let uri = ds.get("source_uri").and_then(|v| v.as_str()).unwrap_or("");

                if uri.is_empty() {
                    results.upstream_checks.push(UpstreamCheck {
                        dataset_id: id.into(),
                        source_uri: String::new(),
                        status: "no_uri".into(),
                        detail: Some("no source URI configured".into()),
                    });
                    continue;
                }

                let probe = probe_upstream(uri);
                if !json_output {
                    match &probe.status[..] {
                        "reachable" => println!("  [{id}] {uri} — reachable"),
                        "unreachable" => println!("  [{id}] {uri} — UNREACHABLE: {}", probe.detail.as_deref().unwrap_or("?")),
                        _ => println!("  [{id}] {uri} — {}", probe.status),
                    }
                }
                results.upstream_checks.push(probe);
            }
        }
    }

    // Phase 3: summary
    let local_ok = results.local_checks.iter().filter(|c| c.status == "ok").count();
    let local_total = results.local_checks.len();
    let local_drift = results.local_checks.iter().filter(|c| c.status == "DRIFT").count();
    let upstream_reachable = results.upstream_checks.iter().filter(|c| c.status == "reachable").count();
    let upstream_total = results.upstream_checks.iter().filter(|c| !c.source_uri.is_empty()).count();

    results.summary = VerifySummary {
        local_files_ok: local_ok,
        local_files_total: local_total,
        local_drift,
        upstream_reachable,
        upstream_total,
        online,
    };

    if json_output {
        println!("{}", serde_json::to_string_pretty(&results).unwrap_or_default());
    } else {
        println!();
        println!("=== Verification Summary ===");
        println!("  Local:    {local_ok}/{local_total} files intact, {local_drift} drifted");
        if online {
            println!("  Upstream: {upstream_reachable}/{upstream_total} sources reachable");
        } else {
            println!("  Upstream: skipped (offline)");
        }
    }

    if local_drift > 0 {
        std::process::exit(1);
    }
}

fn hash_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn check_connectivity() -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    let addrs = [
        "datadryad.org:443",
        "www.ncbi.nlm.nih.gov:443",
        "github.com:443",
    ];
    for addr in &addrs {
        if let Ok(mut iter) = addr.to_socket_addrs() {
            if let Some(sock) = iter.next() {
                if TcpStream::connect_timeout(&sock, std::time::Duration::from_secs(3)).is_ok() {
                    return true;
                }
            }
        }
    }
    false
}

fn probe_upstream(uri: &str) -> UpstreamCheck {
    use std::net::{TcpStream, ToSocketAddrs};

    let host = uri
        .strip_prefix("https://").or_else(|| uri.strip_prefix("http://"))
        .and_then(|s| s.split('/').next())
        .unwrap_or("");

    if host.is_empty() {
        return UpstreamCheck {
            dataset_id: String::new(),
            source_uri: uri.into(),
            status: "invalid_uri".into(),
            detail: Some("cannot parse host from URI".into()),
        };
    }

    let addr_str = format!("{host}:443");
    match addr_str.to_socket_addrs() {
        Ok(mut iter) => {
            if let Some(sock) = iter.next() {
                match TcpStream::connect_timeout(&sock, std::time::Duration::from_secs(5)) {
                    Ok(_) => UpstreamCheck {
                        dataset_id: String::new(),
                        source_uri: uri.into(),
                        status: "reachable".into(),
                        detail: None,
                    },
                    Err(e) => UpstreamCheck {
                        dataset_id: String::new(),
                        source_uri: uri.into(),
                        status: "unreachable".into(),
                        detail: Some(format!("TCP connect failed: {e}")),
                    },
                }
            } else {
                UpstreamCheck {
                    dataset_id: String::new(),
                    source_uri: uri.into(),
                    status: "unreachable".into(),
                    detail: Some("DNS resolved but no addresses".into()),
                }
            }
        }
        Err(e) => UpstreamCheck {
            dataset_id: String::new(),
            source_uri: uri.into(),
            status: "unreachable".into(),
            detail: Some(format!("DNS resolution failed: {e}")),
        },
    }
}

#[derive(Default, serde::Serialize)]
struct VerifyResults {
    online: bool,
    local_checks: Vec<FileCheck>,
    upstream_checks: Vec<UpstreamCheck>,
    summary: VerifySummary,
}

#[derive(serde::Serialize)]
struct FileCheck {
    path: String,
    status: String,
    expected: String,
    actual: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(serde::Serialize)]
struct UpstreamCheck {
    dataset_id: String,
    source_uri: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Default, serde::Serialize)]
struct VerifySummary {
    local_files_ok: usize,
    local_files_total: usize,
    local_drift: usize,
    upstream_reachable: usize,
    upstream_total: usize,
    online: bool,
}
