// SPDX-License-Identifier: AGPL-3.0-or-later

//! Operational subcommands: refresh, status, spore.

use crate::registry;

pub fn cmd_refresh(root: &str) {
    println!("litho refresh: re-fetching all datasets via litho fetch...");
    crate::fetch::run(root, None, true, false);
}

pub fn cmd_status(root: &str) {
    let root_path = std::path::Path::new(root);
    let scope_name = registry::load_scope_name(root_path);
    let modules = registry::load_module_table(root_path);

    println!("lithoSpore v{} — {scope_name}", env!("CARGO_PKG_VERSION"));
    println!("  Artifact root: {root}");

    let mut live = 0_u32;
    let total = modules.len();
    for entry in &modules {
        let has_expected = root_path.join(&entry.expected).exists();
        let has_data = root_path.join(&entry.data_dir).exists();
        if has_expected {
            live += 1;
        }
        println!(
            "  Module {:<25} expected={has_expected} data={has_data}",
            entry.name
        );
    }

    println!(
        "  Modules: {total} ({live} live, {} scaffold)",
        total as u32 - live
    );
    println!("  Tier support: 1 (Python) + 2 (Rust) + 3 (Primal/NUCLEUS)");
}

#[cfg(test)]
mod tests {
    #[test]
    fn registry_module_table_is_seven() {
        assert_eq!(crate::registry::LTEE_MODULES.len(), 7);
    }

    #[test]
    fn resolve_livespore_fallback() {
        let root = std::path::Path::new("/nonexistent");
        let path = crate::resolve_livespore(root);
        assert!(path.to_str().unwrap().contains("artifact/liveSpore.json"));
    }
}

pub fn cmd_self_test(root: &str) {
    let root_path = std::path::Path::new(root);
    let modules = registry::load_module_table(root_path);
    let mut passed = 0u32;
    let mut total = 0u32;

    println!("lithoSpore self-test — artifact integrity check");
    println!("  Root: {root}");
    println!();

    for entry in &modules {
        total += 1;
        let exists = root_path.join(&entry.expected).exists();
        if exists {
            passed += 1;
        }
        println!(
            "  [{}] {}",
            if exists { "OK" } else { "MISSING" },
            entry.expected
        );
    }

    let artifact_files = [
        "artifact/scope.toml",
        "artifact/data.toml",
        "artifact/tolerances.toml",
    ];
    for f in &artifact_files {
        total += 1;
        let exists = root_path.join(f).exists();
        if exists {
            passed += 1;
        }
        println!("  [{}] {f}", if exists { "OK" } else { "MISSING" });
    }

    let doc_files = [
        "papers/registry.toml",
        "papers/READING_ORDER.md",
        "GETTING_STARTED.md",
        "SCIENCE.md",
    ];
    for f in &doc_files {
        total += 1;
        let exists = root_path.join(f).exists();
        if exists {
            passed += 1;
        }
        println!("  [{}] {f}", if exists { "OK" } else { "MISSING" });
    }

    for entry in &modules {
        total += 1;
        let exists = root_path.join(&entry.data_dir).exists();
        if exists {
            passed += 1;
        }
        println!(
            "  [{}] {}/",
            if exists { "OK" } else { "MISSING" },
            entry.data_dir
        );
    }

    // Check figures
    total += 1;
    let fig_count = std::fs::read_dir(root_path.join("figures"))
        .map(|rd| {
            rd.filter(|e| {
                e.as_ref()
                    .map(|e| e.path().extension().is_some_and(|ext| ext == "svg"))
                    .unwrap_or(false)
            })
            .count()
        })
        .unwrap_or(0);
    let fig_ok = fig_count >= 7;
    if fig_ok {
        passed += 1;
    }
    println!(
        "  [{}] figures/*.svg: {fig_count} files (expected ≥ 7)",
        if fig_ok { "OK" } else { "WARN" }
    );

    // Check data_manifest.toml
    total += 1;
    let manifest_path = root_path.join("data_manifest.toml");
    let has_manifest = manifest_path.exists()
        && std::fs::metadata(&manifest_path)
            .map(|m| m.len() > 10)
            .unwrap_or(false);
    if has_manifest {
        passed += 1;
    }
    println!(
        "  [{}] data_manifest.toml",
        if has_manifest { "OK" } else { "MISSING" }
    );

    println!();
    println!("  Self-test: {passed}/{total} checks passed");
    if passed < total {
        std::process::exit(1);
    }
}

pub fn cmd_tier(root: &str) {
    let root_path = std::path::Path::new(root);
    let modules = registry::load_module_table(root_path);

    println!("lithoSpore — tier detection");

    // Tier 1: Python
    let has_python = std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_ok();
    let has_embedded = root_path.join("python/bin/python3").exists();
    println!(
        "  Tier 1 (Python):   {} {}",
        if has_python || has_embedded {
            "AVAILABLE"
        } else {
            "UNAVAILABLE"
        },
        if has_embedded {
            "(embedded)"
        } else if has_python {
            "(system)"
        } else {
            ""
        }
    );

    // Tier 2: Rust binaries
    let mut tier2_bins: Vec<&str> = modules.iter().map(|e| e.binary.as_str()).collect();
    tier2_bins.push("litho");
    let bin_count = tier2_bins
        .iter()
        .filter(|b| {
            root_path.join(format!("bin/{b}")).exists()
                || root_path.join(format!("target/release/{b}")).exists()
        })
        .count();
    let tier2 = bin_count >= modules.len();
    println!(
        "  Tier 2 (Rust):     {} ({bin_count}/{} binaries)",
        if tier2 { "AVAILABLE" } else { "PARTIAL" },
        modules.len() + 1
    );

    // Tier 3: Primals (NUCLEUS)
    let has_nucleus = std::env::var(litho_core::env_vars::NUCLEUS_ROOT).is_ok()
        || std::env::var(litho_core::env_vars::CAPABILITY_PORT).is_ok();
    let graph_file = registry::load_scope(root_path)
        .and_then(|s| {
            let f = &s.guidestone.graph_file;
            if f.is_empty() { None } else { Some(f.clone()) }
        })
        .unwrap_or_else(|| "graphs/ltee_guidestone.toml".to_string());
    let has_graph = root_path.join(&graph_file).exists();
    println!(
        "  Tier 3 (Primals):  {} (graph={has_graph}, nucleus={has_nucleus})",
        if has_nucleus && has_graph {
            "AVAILABLE"
        } else {
            "UNAVAILABLE"
        }
    );

    let max_tier = if has_nucleus && has_graph {
        3
    } else if tier2 {
        2
    } else {
        i32::from(has_python || has_embedded)
    };
    println!();
    println!("  Maximum tier: {max_tier}");
}

pub fn cmd_deploy_report(root: &str, pattern: &str) {
    let root_path = std::path::Path::new(root);
    let modules = registry::load_module_table(root_path);
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut selftest_files: Vec<String> = modules.iter().map(|e| e.expected.clone()).collect();
    for extra in [
        "artifact/scope.toml",
        "artifact/data.toml",
        "artifact/tolerances.toml",
        "papers/registry.toml",
        ".biomeos-spore",
    ] {
        selftest_files.push(extra.to_string());
    }
    let selftest_passed = selftest_files
        .iter()
        .filter(|f| root_path.join(f).exists())
        .count();
    let selftest_total = selftest_files.len();

    let has_python = std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_ok()
        || root_path.join("python/bin/python3").exists();
    let deploy_bins: Vec<&str> = modules
        .iter()
        .map(|e| e.binary.as_str())
        .chain(std::iter::once("litho"))
        .collect();
    let bin_count = deploy_bins
        .iter()
        .filter(|b| {
            root_path.join(format!("bin/{b}")).exists()
                || root_path.join(format!("target/release/{b}")).exists()
        })
        .count();
    let max_tier = if bin_count >= modules.len() {
        2
    } else {
        i32::from(has_python)
    };

    let data_count = modules
        .iter()
        .filter(|e| root_path.join(&e.data_dir).exists())
        .count();

    let fig_count = std::fs::read_dir(root_path.join("figures"))
        .map(|rd| {
            rd.filter(|e| {
                e.as_ref()
                    .map(|e| e.path().extension().is_some_and(|ext| ext == "svg"))
                    .unwrap_or(false)
            })
            .count()
        })
        .unwrap_or(0);

    let manifest_path = root_path.join("data_manifest.toml");
    let hash_count = std::fs::read_to_string(&manifest_path)
        .map(|c| c.matches("[[file]]").count())
        .unwrap_or(0);

    let os_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);

    println!("# lithoSpore Deployment Report");
    println!("# Generated by `litho deploy-report`");
    println!();
    println!("[meta]");
    println!("timestamp = \"{timestamp}\"");
    println!("deployment_pattern = \"{pattern}\"");
    println!("os = \"{os_info}\"");
    println!("artifact_root = \"{}\"", root_path.display());
    println!("version = \"{}\"", env!("CARGO_PKG_VERSION"));
    println!();
    println!("[self_test]");
    println!("checks_passed = {selftest_passed}");
    println!("checks_total = {selftest_total}");
    println!(
        "status = \"{}\"",
        if selftest_passed == selftest_total {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!();
    println!("[tier]");
    println!("max_tier = {max_tier}");
    println!("binaries = {bin_count}");
    println!("python = {has_python}");
    println!();
    println!("[artifact]");
    println!("datasets = {data_count}");
    println!("figures = {fig_count}");
    println!("manifest_files = {hash_count}");
    println!();

    let mut modules_results = Vec::new();
    for entry in &modules {
        let data_path = root_path.join(&entry.data_dir);
        let expected_path = root_path.join(&entry.expected);

        if data_path.exists() && expected_path.exists() {
            let result = registry::dispatch_module(
                &entry.binary,
                data_path.to_str().unwrap_or(&entry.data_dir),
                expected_path.to_str().unwrap_or(&entry.expected),
                2,
            );
            modules_results.push(result);
        }
    }

    if modules_results.is_empty() {
        println!("[validation]");
        println!("status = \"SKIP\"");
        println!("reason = \"No module binaries found\"");
        println!();
    } else {
        let passed = modules_results
            .iter()
            .filter(|m| m.status == litho_core::ValidationStatus::Pass)
            .count();
        let failed = modules_results
            .iter()
            .filter(|m| m.status == litho_core::ValidationStatus::Fail)
            .count();
        let total_checks: u32 = modules_results.iter().map(|m| m.checks).sum();
        let passed_checks: u32 = modules_results.iter().map(|m| m.checks_passed).sum();

        println!("[validation]");
        println!("tier_reached = {max_tier}");
        println!("modules_total = {}", modules_results.len());
        println!("modules_passed = {passed}");
        println!("modules_failed = {failed}");
        println!("checks_total = {total_checks}");
        println!("checks_passed = {passed_checks}");
        println!("status = \"{}\"", if failed == 0 { "PASS" } else { "FAIL" });
        println!();

        for m in &modules_results {
            let status = match m.status {
                litho_core::ValidationStatus::Pass => "PASS",
                litho_core::ValidationStatus::Fail => "FAIL",
                litho_core::ValidationStatus::Skip => "SKIP",
            };
            println!("[[module]]");
            println!("name = \"{}\"", m.name);
            println!("status = \"{status}\"");
            println!("checks = {}", m.checks);
            println!("checks_passed = {}", m.checks_passed);
            println!("runtime_ms = {}", m.runtime_ms);
            println!();
        }
    }
}

pub fn cmd_spore(root: &str) {
    let spore_path = crate::resolve_livespore(std::path::Path::new(root));
    match std::fs::read_to_string(&spore_path) {
        Ok(contents) => {
            let entries: Vec<litho_core::LiveSporeEntry> =
                serde_json::from_str(&contents).unwrap_or_default();
            println!("liveSpore: {} validation runs recorded", entries.len());
            for e in &entries {
                println!(
                    "  {} — {} {} tier {} ({}/{} modules, {}ms)",
                    e.timestamp,
                    e.os,
                    e.arch,
                    e.tier_reached,
                    e.modules_passed,
                    e.modules_total,
                    e.runtime_ms
                );
            }
        }
        Err(_) => println!(
            "No liveSpore.json found at {} — no validation runs recorded yet",
            spore_path.display()
        ),
    }
}
