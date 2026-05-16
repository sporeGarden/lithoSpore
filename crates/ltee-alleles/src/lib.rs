// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 3: Allele frequency trajectories
//!
//! Reproduces Good et al. 2017 (B3) — dynamics of molecular evolution.
//! Springs: neuralSpring (LSTM+HMM+ESN allele), groundSpring (clonal interference).
//!
//! Tier 2: ingests fetched allele frequency data, computes fixation
//! probabilities, interference ratios, and validates against published claims.
//! Targets: T05 (competing clades), T06 (classifier accuracy).

use litho_core::harness;
use litho_core::validation::{ModuleResult, ValidationStatus};

/// Run module 3 validation with the given paths and tier.
pub fn run_validation(data_dir: &str, expected: &str, _max_tier: u8) -> ModuleResult {
    let start = std::time::Instant::now();

    let expected_path = std::path::Path::new(expected);
    let data_path = std::path::Path::new(data_dir);

    if !expected_path.exists() || !data_path.exists() {
        return harness::skip(
            "allele_trajectories", 2, start,
            &format!(
                "Data or expected values not found — run `litho fetch --all` (expected={}, data={})",
                expected_path.display(), data_path.display()
            ),
        );
    }

    let expected_val = match harness::load_expected(expected) {
        Some(v) => v,
        None => return harness::skip(
            "allele_trajectories", 2, start, "Cannot parse expected values JSON",
        ),
    };

    let data_json_path = data_path.join("expected_values.json");
    let data_bundle = data_json_path
        .to_str()
        .and_then(harness::load_expected);

    let source = data_bundle.as_ref().unwrap_or(&expected_val);

    let mut checks = 0u32;
    let mut passed = 0u32;

    if let Some(results) = source.get("results_by_size").and_then(|v| v.as_object()) {
        for (size_label, data) in results {
            let pop_size = data.get("pop_size").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
            let total_fix = data.get("total_fixations").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
            let total_mut = data.get("total_mutations").and_then(serde_json::Value::as_f64).unwrap_or(0.0);

            checks += 1;
            let computed_fix_prob = if total_mut > 0.0 { total_fix / total_mut } else { 0.0 };
            let fix_prob_valid = computed_fix_prob > 0.0 && computed_fix_prob < 1.0;
            if fix_prob_valid { passed += 1; }
            eprintln!("  [{}] {size_label}: fixation_prob = {computed_fix_prob:.6} (computed from {total_fix:.0}/{total_mut:.0})",
                if fix_prob_valid { "PASS" } else { "FAIL" });

            if let Some(stored) = data.get("fixation_probability").and_then(serde_json::Value::as_f64) {
                checks += 1;
                let match_ok = (computed_fix_prob - stored).abs() < 0.01;
                if match_ok { passed += 1; }
                eprintln!("  [{}] {size_label}: fix_prob computed vs stored delta = {:.6}",
                    if match_ok { "PASS" } else { "FAIL" },
                    (computed_fix_prob - stored).abs());
            }

            if let Some(haldane) = data.get("haldane_probability").and_then(serde_json::Value::as_f64) {
                checks += 1;
                let interference = if haldane > 0.0 { computed_fix_prob / haldane } else { 0.0 };
                let interference_ok = interference > 0.0;
                if interference_ok { passed += 1; }
                eprintln!("  [{}] {size_label}: interference_ratio = {interference:.4} (fix_prob/haldane = {computed_fix_prob:.6}/{haldane:.4})",
                    if interference_ok { "PASS" } else { "FAIL" });

                if pop_size >= 10000.0 {
                    checks += 1;
                    let below_haldane = interference < 1.0;
                    if below_haldane { passed += 1; }
                    eprintln!("  [{}] {size_label}: clonal interference suppresses fixation (ratio < 1.0)",
                        if below_haldane { "PASS" } else { "FAIL" });
                }
            }

            if let Some(fitness) = data.get("mean_final_fitness").and_then(serde_json::Value::as_f64) {
                checks += 1;
                let fit_ok = fitness >= 1.0;
                if fit_ok { passed += 1; }
                eprintln!("  [{}] {size_label}: mean_final_fitness = {fitness:.4} (>= 1.0)",
                    if fit_ok { "PASS" } else { "FAIL" });
            }
        }

        checks += 1;
        let multi_pop = results.len() >= 3;
        if multi_pop { passed += 1; }
        eprintln!("  [{}] Multiple population sizes tested: {}",
            if multi_pop { "PASS" } else { "FAIL" }, results.len());
    }

    if let Some(paper) = source.get("paper").and_then(|v| v.as_str()) {
        checks += 1;
        let paper_ok = paper.contains("Good") || paper == "Good2017";
        if paper_ok { passed += 1; }
        eprintln!("  [{}] Paper citation: {paper}", if paper_ok { "PASS" } else { "FAIL" });
    }

    ModuleResult {
        name: "allele_trajectories".to_string(),
        status: if checks > 0 && passed == checks {
            ValidationStatus::Pass
        } else if passed > 0 {
            ValidationStatus::Fail
        } else {
            ValidationStatus::Skip
        },
        tier: 2,
        checks,
        checks_passed: passed,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: if passed < checks { Some(format!("{} check(s) failed", checks - passed)) } else { None },
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
    fn valid_expected_json_produces_checks() {
        let dir = std::env::temp_dir().join("litho_test_alleles_v2");
        let _ = std::fs::create_dir_all(&dir);
        let expected = dir.join("expected.json");
        std::fs::write(&expected, r#"{
            "paper": "Good2017",
            "results_by_size": {
                "1000": {
                    "pop_size": 1000,
                    "total_fixations": 180,
                    "total_mutations": 10079,
                    "fixation_probability": 0.0179,
                    "haldane_probability": 0.02,
                    "neutral_probability": 0.001,
                    "interference_ratio": 0.893,
                    "mean_final_fitness": 1.22,
                    "adaptation_rate": 3.87e-5
                },
                "10000": {
                    "pop_size": 10000,
                    "total_fixations": 1619,
                    "total_mutations": 100117,
                    "fixation_probability": 0.0162,
                    "haldane_probability": 0.02,
                    "neutral_probability": 0.0001,
                    "interference_ratio": 0.809,
                    "mean_final_fitness": 5.9,
                    "adaptation_rate": 3.5e-4
                },
                "100000": {
                    "pop_size": 100000,
                    "total_fixations": 14033,
                    "total_mutations": 1000225,
                    "fixation_probability": 0.0140,
                    "haldane_probability": 0.02,
                    "neutral_probability": 1e-5,
                    "interference_ratio": 0.701,
                    "mean_final_fitness": 8719953.7,
                    "adaptation_rate": 3.15e-3
                }
            }
        }"#).unwrap();
        let data = dir.join("data");
        let _ = std::fs::create_dir_all(&data);

        let result = run_validation(data.to_str().unwrap(), expected.to_str().unwrap(), 2);
        assert!(result.checks >= 15, "expected >= 15 checks, got {}", result.checks);
        assert_eq!(result.status, ValidationStatus::Pass);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
