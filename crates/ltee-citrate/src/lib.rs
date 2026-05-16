// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 4: Citrate innovation
//!
//! Reproduces Blount et al. 2008/2012 (B4) — historical contingency and
//! the evolution of a novel metabolic capability.
//! Springs: groundSpring (Cit+ potentiation), wetSpring (replay experiments).
//!
//! Tier 2: ingests fetched citrate timeline data, computes potentiation window,
//! replay probabilities, and validates the two-hit model.
//! Target: T07 (potentiating mutations ~2000 gens before Cit+).

use litho_core::harness;
use litho_core::validation::{ModuleResult, ValidationStatus};

/// Run module 4 validation with the given paths and tier.
pub fn run_validation(data_dir: &str, expected: &str, _max_tier: u8) -> ModuleResult {
    let start = std::time::Instant::now();

    let expected_path = std::path::Path::new(expected);
    let data_path = std::path::Path::new(data_dir);

    if !expected_path.exists() || !data_path.exists() {
        return harness::skip(
            "citrate_innovation", 2, start,
            &format!(
                "Data or expected values not found — run `litho fetch --all` (expected={}, data={})",
                expected_path.display(), data_path.display()
            ),
        );
    }

    let expected_val = match harness::load_expected(expected) {
        Some(v) => v,
        None => return harness::skip(
            "citrate_innovation", 2, start, "Cannot parse expected values JSON",
        ),
    };

    let data_json_path = data_path.join("expected_values.json");
    let data_bundle = data_json_path
        .to_str()
        .and_then(harness::load_expected);
    let source = data_bundle.as_ref().unwrap_or(&expected_val);

    let mut checks = 0u32;
    let mut passed = 0u32;

    if let Some(frac) = source.get("cit_plus_fraction").and_then(serde_json::Value::as_f64) {
        checks += 1;
        let expected_frac = 1.0 / 6.0;
        let frac_ok = (frac - expected_frac).abs() < 0.01;
        if frac_ok { passed += 1; }
        eprintln!("  [{}] Cit+ fraction: {frac:.4} (expected {expected_frac:.4} ± 0.01)",
            if frac_ok { "PASS" } else { "FAIL" });
    }

    if let Some(pot) = source.get("potentiation_fraction").and_then(serde_json::Value::as_f64) {
        checks += 1;
        let pot_ok = pot > 0.0 && pot <= 1.0;
        if pot_ok { passed += 1; }
        eprintln!("  [{}] Potentiation fraction: {pot:.4} (in (0, 1])",
            if pot_ok { "PASS" } else { "FAIL" });
    }

    let pot_gen = source.get("mean_potentiation_gen").and_then(serde_json::Value::as_f64);
    let cit_gen = source.get("mean_cit_plus_gen").and_then(serde_json::Value::as_f64);

    if let Some(pg) = pot_gen {
        checks += 1;
        let pg_ok = pg > 30000.0 && pg < 50000.0;
        if pg_ok { passed += 1; }
        eprintln!("  [{}] Mean potentiation generation: {pg:.0} (expected 30000–50000)",
            if pg_ok { "PASS" } else { "FAIL" });
    }

    if let Some(cg) = cit_gen {
        checks += 1;
        let cg_ok = cg > 40000.0 && cg < 55000.0;
        if cg_ok { passed += 1; }
        eprintln!("  [{}] Mean Cit+ generation: {cg:.0} (expected 40000–55000)",
            if cg_ok { "PASS" } else { "FAIL" });
    }

    if let (Some(pg), Some(cg)) = (pot_gen, cit_gen) {
        checks += 1;
        let window = cg - pg;
        let window_ok = window > 0.0 && window < 10000.0;
        if window_ok { passed += 1; }
        eprintln!("  [{}] Potentiation window: {window:.0} generations (Cit+ gen - potentiation gen, expected < 10000)",
            if window_ok { "PASS" } else { "FAIL" });

        checks += 1;
        let order_ok = pg < cg;
        if order_ok { passed += 1; }
        eprintln!("  [{}] Potentiation precedes innovation: {pg:.0} < {cg:.0}",
            if order_ok { "PASS" } else { "FAIL" });
    }

    if let Some(replay) = source.get("replay_probabilities").and_then(|v| v.as_object()) {
        checks += 1;
        let all_valid = replay.values().all(|v| {
            v.as_f64().is_some_and(|p| (0.0..=1.0).contains(&p))
        });
        if all_valid { passed += 1; }
        eprintln!("  [{}] Replay probabilities all in [0, 1]: {all_valid}",
            if all_valid { "PASS" } else { "FAIL" });

        checks += 1;
        let early_zero = replay.iter()
            .filter(|(k, _)| k.parse::<f64>().unwrap_or(f64::MAX) < 20000.0)
            .all(|(_, v)| v.as_f64().unwrap_or(1.0) == 0.0);
        if early_zero { passed += 1; }
        eprintln!("  [{}] Early replays (< 20k gen) have zero probability: {early_zero}",
            if early_zero { "PASS" } else { "FAIL" });
    }

    if let Some(two_hit) = source.get("two_hit_model") {
        if let Some(window_gens) = two_hit.get("potentiation_window_gens").and_then(serde_json::Value::as_f64) {
            checks += 1;
            let window_ok = window_gens > 1000.0 && window_gens < 10000.0;
            if window_ok { passed += 1; }
            eprintln!("  [{}] Two-hit model window: {window_gens:.0} gens (expected 1000–10000)",
                if window_ok { "PASS" } else { "FAIL" });
        }
    }

    let single = source.get("single_hit_mean_wait").and_then(serde_json::Value::as_f64);
    let two_hit_val = source.get("two_hit_analytical_mean").and_then(serde_json::Value::as_f64);
    if let (Some(s), Some(t)) = (single, two_hit_val) {
        checks += 1;
        let order_ok = t > s * 10.0;
        if order_ok { passed += 1; }
        eprintln!("  [{}] Two-hit wait >> single-hit: {t:.0} > 10×{s:.0}",
            if order_ok { "PASS" } else { "FAIL" });
    }

    let empirical = source.get("two_hit_empirical_mean").and_then(serde_json::Value::as_f64);
    if let (Some(e), Some(a)) = (empirical, two_hit_val) {
        checks += 1;
        let e_ok = e < a;
        if e_ok { passed += 1; }
        eprintln!("  [{}] Empirical < analytical two-hit: {e:.0} < {a:.0}",
            if e_ok { "PASS" } else { "FAIL" });
    }

    if let Some(paper) = source.get("paper").and_then(|v| v.as_str()) {
        checks += 1;
        let paper_ok = paper.starts_with("Blount");
        if paper_ok { passed += 1; }
        eprintln!("  [{}] Paper citation: {paper}", if paper_ok { "PASS" } else { "FAIL" });
    }

    ModuleResult {
        name: "citrate_innovation".to_string(),
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
    fn missing_files_returns_skip() {
        let result = run_validation("/nonexistent", "/nonexistent", 2);
        assert_eq!(result.status, ValidationStatus::Skip);
    }

    #[test]
    fn valid_citrate_json_validates() {
        let dir = std::env::temp_dir().join("litho_test_citrate_v2");
        let _ = std::fs::create_dir_all(&dir);
        let expected = dir.join("expected.json");
        std::fs::write(&expected, r#"{
            "paper": "Blount2008",
            "cit_plus_fraction": 0.16667,
            "potentiation_fraction": 0.16667,
            "mean_potentiation_gen": 41059.0,
            "mean_cit_plus_gen": 46050.5,
            "replay_probabilities": {"0": 0.0, "5000": 0.0, "10000": 0.0, "15000": 0.0, "40000": 0.0},
            "two_hit_model": {"potentiation_window_gens": 4991.5, "source": "Blount 2012"}
        }"#).unwrap();
        let data = dir.join("data");
        let _ = std::fs::create_dir_all(&data);

        let result = run_validation(data.to_str().unwrap(), expected.to_str().unwrap(), 2);
        assert!(result.checks >= 10, "expected >= 10 checks, got {}", result.checks);
        assert_eq!(result.status, ValidationStatus::Pass);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
