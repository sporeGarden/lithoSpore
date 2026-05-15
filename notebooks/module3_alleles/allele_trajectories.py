#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 3: Allele frequency trajectories — Python Tier 1 baseline.

Reproduces Good et al. 2017 (B3): molecular evolution dynamics.
Validates clonal interference statistics from expected values JSON.
Data: data/good_2017/ (NCBI public-domain)
Upstream: neuralSpring B3 (LSTM+HMM+ESN), groundSpring B3 (clonal interference)
"""
from __future__ import annotations

import json
import sys
import time
from pathlib import Path

EXPECTED_PATH = (
    Path(__file__).resolve().parent.parent.parent
    / "validation"
    / "expected"
    / "module3_alleles.json"
)


def main() -> int:
    start = time.time()
    print("Module 3: Allele trajectories — Python Tier 1 baseline")

    if not EXPECTED_PATH.exists():
        print("  SKIP: Expected values not found — run groundSpring B3 first")
        return 2

    with open(EXPECTED_PATH) as f:
        expected = json.load(f)

    passed = 0
    total = 0

    results = expected.get("results_by_size", {})
    for size_key, data in results.items():
        fix_prob = data.get("fixation_probability")
        if fix_prob is not None:
            total += 1
            if 0.0 < fix_prob < 1.0:
                passed += 1
                print(f"  [PASS] N={size_key}: fixation_probability={fix_prob:.6f} in (0,1)")
            else:
                print(f"  [FAIL] N={size_key}: fixation_probability={fix_prob:.6f} out of (0,1)")

        interference = data.get("interference_ratio")
        if interference is not None:
            total += 1
            if interference > 0.0:
                passed += 1
                print(f"  [PASS] N={size_key}: interference_ratio={interference:.4f} > 0")
            else:
                print(f"  [FAIL] N={size_key}: interference_ratio={interference:.4f} <= 0")

        fitness = data.get("mean_final_fitness")
        if fitness is not None:
            total += 1
            if fitness >= 1.0:
                passed += 1
                print(f"  [PASS] N={size_key}: mean_final_fitness={fitness:.4f} >= 1.0")
            else:
                print(f"  [FAIL] N={size_key}: mean_final_fitness={fitness:.4f} < 1.0")

        rate = data.get("adaptation_rate")
        if rate is not None:
            total += 1
            if rate >= 0.0:
                passed += 1
                print(f"  [PASS] N={size_key}: adaptation_rate={rate:.6e} >= 0")
            else:
                print(f"  [FAIL] N={size_key}: adaptation_rate={rate:.6e} < 0")

    paper = expected.get("paper")
    if paper is not None:
        total += 1
        if paper == "Good2017":
            passed += 1
            print(f"  [PASS] paper={paper}")
        else:
            print(f"  [FAIL] paper={paper} (expected Good2017)")

    elapsed_ms = (time.time() - start) * 1000
    print(f"\nallele_trajectories: {passed}/{total} checks passed ({elapsed_ms:.0f}ms)")

    figures_dir = (
        Path(__file__).resolve().parent.parent.parent / "figures"
    )
    generate_figures(expected, figures_dir)

    return 0 if passed == total else 1


def generate_figures(expected, output_dir):
    """Generate allele trajectory metrics vs population size."""
    import sys as _sys
    _sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from litho_figures import can_generate, apply_style, save_figure, ensure_output_dir

    if not can_generate():
        print("  (matplotlib not available — skipping figures)")
        return

    import matplotlib.pyplot as plt
    import numpy as np
    apply_style()
    out = ensure_output_dir(output_dir)

    results = expected.get("results_by_size", {})
    sizes = sorted(int(k) for k in results.keys())
    fix_prob = [results[str(s)]["fixation_probability"] for s in sizes]
    interference = [results[str(s)]["interference_ratio"] for s in sizes]
    adaptation = [results[str(s)]["adaptation_rate"] for s in sizes]
    fitness = [results[str(s)]["mean_final_fitness"] for s in sizes]
    fitness_std = [results[str(s)]["std_final_fitness"] for s in sizes]

    fig, axes = plt.subplots(2, 2, figsize=(10, 8))

    axes[0, 0].semilogx(sizes, fix_prob, "o-", color="#4e79a7")
    axes[0, 0].set_ylabel("Fixation Probability")
    axes[0, 0].set_title("Fixation Probability")
    axes[0, 0].grid(True, alpha=0.3)

    axes[0, 1].semilogx(sizes, interference, "s-", color="#e15759")
    axes[0, 1].axhline(1.0, ls="--", color="gray", alpha=0.5, label="Haldane = 1.0")
    axes[0, 1].set_ylabel("Interference Ratio")
    axes[0, 1].set_title("Clonal Interference Ratio")
    axes[0, 1].legend()
    axes[0, 1].grid(True, alpha=0.3)

    axes[1, 0].loglog(sizes, adaptation, "^-", color="#59a14f")
    axes[1, 0].set_xlabel("Population Size (N)")
    axes[1, 0].set_ylabel("Adaptation Rate")
    axes[1, 0].set_title("Adaptation Rate")
    axes[1, 0].grid(True, alpha=0.3)

    axes[1, 1].errorbar(sizes, fitness, yerr=fitness_std, fmt="D-", color="#f28e2b",
                         capsize=4)
    axes[1, 1].set_xscale("log")
    axes[1, 1].set_xlabel("Population Size (N)")
    axes[1, 1].set_ylabel("Mean Final Fitness")
    axes[1, 1].set_title("Final Fitness")
    axes[1, 1].grid(True, alpha=0.3)

    fig.suptitle("Module 3: Clonal Interference — Good et al. 2017", fontsize=13)
    fig.tight_layout()
    save_figure(fig, out, "m3_allele_trajectories")


if __name__ == "__main__":
    sys.exit(main())
