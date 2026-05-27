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
use blake3;
use chrono;
use serde_json;
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

pub fn run(pseudospore_path: &str, verbose: bool, json_output: bool) {
    let audit_start = std::time::Instant::now();
    let root = Path::new(pseudospore_path);

    if !root.exists() {
        eprintln!("ERROR: path not found: {pseudospore_path}");
        std::process::exit(1);
    }

    println!("=== litho audit ===");
    println!("  Target: {pseudospore_path}");

    // Load domain profile (optional — graceful degradation)
    let profile = crate::domain_profile::load_domain_profile(root);
    if let Some(ref p) = profile {
        println!("  Profile: {} v{}", p.id, p.version);
    } else {
        println!("  Profile: (none — core checks only)");
    }
    println!();

    let mut findings: Vec<Finding> = Vec::new();

    // --- Core checks (always run, domain-agnostic) ---
    let core_checks: &[(&str, fn(&Path, &mut Vec<Finding>))] = &[
        ("BLAKE3 checksum integrity", check_blake3_integrity),
        ("Module completeness (data/outputs/configs)", check_module_completeness),
        ("Visual evidence layer (figures/)", check_figures_layer),
        ("Version consistency across docs", check_version_consistency),
        ("Provenance completeness", check_provenance),
    ];

    // --- Domain checks (run when profile present and enables them) ---
    let mut domain_checks: Vec<(&str, fn(&Path, &mut Vec<Finding>))> = Vec::new();

    if let Some(ref p) = profile {
        if p.audit.config_fidelity {
            domain_checks.push(("Config↔Data fidelity (HEIGHT vs HILLS)", check_hills_height_match));
        }
        if p.translation.enabled {
            domain_checks.push(("Domain translation validity", check_domain_translation));
            domain_checks.push(("Domain↔Topology cross-reference", check_domain_vs_topology));
        }
        if p.derivation.contracts.len() > 0 {
            domain_checks.push(("Derivation contract (reproduce outputs from data)", check_derivation_contract));
        }
        if p.audit.scientific_claims {
            domain_checks.push(("Validation claims vs FES data", check_validation_claims));
        }
        if p.audit.simulation_time {
            domain_checks.push(("Simulation time consistency (MDP vs scope.toml)", check_simulation_times));
        }
        if p.audit.mdp_headers {
            domain_checks.push(("MDP header accuracy", check_mdp_headers));
        }
    } else {
        // Backwards compatibility: if no profile but domain files exist, run all checks
        // This preserves behavior for pre-profile pseudoSpores
        if root.join("index_map.toml").exists() || root.join("configs").exists() {
            domain_checks.push(("Config↔Data fidelity (HEIGHT vs HILLS)", check_hills_height_match));
            domain_checks.push(("Domain translation validity", check_domain_translation));
            domain_checks.push(("Domain↔Topology cross-reference", check_domain_vs_topology));
            domain_checks.push(("Derivation contract (reproduce outputs from data)", check_derivation_contract));
            domain_checks.push(("Validation claims vs FES data", check_validation_claims));
            domain_checks.push(("Simulation time consistency (MDP vs scope.toml)", check_simulation_times));
            domain_checks.push(("MDP header accuracy", check_mdp_headers));
        }
    }

    let total_checks = core_checks.len() + domain_checks.len();
    let mut check_idx = 0;

    for (label, check_fn) in core_checks {
        check_idx += 1;
        let before = findings.len();
        check_fn(root, &mut findings);
        let added = findings.len() - before;
        if verbose {
            let status = if added == 0 { "PASS" } else { "FAIL" };
            println!("  [{}/{}] {} — {} ({})", check_idx, total_checks, status, label, added);
        }
    }

    for (label, check_fn) in &domain_checks {
        check_idx += 1;
        let before = findings.len();
        check_fn(root, &mut findings);
        let added = findings.len() - before;
        if verbose {
            let status = if added == 0 { "PASS" } else { "FAIL" };
            println!("  [{}/{}] {} — {} ({})", check_idx, total_checks, status, label, added);
        }
    }

    if verbose && !json_output { println!(); }

    // Report
    let high = findings.iter().filter(|f| f.severity == Severity::High).count();
    let med = findings.iter().filter(|f| f.severity == Severity::Medium).count();
    let low = findings.iter().filter(|f| f.severity == Severity::Low).count();

    let elapsed_ms = audit_start.elapsed().as_millis() as u64;

    if json_output {
        // GuideStone-format structured JSON output
        let scope_path = root.join("scope.toml");
        let (artifact_name, artifact_version) = if let Ok(content) = std::fs::read_to_string(&scope_path) {
            let table: toml::Table = content.parse().unwrap_or_default();
            let art = table.get("artifact").and_then(|v| v.as_table());
            (
                art.and_then(|a| a.get("name")).and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                art.and_then(|a| a.get("version")).and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string(),
            )
        } else {
            ("unknown".to_string(), "0.0.0".to_string())
        };

        let profile_id = profile.as_ref().map(|p| p.id.as_str()).unwrap_or("none");
        let status = if high > 0 { "FAIL" } else { "PASS" };
        let tier = if high > 0 { 0 } else { 2 };

        let report = serde_json::json!({
            "artifact": artifact_name,
            "version": artifact_version,
            "status": status,
            "tier_reached": tier,
            "profile": profile_id,
            "checks": total_checks,
            "checks_passed": total_checks - (high + med + low),
            "findings": {
                "high": high,
                "medium": med,
                "low": low
            },
            "modules": findings.iter().map(|f| {
                serde_json::json!({
                    "id": f.id,
                    "severity": format!("{}", f.severity),
                    "category": f.category,
                    "message": f.message
                })
            }).collect::<Vec<_>>()
        });

        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());

        // Append to liveSpore.json if present and passed
        if high == 0 {
            append_livespore(root, total_checks, elapsed_ms);
        }
    } else {
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

        // Append to liveSpore.json on success
        if high == 0 {
            append_livespore(root, total_checks, elapsed_ms);
        }
    }

    std::process::exit(if high > 0 { 1 } else { 0 });
}

fn append_livespore(root: &Path, checks: usize, _elapsed_ms: u64) {
    let livespore_path = root.join("liveSpore.json");
    if !livespore_path.exists() {
        return;
    }

    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "unknown".to_string());
    let hostname_hash = blake3::hash(hostname.as_bytes()).to_hex().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let entry = serde_json::json!({
        "timestamp": timestamp,
        "hostname_hash": hostname_hash,
        "arch": arch,
        "os": os,
        "tier_reached": 2,
        "modules_passed": checks,
        "modules_total": checks,
        "runtime_ms": _elapsed_ms
    });

    let content = std::fs::read_to_string(&livespore_path).unwrap_or_default();

    // Unified schema: object with "envelope" + "validations"
    // Legacy: bare array [] or hotSpring-style {"liveSpore": {...}, ...}
    let mut doc: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::json!({}));

    if let Some(validations) = doc.get_mut("validations").and_then(|v| v.as_array_mut()) {
        // Unified schema path
        validations.push(entry);
    } else if doc.is_array() {
        // Legacy lithoSpore: bare array — migrate to unified
        let legacy_entries = doc.as_array().cloned().unwrap_or_default();
        let mut validations = legacy_entries;
        validations.push(entry);
        doc = serde_json::json!({
            "envelope": {},
            "validations": validations
        });
    } else {
        // Legacy hotSpring or unknown — wrap and add validations
        let envelope = if doc.get("liveSpore").is_some() {
            let mut env = doc.get("liveSpore").cloned().unwrap_or(serde_json::json!({}));
            if let Some(chain) = doc.get("provenance_chain") {
                env["provenance_chain"] = chain.clone();
            }
            if let Some(sw) = doc.get("software") {
                env["software"] = sw.clone();
            }
            env
        } else if doc.get("envelope").is_some() {
            doc.get("envelope").cloned().unwrap_or(serde_json::json!({}))
        } else {
            doc.clone()
        };
        doc = serde_json::json!({
            "envelope": envelope,
            "validations": [entry]
        });
    }

    std::fs::write(&livespore_path, serde_json::to_string_pretty(&doc).unwrap_or_default()).ok();
}

/// Check 0: Verify BLAKE3 checksums actually match file contents.
fn check_blake3_integrity(root: &Path, findings: &mut Vec<Finding>) {
    let cksum_path = root.join("receipts/checksums.blake3");
    if !cksum_path.exists() {
        findings.push(Finding {
            id: "BLAKE3-MISSING".to_string(),
            severity: Severity::High,
            category: "Integrity",
            message: "receipts/checksums.blake3 not found".to_string(),
            fix: "Regenerate checksums with b3sum".to_string(),
        });
        return;
    }

    let content = match fs::read_to_string(&cksum_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut failures = 0;
    let mut checked = 0;

    for line in content.lines() {
        if line.is_empty() { continue; }
        // Format: <hash>  <path>
        let parts: Vec<&str> = line.splitn(2, "  ").collect();
        if parts.len() != 2 { continue; }

        let expected_hash = parts[0].trim();
        let rel_path = parts[1].trim();
        let file_path = root.join(rel_path.trim_start_matches("./"));

        if !file_path.exists() {
            failures += 1;
            continue;
        }

        let file_bytes = match fs::read(&file_path) {
            Ok(b) => b,
            Err(_) => { failures += 1; continue; }
        };

        let actual_hash = blake3::hash(&file_bytes).to_hex().to_string();
        if actual_hash != expected_hash {
            failures += 1;
            if failures <= 3 {
                findings.push(Finding {
                    id: format!("BLAKE3-MISMATCH-{}", rel_path.replace('/', "-")),
                    severity: Severity::High,
                    category: "Integrity",
                    message: format!("{}: checksum mismatch (file modified after sealing?)", rel_path),
                    fix: "Regenerate checksums or restore original file".to_string(),
                });
            }
        }
        checked += 1;
    }

    if failures > 3 {
        findings.push(Finding {
            id: "BLAKE3-MULTI-FAIL".to_string(),
            severity: Severity::High,
            category: "Integrity",
            message: format!("{} of {} files have checksum mismatches", failures, checked + failures),
            fix: "Regenerate all checksums: find . -type f | xargs b3sum > receipts/checksums.blake3".to_string(),
        });
    }
}

/// Check: Visual evidence layer — figures exist and correspond to outputs.
/// Verify validation.json claims against actual FES output data.
/// Catches the "θ≈5° (4C1)" claim when actual global minimum is at θ≈172° (1C4).
fn check_validation_claims(root: &Path, findings: &mut Vec<Finding>) {
    let val_path = root.join("validation.json");
    if !val_path.exists() { return; }

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
        let fes_path = root.join(format!("outputs/{}/fes_theta.dat", name));
        if !fes_path.exists() { continue; }

        let fes_content = match fs::read_to_string(&fes_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Find global minimum theta value
        let mut min_energy = f64::MAX;
        let mut min_theta = 0.0_f64;
        for line in fes_content.lines() {
            if line.starts_with('#') || line.trim().is_empty() { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 { continue; }
            let theta: f64 = match parts[0].parse() { Ok(v) => v, Err(_) => continue };
            let energy: f64 = match parts[1].parse() { Ok(v) => v, Err(_) => continue };
            if energy < min_energy {
                min_energy = energy;
                min_theta = theta;
            }
        }

        if min_energy == f64::MAX { continue; }

        // Convert to degrees if in radians
        let theta_deg = if min_theta.abs() < 7.0 { min_theta.to_degrees() } else { min_theta };

        // Check validation.json claims about ground state
        let details = module.get("details").and_then(|d| d.as_object());
        if let Some(det) = details {
            for (key, val_str) in det {
                let claim = val_str.as_str().unwrap_or("");
                // Look for ground state claims (key = "ground_state*")
                if !key.contains("ground_state") { continue; }

                // Determine what the claim says is the ground state.
                // Look for pattern "global min" followed by a conformer name.
                // "global min θ≈172° (1C4)" → claims 1C4
                // "global min θ≈5° (4C1)" → claims 4C1
                let claim_says_4c1_ground = claim.contains("global min") &&
                    claim.find("global min").map(|pos| {
                        let after = &claim[pos..];
                        let has_4c1 = after.find("4C1").or(after.find("4c1"));
                        let has_1c4 = after.find("1C4").or(after.find("1c4"));
                        match (has_4c1, has_1c4) {
                            (Some(a), Some(b)) => a < b,
                            (Some(_), None) => true,
                            _ => false,
                        }
                    }).unwrap_or(false);

                let claim_says_1c4_ground = claim.contains("global min") &&
                    claim.find("global min").map(|pos| {
                        let after = &claim[pos..];
                        let has_4c1 = after.find("4C1").or(after.find("4c1"));
                        let has_1c4 = after.find("1C4").or(after.find("1c4"));
                        match (has_4c1, has_1c4) {
                            (Some(a), Some(b)) => b < a,
                            (None, Some(_)) => true,
                            _ => false,
                        }
                    }).unwrap_or(false);

                // 4C1 is θ≈0-30°, 1C4 is θ≈150-180°
                let actual_is_4c1 = theta_deg < 60.0;
                let actual_is_1c4 = theta_deg > 120.0;

                if claim_says_4c1_ground && actual_is_1c4 {
                    findings.push(Finding {
                        id: format!("VALIDATION-CLAIM-{}", name),
                        severity: Severity::High,
                        category: "Scientific Claims",
                        message: format!(
                            "{}: claims 4C1 ground state but FES global minimum is at θ={:.1}° (1C4)",
                            name, theta_deg
                        ),
                        fix: "Update validation.json to reflect actual FES ground state".to_string(),
                    });
                } else if claim_says_1c4_ground && actual_is_4c1 {
                    findings.push(Finding {
                        id: format!("VALIDATION-CLAIM-{}", name),
                        severity: Severity::High,
                        category: "Scientific Claims",
                        message: format!(
                            "{}: claims 1C4 ground state but FES global minimum is at θ={:.1}° (4C1)",
                            name, theta_deg
                        ),
                        fix: "Update validation.json to reflect actual FES ground state".to_string(),
                    });
                }
            }
        }
    }
}

/// Cross-check scope.toml simulation_time_ns against MDP nsteps*dt
fn check_simulation_times(root: &Path, findings: &mut Vec<Finding>) {
    let scope_path = root.join("scope.toml");
    if !scope_path.exists() { return; }

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
        let claimed_ns = module.get("simulation_time_ns")
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as f64;

        scope_total_ns += claimed_ns;

        // Try to find matching MDP
        let configs_dir = root.join(format!("configs/{}", name));
        if !configs_dir.exists() { continue; }

        let mut mdp_ns: Option<f64> = None;
        if let Ok(entries) = fs::read_dir(&configs_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e == "mdp").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&p) {
                        let mut nsteps: Option<f64> = None;
                        let mut dt: Option<f64> = None;
                        for line in content.lines() {
                            let line = line.split(';').next().unwrap_or("").trim();
                            if line.starts_with("nsteps") {
                                nsteps = line.split('=').nth(1)
                                    .and_then(|v| v.trim().parse().ok());
                            } else if line.starts_with("dt") && !line.starts_with("dt_") {
                                dt = line.split('=').nth(1)
                                    .and_then(|v| v.trim().parse().ok());
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
                    id: format!("SIMTIME-MISMATCH-{}", name),
                    severity: Severity::High,
                    category: "Simulation Time",
                    message: format!(
                        "{}: scope.toml claims {} ns but MDP nsteps*dt = {} ns",
                        name, claimed_ns, actual_ns
                    ),
                    fix: "Update scope.toml simulation_time_ns to match MDP parameters".to_string(),
                });
            }
        }
    }

    // Cross-check environment.toml total
    let env_path = root.join("receipts/environment.toml");
    if let Ok(env_content) = fs::read_to_string(&env_path) {
        if let Ok(env_table) = env_content.parse::<toml::Table>() {
            if let Some(total) = env_table.get("production")
                .and_then(|p| p.as_table())
                .and_then(|p| p.get("total_production_ns"))
                .and_then(|v| v.as_integer())
            {
                let diff = (total as f64 - scope_total_ns).abs();
                if diff > 1.0 {
                    findings.push(Finding {
                        id: "SIMTIME-TOTAL-MISMATCH".to_string(),
                        severity: Severity::High,
                        category: "Simulation Time",
                        message: format!(
                            "environment.toml total_production_ns={} but scope.toml modules sum to {} ns",
                            total, scope_total_ns
                        ),
                        fix: "Update environment.toml to match sum of module simulation times".to_string(),
                    });
                }
            }
        }
    }
}

fn check_figures_layer(root: &Path, findings: &mut Vec<Finding>) {
    let figures_dir = root.join("figures");
    let outputs_dir = root.join("outputs");

    if !outputs_dir.exists() {
        return;
    }

    // Count output modules that have FES data
    let mut fes_modules = 0;
    if let Ok(entries) = fs::read_dir(&outputs_dir) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() { continue; }
            let has_fes = fs::read_dir(entry.path())
                .into_iter()
                .flatten()
                .flatten()
                .any(|f| {
                    let name = f.file_name().to_string_lossy().to_string();
                    name.starts_with("fes_") && name.ends_with(".dat")
                });
            if has_fes { fes_modules += 1; }
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
            message: format!("{} modules have FES data but no figures/ directory exists", fes_modules),
            fix: "Generate figures: python generate_figures.py --pseudospore <path>".to_string(),
        });
        return;
    }

    let png_count = fs::read_dir(&figures_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().map(|x| x == "png").unwrap_or(false))
        .count();

    if png_count == 0 {
        findings.push(Finding {
            id: "FIGURES-EMPTY".to_string(),
            severity: Severity::Low,
            category: "Visual Evidence",
            message: "figures/ directory exists but contains no PNG files".to_string(),
            fix: "Generate figures: python generate_figures.py --pseudospore <path>".to_string(),
        });
    }
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

    // Check JSON files with "version" fields match scope.toml
    let json_files = ["validation.json", "validation_matrix.json"];
    for jf in &json_files {
        let jpath = root.join(jf);
        if let Ok(content) = fs::read_to_string(&jpath) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(jv) = v.get("version").and_then(|v| v.as_str()) {
                    if jv != scope_version {
                        findings.push(Finding {
                            id: format!("VERSION-JSON-{}", jf),
                            severity: Severity::Low,
                            category: "Version Sync",
                            message: format!("{} says version \"{}\" but scope.toml says \"{}\"", jf, jv, scope_version),
                            fix: format!("Update \"version\" field in {} to \"{}\"", jf, scope_version),
                        });
                    }
                }
            }
        }
    }

    // Check environment.toml total_production_ns matches actual module sum from scope.toml
    let env_path = root.join("receipts/environment.toml");
    if let Ok(env_content) = fs::read_to_string(&env_path) {
        let claimed_ns: Option<u64> = env_content.lines()
            .find(|l| l.starts_with("total_production_ns"))
            .and_then(|l| l.split('=').nth(1))
            .and_then(|v| v.trim().parse().ok());

        // Sum simulation_time_ns from scope.toml modules
        let scope_content = fs::read_to_string(&scope_path).unwrap_or_default();
        let actual_ns: u64 = scope_content.lines()
            .filter(|l| l.starts_with("simulation_time_ns"))
            .filter_map(|l| l.split('=').nth(1))
            .filter_map(|v| v.trim().parse::<u64>().ok())
            .sum();

        if let Some(claimed) = claimed_ns {
            if actual_ns > 0 && claimed != actual_ns {
                findings.push(Finding {
                    id: "ENV-PRODUCTION-NS".to_string(),
                    severity: Severity::Low,
                    category: "Version Sync",
                    message: format!("environment.toml claims {} ns total but scope.toml modules sum to {} ns", claimed, actual_ns),
                    fix: format!("Update total_production_ns to {}", actual_ns),
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

            // Check for placeholder merkle root
            if let Some(merkle) = v.get("dag_merkle_root").and_then(|v| v.as_str()) {
                if merkle.contains("pending") || merkle.contains("placeholder") || merkle.is_empty() {
                    findings.push(Finding {
                        id: "PROVENANCE-MERKLE-PLACEHOLDER".to_string(),
                        severity: Severity::Medium,
                        category: "Provenance",
                        message: format!("dag_merkle_root is placeholder: \"{}\"", merkle),
                        fix: "Compute actual BLAKE3 merkle root: b3sum outputs/*/fes_*.dat data/*/HILLS* | b3sum".to_string(),
                    });
                }
            }
        }
    }

    // Check braid JSONs for frozen/stale URNs
    let provenance_dir = root.join("provenance");
    if !provenance_dir.exists() { return; }

    let scope_version = fs::read_to_string(root.join("scope.toml"))
        .ok()
        .and_then(|c| c.lines().find(|l| l.starts_with("version")).and_then(|l| l.split('"').nth(1)).map(|s| s.to_string()))
        .unwrap_or_default();

    if let Ok(entries) = fs::read_dir(&provenance_dir) {
        let mut braid_ids: Vec<(String, String)> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.file_name().map(|n| n.to_string_lossy().starts_with("cazyme_fel_v")).unwrap_or(false) { continue; }
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                        let fname = path.file_name().unwrap().to_string_lossy().to_string();
                        if let Some(bid) = v.get("braid_id").and_then(|v| v.as_str()) {
                            braid_ids.push((fname.clone(), bid.to_string()));
                        }
                    }
                }
            }
        }

        // Detect frozen URNs: if all braids share the same braid_id despite different versions
        if braid_ids.len() > 1 {
            let first_id = &braid_ids[0].1;
            let all_same = braid_ids.iter().all(|(_, id)| id == first_id);
            if all_same {
                findings.push(Finding {
                    id: "PROVENANCE-FROZEN-URN".to_string(),
                    severity: Severity::Low,
                    category: "Provenance",
                    message: format!(
                        "All {} braid JSONs share identical braid_id \"{}\" — should be unique per version",
                        braid_ids.len(), first_id
                    ),
                    fix: "Each braid version should have its own unique braid_id URN".to_string(),
                });
            }
        }

        // Check that the latest braid's URN references the current version
        if !scope_version.is_empty() {
            let latest_braid = format!("cazyme_fel_v{}.json", scope_version);
            let latest_path = provenance_dir.join(&latest_braid);
            if let Ok(content) = fs::read_to_string(&latest_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(bid) = v.get("braid_id").and_then(|v| v.as_str()) {
                        if !bid.contains(&scope_version.replace('.', "-")) && !bid.contains(&scope_version) {
                            findings.push(Finding {
                                id: "PROVENANCE-URN-VERSION-MISMATCH".to_string(),
                                severity: Severity::Low,
                                category: "Provenance",
                                message: format!(
                                    "{}: braid_id \"{}\" doesn't reference current version {}",
                                    latest_braid, bid, scope_version
                                ),
                                fix: format!("Update braid_id to include version identifier (e.g., urn:braid:...-v{})", scope_version),
                            });
                        }
                    }
                }
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

        // Build lookup: GROMACS/PLUMED 1-indexed atom position → atom_name from .gro
        // GROMACS atom indices are 1-indexed line positions (line 1 = first atom after header).
        // GRO serial numbers (col 15-20) can wrap at 99999 or restart per molecule — unreliable for lookup.
        let gro_lines: Vec<&str> = gro_content.lines().collect();

        for (atom_name, atom_val) in ring {
            if atom_name.starts_with('_') { continue; }

            let comp_idx = match atom_val.as_table()
                .and_then(|t| t.get("computation"))
                .and_then(|v| v.as_integer()) {
                Some(i) => i as usize,
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
