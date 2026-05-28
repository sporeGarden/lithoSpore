// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module registry — domain-agnostic module resolution for the lithoSpore chassis.
//!
//! Provides a unified way to discover modules from `scope.toml` `[[module]]`
//! entries, with compiled LTEE constants as fallback. All subcommands
//! (validate, parity, status, chaos, deploy-test, visualize) import from
//! here instead of maintaining their own module tables.

use std::path::Path;

/// A runtime-resolved module entry.
#[derive(Debug, Clone)]
pub(crate) struct ModuleEntry {
    pub name: String,
    pub binary: String,
    pub data_dir: String,
    pub expected: String,
    pub tier1_notebook: String,
}

/// LTEE instance: compiled fallback when scope.toml has no `[[module]]` entries.
pub(crate) const LTEE_MODULES: &[(&str, &str, &str, &str)] = &[
    (
        "power_law_fitness",
        "ltee-fitness",
        "artifact/data/wiser_2013",
        "validation/expected/module1_fitness.json",
    ),
    (
        "mutation_accumulation",
        "ltee-mutations",
        "artifact/data/barrick_2009",
        "validation/expected/module2_mutations.json",
    ),
    (
        "allele_trajectories",
        "ltee-alleles",
        "artifact/data/good_2017",
        "validation/expected/module3_alleles.json",
    ),
    (
        "citrate_innovation",
        "ltee-citrate",
        "artifact/data/blount_2012",
        "validation/expected/module4_citrate.json",
    ),
    (
        "biobrick_burden",
        "ltee-biobricks",
        "artifact/data/biobricks_2024",
        "validation/expected/module5_biobricks.json",
    ),
    (
        "breseq_264_genomes",
        "ltee-breseq",
        "artifact/data/tenaillon_2016",
        "validation/expected/module6_breseq.json",
    ),
    (
        "anderson_qs_predictions",
        "ltee-anderson",
        "artifact/data/anderson_predictions",
        "validation/expected/module7_anderson.json",
    ),
];

pub(crate) const LTEE_NOTEBOOKS: &[(&str, &str)] = &[
    (
        "power_law_fitness",
        "notebooks/module1_fitness/power_law_fitness.py",
    ),
    (
        "mutation_accumulation",
        "notebooks/module2_mutations/mutation_accumulation.py",
    ),
    (
        "allele_trajectories",
        "notebooks/module3_alleles/allele_trajectories.py",
    ),
    (
        "citrate_innovation",
        "notebooks/module4_citrate/citrate_innovation.py",
    ),
    (
        "biobrick_burden",
        "notebooks/module5_biobricks/biobrick_burden.py",
    ),
    (
        "breseq_264_genomes",
        "notebooks/module6_breseq/breseq_comparison.py",
    ),
    (
        "anderson_qs_predictions",
        "notebooks/module7_anderson/anderson_predictions.py",
    ),
];

pub(crate) type ModuleFn = fn(&str, &str, u8) -> litho_core::ModuleResult;

/// Compiled dispatch table. Modules are linked at compile time via Cargo deps.
/// A future evolution would use dynamic loading or feature gates per instance.
pub(crate) const MODULE_DISPATCH: &[(&str, ModuleFn)] = &[
    ("ltee-fitness", ltee_fitness::run_validation),
    ("ltee-mutations", ltee_mutations::run_validation),
    ("ltee-alleles", ltee_alleles::run_validation),
    ("ltee-citrate", ltee_citrate::run_validation),
    ("ltee-biobricks", ltee_biobricks::run_validation),
    ("ltee-breseq", ltee_breseq::run_validation),
    ("ltee-anderson", ltee_anderson::run_validation),
];

/// Load modules from `scope.toml` when present, otherwise the compiled LTEE table.
pub(crate) fn load_modules(scope_path: Option<&Path>) -> Vec<ModuleEntry> {
    if let Some(path) = scope_path
        && let Ok(scope) = litho_core::ScopeManifest::load(path)
        && !scope.module.is_empty()
    {
        return scope
            .module
            .iter()
            .map(|m| ModuleEntry {
                name: m.name.clone(),
                binary: m.binary.clone(),
                data_dir: m.data_dir.clone(),
                expected: m.expected.clone(),
                tier1_notebook: m.tier1_notebook.clone(),
            })
            .collect();
    }
    compiled_module_fallback()
}

fn compiled_module_fallback() -> Vec<ModuleEntry> {
    LTEE_MODULES
        .iter()
        .map(|(name, binary, data_dir, expected)| ModuleEntry {
            name: name.to_string(),
            binary: binary.to_string(),
            data_dir: data_dir.to_string(),
            expected: expected.to_string(),
            tier1_notebook: find_notebook(name),
        })
        .collect()
}

/// Load the module table. Priority:
///
/// 1. `scope.toml` `[[module]]` entries (domain-agnostic)
/// 2. `scope.toml` springs × `data.toml` datasets (legacy)
/// 3. Compiled `LTEE_MODULES` fallback
pub(crate) fn load_module_table(root: &Path) -> Vec<ModuleEntry> {
    let scope_path = root.join("artifact/scope.toml");

    let from_scope = load_modules(Some(&scope_path));
    if let Ok(scope) = litho_core::ScopeManifest::load(&scope_path) {
        if !scope.module.is_empty() {
            return from_scope;
        }

        // Priority 2: derive from springs × data.toml
        if let Ok(data_content) = std::fs::read_to_string(root.join("artifact/data.toml"))
            && let Ok(data_toml) = data_content.parse::<toml::Value>()
        {
            let datasets = data_toml.get("dataset").and_then(|v| v.as_array());
            let module_bins = scope.module_binaries();
            let mut entries = Vec::new();

            for bin_name in &module_bins {
                let matching: Vec<&toml::Value> = datasets
                    .map(|arr| {
                        arr.iter()
                            .filter(|d| d.get("module").and_then(|v| v.as_str()) == Some(bin_name))
                            .collect()
                    })
                    .unwrap_or_default();

                let ds = matching
                    .iter()
                    .find(|d| {
                        d.get("local_path")
                            .and_then(|v| v.as_str())
                            .is_some_and(|p| root.join(p.trim_end_matches('/')).exists())
                    })
                    .or_else(|| matching.first());

                let data_dir = ds
                    .and_then(|d| d.get("local_path").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .trim_end_matches('/')
                    .to_string();

                let expected = find_expected_json(root, bin_name);

                let name = derive_logical_name(bin_name);

                if !data_dir.is_empty() || !expected.is_empty() {
                    entries.push(ModuleEntry {
                        name,
                        binary: bin_name.to_string(),
                        data_dir,
                        expected,
                        tier1_notebook: find_notebook(&name_from_binary(bin_name)),
                    });
                }
            }

            if !entries.is_empty() {
                return entries;
            }
        }
    }

    // Priority 3: compiled LTEE fallback
    compiled_module_fallback()
}

/// Load the guideStone identity name from scope.toml, with fallback.
pub(crate) fn load_scope_name(root: &Path) -> String {
    litho_core::ScopeManifest::load(&root.join("artifact/scope.toml")).map_or_else(
        |_| "ltee-guidestone".to_string(),
        |s| s.guidestone.name.clone(),
    )
}

/// Load the scope manifest (if available).
pub(crate) fn load_scope(root: &Path) -> Option<litho_core::ScopeManifest> {
    litho_core::ScopeManifest::load(&root.join("artifact/scope.toml")).ok()
}

/// Domain-agnostic name matching: does a result module name correspond
/// to a given binary/module identifier? Uses the module registry to
/// resolve, falling back to the legacy LTEE mapping.
pub(crate) fn module_name_matches(
    registry: &[ModuleEntry],
    result_name: &str,
    target_module: &str,
) -> bool {
    // Check registry: binary → logical name
    if let Some(entry) = registry.iter().find(|e| e.binary == target_module) {
        return result_name == entry.name;
    }
    // Fallback: strip common prefixes and compare
    let normalized_target = derive_logical_name(target_module);
    result_name == normalized_target
}

/// Derive a logical module name from a binary crate name.
/// Strips known prefixes and converts hyphens to underscores.
fn derive_logical_name(binary: &str) -> String {
    let stripped = binary
        .strip_prefix("ltee-")
        .or_else(|| binary.strip_prefix("milc-"))
        .or_else(|| binary.strip_prefix("lattice-"))
        .unwrap_or(binary);
    stripped.replace('-', "_")
}

fn name_from_binary(binary: &str) -> String {
    derive_logical_name(binary)
}

/// Find notebook path from `LTEE_NOTEBOOKS` or return empty.
fn find_notebook(logical_name: &str) -> String {
    LTEE_NOTEBOOKS
        .iter()
        .find(|(n, _)| *n == logical_name)
        .map(|(_, p)| p.to_string())
        .unwrap_or_default()
}

/// Find the expected JSON file for a module by scanning `validation/expected/`.
pub(crate) fn find_expected_json(root: &Path, module_binary: &str) -> String {
    let expected_dir = root.join("validation/expected");
    if !expected_dir.is_dir() {
        return String::new();
    }
    let entries = match std::fs::read_dir(&expected_dir) {
        Ok(e) => e,
        Err(_) => return String::new(),
    };
    let suffix = module_binary.replace('-', "_");
    let short = suffix
        .strip_prefix("ltee_")
        .or_else(|| suffix.strip_prefix("milc_"))
        .or_else(|| suffix.strip_prefix("lattice_"))
        .unwrap_or(&suffix);
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".json")
            && (name_str.contains(&suffix) || name_str.contains(short))
            && let Ok(rel) = entry.path().strip_prefix(root)
        {
            return rel.to_string_lossy().to_string();
        }
    }
    String::new()
}

/// Dispatch to a module's in-process validation function by binary name.
pub(crate) fn dispatch_module(
    binary: &str,
    data_dir: &str,
    expected: &str,
    max_tier: u8,
) -> litho_core::ModuleResult {
    if let Some((_, func)) = MODULE_DISPATCH.iter().find(|(name, _)| *name == binary) {
        func(data_dir, expected, max_tier)
    } else {
        litho_core::ModuleResult {
            name: binary.to_string(),
            status: litho_core::ValidationStatus::Skip,
            tier: 0,
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some(format!("No in-process dispatch for {binary}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ltee_tables_consistent() {
        assert_eq!(LTEE_MODULES.len(), 7);
        assert_eq!(LTEE_NOTEBOOKS.len(), 7);
        assert_eq!(MODULE_DISPATCH.len(), 7);
        for (name, _, _, _) in LTEE_MODULES {
            assert!(
                LTEE_NOTEBOOKS.iter().any(|(n, _)| n == name),
                "missing notebook for {name}"
            );
        }
        for (_, binary, _, _) in LTEE_MODULES {
            assert!(
                MODULE_DISPATCH.iter().any(|(n, _)| n == binary),
                "missing dispatch for {binary}"
            );
        }
    }

    #[test]
    fn derive_logical_name_strips_prefixes() {
        assert_eq!(derive_logical_name("ltee-fitness"), "fitness");
        assert_eq!(derive_logical_name("milc-eos"), "eos");
        assert_eq!(derive_logical_name("lattice-pressure"), "pressure");
        assert_eq!(derive_logical_name("plain-module"), "plain_module");
    }

    #[test]
    fn module_name_matches_from_registry() {
        let registry = vec![ModuleEntry {
            name: "power_law_fitness".into(),
            binary: "ltee-fitness".into(),
            data_dir: String::new(),
            expected: String::new(),
            tier1_notebook: String::new(),
        }];
        assert!(module_name_matches(
            &registry,
            "power_law_fitness",
            "ltee-fitness"
        ));
        assert!(!module_name_matches(
            &registry,
            "mutation_accumulation",
            "ltee-fitness"
        ));
    }

    #[test]
    fn module_name_matches_fallback() {
        let registry: Vec<ModuleEntry> = Vec::new();
        assert!(module_name_matches(&registry, "fitness", "ltee-fitness"));
        assert!(module_name_matches(&registry, "eos", "milc-eos"));
    }
}
