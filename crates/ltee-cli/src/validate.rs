// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho validate` — run science modules in-process and produce structured output.
//!
//! The module table is loaded from `scope.toml` + `data.toml` when available,
//! making the validation pipeline domain-agnostic. The compiled `LTEE_MODULES`
//! constant serves as the fallback for the LTEE instance (first lithoSpore).

use crate::resolve_livespore;

/// LTEE instance module table — compiled fallback when scope.toml is absent.
/// Also used by other subcommands (visualize, chaos, ops, deploy_test) as the
/// static module registry for the LTEE instance.
pub(crate) const LTEE_MODULES: &[(&str, &str, &str, &str)] = &[
    ("power_law_fitness", "ltee-fitness", "artifact/data/wiser_2013", "validation/expected/module1_fitness.json"),
    ("mutation_accumulation", "ltee-mutations", "artifact/data/barrick_2009", "validation/expected/module2_mutations.json"),
    ("allele_trajectories", "ltee-alleles", "artifact/data/good_2017", "validation/expected/module3_alleles.json"),
    ("citrate_innovation", "ltee-citrate", "artifact/data/blount_2012", "validation/expected/module4_citrate.json"),
    ("biobrick_burden", "ltee-biobricks", "artifact/data/biobricks_2024", "validation/expected/module5_biobricks.json"),
    ("breseq_264_genomes", "ltee-breseq", "artifact/data/tenaillon_2016", "validation/expected/module6_breseq.json"),
    ("anderson_qs_predictions", "ltee-anderson", "artifact/data/anderson_predictions", "validation/expected/module7_anderson.json"),
];

const LTEE_NOTEBOOKS: &[(&str, &str)] = &[
    ("power_law_fitness", "notebooks/module1_fitness/power_law_fitness.py"),
    ("mutation_accumulation", "notebooks/module2_mutations/mutation_accumulation.py"),
    ("allele_trajectories", "notebooks/module3_alleles/allele_trajectories.py"),
    ("citrate_innovation", "notebooks/module4_citrate/citrate_innovation.py"),
    ("biobrick_burden", "notebooks/module5_biobricks/biobrick_burden.py"),
    ("breseq_264_genomes", "notebooks/module6_breseq/breseq_comparison.py"),
    ("anderson_qs_predictions", "notebooks/module7_anderson/anderson_predictions.py"),
];

/// A runtime-resolved module entry: logical name, binary crate, data dir, expected JSON.
struct ModuleEntry {
    name: String,
    binary: String,
    data_dir: String,
    expected: String,
}

/// Build the module table from `scope.toml` + `data.toml` if present,
/// otherwise fall back to the compiled LTEE constant.
fn load_module_table(root: &std::path::Path) -> Vec<ModuleEntry> {
    let scope_path = root.join("artifact/scope.toml");
    let data_path = root.join("artifact/data.toml");

    if let (Ok(scope), Ok(data_content)) = (
        litho_core::ScopeManifest::load(&scope_path),
        std::fs::read_to_string(&data_path),
    ) {
        if let Ok(data_toml) = data_content.parse::<toml::Value>() {
            let datasets = data_toml.get("dataset")
                .and_then(|v| v.as_array());

            let module_bins = scope.module_binaries();
            let mut entries = Vec::new();

            for bin_name in &module_bins {
                let ds = datasets.and_then(|arr| {
                    arr.iter().find(|d| {
                        d.get("module").and_then(|v| v.as_str()) == Some(bin_name)
                    })
                });

                let data_dir = ds
                    .and_then(|d| d.get("local_path").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .trim_end_matches('/')
                    .to_string();

                let expected = find_expected_json(root, bin_name);

                let name = bin_name.strip_prefix("ltee-")
                    .unwrap_or(bin_name)
                    .replace('-', "_");

                if !data_dir.is_empty() || !expected.is_empty() {
                    entries.push(ModuleEntry {
                        name,
                        binary: bin_name.to_string(),
                        data_dir,
                        expected,
                    });
                }
            }

            if !entries.is_empty() {
                return entries;
            }
        }
    }

    LTEE_MODULES.iter().map(|(name, binary, data_dir, expected)| ModuleEntry {
        name: name.to_string(),
        binary: binary.to_string(),
        data_dir: data_dir.to_string(),
        expected: expected.to_string(),
    }).collect()
}

/// Find the expected JSON file for a module by scanning `validation/expected/`.
fn find_expected_json(root: &std::path::Path, module_binary: &str) -> String {
    let expected_dir = root.join("validation/expected");
    if !expected_dir.is_dir() {
        return String::new();
    }
    let entries = match std::fs::read_dir(&expected_dir) {
        Ok(e) => e,
        Err(_) => return String::new(),
    };
    let suffix = module_binary.replace('-', "_");
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".json") && name_str.contains(&suffix) {
            if let Ok(rel) = entry.path().strip_prefix(root) {
                return rel.to_string_lossy().to_string();
            }
        }
    }
    String::new()
}

type ModuleFn = fn(&str, &str, u8) -> litho_core::ModuleResult;

const MODULE_DISPATCH: &[(&str, ModuleFn)] = &[
    ("ltee-fitness", ltee_fitness::run_validation),
    ("ltee-mutations", ltee_mutations::run_validation),
    ("ltee-alleles", ltee_alleles::run_validation),
    ("ltee-citrate", ltee_citrate::run_validation),
    ("ltee-biobricks", ltee_biobricks::run_validation),
    ("ltee-breseq", ltee_breseq::run_validation),
    ("ltee-anderson", ltee_anderson::run_validation),
];

pub fn run(root: &str, json: bool, max_tier: u8) {
    let root_path = std::path::Path::new(root);

    let scope_name = litho_core::ScopeManifest::load(&root_path.join("artifact/scope.toml"))
        .map(|s| s.guidestone.name.clone())
        .unwrap_or_else(|_| "ltee-guidestone".to_string());

    let mut report = litho_core::ValidationReport::new(&scope_name, env!("CARGO_PKG_VERSION"));
    let modules = load_module_table(root_path);

    for entry in &modules {
        let data_path = root_path.join(&entry.data_dir);
        let expected_path = root_path.join(&entry.expected);

        if data_path.exists() && expected_path.exists() {
            let result = run_module_in_process(
                &entry.binary,
                data_path.to_str().unwrap_or(&entry.data_dir),
                expected_path.to_str().unwrap_or(&entry.expected),
                max_tier,
            );
            report.add_module(result);
        } else {
            report.add_module(dispatch_python_tier1(&entry.name, root, &data_path, &expected_path));
        }
    }

    wire_target_coverage(root_path, &mut report);

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
        println!("lithoSpore v{} — {scope_name}", env!("CARGO_PKG_VERSION"));
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
        if !report.target_coverage.is_empty() {
            println!("\nTarget Coverage (T01–T14):");
            for tc in &report.target_coverage {
                println!("  {} — {} [{}]: {}", tc.id, tc.status, tc.module, tc.claim);
            }
        }
    }

    write_livespore(root, &report);
    std::process::exit(report.exit_code());
}

    /// Resolve a module binary, checking USB layout (`bin/`) first, then dev layout.
    #[allow(dead_code)]
    pub(crate) fn resolve_binary(root: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
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

fn run_module_in_process(
    binary_name: &str,
    data_dir: &str,
    expected: &str,
    max_tier: u8,
) -> litho_core::ModuleResult {
    if let Some((_, func)) = MODULE_DISPATCH.iter().find(|(name, _)| *name == binary_name) {
        func(data_dir, expected, max_tier)
    } else {
        litho_core::ModuleResult {
            name: binary_name.to_string(),
            status: litho_core::ValidationStatus::Skip,
            tier: 0,
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some(format!("No in-process dispatch for {binary_name}")),
        }
    }
}

fn wire_target_coverage(root_path: &std::path::Path, report: &mut litho_core::ValidationReport) {
    let targets_path = root_path.join("data/targets/ltee_validation_targets.toml");
    if !targets_path.exists() {
        return;
    }
    let Ok(content) = std::fs::read_to_string(&targets_path) else { return };
    let Ok(targets_toml) = content.parse::<toml::Value>() else { return };
    let Some(targets_arr) = targets_toml.get("targets").and_then(|v| v.as_array()) else { return };

    for target in targets_arr {
        let id = target.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let module = target.get("module").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let claim = target.get("claim").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let module_result = report.modules.iter().find(|m| module_name_matches(&m.name, &module));

        let status = match module_result {
            Some(m) if m.status == litho_core::ValidationStatus::Pass => "PASS",
            Some(m) if m.status == litho_core::ValidationStatus::Fail => "FAIL",
            Some(_) => "SKIP",
            None => "NOT_RUN",
        };

        report.target_coverage.push(litho_core::TargetCoverage {
            id, module, claim, status: status.to_string(),
        });
    }
}

fn write_livespore(root: &str, report: &litho_core::ValidationReport) {
    let spore_path = resolve_livespore(std::path::Path::new(root));

    let mut entries: Vec<litho_core::LiveSporeEntry> = Vec::new();

    if spore_path.exists() {
        match std::fs::read_to_string(&spore_path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<litho_core::LiveSporeEntry>>(&content) {
                    Ok(parsed) => entries = parsed,
                    Err(e) => {
                        let backup = spore_path.with_extension("json.bak");
                        if let Err(be) = std::fs::copy(&spore_path, &backup) {
                            eprintln!("Warning: liveSpore.json is corrupt ({e}) and backup failed ({be})");
                        } else {
                            eprintln!("Warning: liveSpore.json is corrupt ({e}), backed up to {}", backup.display());
                        }
                    }
                }
            }
            Err(e) => eprintln!("Warning: could not read liveSpore.json: {e}"),
        }
    }

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

    let notebook = match LTEE_NOTEBOOKS.iter().find(|(n, _)| *n == name) {
        Some((_, nb)) => *nb,
        None => return litho_core::ModuleResult {
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

fn module_name_matches(result_name: &str, target_module: &str) -> bool {
    match target_module {
        "ltee-fitness" => result_name == "power_law_fitness",
        "ltee-mutations" => result_name == "mutation_accumulation",
        "ltee-alleles" => result_name == "allele_trajectories",
        "ltee-citrate" => result_name == "citrate_innovation",
        "ltee-biobricks" => result_name == "biobrick_burden",
        "ltee-breseq" => result_name == "breseq_264_genomes",
        "ltee-anderson" => result_name == "anderson_qs_predictions",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_binary_returns_none_for_nonexistent() {
        let root = std::path::Path::new("/nonexistent");
        assert!(resolve_binary(root, "ltee-fitness").is_none());
    }

    #[test]
    fn ltee_modules_table_is_complete() {
        assert_eq!(LTEE_MODULES.len(), 7);
        assert_eq!(LTEE_NOTEBOOKS.len(), 7);
        for (name, _, _, _) in LTEE_MODULES {
            assert!(LTEE_NOTEBOOKS.iter().any(|(n, _)| n == name), "missing notebook for {name}");
        }
    }

    #[test]
    fn module_dispatch_covers_all_ltee() {
        assert_eq!(MODULE_DISPATCH.len(), 7);
        for (_, binary, _, _) in LTEE_MODULES {
            assert!(MODULE_DISPATCH.iter().any(|(n, _)| n == binary),
                "missing dispatch for {binary}");
        }
    }
}
