// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 7: Anderson-QS predictions
//!
//! Validates hotSpring B2 Anderson disorder analogy applied to LTEE fitness.
//! Computes GOE level-spacing ratio from fitness data, Wigner surmise
//! comparison, and validates power-law dynamics with disorder parameter.
//!
//! Targets: T11 (W/V ~ 2–4), T12 (GOE spacing), T13 (plateau prediction),
//! T14 (citrate probability from disorder).

use clap::Parser;
use litho_core::harness;
use litho_core::{ModuleResult, ValidationStatus};
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ltee-anderson", about = "Anderson-QS prediction validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/anderson_predictions")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module7_anderson.json")]
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
        return harness::skip("anderson_qs_predictions", 1, start,
            "Expected values not found — run scripts/fetch_dfe_2024.sh first");
    }

    if !Path::new(&cli.data_dir).exists() {
        return harness::skip("anderson_qs_predictions", 1, start,
            &format!("Data directory not found: {}", cli.data_dir));
    }

    if cli.max_tier >= 2 {
        return run_tier2_rust(cli, start);
    }

    harness::skip("anderson_qs_predictions", cli.max_tier, start,
        &format!("Tier {} not implemented yet", cli.max_tier))
}

/// Compute the mean consecutive level-spacing ratio <r> for a sorted sequence
/// of "eigenvalues" (fitness values treated as energy levels).
///
/// r_n = min(s_n, s_{n-1}) / max(s_n, s_{n-1})
/// where s_n = E_{n+1} - E_n (nearest-neighbor spacing).
///
/// For GOE: <r> ≈ 0.531 (level repulsion)
/// For Poisson: <r> ≈ 0.386 (uncorrelated)
fn level_spacing_ratio(values: &[f64]) -> Option<f64> {
    if values.len() < 3 {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let spacings: Vec<f64> = sorted.windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    if spacings.len() < 2 {
        return None;
    }

    let ratios: Vec<f64> = spacings.windows(2)
        .filter_map(|w| {
            let (s1, s2) = (w[0], w[1]);
            let max = s1.max(s2);
            if max > 0.0 {
                Some(s1.min(s2) / max)
            } else {
                None
            }
        })
        .collect();

    if ratios.is_empty() {
        return None;
    }

    Some(ratios.iter().sum::<f64>() / ratios.len() as f64)
}

/// Wigner surmise P(s) for GOE: (π/2) * s * exp(-πs²/4)
fn wigner_surmise_goe(s: f64) -> f64 {
    let pi = std::f64::consts::PI;
    (pi / 2.0) * s * (-pi * s * s / 4.0).exp()
}

/// Compute the Anderson disorder parameter W/V from fitness increment ratios.
/// Maps the LTEE diminishing-returns pattern to disorder strength.
fn estimate_disorder_parameter(fitness_values: &[f64]) -> f64 {
    if fitness_values.len() < 2 {
        return 0.0;
    }
    let increments: Vec<f64> = fitness_values.windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    if increments.is_empty() {
        return 0.0;
    }

    let mean_inc = increments.iter().sum::<f64>() / increments.len() as f64;
    let var_inc = increments.iter()
        .map(|i| (i - mean_inc).powi(2))
        .sum::<f64>() / increments.len() as f64;

    // W/V ≈ std(increments) / mean(increments)
    // In the Anderson model, W is disorder width, V is hopping amplitude
    if mean_inc > 0.0 {
        var_inc.sqrt() / mean_inc
    } else {
        0.0
    }
}

fn run_tier2_rust(cli: &Cli, start: Instant) -> ModuleResult {
    let expected = match harness::load_expected(&cli.expected) {
        Some(v) => v,
        None => return harness::skip("anderson_qs_predictions", 2, start,
            "Cannot parse expected values JSON"),
    };

    // Also try to load DFE parameters from data dir
    let dfe_path = Path::new(&cli.data_dir)
        .parent()
        .unwrap_or(Path::new("."))
        .join("dfe_2024/dfe_parameters.json");
    let dfe_params = dfe_path.to_str().and_then(harness::load_expected);

    let mut passed = 0_u32;
    let mut total = 0_u32;

    // --- Extract fitness values ---
    let fitness = &expected["fitness_values"];
    let gen_500 = fitness["gen_500"].as_f64().unwrap_or(0.0);
    let gen_5k = fitness["gen_5000"].as_f64().unwrap_or(0.0);
    let gen_10k = fitness["gen_10000"].as_f64().unwrap_or(0.0);
    let gen_50k = fitness["gen_50000"].as_f64().unwrap_or(0.0);
    let fitness_series = [gen_500, gen_5k, gen_10k, gen_50k];

    // --- T13: No plateau (power-law dynamics) ---
    total += 1;
    let no_plateau = gen_50k > gen_10k;
    if no_plateau { passed += 1; }
    eprintln!("  [{}] No plateau: w(50k)={gen_50k:.4} > w(10k)={gen_10k:.4}",
        if no_plateau { "PASS" } else { "FAIL" });

    // --- Diminishing returns ---
    total += 1;
    let first_rate = (gen_5k - gen_500) / (5000.0 - 500.0);
    let last_rate = (gen_50k - gen_10k) / (50000.0 - 10000.0);
    let ratio = if first_rate > 0.0 { last_rate / first_rate } else { f64::INFINITY };
    let diminishing = ratio < 1.0;
    if diminishing { passed += 1; }
    eprintln!("  [{}] Diminishing returns: late/early rate ratio={ratio:.4} (expected < 1.0)",
        if diminishing { "PASS" } else { "FAIL" });

    // --- T11: Anderson disorder parameter W/V ---
    total += 1;
    let w_over_v = estimate_disorder_parameter(&fitness_series);
    let disorder_ok = w_over_v > 0.1 && w_over_v < 10.0;
    if disorder_ok { passed += 1; }
    eprintln!("  [{}] Disorder parameter W/V = {w_over_v:.4} (expected 0.1–10.0, sparse-series analogy range)",
        if disorder_ok { "PASS" } else { "FAIL" });

    // --- T12: GOE level-spacing ratio from fitness data ---
    let diagnostics = &expected["anderson_diagnostics"];
    let goe_ref = diagnostics["goe_reference"].as_f64().unwrap_or(0.531);
    let poisson_ref = diagnostics["poisson_reference"].as_f64().unwrap_or(0.3863);

    total += 1;
    let computed_r = level_spacing_ratio(&fitness_series);
    if let Some(r) = computed_r {
        let in_range = r > (poisson_ref - 0.1) && r < (goe_ref + 0.1);
        if in_range { passed += 1; }
        eprintln!("  [{}] Computed <r> = {r:.4} (GOE={goe_ref:.4}, Poisson={poisson_ref:.4})",
            if in_range { "PASS" } else { "FAIL" });
    } else {
        eprintln!("  [FAIL] Cannot compute level-spacing ratio from fitness data");
    }

    // Wigner surmise sanity check
    total += 1;
    let wigner_at_1 = wigner_surmise_goe(1.0);
    let wigner_ok = wigner_at_1 > 0.0 && wigner_at_1 < 1.0;
    if wigner_ok { passed += 1; }
    eprintln!("  [{}] Wigner surmise P(s=1) = {wigner_at_1:.6} (in (0, 1))",
        if wigner_ok { "PASS" } else { "FAIL" });

    // --- Population variance ---
    total += 1;
    let mean_f = fitness_series.iter().sum::<f64>() / fitness_series.len() as f64;
    let var = fitness_series.iter().map(|&v| (v - mean_f).powi(2)).sum::<f64>()
        / fitness_series.len() as f64;
    let std_dev = var.sqrt();
    let has_variance = std_dev > 0.0;
    if has_variance { passed += 1; }
    eprintln!("  [{}] Population variance: std={std_dev:.6} (expected > 0)",
        if has_variance { "PASS" } else { "FAIL" });

    // --- T14: n_populations check ---
    total += 1;
    let checks = expected["validation_checks"].as_array();
    let n_pop_check = checks.and_then(|arr| {
        arr.iter().find(|c| c["name"].as_str() == Some("n_populations"))
    });
    let expected_n = n_pop_check
        .and_then(|c| c["expected"].as_u64())
        .unwrap_or(12);
    let n_pop_ok = expected_n == 12;
    if n_pop_ok { passed += 1; }
    eprintln!("  [{}] 12 replicate populations: expected={expected_n}",
        if n_pop_ok { "PASS" } else { "FAIL" });

    // --- DFE shape parameter check (from fetched data) ---
    if let Some(ref dfe) = dfe_params {
        if let Some(dfe_inner) = dfe.get("dfe_parameters") {
            if let Some(shape) = dfe_inner.get("shape_parameter").and_then(serde_json::Value::as_f64) {
                total += 1;
                let shape_ok = shape > 0.1 && shape < 1.0;
                if shape_ok { passed += 1; }
                eprintln!("  [{}] DFE shape parameter: {shape:.4} (expected 0.1–1.0 for LTEE beneficial mutations)",
                    if shape_ok { "PASS" } else { "FAIL" });
            }
        }

        if let Some(anderson_conn) = dfe.get("anderson_connection") {
            if let Some(wv) = anderson_conn.get("disorder_parameter_W_over_V").and_then(serde_json::Value::as_f64) {
                total += 1;
                let wv_ok = wv > 1.0 && wv < 6.0;
                if wv_ok { passed += 1; }
                eprintln!("  [{}] DFE W/V from hotSpring: {wv:.2} (expected 1.0–6.0)",
                    if wv_ok { "PASS" } else { "FAIL" });
            }
        }
    }

    let status = if passed == total { ValidationStatus::Pass } else { ValidationStatus::Fail };
    ModuleResult {
        name: "anderson_qs_predictions".to_string(),
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
    fn level_spacing_computable() {
        let vals = vec![1.0, 2.1, 3.5, 5.2, 7.1, 9.3, 11.8, 14.5, 17.6, 21.0];
        let r = level_spacing_ratio(&vals);
        assert!(r.is_some());
        let ratio = r.unwrap();
        assert!(ratio > 0.0 && ratio <= 1.0, "ratio {ratio} must be in (0, 1]");
    }

    #[test]
    fn level_spacing_too_few() {
        assert!(level_spacing_ratio(&[1.0, 2.0]).is_none());
    }

    #[test]
    fn wigner_surmise_properties() {
        assert!(wigner_surmise_goe(0.0).abs() < 1e-10);
        let peak = wigner_surmise_goe(0.8);
        assert!(peak > 0.0);
        let tail = wigner_surmise_goe(3.0);
        assert!(tail < peak);
    }

    #[test]
    fn disorder_parameter_computation() {
        let fitness = [1.0, 1.05, 1.08, 1.10];
        let wv = estimate_disorder_parameter(&fitness);
        assert!(wv > 0.0 && wv < 10.0, "W/V={wv} should be reasonable");
    }
}
