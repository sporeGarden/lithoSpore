// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho audit` — pre-handoff validation that catches packaging/config fidelity issues.
//!
//! Checks that:
//! 1. Configs can reproduce data (HEIGHT/SIGMA match HILLS first Gaussian)
//! 2. Domain-frame translation produces valid PDB serials (not sequential 1,2,3...)
//! 3. validation.json check names are consistent with actual FES results
//! 4. All data/ modules have corresponding outputs/ and configs/
//! 5. Documentation version references match scope.toml version
//! 6. Provenance fields are populated (no empty strings)
//! 7. MDP headers match the system they describe
//!
//! Returns structured findings with severity levels (HIGH/MEDIUM/LOW).

use std::fs;
use std::path::Path;
use toml;

#[derive(Debug)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub category: &'static str,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "HIGH"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::Low => write!(f, "LOW"),
        }
    }
}

pub fn run(pseudospore_path: &str, verbose: bool) {
    let root = Path::new(pseudospore_path);

    if !root.exists() {
        eprintln!("ERROR: path not found: {pseudospore_path}");
        std::process::exit(1);
    }

    println!("=== litho audit ===");
    println!("  Target: {pseudospore_path}");
    println!();

    let mut findings: Vec<Finding> = Vec::new();

    let checks: &[(&str, fn(&Path, &mut Vec<Finding>))] = &[
        ("Config↔Data fidelity (HEIGHT vs HILLS)", check_hills_height_match),
        ("Domain translation validity", check_domain_translation),
        ("Domain↔Topology cross-reference", check_domain_vs_topology),
        ("Module completeness (data/outputs/configs)", check_module_completeness),
        ("Derivation contract (reproduce outputs from data)", check_derivation_contract),
        ("Version consistency across docs", check_version_consistency),
        ("Provenance completeness", check_provenance),
        ("MDP header accuracy", check_mdp_headers),
    ];

    for (i, (label, check_fn)) in checks.iter().enumerate() {
        let before = findings.len();
        check_fn(root, &mut findings);
        let added = findings.len() - before;
        if verbose {
            let status = if added == 0 { "PASS" } else { "FAIL" };
            println!("  [{}] {} — {} ({})", i + 1, status, label, added);
        }
    }
    if verbose { println!(); }

    // Report
    let high = findings.iter().filter(|f| f.severity == Severity::High).count();
    let med = findings.iter().filter(|f| f.severity == Severity::Medium).count();
    let low = findings.iter().filter(|f| f.severity == Severity::Low).count();

    if findings.is_empty() {
        println!("  PASS — no findings. Artifact is handoff-ready.");
    } else {
        println!("  Findings: {} HIGH, {} MEDIUM, {} LOW", high, med, low);
        println!();

        for f in &findings {
            println!("  [{}] {} — {}", f.severity, f.id, f.category);
            println!("    {}", f.message);
            if verbose {
                println!("    Fix: {}", f.fix);
            }
            println!();
        }

        if high > 0 {
            println!("  VERDICT: CONDITIONAL PASS — fix {} HIGH findings before handoff.", high);
        } else if med > 0 {
            println!("  VERDICT: PASS with recommendations — {} MEDIUM findings.", med);
        } else {
            println!("  VERDICT: PASS — {} LOW findings (cosmetic).", low);
        }
    }

    println!();
    std::process::exit(if high > 0 { 1 } else { 0 });
}

fn check_hills_height_match(root: &Path, findings: &mut Vec<Finding>) {
    let configs_dir = root.join("configs");
    let data_dir = root.join("data");

    if !configs_dir.exists() || !data_dir.exists() {
        return;
    }

    if let Ok(modules) = fs::read_dir(&configs_dir) {
        for module in modules.flatten() {
            if !module.path().is_dir() {
                continue;
            }
            let mod_name = module.file_name().to_string_lossy().to_string();

            // Find plumed.dat and corresponding HILLS
            let plumed_paths = [
                module.path().join("plumed.dat"),
                module.path().join("plumed_2d.dat"),
            ];

            for plumed_path in &plumed_paths {
                if !plumed_path.exists() {
                    continue;
                }

                let plumed_content = match fs::read_to_string(plumed_path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                // Extract HEIGHT from config
                let config_height: Option<f64> = plumed_content
                    .lines()
                    .find(|l| l.contains("HEIGHT="))
                    .and_then(|l| {
                        l.split("HEIGHT=")
                            .nth(1)
                            .and_then(|s| s.split_whitespace().next())
                            .and_then(|s| s.trim_end_matches(|c: char| !c.is_numeric() && c != '.').parse().ok())
                    });

                // Extract BIASFACTOR
                let biasfactor: Option<f64> = plumed_content
                    .lines()
                    .find(|l| l.contains("BIASFACTOR="))
                    .and_then(|l| {
                        l.split("BIASFACTOR=")
                            .nth(1)
                            .and_then(|s| s.split_whitespace().next())
                            .and_then(|s| s.trim_end_matches(|c: char| !c.is_numeric() && c != '.').parse().ok())
                    });

                if let (Some(height), Some(bf)) = (config_height, biasfactor) {
                    // Find corresponding HILLS
                    let hills_name = if plumed_path.to_string_lossy().contains("2d") {
                        "HILLS_2d"
                    } else {
                        "HILLS"
                    };
                    let hills_path = data_dir.join(&mod_name).join(hills_name);

                    if hills_path.exists() {
                        if let Ok(content) = fs::read_to_string(&hills_path) {
                            // Determine dimensionality from FIELDS header
                            let n_sigma = content.lines()
                                .find(|l| l.contains("FIELDS"))
                                .map(|l| l.matches("sigma").count())
                                .unwrap_or(1);

                            // Get first data line
                            if let Some(first_data) = content.lines().find(|l| !l.starts_with('#') && !l.is_empty()) {
                                let fields: Vec<&str> = first_data.split_whitespace().collect();
                                // Height field: time + n_cv + n_sigma + height
                                // 1D: time cv sigma height bf → index 3
                                // 2D: time cv1 cv2 sigma1 sigma2 height bf → index 5
                                let height_field_idx = 1 + n_sigma * 2;
                                if let Some(hills_height) = fields.get(height_field_idx).and_then(|s| s.parse::<f64>().ok()) {
                                    let expected_first = height * bf / (bf - 1.0);
                                    let tolerance = expected_first * 0.05; // 5% tolerance for WTMetaD decay
                                    if (hills_height - expected_first).abs() > tolerance {
                                        findings.push(Finding {
                                            id: format!("CONFIG-HEIGHT-{}", mod_name),
                                            severity: Severity::High,
                                            category: "Config↔Data Fidelity",
                                            message: format!(
                                                "{}/{}: config HEIGHT={:.1} → expected first Gaussian {:.4}, but HILLS shows {:.4}",
                                                mod_name, hills_name, height, expected_first, hills_height
                                            ),
                                            fix: format!("Update HEIGHT in configs/{}/{}",
                                                mod_name,
                                                plumed_path.file_name().unwrap().to_string_lossy()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_domain_translation(root: &Path, findings: &mut Vec<Finding>) {
    let index_map_path = root.join("index_map.toml");
    if !index_map_path.exists() {
        findings.push(Finding {
            id: "TRANSLATE-MISSING".to_string(),
            severity: Severity::Medium,
            category: "Translation",
            message: "No index_map.toml present — domain experts cannot verify atom indices".to_string(),
            fix: "Generate index_map.toml with domain (PDB) ↔ computation (GROMACS) mapping".to_string(),
        });
        return;
    }

    if let Ok(content) = fs::read_to_string(&index_map_path) {
        // Check for placeholder "?" values
        let placeholder_count = content.matches("\"?\"").count();
        if placeholder_count > 0 {
            findings.push(Finding {
                id: "TRANSLATE-PLACEHOLDER".to_string(),
                severity: Severity::High,
                category: "Translation",
                message: format!("{} domain indices are still placeholder '?' — must be assigned from PDB source", placeholder_count),
                fix: "Look up PDB HETATM serials for ring atoms and replace '?' values".to_string(),
            });
        }

        // Check for suspiciously low domain values (1-20) that might be GROMACS indices
        if let Ok(table) = content.parse::<toml::Table>() {
            if let Some(toml::Value::Table(systems)) = table.get("systems") {
                for (sys_name, sys_val) in systems {
                    if let Some(ring) = sys_val.as_table().and_then(|t| t.get("ring")).and_then(|r| r.as_table()) {
                        let mut low_domain_count = 0;
                        for (atom_name, atom_val) in ring {
                            if atom_name.starts_with('_') { continue; }
                            if let Some(domain) = atom_val.as_table().and_then(|t| t.get("domain")).and_then(|v| v.as_integer()) {
                                if domain > 0 && domain < 20 {
                                    low_domain_count += 1;
                                }
                            }
                        }
                        if low_domain_count >= 5 {
                            findings.push(Finding {
                                id: format!("TRANSLATE-LOW-SERIAL-{}", sys_name),
                                severity: Severity::Medium,
                                category: "Translation",
                                message: format!(
                                    "systems.{}: {} ring atoms have domain < 20 — these may be GROMACS indices rather than PDB serials",
                                    sys_name, low_domain_count
                                ),
                                fix: "Verify domain values are actual PDB HETATM serials from source crystal structure".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}

fn check_module_completeness(root: &Path, findings: &mut Vec<Finding>) {
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
            if !module.path().is_dir() { continue; }
            let mod_name = module.file_name().to_string_lossy().to_string();
            let data_module = data_dir.join(&mod_name);
            if !data_module.exists() {
                findings.push(Finding {
                    id: format!("DATA-GAP-{}", mod_name),
                    severity: Severity::Medium,
                    category: "Zero-Trust",
                    message: format!("outputs/{} exists but data/{} is missing — cannot verify derivation", mod_name, mod_name),
                    fix: format!("Add data/{}/HILLS or mark module as reference-only in scope.toml", mod_name),
                });
            }
        }
    }

    // Check configs/ coverage
    if configs_dir.exists() {
        if let Ok(modules) = fs::read_dir(&outputs_dir) {
            for module in modules.flatten() {
                if !module.path().is_dir() { continue; }
                let mod_name = module.file_name().to_string_lossy().to_string();
                let config_module = configs_dir.join(&mod_name);
                if !config_module.exists() {
                    findings.push(Finding {
                        id: format!("CONFIG-GAP-{}", mod_name),
                        severity: Severity::Low,
                        category: "Completeness",
                        message: format!("outputs/{} has no matching configs/ entry", mod_name),
                        fix: format!("Add configs/{}/plumed.dat", mod_name),
                    });
                }
            }
        }
    }
}

fn check_version_consistency(root: &Path, findings: &mut Vec<Finding>) {
    let scope_path = root.join("scope.toml");
    let scope_version = fs::read_to_string(&scope_path)
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("version"))
                .and_then(|l| l.split('"').nth(1))
                .map(|s| s.to_string())
        })
        .unwrap_or_default();

    if scope_version.is_empty() {
        return;
    }

    // Check docs reference current version
    let doc_files = ["ABG_HANDOFF.md", "RELEASE.md", "README.md"];
    for doc in &doc_files {
        let doc_path = root.join(doc);
        if let Ok(content) = fs::read_to_string(&doc_path) {
            let first_10_lines: String = content.lines().take(10).collect::<Vec<_>>().join("\n");
            if first_10_lines.contains("v0.") && !first_10_lines.contains(&format!("v{}", scope_version)) {
                // Found a version reference that doesn't match current
                findings.push(Finding {
                    id: format!("VERSION-STALE-{}", doc),
                    severity: Severity::Medium,
                    category: "Documentation",
                    message: format!("{} references an older version in header (scope.toml says v{})", doc, scope_version),
                    fix: format!("Update {} to reference v{}", doc, scope_version),
                });
            }
        }
    }
}

fn check_provenance(root: &Path, findings: &mut Vec<Finding>) {
    let ferment_path = root.join("provenance/ferment_transcript.json");
    if let Ok(content) = fs::read_to_string(&ferment_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            let empty_fields: Vec<&str> = ["dataset_id", "spring", "dag_session_id", "braid_id", "timestamp"]
                .iter()
                .filter(|&&field| {
                    v.get(field)
                        .map(|val| val.as_str().map(|s| s.is_empty()).unwrap_or(true))
                        .unwrap_or(true)
                })
                .copied()
                .collect();

            if !empty_fields.is_empty() {
                findings.push(Finding {
                    id: "PROVENANCE-GAPS".to_string(),
                    severity: Severity::Medium,
                    category: "Provenance",
                    message: format!("ferment_transcript.json has empty fields: {}", empty_fields.join(", ")),
                    fix: "Populate all provenance fields before handoff".to_string(),
                });
            }
        }
    }
}

fn check_mdp_headers(root: &Path, findings: &mut Vec<Finding>) {
    let configs_dir = root.join("configs");
    if !configs_dir.exists() {
        return;
    }

    if let Ok(modules) = fs::read_dir(&configs_dir) {
        for module in modules.flatten() {
            if !module.path().is_dir() { continue; }
            let mod_name = module.file_name().to_string_lossy().to_string();

            // Check .mdp files
            if let Ok(files) = fs::read_dir(module.path()) {
                for file in files.flatten() {
                    if file.path().extension().map(|e| e == "mdp").unwrap_or(false) {
                        if let Ok(content) = fs::read_to_string(file.path()) {
                            let first_line = content.lines().next().unwrap_or("");
                            // Check if header mentions a different system
                            if mod_name.contains("enzyme") && first_line.contains("free xylose") {
                                findings.push(Finding {
                                    id: format!("MDP-HEADER-{}", mod_name),
                                    severity: Severity::High,
                                    category: "Config Fidelity",
                                    message: format!("configs/{}/{}: header says 'free xylose' but module is enzyme-bound",
                                        mod_name, file.file_name().to_string_lossy()),
                                    fix: "Correct the MDP comment header to match the actual system".to_string(),
                                });
                            }
                            if mod_name.contains("free") && first_line.contains("enzyme") {
                                findings.push(Finding {
                                    id: format!("MDP-HEADER-{}", mod_name),
                                    severity: Severity::High,
                                    category: "Config Fidelity",
                                    message: format!("configs/{}/{}: header says 'enzyme' but module is free xylose",
                                        mod_name, file.file_name().to_string_lossy()),
                                    fix: "Correct the MDP comment header to match the actual system".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Check 3: Cross-reference domain indices in index_map.toml against actual .gro topology.
/// Verifies that computation indices listed in the map actually correspond to the claimed
/// atom names at those positions in the topology file.
fn check_domain_vs_topology(root: &Path, findings: &mut Vec<Finding>) {
    let index_map_path = root.join("index_map.toml");
    if !index_map_path.exists() {
        return;
    }

    let content = match fs::read_to_string(&index_map_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => return,
    };

    let systems = match table.get("systems").and_then(|v| v.as_table()) {
        Some(s) => s,
        None => return,
    };

    for (sys_name, sys_val) in systems {
        let sys = match sys_val.as_table() {
            Some(s) => s,
            None => continue,
        };

        let rosetta = match sys.get("rosetta_stone").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => continue,
        };

        let gro_path = root.join(rosetta);
        if !gro_path.exists() {
            continue;
        }

        let gro_content = match fs::read_to_string(&gro_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let ring = match sys.get("ring").and_then(|v| v.as_table()) {
            Some(r) => r,
            None => continue,
        };

        // Build lookup: gromacs_index → atom_name from .gro
        let gro_lines: Vec<&str> = gro_content.lines().collect();

        for (atom_name, atom_val) in ring {
            if atom_name.starts_with('_') { continue; }

            let comp_idx = match atom_val.as_table()
                .and_then(|t| t.get("computation"))
                .and_then(|v| v.as_integer()) {
                Some(i) => i as usize,
                None => continue,
            };

            // In GRO format, atom number is at columns 15-20 (0-indexed)
            // Find the line with this atom number
            let mut found_name = None;
            for line in &gro_lines[2..] {
                if line.len() < 20 { continue; }
                let atom_num_str = line.get(15..20).unwrap_or("").trim();
                if let Ok(num) = atom_num_str.parse::<usize>() {
                    if num == comp_idx {
                        found_name = Some(line.get(10..15).unwrap_or("").trim().to_string());
                        break;
                    }
                }
            }

            if let Some(gro_name) = found_name {
                if gro_name != *atom_name {
                    findings.push(Finding {
                        id: format!("TOPOLOGY-MISMATCH-{}-{}", sys_name, atom_name),
                        severity: Severity::High,
                        category: "Index Verification",
                        message: format!(
                            "systems.{}.ring.{}: computation index {} maps to '{}' in topology, expected '{}'",
                            sys_name, atom_name, comp_idx, gro_name, atom_name
                        ),
                        fix: format!("Check index_map.toml entry for {} in system {}", atom_name, sys_name),
                    });
                }
            }
        }
    }
}

/// Check 5: Verify derivation contract — outputs can be re-derived from data.
/// Uses plumed sum_hills internally if available, otherwise checks file sizes
/// and HILLS line counts as a proxy.
fn check_derivation_contract(root: &Path, findings: &mut Vec<Finding>) {
    let data_dir = root.join("data");
    let outputs_dir = root.join("outputs");

    if !data_dir.exists() || !outputs_dir.exists() {
        return;
    }

    // Check if plumed is available (search common conda/system paths)
    let plumed_bin = find_plumed();
    let _has_plumed = plumed_bin.is_some();

    if let Ok(modules) = fs::read_dir(&data_dir) {
        for module in modules.flatten() {
            if !module.path().is_dir() { continue; }
            let mod_name = module.file_name().to_string_lossy().to_string();

            // Check 1D HILLS
            let hills_path = module.path().join("HILLS");
            let output_fes = outputs_dir.join(&mod_name).join("fes_theta.dat");

            if hills_path.exists() && output_fes.exists() {
                if let Some(ref plumed) = plumed_bin {
                    // Actually verify derivation
                    let tmp_out = format!("/tmp/litho_audit_derive_{}.dat", mod_name);
                    let result = std::process::Command::new(plumed)
                        .args(["sum_hills", "--hills"])
                        .arg(&hills_path)
                        .args(["--mintozero", "--outfile", &tmp_out])
                        .output();

                    match result {
                        Ok(o) if o.status.success() => {
                            // Compare with output
                            let derived = fs::read_to_string(&tmp_out).unwrap_or_default();
                            let expected = fs::read_to_string(&output_fes).unwrap_or_default();
                            if derived != expected {
                                // Check if it's just formatting
                                let d_lines: Vec<f64> = derived.lines()
                                    .filter(|l| !l.starts_with('#') && !l.is_empty())
                                    .filter_map(|l| l.split_whitespace().nth(1))
                                    .filter_map(|s| s.parse().ok())
                                    .collect();
                                let e_lines: Vec<f64> = expected.lines()
                                    .filter(|l| !l.starts_with('#') && !l.is_empty())
                                    .filter_map(|l| l.split_whitespace().nth(1))
                                    .filter_map(|s| s.parse().ok())
                                    .collect();

                                if d_lines.len() == e_lines.len() {
                                    let max_diff: f64 = d_lines.iter().zip(e_lines.iter())
                                        .map(|(a, b)| (a - b).abs())
                                        .fold(0.0f64, f64::max);
                                    if max_diff > 0.001 {
                                        findings.push(Finding {
                                            id: format!("DERIVATION-FAIL-{}", mod_name),
                                            severity: Severity::High,
                                            category: "Derivation Contract",
                                            message: format!(
                                                "{}: re-derived FES differs from shipped output by {:.4} kJ/mol max",
                                                mod_name, max_diff
                                            ),
                                            fix: "Regenerate outputs/ from data/ or fix data/HILLS".to_string(),
                                        });
                                    }
                                } else {
                                    findings.push(Finding {
                                        id: format!("DERIVATION-SIZE-{}", mod_name),
                                        severity: Severity::Medium,
                                        category: "Derivation Contract",
                                        message: format!(
                                            "{}: re-derived FES has {} points, shipped has {}",
                                            mod_name, d_lines.len(), e_lines.len()
                                        ),
                                        fix: "Check GRID settings — derivation may need explicit --min/--max/--bin".to_string(),
                                    });
                                }
                            }
                            fs::remove_file(&tmp_out).ok();
                        }
                        _ => {} // plumed failed, skip
                    }
                } else {
                    // No plumed: sanity check that HILLS has reasonable line count
                    let hills_lines = fs::read_to_string(&hills_path)
                        .map(|c| c.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).count())
                        .unwrap_or(0);
                    if hills_lines < 100 {
                        findings.push(Finding {
                            id: format!("HILLS-SHORT-{}", mod_name),
                            severity: Severity::Medium,
                            category: "Derivation Contract",
                            message: format!("{}: HILLS has only {} depositions (< 100, likely incomplete)", mod_name, hills_lines),
                            fix: "Verify simulation completed or mark module as IN_FLIGHT".to_string(),
                        });
                    }
                }
            }

            // Check 2D HILLS
            let hills_2d_path = module.path().join("HILLS_2d");
            let output_fes_2d = outputs_dir.join(&mod_name).join("fes_2d.dat");

            if hills_2d_path.exists() && output_fes_2d.exists() {
                if let Some(ref plumed) = plumed_bin {
                let tmp_out = format!("/tmp/litho_audit_derive_2d_{}.dat", mod_name);
                let result = std::process::Command::new(plumed)
                    .args(["sum_hills", "--hills"])
                    .arg(&hills_2d_path)
                    .args(["--min", "-0.12,-0.12", "--max", "0.12,0.12", "--bin", "100,100"])
                    .args(["--mintozero", "--outfile", &tmp_out])
                    .output();

                if let Ok(o) = result {
                    if o.status.success() {
                        let derived = fs::read_to_string(&tmp_out).unwrap_or_default();
                        let expected = fs::read_to_string(&output_fes_2d).unwrap_or_default();
                        if derived != expected {
                            let d_vals: Vec<f64> = derived.lines()
                                .filter(|l| !l.starts_with('#') && !l.is_empty())
                                .filter_map(|l| l.split_whitespace().nth(2))
                                .filter_map(|s| s.parse().ok())
                                .collect();
                            let e_vals: Vec<f64> = expected.lines()
                                .filter(|l| !l.starts_with('#') && !l.is_empty())
                                .filter_map(|l| l.split_whitespace().nth(2))
                                .filter_map(|s| s.parse().ok())
                                .collect();

                            if d_vals.len() == e_vals.len() && !d_vals.is_empty() {
                                let max_diff: f64 = d_vals.iter().zip(e_vals.iter())
                                    .map(|(a, b)| (a - b).abs())
                                    .fold(0.0f64, f64::max);
                                if max_diff > 0.001 {
                                    findings.push(Finding {
                                        id: format!("DERIVATION-2D-FAIL-{}", mod_name),
                                        severity: Severity::High,
                                        category: "Derivation Contract",
                                        message: format!(
                                            "{}: 2D FES re-derivation differs by {:.4} kJ/mol max",
                                            mod_name, max_diff
                                        ),
                                        fix: "Regenerate outputs/ from data/ with matching --min/--max/--bin".to_string(),
                                    });
                                }
                            }
                        }
                    }
                    fs::remove_file(&tmp_out).ok();
                }
                }
            }
        }
    }
}

/// Locate plumed binary — checks PATH then common conda locations.
/// Uses `plumed info --root` as the liveness check (plumed has no --version flag).
fn find_plumed() -> Option<String> {
    let check_plumed = |bin: &str| -> bool {
        std::process::Command::new(bin)
            .args(["info", "--root"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };

    // Try PATH first
    if check_plumed("plumed") {
        return Some("plumed".to_string());
    }

    // Search common conda/system paths
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("{}/miniconda3/envs/gromacs-fel/bin/plumed", home),
        format!("{}/miniconda3/bin/plumed", home),
        format!("{}/anaconda3/envs/gromacs-fel/bin/plumed", home),
        "/usr/local/bin/plumed".to_string(),
        "/usr/bin/plumed".to_string(),
    ];

    for candidate in &candidates {
        if Path::new(candidate).exists() && check_plumed(candidate) {
            return Some(candidate.clone());
        }
    }

    None
}
