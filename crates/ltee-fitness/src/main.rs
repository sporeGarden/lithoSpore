// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 1: Power-law fitness trajectories
//!
//! Reproduces Wiser et al. 2013 (B2) — long-term fitness dynamics.
//! Springs: groundSpring (jackknife + AIC/BIC), wetSpring (diversity metrics).
//!
//! Tier 1: dispatches to Python baseline.
//! Tier 2: pure Rust curve fitting (Nelder-Mead + AIC/BIC model selection).

use clap::Parser;
use litho_core::{ModuleResult, ValidationStatus};
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ltee-fitness", about = "Power-law fitness trajectory validation")]
struct Cli {
    #[arg(long, default_value = "data/wiser_2013")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module1_fitness.json")]
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
            "Module 1 (fitness): {} — {}/{} checks ({}ms)",
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

fn run_validation(cli: &Cli) -> ModuleResult {
    let start = Instant::now();

    if !Path::new(&cli.expected).exists() {
        return skip_result("power_law_fitness", 1, start,
            "Expected values not found — run groundSpring B2 first");
    }

    if !Path::new(&cli.data_dir).exists() {
        return skip_result("power_law_fitness", 1, start,
            "Data not fetched — run scripts/fetch_wiser_2013.sh");
    }

    if cli.max_tier >= 2 {
        return run_tier2_rust(cli, start);
    }
    if cli.max_tier >= 1 {
        return run_tier1_python(start);
    }

    skip_result("power_law_fitness", cli.max_tier, start,
        &format!("Tier {} not implemented yet", cli.max_tier))
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

// ── Tier 2: Pure Rust curve fitting ──────────────────────────────────

#[derive(Debug, serde::Serialize)]
struct ModelFit {
    model: String,
    params: Vec<f64>,
    k: usize,
    rss: f64,
    r_squared: f64,
    aic: f64,
    bic: f64,
}

fn power_law(t: f64, a: f64, b: f64) -> f64 {
    1.0 + a * t.max(1e-12).powf(b)
}

fn hyperbolic(t: f64, a: f64, b: f64) -> f64 {
    1.0 + a * t / (1.0 + b * t)
}

fn logarithmic(t: f64, c: f64, d: f64) -> f64 {
    1.0 + c * t.max(1e-12).ln() + d
}

fn rss_for_model(
    gens: &[f64], fitness: &[f64],
    model: fn(f64, f64, f64) -> f64, p: &[f64; 2],
) -> f64 {
    gens.iter().zip(fitness)
        .filter(|(&t, _)| t > 0.0)
        .map(|(&t, &y)| { let d = y - model(t, p[0], p[1]); d * d })
        .sum()
}

/// Nelder-Mead simplex for 2 parameters — finds minimum RSS.
fn nelder_mead_2d(
    gens: &[f64], fitness: &[f64],
    model: fn(f64, f64, f64) -> f64,
    p0: [f64; 2],
) -> Option<[f64; 2]> {
    let obj = |p: &[f64; 2]| -> f64 { rss_for_model(gens, fitness, model, p) };

    let mut simplex = [
        (p0, obj(&p0)),
        ([p0[0] * 1.5 + 1e-6, p0[1]], 0.0),
        ([p0[0], p0[1] * 1.5 + 1e-6], 0.0),
    ];
    simplex[1].1 = obj(&simplex[1].0);
    simplex[2].1 = obj(&simplex[2].0);

    for _ in 0..5000 {
        simplex.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let (best, _, worst) = (simplex[0].1, simplex[1].1, simplex[2].1);

        if (worst - best).abs() < 1e-14 {
            break;
        }

        let cx = [(simplex[0].0[0] + simplex[1].0[0]) / 2.0,
                   (simplex[0].0[1] + simplex[1].0[1]) / 2.0];

        // Reflect
        let xr = [2.0 * cx[0] - simplex[2].0[0], 2.0 * cx[1] - simplex[2].0[1]];
        let fr = obj(&xr);

        if fr < simplex[1].1 && fr >= best {
            simplex[2] = (xr, fr);
            continue;
        }

        if fr < best {
            let xe = [3.0 * cx[0] - 2.0 * simplex[2].0[0],
                       3.0 * cx[1] - 2.0 * simplex[2].0[1]];
            let fe = obj(&xe);
            simplex[2] = if fe < fr { (xe, fe) } else { (xr, fr) };
            continue;
        }

        // Contract
        let xc = [(cx[0] + simplex[2].0[0]) / 2.0,
                   (cx[1] + simplex[2].0[1]) / 2.0];
        let fc = obj(&xc);
        if fc < worst {
            simplex[2] = (xc, fc);
            continue;
        }

        // Shrink toward best
        let b0 = simplex[0].0;
        for v in &mut simplex[1..] {
            v.0[0] = b0[0] + 0.5 * (v.0[0] - b0[0]);
            v.0[1] = b0[1] + 0.5 * (v.0[1] - b0[1]);
            v.1 = obj(&v.0);
        }
    }

    simplex.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    if simplex[0].1.is_finite() { Some(simplex[0].0) } else { None }
}

fn fit_model(
    gens: &[f64], fitness: &[f64],
    name: &str,
    model: fn(f64, f64, f64) -> f64,
    p0: [f64; 2],
) -> Option<ModelFit> {
    let popt = nelder_mead_2d(gens, fitness, model, p0)?;

    let (mut ss_res, mut ss_tot, mut n) = (0.0_f64, 0.0_f64, 0_usize);
    let mean_y: f64 = {
        let (s, c) = fitness.iter().zip(gens).filter(|(_, &t)| t > 0.0)
            .fold((0.0, 0), |(s, c), (&y, _)| (s + y, c + 1));
        s / c as f64
    };
    for (&t, &y) in gens.iter().zip(fitness) {
        if t <= 0.0 { continue; }
        let pred = model(t, popt[0], popt[1]);
        ss_res += (y - pred).powi(2);
        ss_tot += (y - mean_y).powi(2);
        n += 1;
    }

    let r_squared = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 1.0 };
    let ss_res_safe = ss_res.max(1e-30);
    let k = 2;
    let nf = n as f64;
    let aic = nf * (ss_res_safe / nf).ln() + 2.0 * k as f64;
    let bic = nf * (ss_res_safe / nf).ln() + k as f64 * nf.ln();

    Some(ModelFit {
        model: name.to_string(),
        params: popt.to_vec(),
        k,
        rss: ss_res,
        r_squared,
        aic,
        bic,
    })
}

fn load_csv(data_dir: &str) -> Option<(Vec<f64>, Vec<f64>)> {
    let path = Path::new(data_dir).join("fitness_data.csv");
    let content = std::fs::read_to_string(path).ok()?;
    let mut gens = Vec::new();
    let mut fitness = Vec::new();
    for line in content.lines().skip(1) {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() >= 2 {
            if let (Ok(g), Ok(f)) = (cols[0].trim().parse::<f64>(), cols[1].trim().parse::<f64>()) {
                gens.push(g);
                fitness.push(f);
            }
        }
    }
    if gens.is_empty() { None } else { Some((gens, fitness)) }
}

fn load_expected(path: &str) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn run_tier2_rust(cli: &Cli, start: Instant) -> ModuleResult {
    let expected = match load_expected(&cli.expected) {
        Some(v) => v,
        None => return skip_result("power_law_fitness", 2, start,
            "Cannot parse expected values JSON"),
    };

    let (gens, fitness) = match load_csv(&cli.data_dir) {
        Some(d) => d,
        None => return skip_result("power_law_fitness", 2, start,
            "No fitness_data.csv in data directory"),
    };

    eprintln!("  Tier 2 (Rust): {} data points, gen range {:.0}–{:.0}",
        gens.len(), gens.first().unwrap_or(&0.0), gens.last().unwrap_or(&0.0));

    let mut passed = 0_u32;
    let mut total = 0_u32;

    // Check 1: fitness generally increasing
    total += 1;
    let increasing_count = fitness.windows(2).filter(|w| w[1] >= w[0]).count();
    let increasing = increasing_count >= (fitness.len() - 1) * 4 / 5;
    if increasing { passed += 1; }
    eprintln!("  [{}] Fitness trajectory is increasing",
        if increasing { "PASS" } else { "FAIL" });

    // Fit all three models
    let pl = fit_model(&gens, &fitness, "power_law", power_law, [0.01, 0.5]);
    let hyp = fit_model(&gens, &fitness, "hyperbolic", hyperbolic, [1e-3, 1e-4]);
    let log = fit_model(&gens, &fitness, "logarithmic", logarithmic, [0.1, 0.0]);

    let mut results: Vec<&ModelFit> = Vec::new();
    if let Some(ref r) = pl { results.push(r); }
    if let Some(ref r) = hyp { results.push(r); }
    if let Some(ref r) = log { results.push(r); }

    for r in &results {
        eprintln!("  {:<15} R²={:.5} AIC={:.3} BIC={:.3}",
            r.model, r.r_squared, r.aic, r.bic);
    }

    // Check 2: AIC selects power_law
    total += 1;
    if let Some(best) = results.iter().min_by(|a, b| a.aic.partial_cmp(&b.aic).unwrap()) {
        let ok = best.model == "power_law";
        if ok { passed += 1; }
        eprintln!("  [{}] Best model by AIC: {} (expected: power_law)",
            if ok { "PASS" } else { "FAIL" }, best.model);
    }

    // Check 3: BIC selects power_law
    total += 1;
    if let Some(best) = results.iter().min_by(|a, b| a.bic.partial_cmp(&b.bic).unwrap()) {
        let ok = best.model == "power_law";
        if ok { passed += 1; }
        eprintln!("  [{}] Best model by BIC: {} (expected: power_law)",
            if ok { "PASS" } else { "FAIL" }, best.model);
    }

    if let Some(ref pl_fit) = pl {
        // Check 4: R² >= 0.99
        total += 1;
        let ok = pl_fit.r_squared >= 0.99;
        if ok { passed += 1; }
        eprintln!("  [{}] Power-law R² = {:.5} (min: 0.99)",
            if ok { "PASS" } else { "FAIL" }, pl_fit.r_squared);

        // Check 5: exponent in [0.40, 0.70]
        total += 1;
        let b_exp = pl_fit.params[1];
        let ok = (0.40..=0.70).contains(&b_exp);
        if ok { passed += 1; }
        eprintln!("  [{}] Power-law exponent b = {:.4} (expected: [0.40, 0.70])",
            if ok { "PASS" } else { "FAIL" }, b_exp);

        // Check 6: AIC(power_law) < AIC(hyperbolic)
        if let Some(ref h) = hyp {
            total += 1;
            let ok = pl_fit.aic < h.aic;
            if ok { passed += 1; }
            eprintln!("  [{}] AIC(power_law) < AIC(hyperbolic)",
                if ok { "PASS" } else { "FAIL" });
        }

        // Check 7: R² matches expected within 0.01
        total += 1;
        let exp_r2 = expected["model_fits"]["power_law"]["r_squared"]
            .as_f64().unwrap_or(0.0);
        let ok = (pl_fit.r_squared - exp_r2).abs() < 0.01;
        if ok { passed += 1; }
        eprintln!("  [{}] R² matches expected: {:.5} vs {:.5}",
            if ok { "PASS" } else { "FAIL" }, pl_fit.r_squared, exp_r2);

        // Check 8: exponent matches expected within 0.05
        total += 1;
        let exp_b = expected["model_fits"]["power_law"]["params"][1]
            .as_f64().unwrap_or(0.0);
        let ok = (b_exp - exp_b).abs() < 0.05;
        if ok { passed += 1; }
        eprintln!("  [{}] Exponent matches expected: {:.4} vs {:.4}",
            if ok { "PASS" } else { "FAIL" }, b_exp, exp_b);
    }

    let status = if passed == total { ValidationStatus::Pass } else { ValidationStatus::Fail };
    ModuleResult {
        name: "power_law_fitness".to_string(),
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
    let notebook_path = Path::new("notebooks/module1_fitness/power_law_fitness.py");
    if !notebook_path.exists() {
        return skip_result("power_law_fitness", 1, start, "Python baseline not found");
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
                name: "power_law_fitness".to_string(),
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
        Err(e) => skip_result("power_law_fitness", 1, start,
            &format!("Python dispatch failed: {e}")),
    }
}
