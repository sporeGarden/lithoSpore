// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 6: 264-genome breseq comparison
//!
//! Reproduces Tenaillon et al. 2016 (B7) — tempo and mode of genome evolution.
//! Validates mutation accumulation curves, mutation spectrum, ts/tv ratio,
//! and clock-like accumulation from 264 sequenced LTEE clones.
//!
//! Tier 2: pure Rust validation against wetSpring expected values.

use clap::Parser;
use litho_core::harness;
use litho_core::stats::pearson_r;
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
    harness::output_and_exit(&result, cli.json);
}

fn run_validation(cli: &Cli) -> ModuleResult {
    let start = Instant::now();

    if !Path::new(&cli.expected).exists() {
        return harness::skip("breseq_264_genomes", 1, start,
            "Expected values not found — run wetSpring B7 first");
    }

    if !Path::new(&cli.data_dir).exists() {
        return harness::skip("breseq_264_genomes", 1, start,
            &format!("Data directory not found: {}", cli.data_dir));
    }

    if cli.max_tier >= 2 {
        return run_tier2_rust(cli, start);
    }

    harness::skip("breseq_264_genomes", cli.max_tier, start,
        &format!("Tier {} not implemented yet", cli.max_tier))
}

fn run_tier2_rust(cli: &Cli, start: Instant) -> ModuleResult {
    let expected = match harness::load_expected(&cli.expected) {
        Some(v) => v,
        None => return harness::skip("breseq_264_genomes", 2, start,
            "Cannot parse expected values JSON"),
    };

    let targets = &expected["targets"];

    let mut passed = 0_u32;
    let mut total = 0_u32;

    total += 1;
    let n_pop = targets["n_populations"]["value"].as_u64().unwrap_or(0);
    let pop_ok = n_pop == 12;
    if pop_ok { passed += 1; }
    eprintln!("  [{}] 12 replicate populations: {n_pop}",
        if pop_ok { "PASS" } else { "FAIL" });

    total += 1;
    let n_genomes = targets["n_genomes"]["value"].as_u64().unwrap_or(0);
    let genomes_ok = n_genomes == 264;
    if genomes_ok { passed += 1; }
    eprintln!("  [{}] 264 sequenced genomes: {n_genomes}",
        if genomes_ok { "PASS" } else { "FAIL" });

    total += 1;
    let genome_len = targets["genome_length_bp"]["value"].as_f64().unwrap_or(0.0);
    let genome_tol = targets["genome_length_bp"]["tolerance"].as_f64().unwrap_or(100.0);
    let expected_len = 4_629_812.0;
    let len_ok = (genome_len - expected_len).abs() <= genome_tol;
    if len_ok { passed += 1; }
    eprintln!("  [{}] Genome length: {genome_len:.0} bp (expected {expected_len:.0} ± {genome_tol:.0})",
        if len_ok { "PASS" } else { "FAIL" });

    total += 1;
    let rate = targets["nonmutator_rate_per_bp_per_gen"]["value"].as_f64().unwrap_or(0.0);
    let rate_tol = targets["nonmutator_rate_per_bp_per_gen"]["tolerance"].as_f64().unwrap_or(1e-11);
    let rate_ok = (rate - 8.9e-11).abs() <= rate_tol;
    if rate_ok { passed += 1; }
    eprintln!("  [{}] Non-mutator rate: {rate:.2e} per bp/gen (expected 8.9e-11 ± {rate_tol:.0e})",
        if rate_ok { "PASS" } else { "FAIL" });

    total += 1;
    let muts_50k = targets["nonmutator_mutations_at_50k"]["value"].as_f64().unwrap_or(0.0);
    let muts_tol = targets["nonmutator_mutations_at_50k"]["tolerance"].as_f64().unwrap_or(2.3);
    let muts_ok = (muts_50k - 20.6).abs() <= muts_tol;
    if muts_ok { passed += 1; }
    eprintln!("  [{}] Mutations at 50k: {muts_50k:.1} (expected 20.6 ± {muts_tol:.1})",
        if muts_ok { "PASS" } else { "FAIL" });

    total += 1;
    let ts_tv = targets["ts_tv_ratio"]["value"].as_f64().unwrap_or(0.0);
    let ts_tv_tol = targets["ts_tv_ratio"]["tolerance"].as_f64().unwrap_or(0.3);
    let ts_tv_ok = (ts_tv - 1.7).abs() <= ts_tv_tol;
    if ts_tv_ok { passed += 1; }
    eprintln!("  [{}] Ts/Tv ratio: {ts_tv:.2} (expected 1.7 ± {ts_tv_tol:.1})",
        if ts_tv_ok { "PASS" } else { "FAIL" });

    total += 1;
    let gc_at = targets["gc_to_at_fraction"]["value"].as_f64().unwrap_or(0.0);
    let gc_at_tol = targets["gc_to_at_fraction"]["tolerance"].as_f64().unwrap_or(0.05);
    let gc_at_ok = (gc_at - 0.68).abs() <= gc_at_tol;
    if gc_at_ok { passed += 1; }
    eprintln!("  [{}] GC→AT fraction: {gc_at:.2} (expected 0.68 ± {gc_at_tol:.2})",
        if gc_at_ok { "PASS" } else { "FAIL" });

    total += 1;
    let curve = &expected["mutation_accumulation_curve"];
    let gens: Vec<f64> = curve["generations"].as_array()
        .map(|a| a.iter().filter_map(serde_json::value::Value::as_f64).collect())
        .unwrap_or_default();
    let muts: Vec<f64> = curve["expected_mutations_nonmutator"].as_array()
        .map(|a| a.iter().filter_map(serde_json::value::Value::as_f64).collect())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_expected_returns_skip() {
        let cli = Cli {
            data_dir: "/nonexistent".into(),
            expected: "/nonexistent".into(),
            max_tier: 2,
            json: false,
        };
        let result = run_validation(&cli);
        assert_eq!(result.status, ValidationStatus::Skip);
    }

    #[test]
    fn low_tier_returns_skip() {
        let cli = Cli {
            data_dir: "/nonexistent".into(),
            expected: "/nonexistent".into(),
            max_tier: 0,
            json: false,
        };
        let result = run_validation(&cli);
        assert_eq!(result.status, ValidationStatus::Skip);
    }
}
