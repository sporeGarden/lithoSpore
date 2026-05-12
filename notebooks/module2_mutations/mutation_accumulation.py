#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 2: Mutation accumulation curves — Python Tier 1 baseline.

Reproduces Barrick et al. 2009 (B1): neutral mutation dynamics.

Under strict neutrality, mutation fixation probability = 1/N (haploid).
The molecular clock rate equals the genomic mutation rate μ, independent
of population size. Drift dominates selection when |s| < 1/N.

Data: artifact/data/barrick_2009/ (NCBI public-domain)
Upstream: groundSpring B1 (drift vs selection null model)

Ported from groundSpring/control/ltee_neutral_mutation/ltee_neutral_mutation.py
with lithoSpore-specific validation harness wrapping.
"""
from __future__ import annotations

import json
import sys
import time
from pathlib import Path

import numpy as np

ARTIFACT_ROOT = Path(__file__).resolve().parent.parent.parent / "artifact"
DATA_DIR = ARTIFACT_ROOT / "data" / "barrick_2009"
EXPECTED_PATH = Path(__file__).resolve().parent.parent.parent / "validation" / "expected" / "module2_mutations.json"


def kimura_fixation_prob(pop_size: int, selection: float, initial_freq: float = None) -> float:
    """Kimura fixation probability for a new mutation in a haploid population."""
    if initial_freq is None:
        initial_freq = 1.0 / pop_size

    if abs(selection) < 1e-10:
        return initial_freq

    numerator = 1.0 - np.exp(-2.0 * selection * pop_size * initial_freq)
    denominator = 1.0 - np.exp(-2.0 * selection * pop_size)
    return float(numerator / denominator)


def neutral_accumulation_rate(genomic_mu: float) -> float:
    """Under neutrality, substitution rate = genomic mutation rate μ."""
    return genomic_mu


def simulate_neutral_fixations(pop_size, mu, n_gens, seed):
    """Simulate neutral mutation accumulation via Poisson process."""
    rng = np.random.default_rng(seed)
    fixations_per_gen = rng.poisson(mu, size=n_gens)
    cumulative = np.cumsum(fixations_per_gen)
    return cumulative


def load_expected():
    """Load expected values from groundSpring B1 reproduction."""
    if not EXPECTED_PATH.exists():
        return None
    with open(EXPECTED_PATH) as f:
        return json.load(f)


def load_mutation_params(data_dir: Path):
    """Load mutation parameters from the data directory."""
    params_path = data_dir / "mutation_parameters.json"
    if not params_path.exists():
        return None
    with open(params_path) as f:
        return json.load(f)


def main():
    t0 = time.monotonic()
    print("=" * 72)
    print("  Module 2: Mutation Accumulation — Python Tier 1 Baseline")
    print("  Barrick et al. 2009 (B1) | groundSpring reproduction")
    print("=" * 72)
    print(f"  Data dir: {DATA_DIR}")
    print(f"  Expected: {EXPECTED_PATH}")

    if not DATA_DIR.exists():
        print("\n  SKIP: Data not yet fetched. Run: scripts/fetch_barrick_2009.sh")
        return 2

    expected = load_expected()
    if expected is None:
        print("\n  SKIP: Expected values not found at validation/expected/module2_mutations.json")
        return 2

    params = load_mutation_params(DATA_DIR)

    pop_size = 500_000
    mu = 8.9e-4
    n_gens = 20_000
    n_reps = 12
    seed = 42

    if params:
        pop_size = params.get("population_size", pop_size)
        mu = params.get("genomic_mutation_rate", mu)

    print(f"\n  Population size: {pop_size:,}")
    print(f"  Genomic mutation rate: {mu:.4e}")
    print(f"  Generations: {n_gens:,}")

    checks_passed = 0
    checks_total = 0

    # Check 1: Kimura fixation probability for neutral mutation = 1/N
    checks_total += 1
    pfix = kimura_fixation_prob(pop_size, 0.0)
    expected_pfix = expected["kimura_fixation_prob_neutral"]
    pfix_pass = abs(pfix - expected_pfix) / expected_pfix < 0.01
    print(f"\n  Kimura P_fix(s=0, N={pop_size}): {pfix:.2e} (expected: {expected_pfix:.2e})")
    status = "PASS" if pfix_pass else "FAIL"
    print(f"  [{status}] Neutral fixation probability matches 1/N")
    if pfix_pass:
        checks_passed += 1

    # Check 2: Molecular clock rate = μ
    checks_total += 1
    rate = neutral_accumulation_rate(mu)
    rate_pass = abs(rate - mu) < 1e-10
    print(f"\n  Neutral substitution rate: {rate:.4e} (expected: {mu:.4e})")
    status = "PASS" if rate_pass else "FAIL"
    print(f"  [{status}] Molecular clock rate = μ")
    if rate_pass:
        checks_passed += 1

    # Check 3: Simulated accumulation is linear (molecular clock)
    checks_total += 1
    all_trajectories = []
    for i in range(n_reps):
        traj = simulate_neutral_fixations(pop_size, mu, n_gens, seed + i)
        all_trajectories.append(traj)

    mean_traj = np.mean(all_trajectories, axis=0)
    gens = np.arange(1, n_gens + 1, dtype=float)

    from numpy.polynomial.polynomial import polyfit
    coeffs = polyfit(gens, mean_traj, 1)
    slope = coeffs[1]
    from scipy.stats import pearsonr
    r_val, _ = pearsonr(gens, mean_traj)
    linear_pass = r_val > 0.998
    print(f"\n  Mean trajectory over {n_reps} replicates:")
    print(f"  Linear fit slope: {slope:.6f} (expected ~μ = {mu:.4e})")
    print(f"  Pearson r: {r_val:.6f}")
    status = "PASS" if linear_pass else "FAIL"
    print(f"  [{status}] Molecular clock is linear (r > 0.998)")
    if linear_pass:
        checks_passed += 1

    # Check 4: Drift dominates for small |s|
    checks_total += 1
    s_threshold = 1.0 / pop_size
    pfix_small_s = kimura_fixation_prob(pop_size, s_threshold)
    drift_ratio = pfix_small_s / (1.0 / pop_size)
    drift_pass = drift_ratio < 5.0
    print(f"\n  P_fix(s={s_threshold:.2e}) = {pfix_small_s:.6e}")
    print(f"  Ratio to neutral: {drift_ratio:.2f}")
    status = "PASS" if drift_pass else "FAIL"
    print(f"  [{status}] Drift dominates at |s| = 1/N (ratio < 5×)")
    if drift_pass:
        checks_passed += 1

    # Check 5: Selection detectable for |s| >> 1/N
    checks_total += 1
    s_large = 0.01
    pfix_large = kimura_fixation_prob(pop_size, s_large)
    sel_detectable = pfix_large > 10.0 / pop_size
    print(f"\n  P_fix(s={s_large}) = {pfix_large:.6e}")
    status = "PASS" if sel_detectable else "FAIL"
    print(f"  [{status}] Selection detectable at s = {s_large}")
    if sel_detectable:
        checks_passed += 1

    # Check 6: Cross-validate drift ratio against groundSpring expected
    checks_total += 1
    exp_ratio = expected["drift_dominance_ratio"]
    ratio_match = abs(drift_ratio - exp_ratio) / exp_ratio < 0.01
    status = "PASS" if ratio_match else "FAIL"
    print(f"\n  [{status}] Drift ratio matches groundSpring expected: "
          f"{drift_ratio:.4f} vs {exp_ratio:.4f}")
    if ratio_match:
        checks_passed += 1

    # Check 7: Determinism
    checks_total += 1
    traj2 = simulate_neutral_fixations(pop_size, mu, n_gens, seed)
    det_pass = np.array_equal(all_trajectories[0], traj2)
    status = "PASS" if det_pass else "FAIL"
    print(f"\n  [{status}] Deterministic (same seed → same data)")
    if det_pass:
        checks_passed += 1

    elapsed_ms = int((time.monotonic() - t0) * 1000)

    print(f"\n{'=' * 72}")
    overall = "PASS" if checks_passed == checks_total else "FAIL"
    print(f"  RESULT: {overall} — {checks_passed}/{checks_total} checks ({elapsed_ms}ms)")
    print(f"{'=' * 72}")

    result_json = {
        "module": "mutation_accumulation",
        "status": overall,
        "tier": 1,
        "checks": checks_total,
        "checks_passed": checks_passed,
        "runtime_ms": elapsed_ms,
    }
    print(json.dumps(result_json, indent=2))

    return 0 if checks_passed == checks_total else 1


if __name__ == "__main__":
    sys.exit(main())
