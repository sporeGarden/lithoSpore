#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 3: Allele frequency trajectories — Python Tier 1 baseline.

Reproduces Good et al. 2017 (B3): clonal interference dynamics.

Chain: data/good_2017/expected_values.json (simulation counts)
     → compute fixation probabilities, interference ratios
     → compare to validation/expected/module3_alleles.json

The data bundle contains raw simulation tallies (total_fixations,
total_mutations, haldane_probability) at multiple population sizes.
This script computes the derived statistics from those tallies —
the same computation the Tier 2 Rust implementation performs.
"""
from __future__ import annotations

import json
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
DATA_PATH = ROOT / "artifact" / "data" / "good_2017" / "expected_values.json"
EXPECTED_PATH = ROOT / "validation" / "expected" / "module3_alleles.json"


def main() -> int:
    start = time.time()
    print("Module 3: Allele trajectories — Python Tier 1 baseline")

    if not DATA_PATH.exists():
        print(f"  SKIP: Data not found at {DATA_PATH}")
        print("  Run: litho fetch --module ltee-alleles")
        return 2

    with open(DATA_PATH) as f:
        data = json.load(f)

    expected = None
    if EXPECTED_PATH.exists():
        with open(EXPECTED_PATH) as f:
            expected = json.load(f)

    passed = 0
    total = 0

    results = data.get("results_by_size", {})
    if not results:
        print("  FAIL: No results_by_size in data bundle")
        return 1

    print(f"  Computing from {len(results)} population sizes...")
    for size_key in sorted(results.keys(), key=lambda k: int(k)):
        entry = results[size_key]
        total_fix = entry.get("total_fixations", 0)
        total_mut = entry.get("total_mutations", 0)
        haldane = entry.get("haldane_probability", 0.02)

        fix_prob = total_fix / total_mut if total_mut > 0 else 0.0

        total += 1
        if 0.0 < fix_prob < 1.0:
            passed += 1
            print(f"  [PASS] N={size_key}: fix_prob = {fix_prob:.6f} "
                  f"(computed: {total_fix}/{total_mut})")
        else:
            print(f"  [FAIL] N={size_key}: fix_prob = {fix_prob:.6f} out of (0,1)")

        if expected:
            exp_entry = expected.get("results_by_size", {}).get(size_key, {})
            exp_fix = exp_entry.get("fixation_probability")
            if exp_fix is not None:
                total += 1
                delta = abs(fix_prob - exp_fix)
                if delta < 0.01:
                    passed += 1
                    print(f"  [PASS] N={size_key}: computed vs expected delta = {delta:.6f}")
                else:
                    print(f"  [FAIL] N={size_key}: computed {fix_prob:.6f} vs expected {exp_fix:.6f}")

        interference = fix_prob / haldane if haldane > 0 else 0.0
        total += 1
        if interference > 0:
            passed += 1
            print(f"  [PASS] N={size_key}: interference_ratio = {interference:.4f} "
                  f"(fix_prob/haldane = {fix_prob:.6f}/{haldane:.4f})")
        else:
            print(f"  [FAIL] N={size_key}: interference_ratio = {interference:.4f}")

        pop_size = entry.get("pop_size", int(size_key))
        if pop_size >= 10000:
            total += 1
            if interference < 1.0:
                passed += 1
                print(f"  [PASS] N={size_key}: clonal interference suppresses fixation (ratio < 1.0)")
            else:
                print(f"  [FAIL] N={size_key}: ratio >= 1.0 (no suppression)")

        fitness = entry.get("mean_final_fitness")
        if fitness is not None:
            total += 1
            if fitness >= 1.0:
                passed += 1
                print(f"  [PASS] N={size_key}: mean_final_fitness = {fitness:.4f}")
            else:
                print(f"  [FAIL] N={size_key}: mean_final_fitness = {fitness:.4f} < 1.0")

    total += 1
    if len(results) >= 3:
        passed += 1
        print(f"  [PASS] Multiple population sizes tested: {len(results)}")
    else:
        print(f"  [FAIL] Only {len(results)} population sizes (need >= 3)")

    paper = data.get("paper", "")
    total += 1
    if "Good" in paper or paper == "Good2017":
        passed += 1
        print(f"  [PASS] Paper citation: {paper}")
    else:
        print(f"  [FAIL] Paper: {paper} (expected Good*)")

    elapsed_ms = (time.time() - start) * 1000
    status = "PASS" if passed == total else "FAIL"
    print(f"\nallele_trajectories: {status} — {passed}/{total} checks ({elapsed_ms:.0f}ms)")

    figures_dir = ROOT / "figures"
    generate_figures(data, figures_dir)

    return 0 if passed == total else 1


def generate_figures(data, output_dir):
    """Generate allele trajectory metrics vs population size."""
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from litho_figures import can_generate, apply_style, save_figure, ensure_output_dir

    if not can_generate():
        print("  (matplotlib not available — skipping figures)")
        return

    import matplotlib.pyplot as plt
    import numpy as np
    apply_style()
    out = ensure_output_dir(output_dir)

    results = data.get("results_by_size", {})
    sizes = sorted(int(k) for k in results.keys())

    fix_prob = []
    interference = []
    fitness = []
    fitness_std = []
    for s in sizes:
        entry = results[str(s)]
        tf = entry.get("total_fixations", 0)
        tm = entry.get("total_mutations", 1)
        h = entry.get("haldane_probability", 0.02)
        fp = tf / tm if tm > 0 else 0
        fix_prob.append(fp)
        interference.append(fp / h if h > 0 else 0)
        fitness.append(entry.get("mean_final_fitness", 1.0))
        fitness_std.append(entry.get("std_final_fitness", 0.0))

    fig, axes = plt.subplots(2, 2, figsize=(10, 8))

    axes[0, 0].semilogx(sizes, fix_prob, "o-", color="#4e79a7")
    axes[0, 0].set_ylabel("Fixation Probability")
    axes[0, 0].set_title("Fixation Probability (computed)")
    axes[0, 0].grid(True, alpha=0.3)

    axes[0, 1].semilogx(sizes, interference, "s-", color="#e15759")
    axes[0, 1].axhline(1.0, ls="--", color="gray", alpha=0.5, label="Haldane = 1.0")
    axes[0, 1].set_ylabel("Interference Ratio")
    axes[0, 1].set_title("Clonal Interference (computed)")
    axes[0, 1].legend()
    axes[0, 1].grid(True, alpha=0.3)

    axes[1, 0].loglog(sizes, [r.get("adaptation_rate", 0) for r in
                               [results[str(s)] for s in sizes]], "^-", color="#59a14f")
    axes[1, 0].set_xlabel("Population Size (N)")
    axes[1, 0].set_ylabel("Adaptation Rate")
    axes[1, 0].set_title("Adaptation Rate")
    axes[1, 0].grid(True, alpha=0.3)

    axes[1, 1].errorbar(sizes, fitness, yerr=fitness_std, fmt="D-", color="#f28e2b", capsize=4)
    axes[1, 1].set_xscale("log")
    axes[1, 1].set_xlabel("Population Size (N)")
    axes[1, 1].set_ylabel("Mean Final Fitness")
    axes[1, 1].set_title("Final Fitness")
    axes[1, 1].grid(True, alpha=0.3)

    fig.suptitle("Module 3: Clonal Interference — Good et al. 2017\n"
                 "(computed from simulation tallies)", fontsize=12)
    fig.tight_layout()
    save_figure(fig, out, "m3_allele_trajectories")


if __name__ == "__main__":
    sys.exit(main())
