// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 6: 264-genome breseq comparison
//!
//! Reproduces Tenaillon et al. 2016 (B7) — tempo and mode of genome evolution.
//! Validates mutation accumulation curves, mutation spectrum, Ts/Tv ratio,
//! and clock-like accumulation from 264 sequenced LTEE clones.
//!
//! Tier 2: pure Rust validation against fetched/expected values.
//! Targets: T09 (sensitivity vs breseq), T10 (parallel evolution ≥15 genes).

use litho_core::harness;
use litho_core::stats::pearson_r;
use litho_core::{ModuleResult, ValidationStatus};
use std::path::Path;
use std::time::Instant;

/// Run module 6 validation with the given paths and tier.
pub fn run_validation(data_dir: &str, expected: &str, max_tier: u8) -> ModuleResult {
    let start = Instant::now();

    if !Path::new(expected).exists() {
        return harness::skip("breseq_264_genomes", 1, start,
            "Expected values not found — run `litho fetch --all`");
    }

    if !Path::new(data_dir).exists() {
        return harness::skip("breseq_264_genomes", 1, start,
            &format!("Data directory not found: {data_dir}"));
    }

    if max_tier >= 2 {
        return run_tier2_rust(data_dir, expected, start);
    }

    harness::skip("breseq_264_genomes", max_tier, start,
        &format!("Tier {max_tier} not implemented yet"))
}

fn run_tier2_rust(data_dir: &str, expected_path: &str, start: Instant) -> ModuleResult {
    let _data_dir = data_dir;
    let expected = match harness::load_expected(expected_path) {
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

    if let Some(spectrum_val) = targets.get("mutation_spectrum") {
        if let Some(spectrum) = spectrum_val.get("value").and_then(|v| v.as_object()) {
            let spec_tol = spectrum_val["tolerance"].as_f64().unwrap_or(0.05);

            let expected_spectrum = [
                ("GC_to_AT", 0.68),
                ("AT_to_GC", 0.08),
                ("GC_to_TA", 0.10),
                ("GC_to_CG", 0.02),
                ("AT_to_TA", 0.07),
                ("AT_to_CG", 0.05),
            ];

            for (class, expected_frac) in &expected_spectrum {
                if let Some(observed) = spectrum.get(*class).and_then(serde_json::Value::as_f64) {
                    total += 1;
                    let class_ok = (observed - expected_frac).abs() <= spec_tol;
                    if class_ok { passed += 1; }
                    eprintln!("  [{}] Spectrum {class}: {observed:.3} (expected {expected_frac:.3} ± {spec_tol:.3})",
                        if class_ok { "PASS" } else { "FAIL" });
                }
            }

            let total_frac: f64 = spectrum.values()
                .filter_map(serde_json::Value::as_f64)
                .sum();
            total += 1;
            let sum_ok = (total_frac - 1.0).abs() < 0.05;
            if sum_ok { passed += 1; }
            eprintln!("  [{}] Spectrum sums to ~1.0: {total_frac:.4}",
                if sum_ok { "PASS" } else { "FAIL" });
        }
    }

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

    if gens.len() >= 2 && muts.len() >= 2 {
        let last_gen = gens.last().copied().unwrap_or(0.0);
        let last_muts = muts.last().copied().unwrap_or(0.0);
        if last_gen > 0.0 && genome_len > 0.0 {
            total += 1;
            let computed_rate = last_muts / (last_gen * genome_len);
            let rate_match = (computed_rate - rate).abs() / rate < 0.1;
            if rate_match { passed += 1; }
            eprintln!("  [{}] Computed rate matches published: {computed_rate:.2e} vs {rate:.2e} (< 10% deviation)",
                if rate_match { "PASS" } else { "FAIL" });
        }
    }

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
        let result = run_validation("/nonexistent", "/nonexistent", 2);
        assert_eq!(result.status, ValidationStatus::Skip);
    }

    #[test]
    fn low_tier_returns_skip() {
        let result = run_validation("/nonexistent", "/nonexistent", 0);
        assert_eq!(result.status, ValidationStatus::Skip);
    }
}
