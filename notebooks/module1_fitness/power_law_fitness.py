#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 1: Power-law fitness trajectories — Python Tier 1 baseline.

Reproduces Wiser et al. 2013 (B2): fits power-law, hyperbolic, and
logarithmic models to LTEE fitness data. Compares via AIC/BIC.

Data: artifact/data/wiser_2013/ (Dryad CC0)
Upstream: groundSpring B2 (model selection), wetSpring B2 (Anderson-QS)

Ported from groundSpring/control/ltee_fitness_dynamics/ltee_fitness_dynamics.py
with lithoSpore-specific validation harness wrapping.
"""
from __future__ import annotations

import csv
import json
import sys
import time
from pathlib import Path

import numpy as np
from scipy.optimize import curve_fit

ARTIFACT_ROOT = Path(__file__).resolve().parent.parent.parent / "artifact"
DATA_DIR = ARTIFACT_ROOT / "data" / "wiser_2013"
EXPECTED_PATH = Path(__file__).resolve().parent.parent.parent / "validation" / "expected" / "module1_fitness.json"


def power_law(t, a, b):
    """w(t) = 1 + A·t^b for t > 0."""
    return 1.0 + a * np.power(np.maximum(t, 1e-12), b)


def hyperbolic(t, a, b):
    """w(t) = 1 + a·t/(1 + b·t)."""
    return 1.0 + a * t / (1.0 + b * t)


def logarithmic(t, c, d):
    """w(t) = 1 + c·ln(t) + d for t > 0."""
    return 1.0 + c * np.log(np.maximum(t, 1e-12)) + d


MODELS = {
    "power_law": (power_law, [0.01, 0.5], 2),
    "hyperbolic": (hyperbolic, [1e-3, 1e-4], 2),
    "logarithmic": (logarithmic, [0.1, 0.0], 2),
}


def load_fitness_data(data_dir: Path):
    """Load fitness time-series from CSV (generation, mean_fitness)."""
    csv_path = data_dir / "fitness_data.csv"
    if not csv_path.exists():
        return None, None

    gens = []
    fitness = []
    with open(csv_path) as f:
        reader = csv.DictReader(f)
        for row in reader:
            gens.append(float(row["generation"]))
            fitness.append(float(row["mean_fitness"]))

    return np.array(gens), np.array(fitness)


def fit_model(gens, fitness_mean, model_name):
    """Fit a single model; return (params, rss, r_squared, aic, bic)."""
    func, p0, k = MODELS[model_name]
    mask = gens > 0
    t = gens[mask]
    y = fitness_mean[mask]
    n = len(t)

    try:
        popt, _ = curve_fit(func, t, y, p0=p0, maxfev=10000)
    except RuntimeError:
        return None

    predicted = func(t, *popt)
    ss_res = float(np.sum((y - predicted) ** 2))
    ss_tot = float(np.sum((y - np.mean(y)) ** 2))
    r_squared = 1.0 - ss_res / ss_tot if ss_tot > 0 else 1.0

    if ss_res <= 0:
        ss_res = 1e-30

    aic_val = n * np.log(ss_res / n) + 2 * k
    bic_val = n * np.log(ss_res / n) + k * np.log(n)

    return {
        "model": model_name,
        "params": [float(p) for p in popt],
        "k": k,
        "rss": float(ss_res),
        "r_squared": r_squared,
        "aic": float(aic_val),
        "bic": float(bic_val),
    }


def load_expected():
    """Load expected values from groundSpring B2 reproduction."""
    if not EXPECTED_PATH.exists():
        return None
    with open(EXPECTED_PATH) as f:
        return json.load(f)


def main():
    t0 = time.monotonic()
    print("=" * 72)
    print("  Module 1: Power-law Fitness — Python Tier 1 Baseline")
    print("  Wiser et al. 2013 (B2) | groundSpring reproduction")
    print("=" * 72)
    print(f"  Data dir: {DATA_DIR}")
    print(f"  Expected: {EXPECTED_PATH}")

    if not DATA_DIR.exists():
        print("\n  SKIP: Data not yet fetched. Run: scripts/fetch_wiser_2013.sh")
        return 2

    expected = load_expected()
    if expected is None:
        print("\n  SKIP: Expected values not found at validation/expected/module1_fitness.json")
        return 2

    gens, mean_fitness = load_fitness_data(DATA_DIR)
    if gens is None:
        print("\n  SKIP: No fitness_data.csv in data directory")
        return 2

    print(f"\n  Data points: {len(gens)}")
    print(f"  Generation range: {gens[0]:.0f} — {gens[-1]:.0f}")
    print(f"  Final mean fitness: {mean_fitness[-1]:.4f}")

    checks_passed = 0
    checks_total = 0

    # Check 1: fitness generally increasing
    checks_total += 1
    diffs = np.diff(mean_fitness)
    increasing = np.sum(diffs >= 0) >= len(diffs) * 0.8
    status = "PASS" if increasing else "FAIL"
    print(f"\n  [{status}] Fitness trajectory is increasing")
    if increasing:
        checks_passed += 1

    # Check 2-4: fit all three models and compare
    print("\n  Model Comparison:")
    print(f"  {'Model':<15} {'R²':>8} {'AIC':>10} {'BIC':>10}")
    print(f"  {'-' * 50}")

    results = {}
    for name in ["power_law", "hyperbolic", "logarithmic"]:
        result = fit_model(gens, mean_fitness, name)
        if result is not None:
            results[name] = result
            print(
                f"  {name:<15} {result['r_squared']:>8.5f} "
                f"{result['aic']:>10.3f} {result['bic']:>10.3f}"
            )

    # AIC selection: power_law should win
    checks_total += 1
    if results:
        best_aic = min(results.values(), key=lambda r: r["aic"])
        aic_pass = best_aic["model"] == "power_law"
        status = "PASS" if aic_pass else "FAIL"
        print(f"\n  [{status}] Best model by AIC: {best_aic['model']} (expected: power_law)")
        if aic_pass:
            checks_passed += 1

    # BIC selection
    checks_total += 1
    if results:
        best_bic = min(results.values(), key=lambda r: r["bic"])
        bic_pass = best_bic["model"] == "power_law"
        status = "PASS" if bic_pass else "FAIL"
        print(f"  [{status}] Best model by BIC: {best_bic['model']} (expected: power_law)")
        if bic_pass:
            checks_passed += 1

    # Power-law R² >= 0.99
    checks_total += 1
    if "power_law" in results:
        pl_r2 = results["power_law"]["r_squared"]
        r2_pass = pl_r2 >= 0.99
        status = "PASS" if r2_pass else "FAIL"
        print(f"  [{status}] Power-law R² = {pl_r2:.5f} (min: 0.99)")
        if r2_pass:
            checks_passed += 1

    # Power-law exponent in expected range [0.40, 0.70]
    checks_total += 1
    if "power_law" in results:
        b_exp = results["power_law"]["params"][1]
        in_range = 0.40 <= b_exp <= 0.70
        status = "PASS" if in_range else "FAIL"
        print(f"  [{status}] Power-law exponent b = {b_exp:.4f} (expected: [0.40, 0.70])")
        if in_range:
            checks_passed += 1

    # AIC(power_law) < AIC(hyperbolic)
    checks_total += 1
    if "power_law" in results and "hyperbolic" in results:
        aic_lt = results["power_law"]["aic"] < results["hyperbolic"]["aic"]
        status = "PASS" if aic_lt else "FAIL"
        print(f"  [{status}] AIC(power_law) < AIC(hyperbolic)")
        if aic_lt:
            checks_passed += 1

    # Cross-validate against groundSpring expected values
    checks_total += 1
    exp_r2 = expected["model_fits"]["power_law"]["r_squared"]
    if "power_law" in results:
        r2_match = abs(results["power_law"]["r_squared"] - exp_r2) < 0.01
        status = "PASS" if r2_match else "FAIL"
        print(f"\n  [{status}] R² matches groundSpring expected: "
              f"{results['power_law']['r_squared']:.5f} vs {exp_r2:.5f}")
        if r2_match:
            checks_passed += 1

    # Cross-validate exponent
    checks_total += 1
    exp_b = expected["model_fits"]["power_law"]["params"][1]
    if "power_law" in results:
        b_match = abs(results["power_law"]["params"][1] - exp_b) < 0.05
        status = "PASS" if b_match else "FAIL"
        print(f"  [{status}] Exponent matches groundSpring expected: "
              f"{results['power_law']['params'][1]:.4f} vs {exp_b:.4f}")
        if b_match:
            checks_passed += 1

    elapsed_ms = int((time.monotonic() - t0) * 1000)

    print(f"\n{'=' * 72}")
    overall = "PASS" if checks_passed == checks_total else "FAIL"
    print(f"  RESULT: {overall} — {checks_passed}/{checks_total} checks ({elapsed_ms}ms)")
    print(f"{'=' * 72}")

    result_json = {
        "module": "power_law_fitness",
        "status": overall,
        "tier": 1,
        "checks": checks_total,
        "checks_passed": checks_passed,
        "runtime_ms": elapsed_ms,
        "model_fits": results,
    }
    print(json.dumps(result_json, indent=2))

    figures_dir = Path(__file__).resolve().parent.parent.parent / "figures"
    generate_figures(gens, mean_fitness, results, figures_dir)

    return 0 if checks_passed == checks_total else 1


def generate_figures(gens, mean_fitness, model_results, output_dir):
    """Generate publication-quality fitness trajectory figure."""
    import sys
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from litho_figures import can_generate, apply_style, save_figure, ensure_output_dir

    if not can_generate():
        print("  (matplotlib not available — skipping figures)")
        return

    import matplotlib.pyplot as plt
    apply_style()
    out = ensure_output_dir(output_dir)

    fig, ax = plt.subplots(figsize=(9, 5.5))
    ax.scatter(gens, mean_fitness, s=50, zorder=5, label="Observed (12-pop mean)")

    t_fine = np.linspace(1, max(gens), 500)
    if "power_law" in model_results:
        p = model_results["power_law"]["params"]
        ax.plot(t_fine, power_law(t_fine, *p), "-",
                label=f"Power-law (R²={model_results['power_law']['r_squared']:.4f})")
    if "hyperbolic" in model_results:
        p = model_results["hyperbolic"]["params"]
        ax.plot(t_fine, hyperbolic(t_fine, *p), "--",
                label=f"Hyperbolic (R²={model_results['hyperbolic']['r_squared']:.4f})")
    if "logarithmic" in model_results:
        p = model_results["logarithmic"]["params"]
        ax.plot(t_fine, logarithmic(t_fine, *p), ":",
                label=f"Logarithmic (R²={model_results['logarithmic']['r_squared']:.4f})")

    ax.set_xlabel("Generation")
    ax.set_ylabel("Mean Relative Fitness")
    ax.set_title("Module 1: LTEE Fitness Dynamics — Wiser et al. 2013")
    ax.legend(loc="upper left", framealpha=0.9)
    ax.grid(True, alpha=0.3)
    save_figure(fig, out, "m1_fitness_trajectory")


if __name__ == "__main__":
    sys.exit(main())
