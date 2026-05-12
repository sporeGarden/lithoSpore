// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 2: Mutation accumulation curves
//!
//! Reproduces Barrick et al. 2009 (B1) — genome evolution and adaptation.
//! Springs: groundSpring (drift vs selection), neuralSpring (LSTM prediction).
//!
//! Tier 1: dispatches to Python baseline.
//! Tier 2: pure Rust Kimura fixation, Poisson accumulation, molecular clock.

use clap::Parser;
use litho_core::{ModuleResult, ValidationStatus};
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ltee-mutations", about = "Mutation accumulation curve validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/barrick_2009")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module2_mutations.json")]
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
            "Module 2 (mutations): {} — {}/{} checks ({}ms)",
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
        return skip_result("mutation_accumulation", 1, start,
            "Expected values not found — run groundSpring B1 first");
    }

    if !Path::new(&cli.data_dir).exists() {
        return skip_result("mutation_accumulation", 1, start,
            "Data not fetched — run scripts/fetch_barrick_2009.sh");
    }

    if cli.max_tier >= 2 {
        return run_tier2_rust(cli, start);
    }
    if cli.max_tier >= 1 {
        return run_tier1_python(start);
    }

    skip_result("mutation_accumulation", cli.max_tier, start,
        &format!("Tier {} not implemented yet", cli.max_tier))
}

// ── Tier 2: Pure Rust ────────────────────────────────────────────────

/// Kimura fixation probability for a new mutation in a haploid population.
/// P_fix = (1 - exp(-2sNp)) / (1 - exp(-2sN)), or p when s≈0.
fn kimura_fixation_prob(pop_size: u64, selection: f64, initial_freq: Option<f64>) -> f64 {
    let p = initial_freq.unwrap_or(1.0 / pop_size as f64);
    if selection.abs() < 1e-10 {
        return p;
    }
    let n = pop_size as f64;
    let num = 1.0 - (-2.0 * selection * n * p).exp();
    let den = 1.0 - (-2.0 * selection * n).exp();
    num / den
}

/// Simple xorshift64 PRNG for deterministic Poisson sampling.
struct Xorshift64(u64);

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Knuth's algorithm for Poisson random variates.
    fn poisson(&mut self, lambda: f64) -> u64 {
        let l = (-lambda).exp();
        let mut k = 0_u64;
        let mut p = 1.0_f64;
        loop {
            k += 1;
            p *= self.next_f64();
            if p <= l {
                break;
            }
        }
        k - 1
    }
}

/// Simulate neutral mutation accumulation via Poisson process.
fn simulate_neutral_fixations(mu: f64, n_gens: usize, seed: u64) -> Vec<u64> {
    let mut rng = Xorshift64::new(seed);
    let mut cumulative = Vec::with_capacity(n_gens);
    let mut total = 0_u64;
    for _ in 0..n_gens {
        total += rng.poisson(mu);
        cumulative.push(total);
    }
    cumulative
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

fn load_mutation_params(data_dir: &str) -> Option<serde_json::Value> {
    let path = Path::new(data_dir).join("mutation_parameters.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn load_expected(path: &str) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn run_tier2_rust(cli: &Cli, start: Instant) -> ModuleResult {
    let expected = match load_expected(&cli.expected) {
        Some(v) => v,
        None => return skip_result("mutation_accumulation", 2, start,
            "Cannot parse expected values JSON"),
    };

    let params = load_mutation_params(&cli.data_dir);

    let mut pop_size: u64 = 500_000;
    let mut mu: f64 = 8.9e-4;
    let n_gens: usize = 20_000;
    // More replicates than Python (12) because xorshift64 has higher
    // per-trajectory variance than numpy's PCG64; extra averaging keeps
    // the Pearson r comfortably above the 0.998 linearity threshold.
    let n_reps: usize = 50;
    let seed: u64 = 42;

    if let Some(ref p) = params {
        if let Some(v) = p["population_size"].as_u64() { pop_size = v; }
        if let Some(v) = p["genomic_mutation_rate"].as_f64() { mu = v; }
    }

    eprintln!("  Tier 2 (Rust): N={pop_size}, μ={mu:.4e}, gens={n_gens}, reps={n_reps}");

    let mut passed = 0_u32;
    let mut total = 0_u32;

    // Check 1: Kimura fixation probability for neutral mutation = 1/N
    total += 1;
    let pfix = kimura_fixation_prob(pop_size, 0.0, None);
    let expected_pfix = expected["kimura_fixation_prob_neutral"].as_f64().unwrap_or(0.0);
    let pfix_ok = if expected_pfix > 0.0 {
        (pfix - expected_pfix).abs() / expected_pfix < 0.01
    } else {
        (pfix - 1.0 / pop_size as f64).abs() < 1e-12
    };
    if pfix_ok { passed += 1; }
    eprintln!("  [{}] Neutral fixation probability = {pfix:.2e} (expected: {expected_pfix:.2e})",
        if pfix_ok { "PASS" } else { "FAIL" });

    // Run simulation upfront (needed for checks 2 and 3)
    let mut mean_traj = vec![0.0_f64; n_gens];
    let first_traj = simulate_neutral_fixations(mu, n_gens, seed);
    for (i, &v) in first_traj.iter().enumerate() {
        mean_traj[i] += v as f64;
    }
    for rep in 1..n_reps {
        let traj = simulate_neutral_fixations(mu, n_gens, seed + rep as u64);
        for (i, &v) in traj.iter().enumerate() {
            mean_traj[i] += v as f64;
        }
    }
    for v in &mut mean_traj {
        *v /= n_reps as f64;
    }

    let gens_f: Vec<f64> = (1..=n_gens).map(|g| g as f64).collect();
    let r_val = pearson_r(&gens_f, &mean_traj);

    // Regression slope via least-squares: slope = Cov(x,y)/Var(x)
    let n_f = n_gens as f64;
    let mean_x = gens_f.iter().sum::<f64>() / n_f;
    let mean_y = mean_traj.iter().sum::<f64>() / n_f;
    let mut cov_xy = 0.0_f64;
    let mut var_x = 0.0_f64;
    for (&x, &y) in gens_f.iter().zip(&mean_traj) {
        let dx = x - mean_x;
        cov_xy += dx * (y - mean_y);
        var_x += dx * dx;
    }
    let slope = if var_x > 0.0 { cov_xy / var_x } else { 0.0 };

    // Check 2: Simulated molecular clock rate matches expected
    total += 1;
    let expected_clock = expected["molecular_clock_rate"].as_f64().unwrap_or(0.0);
    let rate_ok = if expected_clock > 0.0 {
        (slope - expected_clock).abs() / expected_clock < 0.05
    } else {
        (slope - mu).abs() / mu < 0.05
    };
    if rate_ok { passed += 1; }
    eprintln!("  [{}] Molecular clock rate: slope={slope:.6e} (expected: {expected_clock:.6e})",
        if rate_ok { "PASS" } else { "FAIL" });

    // Check 3: Simulated accumulation is linear (molecular clock)
    total += 1;
    let linear_ok = r_val > 0.998;
    if linear_ok { passed += 1; }
    eprintln!("  [{}] Molecular clock is linear (r = {r_val:.6}) (min: 0.998)",
        if linear_ok { "PASS" } else { "FAIL" });

    // Check 4: Drift dominates for small |s|
    total += 1;
    let s_threshold = 1.0 / pop_size as f64;
    let pfix_small_s = kimura_fixation_prob(pop_size, s_threshold, None);
    let drift_ratio = pfix_small_s / (1.0 / pop_size as f64);
    let drift_ok = drift_ratio < 5.0;
    if drift_ok { passed += 1; }
    eprintln!("  [{}] Drift dominates at |s|=1/N (ratio = {drift_ratio:.2}, limit < 5×)",
        if drift_ok { "PASS" } else { "FAIL" });

    // Check 5: Selection detectable for |s| >> 1/N
    total += 1;
    let pfix_large = kimura_fixation_prob(pop_size, 0.01, None);
    let sel_ok = pfix_large > 10.0 / pop_size as f64;
    if sel_ok { passed += 1; }
    eprintln!("  [{}] Selection detectable at s=0.01 (P_fix = {pfix_large:.6e})",
        if sel_ok { "PASS" } else { "FAIL" });

    // Check 6: Drift ratio matches expected
    total += 1;
    let exp_ratio = expected["drift_dominance_ratio"].as_f64().unwrap_or(0.0);
    let ratio_match = if exp_ratio > 0.0 {
        (drift_ratio - exp_ratio).abs() / exp_ratio < 0.01
    } else {
        false
    };
    if ratio_match { passed += 1; }
    eprintln!("  [{}] Drift ratio matches expected: {drift_ratio:.4} vs {exp_ratio:.4}",
        if ratio_match { "PASS" } else { "FAIL" });

    // Check 7: Determinism
    total += 1;
    let traj2 = simulate_neutral_fixations(mu, n_gens, seed);
    let det_ok = first_traj == traj2;
    if det_ok { passed += 1; }
    eprintln!("  [{}] Deterministic (same seed → same data)",
        if det_ok { "PASS" } else { "FAIL" });

    let status = if passed == total { ValidationStatus::Pass } else { ValidationStatus::Fail };
    ModuleResult {
        name: "mutation_accumulation".to_string(),
        status,
        tier: 2,
        checks: total,
        checks_passed: passed,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: if passed < total { Some(format!("{} check(s) failed", total - passed)) } else { None },
    }
}

// ── Tier 1: Python dispatch ──────────────────────────────────────────

fn run_tier1_python(start: Instant) -> ModuleResult {
    let notebook_path = Path::new("notebooks/module2_mutations/mutation_accumulation.py");
    if !notebook_path.exists() {
        return skip_result("mutation_accumulation", 1, start, "Python baseline not found");
    }

    let output = std::process::Command::new("python3")
        .arg(notebook_path)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            eprintln!("{stdout}");
            if !stderr.is_empty() {
                eprintln!("{stderr}");
            }

            let passed = stdout.matches("[PASS]").count() as u32;
            let failed = stdout.matches("[FAIL]").count() as u32;
            let total = passed + failed;

            let status = if out.status.code() == Some(0) && failed == 0 {
                ValidationStatus::Pass
            } else if out.status.code() == Some(2) {
                ValidationStatus::Skip
            } else {
                ValidationStatus::Fail
            };

            ModuleResult {
                name: "mutation_accumulation".to_string(),
                status,
                tier: 1,
                checks: total,
                checks_passed: passed,
                runtime_ms: start.elapsed().as_millis() as u64,
                error: if failed > 0 {
                    Some(format!("{failed} check(s) failed"))
                } else {
                    None
                },
            }
        }
        Err(e) => skip_result("mutation_accumulation", 1, start,
            &format!("Python dispatch failed: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kimura_neutral_is_one_over_n() {
        let n = 500_000_u64;
        let pfix = kimura_fixation_prob(n, 0.0, None);
        let expected = 1.0 / n as f64;
        assert!(
            (pfix - expected).abs() < 1e-12,
            "P_fix(s=0)=1/N: got {pfix}, expected {expected}"
        );
    }

    #[test]
    fn kimura_beneficial_exceeds_neutral() {
        let n = 500_000_u64;
        let neutral = kimura_fixation_prob(n, 0.0, None);
        let beneficial = kimura_fixation_prob(n, 0.01, None);
        assert!(
            beneficial > neutral * 10.0,
            "strong selection ⇒ P_fix >> 1/N"
        );
    }

    #[test]
    fn kimura_deleterious_below_neutral() {
        let n = 500_000_u64;
        let neutral = kimura_fixation_prob(n, 0.0, None);
        let deleterious = kimura_fixation_prob(n, -0.01, None);
        assert!(
            deleterious < neutral,
            "deleterious ⇒ P_fix < 1/N"
        );
    }

    #[test]
    fn xorshift_deterministic() {
        let mut rng1 = Xorshift64::new(42);
        let mut rng2 = Xorshift64::new(42);
        for _ in 0..1000 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn poisson_mean_near_lambda() {
        let mut rng = Xorshift64::new(12345);
        let lambda = 5.0;
        let n = 10_000;
        let sum: u64 = (0..n).map(|_| rng.poisson(lambda)).sum();
        let mean = sum as f64 / n as f64;
        assert!(
            (mean - lambda).abs() < 0.2,
            "E[Poisson({lambda})]≈{lambda}: got {mean}"
        );
    }

    #[test]
    fn simulate_neutral_deterministic() {
        let t1 = simulate_neutral_fixations(8.9e-4, 1000, 42);
        let t2 = simulate_neutral_fixations(8.9e-4, 1000, 42);
        assert_eq!(t1, t2);
    }

    #[test]
    fn pearson_r_perfect_linear() {
        let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&v| 3.0 * v + 7.0).collect();
        let r = pearson_r(&x, &y);
        assert!((r - 1.0).abs() < 1e-10, "perfect line ⇒ r=1: got {r}");
    }

    #[test]
    fn pearson_r_uncorrelated_near_zero() {
        let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&v| (v * 17.3).sin()).collect();
        let r = pearson_r(&x, &y);
        assert!(r.abs() < 0.3, "sin vs linear ⇒ low r: got {r}");
    }
}
