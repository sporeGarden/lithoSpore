// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho audit` — pre-handoff validation that catches packaging/config fidelity issues.
//!
//! Orchestrates core integrity/completeness/provenance checks and optional domain-profile
//! checks. Returns structured [`Finding`]s with [`Severity`] levels (HIGH/MEDIUM/LOW).

mod completeness;
mod domain;
mod integrity;
mod provenance;

use std::path::Path;

use completeness::{check_figures_layer, check_module_completeness};
use domain::{
    check_derivation_contract, check_domain_translation, check_domain_vs_topology,
    check_hills_height_match, check_mdp_headers, check_simulation_times, check_validation_claims,
};
use integrity::check_blake3_integrity;
use provenance::{check_provenance, check_version_consistency};

type AuditCheckFn = fn(&Path, &mut Vec<Finding>);
type AuditCheck = (&'static str, AuditCheckFn);

#[derive(Debug)]
pub(crate) struct Finding {
    pub id: String,
    pub severity: Severity,
    pub category: &'static str,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Severity {
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

pub(crate) fn run(pseudospore_path: &str, verbose: bool, json_output: bool) {
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
    let core_checks: &[AuditCheck] = &[
        ("BLAKE3 checksum integrity", check_blake3_integrity),
        (
            "Module completeness (data/outputs/configs)",
            check_module_completeness,
        ),
        ("Visual evidence layer (figures/)", check_figures_layer),
        ("Version consistency across docs", check_version_consistency),
        ("Provenance completeness", check_provenance),
    ];

    // --- Domain checks (run when profile present and enables them) ---
    let mut domain_checks: Vec<AuditCheck> = Vec::new();

    if let Some(ref p) = profile {
        if p.audit.as_ref().is_some_and(|a| a.domain.config_fidelity) {
            domain_checks.push((
                "Config↔Data fidelity (HEIGHT vs HILLS)",
                check_hills_height_match,
            ));
        }
        if p.translation_enabled() {
            domain_checks.push(("Domain translation validity", check_domain_translation));
            domain_checks.push(("Domain↔Topology cross-reference", check_domain_vs_topology));
        }
        if p.derivation
            .as_ref()
            .is_some_and(|d| !d.contracts.is_empty())
        {
            domain_checks.push((
                "Derivation contract (reproduce outputs from data)",
                check_derivation_contract,
            ));
        }
        if p.audit
            .as_ref()
            .is_some_and(|a| a.validation.scientific_claims)
        {
            domain_checks.push(("Validation claims vs FES data", check_validation_claims));
        }
        if p.audit
            .as_ref()
            .is_some_and(|a| a.validation.simulation_time)
        {
            domain_checks.push((
                "Simulation time consistency (MDP vs scope.toml)",
                check_simulation_times,
            ));
        }
        if p.audit.as_ref().is_some_and(|a| a.domain.mdp_headers) {
            domain_checks.push(("MDP header accuracy", check_mdp_headers));
        }
    } else {
        // Backwards compatibility: if no profile but domain files exist, run all checks
        // This preserves behavior for pre-profile pseudoSpores
        if root.join("index_map.toml").exists() || root.join("configs").exists() {
            domain_checks.push((
                "Config↔Data fidelity (HEIGHT vs HILLS)",
                check_hills_height_match,
            ));
            domain_checks.push(("Domain translation validity", check_domain_translation));
            domain_checks.push(("Domain↔Topology cross-reference", check_domain_vs_topology));
            domain_checks.push((
                "Derivation contract (reproduce outputs from data)",
                check_derivation_contract,
            ));
            domain_checks.push(("Validation claims vs FES data", check_validation_claims));
            domain_checks.push((
                "Simulation time consistency (MDP vs scope.toml)",
                check_simulation_times,
            ));
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
            println!("  [{check_idx}/{total_checks}] {status} — {label} ({added})");
        }
    }

    for (label, check_fn) in &domain_checks {
        check_idx += 1;
        let before = findings.len();
        check_fn(root, &mut findings);
        let added = findings.len() - before;
        if verbose {
            let status = if added == 0 { "PASS" } else { "FAIL" };
            println!("  [{check_idx}/{total_checks}] {status} — {label} ({added})");
        }
    }

    if verbose && !json_output {
        println!();
    }

    // Report
    let high = findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .count();
    let med = findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .count();
    let low = findings
        .iter()
        .filter(|f| f.severity == Severity::Low)
        .count();

    let elapsed_ms = audit_start.elapsed().as_millis() as u64;

    if json_output {
        // GuideStone-format structured JSON output
        let scope_path = root.join("scope.toml");
        let (artifact_name, artifact_version) =
            if let Ok(content) = std::fs::read_to_string(&scope_path) {
                let table: toml::Table = content.parse().unwrap_or_default();
                let art = table.get("artifact").and_then(|v| v.as_table());
                (
                    art.and_then(|a| a.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    art.and_then(|a| a.get("version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.0.0")
                        .to_string(),
                )
            } else {
                ("unknown".to_string(), "0.0.0".to_string())
            };

        let profile_id = profile.as_ref().map_or("none", |p| p.id.as_str());
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

        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );

        // Append to liveSpore.json if present and passed
        if high == 0 {
            append_livespore(root, total_checks, elapsed_ms);
        }
    } else {
        if findings.is_empty() {
            println!("  PASS — no findings. Artifact is handoff-ready.");
        } else {
            println!("  Findings: {high} HIGH, {med} MEDIUM, {low} LOW");
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
                println!("  VERDICT: CONDITIONAL PASS — fix {high} HIGH findings before handoff.");
            } else if med > 0 {
                println!("  VERDICT: PASS with recommendations — {med} MEDIUM findings.");
            } else {
                println!("  VERDICT: PASS — {low} LOW findings (cosmetic).");
            }
        }
        println!();

        // Append to liveSpore.json on success
        if high == 0 {
            append_livespore(root, total_checks, elapsed_ms);
        }
    }

    std::process::exit(i32::from(high > 0));
}

fn append_livespore(root: &Path, checks: usize, elapsed_ms: u64) {
    let livespore_path = root.join("liveSpore.json");
    if !livespore_path.exists() {
        return;
    }

    let hostname = std::env::var(litho_core::env_vars::HOSTNAME)
        .or_else(|_| std::env::var(litho_core::env_vars::HOST))
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
        "runtime_ms": elapsed_ms
    });

    let content = std::fs::read_to_string(&livespore_path).unwrap_or_default();

    // Unified schema: object with "envelope" + "validations"
    // Legacy: bare array [] or hotSpring-style {"liveSpore": {...}, ...}
    let mut doc: serde_json::Value =
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}));

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
            let mut env = doc
                .get("liveSpore")
                .cloned()
                .unwrap_or(serde_json::json!({}));
            if let Some(chain) = doc.get("provenance_chain") {
                env["provenance_chain"] = chain.clone();
            }
            if let Some(sw) = doc.get("software") {
                env["software"] = sw.clone();
            }
            env
        } else if doc.get("envelope").is_some() {
            doc.get("envelope")
                .cloned()
                .unwrap_or(serde_json::json!({}))
        } else {
            doc.clone()
        };
        doc = serde_json::json!({
            "envelope": envelope,
            "validations": [entry]
        });
    }

    std::fs::write(
        &livespore_path,
        serde_json::to_string_pretty(&doc).unwrap_or_default(),
    )
    .ok();
}
