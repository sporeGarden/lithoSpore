// SPDX-License-Identifier: AGPL-3.0-or-later

//! Structural completeness — `data/`/`outputs/`/`configs/` alignment and visual evidence in `figures/`.

use std::fs;
use std::path::Path;

use super::{Finding, Severity};

pub(super) fn check_figures_layer(root: &Path, findings: &mut Vec<Finding>) {
    let figures_dir = root.join("figures");
    let outputs_dir = root.join("outputs");

    if !outputs_dir.exists() {
        return;
    }

    // Count output modules that have FES data
    let mut fes_modules = 0;
    if let Ok(entries) = fs::read_dir(&outputs_dir) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let has_fes = fs::read_dir(entry.path())
                .into_iter()
                .flatten()
                .flatten()
                .any(|f| {
                    let name = f.file_name().to_string_lossy().to_string();
                    name.starts_with("fes_")
                        && name
                            .rsplit_once('.')
                            .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("dat"))
                });
            if has_fes {
                fes_modules += 1;
            }
        }
    }

    if fes_modules == 0 {
        return;
    }

    if !figures_dir.exists() {
        findings.push(Finding {
            id: "FIGURES-MISSING".to_string(),
            severity: Severity::Low,
            category: "Visual Evidence",
            message: format!(
                "{fes_modules} modules have FES data but no figures/ directory exists"
            ),
            fix: "Generate figures: run `litho emit-pseudospore` (auto-generates from outputs/) or re-run the module notebooks".to_string(),
        });
        return;
    }

    let png_count = fs::read_dir(&figures_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .count();

    if png_count == 0 {
        findings.push(Finding {
            id: "FIGURES-EMPTY".to_string(),
            severity: Severity::Low,
            category: "Visual Evidence",
            message: "figures/ directory exists but contains no PNG files".to_string(),
            fix: "Generate figures: run `litho emit-pseudospore` (auto-generates from outputs/) or re-run the module notebooks".to_string(),
        });
    }
}

pub(super) fn check_module_completeness(root: &Path, findings: &mut Vec<Finding>) {
    let data_dir = root.join("data");
    let outputs_dir = root.join("outputs");
    let configs_dir = root.join("configs");

    if !data_dir.exists() {
        findings.push(Finding {
            id: "DATA-MISSING".to_string(),
            severity: Severity::Medium,
            category: "Zero-Trust",
            message: "No data/ directory — cannot verify derivation independently".to_string(),
            fix: "Add data/<module>/HILLS files for zero-trust verification".to_string(),
        });
        return;
    }

    // Check each module in outputs/ has corresponding data/
    if let Ok(modules) = fs::read_dir(&outputs_dir) {
        for module in modules.flatten() {
            if !module.path().is_dir() {
                continue;
            }
            let mod_name = module.file_name().to_string_lossy().to_string();
            let data_module = data_dir.join(&mod_name);
            if !data_module.exists() {
                findings.push(Finding {
                    id: format!("DATA-GAP-{mod_name}"),
                    severity: Severity::Medium,
                    category: "Zero-Trust",
                    message: format!(
                        "outputs/{mod_name} exists but data/{mod_name} is missing — cannot verify derivation"
                    ),
                    fix: format!(
                        "Add data/{mod_name}/HILLS or mark module as reference-only in scope.toml"
                    ),
                });
            }
        }
    }

    // Check configs/ coverage
    if configs_dir.exists()
        && let Ok(modules) = fs::read_dir(&outputs_dir)
    {
        for module in modules.flatten() {
            if !module.path().is_dir() {
                continue;
            }
            let mod_name = module.file_name().to_string_lossy().to_string();
            let config_module = configs_dir.join(&mod_name);
            if !config_module.exists() {
                findings.push(Finding {
                    id: format!("CONFIG-GAP-{mod_name}"),
                    severity: Severity::Low,
                    category: "Completeness",
                    message: format!("outputs/{mod_name} has no matching configs/ entry"),
                    fix: format!("Add configs/{mod_name}/plumed.dat"),
                });
            }
        }
    }
}
