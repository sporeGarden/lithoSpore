// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho emit-pseudospore` — assemble a pseudoSpore directory from module outputs.
//!
//! Generates the standard directory structure, computes BLAKE3 checksums,
//! captures environment metadata, and creates a README from scope metadata.

use litho_core::pseudospore;
use std::path::Path;

pub fn run(
    name: &str,
    version: &str,
    origin: &str,
    output_dir: &str,
    outputs_dir: Option<&str>,
    configs_dir: Option<&str>,
    braids_dir: Option<&str>,
) {
    let out = Path::new(output_dir);
    let dir_name = format!("pseudoSpore_{name}_v{version}");
    let root = out.join(&dir_name);

    println!("=== litho emit-pseudospore ===");
    println!("  Name:    {name}");
    println!("  Version: {version}");
    println!("  Origin:  {origin}");
    println!("  Output:  {}", root.display());
    println!();

    // Create directory structure
    std::fs::create_dir_all(root.join("receipts")).expect("Failed to create receipts/");
    std::fs::create_dir_all(root.join("provenance/braids")).expect("Failed to create provenance/braids/");
    std::fs::create_dir_all(root.join("outputs")).expect("Failed to create outputs/");
    std::fs::create_dir_all(root.join("configs")).expect("Failed to create configs/");

    // 1. Generate scope.toml
    let scope_content = generate_scope(name, version, origin);
    std::fs::write(root.join("scope.toml"), &scope_content).expect("Failed to write scope.toml");
    println!("  [+] scope.toml");

    // 2. Generate stub validation.json
    let validation_content = generate_validation_stub(name, version);
    std::fs::write(root.join("validation.json"), &validation_content)
        .expect("Failed to write validation.json");
    println!("  [+] validation.json (stub — populate with results)");

    // 3. Capture environment
    let env_content = capture_environment();
    std::fs::write(root.join("receipts/environment.toml"), &env_content)
        .expect("Failed to write receipts/environment.toml");
    println!("  [+] receipts/environment.toml");

    // 4. Copy outputs if provided
    if let Some(src) = outputs_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("outputs"));
            println!("  [+] outputs/ (copied from {src})");
        }
    }

    // 5. Copy configs if provided
    if let Some(src) = configs_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("configs"));
            println!("  [+] configs/ (copied from {src})");
        }
    }

    // 6. Copy braids if provided
    if let Some(src) = braids_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("provenance/braids"));
            println!("  [+] provenance/braids/ (copied from {src})");
        }
    }

    // 7. Generate ferment transcript stub
    let ferment_content = generate_ferment_stub(name, version, origin);
    std::fs::write(root.join("provenance/ferment_transcript.json"), &ferment_content)
        .expect("Failed to write provenance/ferment_transcript.json");
    println!("  [+] provenance/ferment_transcript.json (stub)");

    // 8. Compute checksums for outputs/ and provenance/
    let checksums = pseudospore::compute_checksums(&root, &["outputs", "provenance"]);
    let cksum_content = pseudospore::format_checksums(&checksums);
    std::fs::write(root.join("receipts/checksums.blake3"), &cksum_content)
        .expect("Failed to write receipts/checksums.blake3");
    println!("  [+] receipts/checksums.blake3 ({} entries)", checksums.len());

    // 9. Generate README
    let readme = generate_readme(name, version, origin);
    std::fs::write(root.join("README.md"), &readme).expect("Failed to write README.md");
    println!("  [+] README.md");

    println!();
    println!("Done. pseudoSpore emitted to: {}", root.display());
    println!();
    println!("Next steps:");
    println!("  1. Populate validation.json with actual module results");
    println!("  2. Add outputs/<module>/ result files if not already copied");
    println!("  3. Update provenance/ferment_transcript.json with real braid data");
    println!("  4. Re-run `litho emit-pseudospore` or manually update checksums");
    println!("  5. Run `litho ingest-pseudospore {}` to validate", root.display());
}

fn generate_scope(name: &str, version: &str, origin: &str) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    format!(
        r#"[artifact]
name = "{name}"
version = "{version}"
type = "pseudoSpore"
date = "{date}"
origin = "{origin}"
license = "AGPL-3.0-or-later"

# [target]
# paper_doi = ""
# paper_title = ""
# paper_authors = ""
# paper_year = 2026

# [[module]]
# name = "module-name"
# status = "PASS"
# checks = 0
# description = ""

[evolution]
tier_0 = "Industry control"
tier_1 = "Python sovereign implementation"
tier_2 = "Rust sovereign implementation"
tier_3 = "NUCLEUS IPC composition (future)"

[source]
repo = ""
commit = ""
branch = "main"
"#
    )
}

fn generate_validation_stub(name: &str, version: &str) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    format!(
        r#"{{
  "artifact": "{name}",
  "version": "{version}",
  "date": "{date}",
  "modules": [],
  "summary": {{
    "modules_total": 0,
    "modules_pass": 0,
    "modules_in_flight": 0
  }}
}}
"#
    )
}

fn generate_ferment_stub(name: &str, version: &str, origin: &str) -> String {
    let spring = origin.split('/').last().unwrap_or("unknown");
    let timestamp = chrono::Utc::now().to_rfc3339();
    format!(
        r#"{{
  "dataset_id": "{name}_v{version}",
  "spring": "{spring}",
  "spring_version": "{version}",
  "braid_id": "braid-{name}-{version}",
  "timestamp": "{timestamp}",
  "computation": {{}}
}}
"#
    )
}

fn capture_environment() -> String {
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "unknown".to_string());

    let os_info = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Linux".to_string());

    let timestamp = chrono::Utc::now().to_rfc3339();

    format!(
        r#"[hardware]
hostname = "{hostname}"
# cpu = ""
# ram_gb = 0
# gpu = ""

[software]
os = "{os_info}"
# Add tool versions relevant to this computation

[timestamps]
captured = "{timestamp}"
"#
    )
}

fn generate_readme(name: &str, version: &str, origin: &str) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    format!(
        r#"# pseudoSpore: {name} v{version}

**Date:** {date}
**Origin:** {origin}
**Type:** pseudoSpore (lightweight braid-first deployment)
**Standard:** specs/PSEUDOSPORE_STANDARD.md

---

## Structure

- `scope.toml` — birth certificate (artifact identity, modules, evolution tiers)
- `validation.json` — machine-readable results with per-module checks
- `receipts/` — compute provenance (environment, checksums, optional compute log)
- `provenance/` — ferment transcript + braids (DAG, spine, sweetGrass)
- `outputs/` — science results (data files, validation reports)
- `configs/` — reproducibility chain (input configs to re-run computation)

## Verification

```bash
litho ingest-pseudospore . --verify
```

## Promotion

This pseudoSpore can be promoted to a full lithoSpore module by adding:
1. Python baseline (Tier 1) — `notebooks/<module>/`
2. Rust implementation (Tier 2) — `crates/<module>/`
3. Expected values JSON — `validation/expected/`
4. Named tolerances — `artifact/tolerances.toml`

See `docs/LITHOSPORE_PROMOTION.md` in the origin repo for the full path.
"#
    )
}

fn copy_tree(src: &Path, dst: &Path) {
    if !src.is_dir() {
        return;
    }
    std::fs::create_dir_all(dst).ok();
    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let dest = dst.join(entry.file_name());
            if path.is_dir() {
                copy_tree(&path, &dest);
            } else {
                std::fs::copy(&path, &dest).ok();
            }
        }
    }
}
