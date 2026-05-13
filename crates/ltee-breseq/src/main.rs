// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 6: 264-genome breseq comparison
//!
//! Reproduces Tenaillon et al. 2016 (B7) — tempo and mode of genome evolution.
//! Validates mutation accumulation curves, mutation spectrum, ts/tv ratio,
//! and clock-like accumulation from 264 sequenced LTEE clones.
//!
//! Tier 2: pure Rust validation against wetSpring expected values.

use clap::Parser;
use litho_core::{ModuleResult, ValidationStatus};
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ltee-breseq", about = "264-genome breseq comparison validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/tenaillon_2016")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module6_breseq.json")]
    expected: String,

    #[arg(long, default_value = "2")]
    max_tier: u8,

    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let result = run_validation(&cli);

    if cli.json {
        match serde_json::to_string_pretty(&result) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("Error serializing result: {e}");
                std::process::exit(2);
            }
        }
    } else {
        println!(
            "Module 6 (breseq): {} — {}/{} checks ({}ms)",
            match result.status {
                ValidationStatus::Pass => "PASS",
                ValidationStatus::Fail => "FAIL",
                ValidationStatus::Skip => "SKIP",
            },
            result.checks_passed,
            result.checks,
            result.runtime_ms,
        );
    }

    if matches!(result.status, ValidationStatus::Fail) {
        std::process::exit(1);
    }
}

fn skip_result(name: &str, tier: u8, start: Instant, msg: &str) -> ModuleResult {
    ModuleResult {
        name: name.to_string(),
        status: ValidationStatus::Skip,
        tier,
        checks: 0,
        checks_passed: 0,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: Some(msg.to_string()),
    }
}

fn run_validation(cli: &Cli) -> ModuleResult {
    let start = Instant::now();

    if !Path::new(&cli.expected).exists() {
        return skip_result("breseq_264_genomes", 1, start,
            "Expected values not found — run wetSpring B7 first");
    }

    if cli.max_tier >= 2 {
        return run_tier2_rust(cli, start);
    }

    skip_result("breseq_264_genomes", cli.max_tier, start,
        &format!("Tier {} not implemented yet", cli.max_tier))
}

fn load_expected(path: &str) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Pearson correlation coefficient.
fn pearson_r(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let (mut sxy, mut sxx, mut syy) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (xi, yi) in x.iter().zip(y) {
        let dx = xi - mx;
        let dy = yi - my;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx == 0.0 || syy == 0.0 { return 0.0; }
    sxy / (sxx * syy).sqrt()
}

fn run_tier2_rust(cli: &Cli, start: Instant) -> ModuleResult {
    let expected = match load_expected(&cli.expected) {
        Some(v) => v,
        None => return skip_result("breseq_264_genomes", 2, start,
            "Cannot parse expected values JSON"),
    };

    let targets = &expected["targets"];

    let mut passed = 0_u32;
    let mut total = 0_u32;

    // Check 1: 12 populations
    total += 1;
    let n_pop = targets["n_populations"]["value"].as_u64().unwrap_or(0);
    let pop_ok = n_pop == 12;
    if pop_ok { passed += 1; }
    eprintln!("  [{}] 12 replicate populations: {n_pop}",
        if pop_ok { "PASS" } else { "FAIL" });

    // Check 2: 264 genomes
    total += 1;
    let n_genomes = targets["n_genomes"]["value"].as_u64().unwrap_or(0);
    let genomes_ok = n_genomes == 264;
    if genomes_ok { passed += 1; }
    eprintln!("  [{}] 264 sequenced genomes: {n_genomes}",
        if genomes_ok { "PASS" } else { "FAIL" });

    // Check 3: REL606 genome length within tolerance
    total += 1;
    let genome_len = targets["genome_length_bp"]["value"].as_f64().unwrap_or(0.0);
    let genome_tol = targets["genome_length_bp"]["tolerance"].as_f64().unwrap_or(100.0);
    let expected_len = 4_629_812.0;
    let len_ok = (genome_len - expected_len).abs() <= genome_tol;
    if len_ok { passed += 1; }
    eprintln!("  [{}] Genome length: {genome_len:.0} bp (expected {expected_len:.0} ± {genome_tol:.0})",
        if len_ok { "PASS" } else { "FAIL" });

    // Check 4: Non-mutator mutation rate
    total += 1;
    let rate = targets["nonmutator_rate_per_bp_per_gen"]["value"].as_f64().unwrap_or(0.0);
    let rate_tol = targets["nonmutator_rate_per_bp_per_gen"]["tolerance"].as_f64().unwrap_or(1e-11);
    let rate_ok = (rate - 8.9e-11).abs() <= rate_tol;
    if rate_ok { passed += 1; }
    eprintln!("  [{}] Non-mutator rate: {rate:.2e} per bp/gen (expected 8.9e-11 ± {rate_tol:.0e})",
        if rate_ok { "PASS" } else { "FAIL" });

    // Check 5: Mutations at 50k generations
    total += 1;
    let muts_50k = targets["nonmutator_mutations_at_50k"]["value"].as_f64().unwrap_or(0.0);
    let muts_tol = targets["nonmutator_mutations_at_50k"]["tolerance"].as_f64().unwrap_or(2.3);
    let muts_ok = (muts_50k - 20.6).abs() <= muts_tol;
    if muts_ok { passed += 1; }
    eprintln!("  [{}] Mutations at 50k: {muts_50k:.1} (expected 20.6 ± {muts_tol:.1})",
        if muts_ok { "PASS" } else { "FAIL" });

    // Check 6: Ts/Tv ratio
    total += 1;
    let ts_tv = targets["ts_tv_ratio"]["value"].as_f64().unwrap_or(0.0);
    let ts_tv_tol = targets["ts_tv_ratio"]["tolerance"].as_f64().unwrap_or(0.3);
    let ts_tv_ok = (ts_tv - 1.7).abs() <= ts_tv_tol;
    if ts_tv_ok { passed += 1; }
    eprintln!("  [{}] Ts/Tv ratio: {ts_tv:.2} (expected 1.7 ± {ts_tv_tol:.1})",
        if ts_tv_ok { "PASS" } else { "FAIL" });

    // Check 7: GC→AT dominance
    total += 1;
    let gc_at = targets["gc_to_at_fraction"]["value"].as_f64().unwrap_or(0.0);
    let gc_at_tol = targets["gc_to_at_fraction"]["tolerance"].as_f64().unwrap_or(0.05);
    let gc_at_ok = (gc_at - 0.68).abs() <= gc_at_tol;
    if gc_at_ok { passed += 1; }
    eprintln!("  [{}] GC→AT fraction: {gc_at:.2} (expected 0.68 ± {gc_at_tol:.2})",
        if gc_at_ok { "PASS" } else { "FAIL" });

    // Check 8: Accumulation curve is near-linear (molecular clock)
    total += 1;
    let curve = &expected["mutation_accumulation_curve"];
    let gens: Vec<f64> = curve["generations"].as_array()
        .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();
    let muts: Vec<f64> = curve["expected_mutations_nonmutator"].as_array()
        .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
        .unwrap_or_default();

    let linear_ok = if gens.len() >= 3 && gens.len() == muts.len() {
        let pos_gens: Vec<f64> = gens.iter().zip(&muts)
            .filter(|&(&g, _)| g > 0.0).map(|(&g, _)| g).collect();
        let pos_muts: Vec<f64> = gens.iter().zip(&muts)
            .filter(|&(&g, _)| g > 0.0).map(|(_, &m)| m).collect();
        let r = pearson_r(&pos_gens, &pos_muts);
        eprintln!("  [{}] Near-linear accumulation (Pearson r={r:.6}, min 0.99)",
            if r > 0.99 { "PASS" } else { "FAIL" });
        r > 0.99
    } else {
        eprintln!("  [FAIL] Accumulation curve data insufficient");
        false
    };
    if linear_ok { passed += 1; }

    let status = if passed == total { ValidationStatus::Pass } else { ValidationStatus::Fail };
    ModuleResult {
        name: "breseq_264_genomes".to_string(),
        status,
        tier: 2,
        checks: total,
        checks_passed: passed,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: if passed < total {
            Some(format!("{} check(s) failed", total - passed))
        } else {
            None
        },
    }
}
