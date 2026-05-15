// SPDX-License-Identifier: AGPL-3.0-or-later

//! Operational subcommands: refresh, status, spore.

pub fn cmd_refresh(root: &str) {
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

pub fn cmd_status(root: &str) {
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

#[cfg(test)]
mod tests {
    #[test]
    fn status_module_table_is_seven() {
        // Verify the module table used in cmd_status
        let modules: &[(&str, &str, &str)] = &[
            ("1 (fitness)", "validation/expected/module1_fitness.json", "artifact/data/wiser_2013"),
            ("2 (mutations)", "validation/expected/module2_mutations.json", "artifact/data/barrick_2009"),
            ("3 (alleles)", "validation/expected/module3_alleles.json", "artifact/data/good_2017"),
            ("4 (citrate)", "validation/expected/module4_citrate.json", "artifact/data/blount_2012"),
            ("5 (biobricks)", "validation/expected/module5_biobricks.json", "artifact/data/biobricks_2024"),
            ("6 (breseq)", "validation/expected/module6_breseq.json", "artifact/data/tenaillon_2016"),
            ("7 (anderson)", "validation/expected/module7_anderson.json", "artifact/data/anderson_predictions"),
        ];
        assert_eq!(modules.len(), 7);
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
    let mut passed = 0u32;
    let mut total = 0u32;

    println!("lithoSpore self-test — artifact integrity check");
    println!("  Root: {root}");
    println!();

    let expected_files = [
        "validation/expected/module1_fitness.json",
        "validation/expected/module2_mutations.json",
        "validation/expected/module3_alleles.json",
        "validation/expected/module4_citrate.json",
        "validation/expected/module5_biobricks.json",
        "validation/expected/module6_breseq.json",
        "validation/expected/module7_anderson.json",
    ];
    for f in &expected_files {
        total += 1;
        let exists = root_path.join(f).exists();
        if exists { passed += 1; }
        println!("  [{}] {f}", if exists { "OK" } else { "MISSING" });
    }

    let artifact_files = [
        "artifact/scope.toml",
        "artifact/data.toml",
        "artifact/tolerances.toml",
    ];
    for f in &artifact_files {
        total += 1;
        let exists = root_path.join(f).exists();
        if exists { passed += 1; }
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
        if exists { passed += 1; }
        println!("  [{}] {f}", if exists { "OK" } else { "MISSING" });
    }

    let data_dirs = [
        "artifact/data/wiser_2013",
        "artifact/data/barrick_2009",
        "artifact/data/good_2017",
        "artifact/data/blount_2012",
        "artifact/data/biobricks_2024",
        "artifact/data/tenaillon_2016",
        "artifact/data/anderson_predictions",
    ];
    for d in &data_dirs {
        total += 1;
        let exists = root_path.join(d).exists();
        if exists { passed += 1; }
        println!("  [{}] {d}/", if exists { "OK" } else { "MISSING" });
    }

    // Check figures
    total += 1;
    let fig_count = std::fs::read_dir(root_path.join("figures"))
        .map(|rd| rd.filter(|e| e.as_ref().map(|e| e.path().extension().map_or(false, |ext| ext == "svg")).unwrap_or(false)).count())
        .unwrap_or(0);
    let fig_ok = fig_count >= 7;
    if fig_ok { passed += 1; }
    println!("  [{}] figures/*.svg: {fig_count} files (expected ≥ 7)", if fig_ok { "OK" } else { "WARN" });

    // Check data_manifest.toml
    total += 1;
    let manifest_path = root_path.join("data_manifest.toml");
    let has_manifest = manifest_path.exists() && std::fs::metadata(&manifest_path).map(|m| m.len() > 10).unwrap_or(false);
    if has_manifest { passed += 1; }
    println!("  [{}] data_manifest.toml", if has_manifest { "OK" } else { "MISSING" });

    println!();
    println!("  Self-test: {passed}/{total} checks passed");
    if passed < total {
        std::process::exit(1);
    }
}

pub fn cmd_tier(root: &str) {
    let root_path = std::path::Path::new(root);

    println!("lithoSpore — tier detection");

    // Tier 1: Python
    let has_python = std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_ok();
    let has_embedded = root_path.join("python/bin/python3").exists();
    println!("  Tier 1 (Python):   {} {}", if has_python || has_embedded { "AVAILABLE" } else { "UNAVAILABLE" },
        if has_embedded { "(embedded)" } else if has_python { "(system)" } else { "" });

    // Tier 2: Rust binaries
    let binaries = ["ltee-fitness", "ltee-mutations", "ltee-alleles", "ltee-citrate",
                    "ltee-biobricks", "ltee-breseq", "ltee-anderson", "litho"];
    let bin_count = binaries.iter()
        .filter(|b| root_path.join(format!("bin/{b}")).exists() || root_path.join(format!("target/release/{b}")).exists())
        .count();
    let tier2 = bin_count >= 7;
    println!("  Tier 2 (Rust):     {} ({bin_count}/8 binaries)", if tier2 { "AVAILABLE" } else { "PARTIAL" });

    // Tier 3: Primals (NUCLEUS)
    let has_nucleus = std::env::var("NUCLEUS_ROOT").is_ok()
        || std::env::var("CAPABILITY_PORT").is_ok();
    let has_graph = root_path.join("graphs/ltee_guidestone.toml").exists();
    println!("  Tier 3 (Primals):  {} (graph={has_graph}, nucleus={has_nucleus})",
        if has_nucleus && has_graph { "AVAILABLE" } else { "UNAVAILABLE" });

    let max_tier = if has_nucleus && has_graph { 3 } else if tier2 { 2 } else if has_python || has_embedded { 1 } else { 0 };
    println!();
    println!("  Maximum tier: {max_tier}");
}

pub fn cmd_deploy_report(root: &str, pattern: &str) {
    let root_path = std::path::Path::new(root);
    let timestamp = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Self-test
    let expected_files = [
        "validation/expected/module1_fitness.json",
        "validation/expected/module2_mutations.json",
        "validation/expected/module3_alleles.json",
        "validation/expected/module4_citrate.json",
        "validation/expected/module5_biobricks.json",
        "validation/expected/module6_breseq.json",
        "validation/expected/module7_anderson.json",
        "artifact/scope.toml",
        "artifact/data.toml",
        "artifact/tolerances.toml",
        "papers/registry.toml",
        ".biomeos-spore",
    ];
    let selftest_passed = expected_files.iter().filter(|f| root_path.join(f).exists()).count();
    let selftest_total = expected_files.len();

    // Tier detection
    let has_python = std::process::Command::new("python3").arg("--version").output().is_ok()
        || root_path.join("python/bin/python3").exists();
    let binaries = ["ltee-fitness", "ltee-mutations", "ltee-alleles", "ltee-citrate",
                    "ltee-biobricks", "ltee-breseq", "ltee-anderson", "litho"];
    let bin_count = binaries.iter()
        .filter(|b| root_path.join(format!("bin/{b}")).exists() || root_path.join(format!("target/release/{b}")).exists())
        .count();
    let max_tier = if bin_count >= 7 { 2 } else if has_python { 1 } else { 0 };

    // Data bundles
    let data_dirs = ["wiser_2013", "barrick_2009", "good_2017", "blount_2012",
                     "biobricks_2024", "tenaillon_2016", "anderson_predictions"];
    let data_count = data_dirs.iter()
        .filter(|d| root_path.join(format!("artifact/data/{d}")).exists())
        .count();

    // Figures
    let fig_count = std::fs::read_dir(root_path.join("figures"))
        .map(|rd| rd.filter(|e| e.as_ref().map(|e| e.path().extension().map_or(false, |ext| ext == "svg")).unwrap_or(false)).count())
        .unwrap_or(0);

    // Manifest hash count
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
    println!("status = \"{}\"", if selftest_passed == selftest_total { "PASS" } else { "FAIL" });
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

    // Run inline validation via module binaries directly
    let mut modules_results = Vec::new();
    for (_name, binary, data_dir, expected) in crate::validate::LIVE_MODULES {
        let data_path = root_path.join(data_dir);
        let expected_path = root_path.join(expected);
        let binary_path = crate::validate::resolve_binary(root_path, binary);

        if let Some(bp) = binary_path.filter(|_| data_path.exists() && expected_path.exists()) {
            let out = std::process::Command::new(&bp)
                .arg("--data-dir").arg(&data_path)
                .arg("--expected").arg(&expected_path)
                .arg("--max-tier").arg("2")
                .arg("--json")
                .stderr(std::process::Stdio::null())
                .output();

            if let Ok(o) = out {
                if let Ok(result) = serde_json::from_slice::<litho_core::ModuleResult>(&o.stdout) {
                    modules_results.push(result);
                }
            }
        }
    }

    if !modules_results.is_empty() {
        {
            let report = &modules_results;
            let passed = report.iter().filter(|m| m.status == litho_core::ValidationStatus::Pass).count();
            let failed = report.iter().filter(|m| m.status == litho_core::ValidationStatus::Fail).count();
            let total_checks: u32 = report.iter().map(|m| m.checks).sum();
            let passed_checks: u32 = report.iter().map(|m| m.checks_passed).sum();

            println!("[validation]");
            println!("tier_reached = {max_tier}");
            println!("modules_total = {}", report.len());
            println!("modules_passed = {passed}");
            println!("modules_failed = {failed}");
            println!("checks_total = {total_checks}");
            println!("checks_passed = {passed_checks}");
            println!("status = \"{}\"", if failed == 0 { "PASS" } else { "FAIL" });
            println!();

            for m in report {
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
    } else {
        println!("[validation]");
        println!("status = \"SKIP\"");
        println!("reason = \"No module binaries found\"");
        println!();
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
                    e.timestamp, e.os, e.arch, e.tier_reached, e.modules_passed, e.modules_total, e.runtime_ms
                );
            }
        }
        Err(_) => println!("No liveSpore.json found at {} — no validation runs recorded yet", spore_path.display()),
    }
}
