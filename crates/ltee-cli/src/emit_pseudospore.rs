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
    data_dir: Option<&str>,
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

    // 7. Copy data if provided
    if let Some(src) = data_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            std::fs::create_dir_all(root.join("data")).expect("Failed to create data/");
            copy_tree(src_path, &root.join("data"));
            println!("  [+] data/ (copied from {src})");
        }
    }

    // 8. Auto-generate index_map.toml from topology files in data/
    let data_root = root.join("data");
    if data_root.exists() {
        if let Some(index_map) = auto_generate_index_map(&data_root) {
            std::fs::write(root.join("index_map.toml"), &index_map)
                .expect("Failed to write index_map.toml");
            println!("  [+] index_map.toml (auto-generated from topology files)");
        }
    }

    // 9. Generate ferment transcript stub
    let ferment_content = generate_ferment_stub(name, version, origin);
    std::fs::write(root.join("provenance/ferment_transcript.json"), &ferment_content)
        .expect("Failed to write provenance/ferment_transcript.json");
    println!("  [+] provenance/ferment_transcript.json (stub)");

    // 10. Compute checksums for outputs/, provenance/, and data/
    let checksums = pseudospore::compute_checksums(&root, &["outputs", "provenance", "data", "configs"]);
    let cksum_content = pseudospore::format_checksums(&checksums);
    std::fs::write(root.join("receipts/checksums.blake3"), &cksum_content)
        .expect("Failed to write receipts/checksums.blake3");
    println!("  [+] receipts/checksums.blake3 ({} entries)", checksums.len());

    // 11. Generate README
    let readme = generate_readme(name, version, origin);
    std::fs::write(root.join("README.md"), &readme).expect("Failed to write README.md");
    println!("  [+] README.md");

    // 12. Generate TRANSLATE.md stub
    let translate = generate_translate_stub();
    std::fs::write(root.join("TRANSLATE.md"), &translate).expect("Failed to write TRANSLATE.md");
    println!("  [+] TRANSLATE.md (stub — populate with derivation commands)");

    println!();
    println!("Done. pseudoSpore emitted to: {}", root.display());
    println!();
    println!("Next steps:");
    println!("  1. Populate validation.json with actual module results");
    println!("  2. Add outputs/<module>/ result files if not already copied");
    println!("  3. Update provenance/ferment_transcript.json with real braid data");
    println!("  4. Populate TRANSLATE.md with derivation commands");
    println!("  5. Review/edit index_map.toml if auto-generated mappings need refinement");
    println!("  6. Re-run `litho emit-pseudospore` or manually update checksums");
    println!("  7. Run `litho ingest-pseudospore {}` to validate", root.display());
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

fn generate_translate_stub() -> String {
    r#"# Translation: Domain ↔ Computation

## Atom Indices

See `index_map.toml` for the machine-readable mapping.

| Ring atom | Domain (PDB serial) | Computation (runtime index) |
|-----------|--------------------|-----------------------------|
| ... | ... | ... |

Rosetta stone: `data/<module>/npt.gro` (topology file)

## Conventions

| | Domain standard | This artifact |
|--|----------------|---------------|
| Numbering | PDB serial | Runtime topology (mapped in index_map.toml) |
| Checksums | — | BLAKE3 |

## Derivations

| Output | Data | Command |
|--------|------|---------|
| `outputs/<module>/...` | `data/<module>/...` | `<tool> <args>` |
"#
    .to_string()
}

/// Auto-generate index_map.toml by scanning data/ for .gro topology files.
/// Parses GROMACS .gro format to extract atom names and indices for ring atoms
/// commonly found in carbohydrate residues (C1-C5, O5).
fn auto_generate_index_map(data_root: &Path) -> Option<String> {
    let mut systems: Vec<(String, String, Vec<(String, usize)>)> = Vec::new();

    // Walk data/ subdirectories looking for .gro files
    if let Ok(entries) = std::fs::read_dir(data_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let module_name = entry.file_name().to_string_lossy().to_string();

            // Look for .gro files in this module
            if let Ok(files) = std::fs::read_dir(&path) {
                for file in files.flatten() {
                    let fpath = file.path();
                    if fpath.extension().map(|e| e == "gro").unwrap_or(false) {
                        if let Some(ring_atoms) = extract_ring_atoms_from_gro(&fpath) {
                            let rosetta = format!("data/{}/{}", module_name, file.file_name().to_string_lossy());
                            systems.push((module_name.clone(), rosetta, ring_atoms));
                        }
                    }
                }
            }
        }
    }

    if systems.is_empty() {
        return None;
    }

    let mut output = String::new();
    output.push_str("# Auto-generated domain ↔ computation index mapping\n");
    output.push_str("# Generated by `litho emit-pseudospore`\n");
    output.push_str("# Review and correct domain indices manually if needed\n\n");
    output.push_str("[meta]\n");
    output.push_str("ring_order = [\"C1\", \"C2\", \"C3\", \"C4\", \"C5\", \"O5\"]\n\n");

    for (module, rosetta, atoms) in &systems {
        let _safe_name = module.replace('-', "_");
        output.push_str(&format!("[systems.\"{}\"]\n", module));
        output.push_str(&format!("description = \"Auto-detected from {}\"\n", rosetta));
        output.push_str(&format!("rosetta_stone = \"{}\"\n\n", rosetta));
        output.push_str(&format!("[systems.\"{}\".ring]\n", module));
        for (name, idx) in atoms {
            output.push_str(&format!(
                "{} = {{ domain = \"?\", computation = {} }}\n",
                name, idx
            ));
        }
        output.push_str(&format!(
            "\n[systems.\"{}\"._note]\n",
            module
        ));
        output.push_str("value = \"Domain indices need manual assignment from PDB source. Computation indices auto-extracted from topology.\"\n\n");
    }

    Some(output)
}

/// Parse a GROMACS .gro file to extract ring atom indices (C1-C5, O5) from
/// carbohydrate residues (XYS, BXYL, GLC, etc.).
fn extract_ring_atoms_from_gro(path: &Path) -> Option<Vec<(String, usize)>> {
    let content = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 3 {
        return None;
    }

    let sugar_residues = ["XYS", "BXYL", "BXY", "GLC", "GAL", "MAN", "FUC", "XYL"];
    let ring_atom_names = ["C1", "C2", "C3", "C4", "C5", "O5"];

    let mut found: Vec<(String, usize)> = Vec::new();

    // GRO format: columns are fixed-width
    // %5d%-5s%5s%5d ...
    // residue_number(5) residue_name(5) atom_name(5) atom_number(5)
    for line in &lines[2..] {
        if line.len() < 20 {
            continue;
        }

        let res_name = line.get(5..10).unwrap_or("").trim();
        let atom_name = line.get(10..15).unwrap_or("").trim();
        let atom_num_str = line.get(15..20).unwrap_or("").trim();

        if sugar_residues.iter().any(|s| res_name == *s) {
            if ring_atom_names.iter().any(|a| atom_name == *a) {
                if let Ok(num) = atom_num_str.parse::<usize>() {
                    if !found.iter().any(|(n, _)| n == atom_name) {
                        found.push((atom_name.to_string(), num));
                    }
                }
            }
        }
    }

    if found.is_empty() {
        None
    } else {
        Some(found)
    }
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
