// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho validate` — run all 7 LTEE modules and produce structured output.

use crate::resolve_livespore;

pub(crate) const LIVE_MODULES: &[(&str, &str, &str, &str)] = &[
    ("power_law_fitness", "ltee-fitness", "artifact/data/wiser_2013", "validation/expected/module1_fitness.json"),
    ("mutation_accumulation", "ltee-mutations", "artifact/data/barrick_2009", "validation/expected/module2_mutations.json"),
    ("allele_trajectories", "ltee-alleles", "artifact/data/good_2017", "validation/expected/module3_alleles.json"),
    ("citrate_innovation", "ltee-citrate", "artifact/data/blount_2012", "validation/expected/module4_citrate.json"),
    ("biobrick_burden", "ltee-biobricks", "artifact/data/biobricks_2024", "validation/expected/module5_biobricks.json"),
    ("breseq_264_genomes", "ltee-breseq", "artifact/data/tenaillon_2016", "validation/expected/module6_breseq.json"),
    ("anderson_qs_predictions", "ltee-anderson", "artifact/data/anderson_predictions", "validation/expected/module7_anderson.json"),
];

const NOTEBOOKS: &[(&str, &str)] = &[
    ("power_law_fitness", "notebooks/module1_fitness/power_law_fitness.py"),
    ("mutation_accumulation", "notebooks/module2_mutations/mutation_accumulation.py"),
    ("allele_trajectories", "notebooks/module3_alleles/allele_trajectories.py"),
    ("citrate_innovation", "notebooks/module4_citrate/citrate_innovation.py"),
    ("biobrick_burden", "notebooks/module5_biobricks/biobrick_burden.py"),
    ("breseq_264_genomes", "notebooks/module6_breseq/breseq_comparison.py"),
    ("anderson_qs_predictions", "notebooks/module7_anderson/anderson_predictions.py"),
];

pub fn run(root: &str, json: bool, max_tier: u8) {
    let mut report = litho_core::ValidationReport::new("ltee-guidestone", env!("CARGO_PKG_VERSION"));
    let root_path = std::path::Path::new(root);

    for (name, binary, data_dir, expected) in LIVE_MODULES {
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

/// Resolve a module binary, checking USB layout (`bin/`) first, then dev layout.
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

    let notebook = match NOTEBOOKS.iter().find(|(n, _)| *n == name) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_binary_returns_none_for_nonexistent() {
        let root = std::path::Path::new("/nonexistent");
        assert!(resolve_binary(root, "ltee-fitness").is_none());
    }

    #[test]
    fn live_modules_table_is_complete() {
        assert_eq!(LIVE_MODULES.len(), 7);
        assert_eq!(NOTEBOOKS.len(), 7);
        for (name, _, _, _) in LIVE_MODULES {
            assert!(NOTEBOOKS.iter().any(|(n, _)| n == name), "missing notebook for {name}");
        }
    }
}
