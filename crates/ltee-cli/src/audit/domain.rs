// SPDX-License-Identifier: AGPL-3.0-or-later

//! Domain-specific fidelity checks — PLUMED/HILLS, translation maps, derivation, validation claims, MDP headers.

use std::{fs, path::Path};

use super::{Finding, Severity};
/// Verify validation.json claims against actual FES output data.
/// Catches the "θ≈5° (4C1)" claim when actual global minimum is at θ≈172° (1C4).
pub(super) fn check_validation_claims(root: &Path, findings: &mut Vec<Finding>) {
    let val_path = root.join("validation.json");
    if !val_path.exists() {
        return;
    }

    let val_content = match fs::read_to_string(&val_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let val: serde_json::Value = match serde_json::from_str(&val_content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let modules = match val.get("modules").and_then(|m| m.as_array()) {
        Some(m) => m,
        None => return,
    };

    for module in modules {
        let name = module.get("name").and_then(|n| n.as_str()).unwrap_or("");

        // Check 1D FES theta modules — verify ground state claim against data
        let fes_path = root.join(format!("outputs/{name}/fes_theta.dat"));
        if !fes_path.exists() {
            continue;
        }

        let fes_content = match fs::read_to_string(&fes_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Find global minimum theta value
        let mut min_energy = f64::MAX;
        let mut min_theta = 0.0_f64;
        for line in fes_content.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let theta: f64 = match parts[0].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let energy: f64 = match parts[1].parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if energy < min_energy {
                min_energy = energy;
                min_theta = theta;
            }
        }

        if min_energy == f64::MAX {
            continue;
        }

        // Convert to degrees if in radians
        let theta_deg = if min_theta.abs() < 7.0 {
            min_theta.to_degrees()
        } else {
            min_theta
        };

        // Check validation.json claims about ground state
        let details = module.get("details").and_then(|d| d.as_object());
        if let Some(det) = details {
            for (key, val_str) in det {
                let claim = val_str.as_str().unwrap_or("");
                // Look for ground state claims (key = "ground_state*")
                if !key.contains("ground_state") {
                    continue;
                }

                // Determine what the claim says is the ground state.
                // Look for pattern "global min" followed by a conformer name.
                // "global min θ≈172° (1C4)" → claims 1C4
                // "global min θ≈5° (4C1)" → claims 4C1
                let claim_says_4c1_ground = claim.contains("global min")
                    && claim.find("global min").is_some_and(|pos| {
                        let after = &claim[pos..];
                        let has_4c1 = after.find("4C1").or_else(|| after.find("4c1"));
                        let has_1c4 = after.find("1C4").or_else(|| after.find("1c4"));
                        match (has_4c1, has_1c4) {
                            (Some(a), Some(b)) => a < b,
                            (Some(_), None) => true,
                            _ => false,
                        }
                    });

                let claim_says_1c4_ground = claim.contains("global min")
                    && claim.find("global min").is_some_and(|pos| {
                        let after = &claim[pos..];
                        let has_4c1 = after.find("4C1").or_else(|| after.find("4c1"));
                        let has_1c4 = after.find("1C4").or_else(|| after.find("1c4"));
                        match (has_4c1, has_1c4) {
                            (Some(a), Some(b)) => b < a,
                            (None, Some(_)) => true,
                            _ => false,
                        }
                    });

                // 4C1 is θ≈0-30°, 1C4 is θ≈150-180°
                let actual_is_4c1 = theta_deg < 60.0;
                let actual_is_1c4 = theta_deg > 120.0;

                if claim_says_4c1_ground && actual_is_1c4 {
                    findings.push(Finding {
                        id: format!("VALIDATION-CLAIM-{name}"),
                        severity: Severity::High,
                        category: "Scientific Claims",
                        message: format!(
                            "{name}: claims 4C1 ground state but FES global minimum is at θ={theta_deg:.1}° (1C4)"
                        ),
                        fix: "Update validation.json to reflect actual FES ground state".to_string(),
                    });
                } else if claim_says_1c4_ground && actual_is_4c1 {
                    findings.push(Finding {
                        id: format!("VALIDATION-CLAIM-{name}"),
                        severity: Severity::High,
                        category: "Scientific Claims",
                        message: format!(
                            "{name}: claims 1C4 ground state but FES global minimum is at θ={theta_deg:.1}° (4C1)"
                        ),
                        fix: "Update validation.json to reflect actual FES ground state".to_string(),
                    });
                }
            }
        }
    }
}
/// Cross-check scope.toml `simulation_time_ns` against MDP nsteps*dt
pub(super) fn check_simulation_times(root: &Path, findings: &mut Vec<Finding>) {
    let scope_path = root.join("scope.toml");
    if !scope_path.exists() {
        return;
    }

    let scope_content = match fs::read_to_string(&scope_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let scope: toml::Table = match scope_content.parse() {
        Ok(t) => t,
        Err(_) => return,
    };

    // Parse modules from scope.toml
    let modules = match scope.get("module") {
        Some(toml::Value::Array(arr)) => arr,
        _ => return,
    };

    let mut scope_total_ns: f64 = 0.0;

    for module in modules {
        let name = module.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let claimed_ns = module
            .get("simulation_time_ns")
            .and_then(toml::Value::as_integer)
            .unwrap_or(0) as f64;

        scope_total_ns += claimed_ns;

        // Try to find matching MDP
        let configs_dir = root.join(format!("configs/{name}"));
        if !configs_dir.exists() {
            continue;
        }

        let mut mdp_ns: Option<f64> = None;
        if let Ok(entries) = fs::read_dir(&configs_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|e| e == "mdp") {
                    if let Ok(content) = fs::read_to_string(&p) {
                        let mut nsteps: Option<f64> = None;
                        let mut dt: Option<f64> = None;
                        for line in content.lines() {
                            let line = line.split(';').next().unwrap_or("").trim();
                            if line.starts_with("nsteps") {
                                nsteps = line.split('=').nth(1).and_then(|v| v.trim().parse().ok());
                            } else if line.starts_with("dt") && !line.starts_with("dt_") {
                                dt = line.split('=').nth(1).and_then(|v| v.trim().parse().ok());
                            }
                        }
                        if let (Some(n), Some(d)) = (nsteps, dt) {
                            // nsteps * dt(ps) / 1000 = time in ns
                            mdp_ns = Some(n * d / 1000.0);
                        }
                    }
                    break;
                }
            }
        }

        if let Some(actual_ns) = mdp_ns {
            let diff = (claimed_ns - actual_ns).abs();
            if diff > 1.0 {
                findings.push(Finding {
                    id: format!("SIMTIME-MISMATCH-{name}"),
                    severity: Severity::High,
                    category: "Simulation Time",
                    message: format!(
                        "{name}: scope.toml claims {claimed_ns} ns but MDP nsteps*dt = {actual_ns} ns"
                    ),
                    fix: "Update scope.toml simulation_time_ns to match MDP parameters".to_string(),
                });
            }
        }
    }

    // Cross-check environment.toml total
    let env_path = root.join("receipts/environment.toml");
    if let Ok(env_content) = fs::read_to_string(&env_path)
        && let Ok(env_table) = env_content.parse::<toml::Table>()
        && let Some(total) = env_table
            .get("production")
            .and_then(|p| p.as_table())
            .and_then(|p| p.get("total_production_ns"))
            .and_then(toml::Value::as_integer)
    {
        let diff = (total as f64 - scope_total_ns).abs();
        if diff > 1.0 {
            findings.push(Finding {
                        id: "SIMTIME-TOTAL-MISMATCH".to_string(),
                        severity: Severity::High,
                        category: "Simulation Time",
                        message: format!(
                            "environment.toml total_production_ns={total} but scope.toml modules sum to {scope_total_ns} ns"
                        ),
                        fix: "Update environment.toml to match sum of module simulation times".to_string(),
                    });
        }
    }
}
pub(super) fn check_hills_height_match(root: &Path, findings: &mut Vec<Finding>) {
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
                            .and_then(|s| {
                                s.trim_end_matches(|c: char| !c.is_numeric() && c != '.')
                                    .parse()
                                    .ok()
                            })
                    });

                // Extract BIASFACTOR
                let biasfactor: Option<f64> = plumed_content
                    .lines()
                    .find(|l| l.contains("BIASFACTOR="))
                    .and_then(|l| {
                        l.split("BIASFACTOR=")
                            .nth(1)
                            .and_then(|s| s.split_whitespace().next())
                            .and_then(|s| {
                                s.trim_end_matches(|c: char| !c.is_numeric() && c != '.')
                                    .parse()
                                    .ok()
                            })
                    });

                if let (Some(height), Some(bf)) = (config_height, biasfactor) {
                    // Find corresponding HILLS
                    let hills_name = if plumed_path.to_string_lossy().contains("2d") {
                        "HILLS_2d"
                    } else {
                        "HILLS"
                    };
                    let hills_path = data_dir.join(&mod_name).join(hills_name);

                    if hills_path.exists()
                        && let Ok(content) = fs::read_to_string(&hills_path)
                    {
                        // Determine dimensionality from FIELDS header
                        let n_sigma = content
                            .lines()
                            .find(|l| l.contains("FIELDS"))
                            .map_or(1, |l| l.matches("sigma").count());

                        // Get first data line
                        if let Some(first_data) = content
                            .lines()
                            .find(|l| !l.starts_with('#') && !l.is_empty())
                        {
                            let fields: Vec<&str> = first_data.split_whitespace().collect();
                            // Height field: time + n_cv + n_sigma + height
                            // 1D: time cv sigma height bf → index 3
                            // 2D: time cv1 cv2 sigma1 sigma2 height bf → index 5
                            let height_field_idx = 1 + n_sigma * 2;
                            if let Some(hills_height) = fields
                                .get(height_field_idx)
                                .and_then(|s| s.parse::<f64>().ok())
                            {
                                let expected_first = height * bf / (bf - 1.0);
                                let tolerance = expected_first * 0.05; // 5% tolerance for WTMetaD decay
                                if (hills_height - expected_first).abs() > tolerance {
                                    findings.push(Finding {
                                            id: format!("CONFIG-HEIGHT-{mod_name}"),
                                            severity: Severity::High,
                                            category: "Config↔Data Fidelity",
                                            message: format!(
                                                "{mod_name}/{hills_name}: config HEIGHT={height:.1} → expected first Gaussian {expected_first:.4}, but HILLS shows {hills_height:.4}"
                                            ),
                                            fix: format!("Update HEIGHT in configs/{}/{}",
                                                mod_name,
                                                plumed_path.file_name().map_or_else(|| "plumed.dat".into(), |f| f.to_string_lossy())),
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
pub(super) fn check_domain_translation(root: &Path, findings: &mut Vec<Finding>) {
    let index_map_path = root.join("index_map.toml");
    if !index_map_path.exists() {
        findings.push(Finding {
            id: "TRANSLATE-MISSING".to_string(),
            severity: Severity::Medium,
            category: "Translation",
            message: "No index_map.toml present — domain experts cannot verify atom indices"
                .to_string(),
            fix: "Generate index_map.toml with domain (PDB) ↔ computation (GROMACS) mapping"
                .to_string(),
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
                message: format!("{placeholder_count} domain indices are still placeholder '?' — must be assigned from PDB source"),
                fix: "Look up PDB HETATM serials for ring atoms and replace '?' values".to_string(),
            });
        }

        // Check for suspiciously low domain values (1-20) that might be GROMACS indices
        if let Ok(table) = content.parse::<toml::Table>()
            && let Some(toml::Value::Table(systems)) = table.get("systems")
        {
            for (sys_name, sys_val) in systems {
                if let Some(ring) = sys_val
                    .as_table()
                    .and_then(|t| t.get("ring"))
                    .and_then(|r| r.as_table())
                {
                    let mut low_domain_count = 0;
                    for (atom_name, atom_val) in ring {
                        if atom_name.starts_with('_') {
                            continue;
                        }
                        if let Some(domain) = atom_val
                            .as_table()
                            .and_then(|t| t.get("domain"))
                            .and_then(toml::Value::as_integer)
                            && domain > 0
                            && domain < 20
                        {
                            low_domain_count += 1;
                        }
                    }
                    if low_domain_count >= 5 {
                        findings.push(Finding {
                                id: format!("TRANSLATE-LOW-SERIAL-{sys_name}"),
                                severity: Severity::Medium,
                                category: "Translation",
                                message: format!(
                                    "systems.{sys_name}: {low_domain_count} ring atoms have domain < 20 — these may be GROMACS indices rather than PDB serials"
                                ),
                                fix: "Verify domain values are actual PDB HETATM serials from source crystal structure".to_string(),
                            });
                    }
                }
            }
        }
    }
}
pub(super) fn check_mdp_headers(root: &Path, findings: &mut Vec<Finding>) {
    let configs_dir = root.join("configs");
    if !configs_dir.exists() {
        return;
    }

    if let Ok(modules) = fs::read_dir(&configs_dir) {
        for module in modules.flatten() {
            if !module.path().is_dir() {
                continue;
            }
            let mod_name = module.file_name().to_string_lossy().to_string();

            // Check .mdp files
            if let Ok(files) = fs::read_dir(module.path()) {
                for file in files.flatten() {
                    if file.path().extension().is_some_and(|e| e == "mdp")
                        && let Ok(content) = fs::read_to_string(file.path())
                    {
                        let first_line = content.lines().next().unwrap_or("");
                        // Check if header mentions a different system
                        if mod_name.contains("enzyme") && first_line.contains("free xylose") {
                            findings.push(Finding {
                                    id: format!("MDP-HEADER-{mod_name}"),
                                    severity: Severity::High,
                                    category: "Config Fidelity",
                                    message: format!("configs/{}/{}: header says 'free xylose' but module is enzyme-bound",
                                        mod_name, file.file_name().to_string_lossy()),
                                    fix: "Correct the MDP comment header to match the actual system".to_string(),
                                });
                        }
                        if mod_name.contains("free") && first_line.contains("enzyme") {
                            findings.push(Finding {
                                id: format!("MDP-HEADER-{mod_name}"),
                                severity: Severity::High,
                                category: "Config Fidelity",
                                message: format!(
                                    "configs/{}/{}: header says 'enzyme' but module is free xylose",
                                    mod_name,
                                    file.file_name().to_string_lossy()
                                ),
                                fix: "Correct the MDP comment header to match the actual system"
                                    .to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
}
/// Cross-reference domain indices in `index_map.toml` against actual .gro topology.
/// Verifies that computation indices listed in the map actually correspond to the claimed
/// atom names at those positions in the topology file.
pub(super) fn check_domain_vs_topology(root: &Path, findings: &mut Vec<Finding>) {
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

        // Build lookup: GROMACS/PLUMED 1-indexed atom position → atom_name from .gro
        // GROMACS atom indices are 1-indexed line positions (line 1 = first atom after header).
        // GRO serial numbers (col 15-20) can wrap at 99999 or restart per molecule — unreliable for lookup.
        let gro_lines: Vec<&str> = gro_content.lines().collect();

        for (atom_name, atom_val) in ring {
            if atom_name.starts_with('_') {
                continue;
            }

            let comp_idx = match atom_val
                .as_table()
                .and_then(|t| t.get("computation"))
                .and_then(toml::Value::as_integer)
            {
                Some(i) => usize::try_from(i).unwrap_or(0),
                None => continue,
            };

            // comp_idx is 1-indexed: atom 1 = gro_lines[2], atom N = gro_lines[N+1]
            let line_idx = comp_idx + 1; // +1 for title line, natom already at index 1
            let mut found_name = None;
            if line_idx < gro_lines.len() {
                let line = gro_lines[line_idx];
                if line.len() >= 15 {
                    found_name = Some(line.get(10..15).unwrap_or("").trim().to_string());
                }
            }

            if let Some(gro_name) = found_name
                && gro_name != *atom_name
            {
                findings.push(Finding {
                        id: format!("TOPOLOGY-MISMATCH-{sys_name}-{atom_name}"),
                        severity: Severity::High,
                        category: "Index Verification",
                        message: format!(
                            "systems.{sys_name}.ring.{atom_name}: computation index {comp_idx} maps to '{gro_name}' in topology, expected '{atom_name}'"
                        ),
                        fix: format!("Check index_map.toml entry for {atom_name} in system {sys_name}"),
                    });
            }
        }
    }
}
pub(super) use super::derivation::check_derivation_contract;
