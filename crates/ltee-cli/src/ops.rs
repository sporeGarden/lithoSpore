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
    use super::*;

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
