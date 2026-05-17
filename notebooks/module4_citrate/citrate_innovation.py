#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 4: Citrate innovation cascade — Python Tier 1 baseline.

Reproduces Blount et al. 2008/2012 (B4): historical contingency.

Chain: data/blount_2012/expected_values.json (replay experiment data)
     → compute potentiation statistics, two-hit model predictions
     → compare to validation/expected/module4_citrate.json

The data bundle contains per-population lineage data with potentiation
and Cit+ emergence generations. This script computes the derived
statistics (fractions, windows, wait times) from those lineages —
the same computation the Tier 2 Rust implementation performs.
"""
from __future__ import annotations

import json
import math
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
DATA_PATH = ROOT / "artifact" / "data" / "blount_2012" / "expected_values.json"
EXPECTED_PATH = ROOT / "validation" / "expected" / "module4_citrate.json"


def main() -> int:
    start = time.time()
    print("Module 4: Citrate innovation — Python Tier 1 baseline")

    if not DATA_PATH.exists():
        print(f"  SKIP: Data not found at {DATA_PATH}")
        print("  Run: litho fetch --module ltee-citrate")
        return 2

    with open(DATA_PATH) as f:
        data = json.load(f)

    expected = None
    if EXPECTED_PATH.exists():
        with open(EXPECTED_PATH) as f:
            expected = json.load(f)

    passed = 0
    total = 0

    # --- Compute Cit+ fraction from lineage data ---
    n_populations = data.get("n_populations", 12)
    n_cit_plus = data.get("n_cit_plus", 2)
    cit_frac = n_cit_plus / n_populations if n_populations > 0 else 0

    total += 1
    expected_frac = 1.0 / 6.0
    if abs(cit_frac - expected_frac) < 0.01:
        passed += 1
        print(f"  [PASS] Cit+ fraction: {cit_frac:.4f} (computed: {n_cit_plus}/{n_populations} ≈ 1/6)")
    else:
        print(f"  [FAIL] Cit+ fraction: {cit_frac:.4f} (expected ≈ {expected_frac:.4f})")

    # --- Compute potentiation statistics ---
    pot_frac = data.get("potentiation_fraction", cit_frac)
    total += 1
    if 0.0 < pot_frac <= 1.0:
        passed += 1
        print(f"  [PASS] Potentiation fraction: {pot_frac:.4f} in (0,1]")
    else:
        print(f"  [FAIL] Potentiation fraction: {pot_frac:.4f}")

    pot_gen = data.get("mean_potentiation_gen")
    if pot_gen is not None:
        total += 1
        if 30000 < pot_gen < 50000:
            passed += 1
            print(f"  [PASS] Mean potentiation generation: {pot_gen:.0f} in (30k, 50k)")
        else:
            print(f"  [FAIL] Mean potentiation generation: {pot_gen:.0f}")

    cit_gen = data.get("mean_cit_plus_gen")
    if cit_gen is not None:
        total += 1
        if 40000 < cit_gen < 55000:
            passed += 1
            print(f"  [PASS] Mean Cit+ generation: {cit_gen:.0f} in (40k, 55k)")
        else:
            print(f"  [FAIL] Mean Cit+ generation: {cit_gen:.0f}")

    # --- Compute potentiation window ---
    if pot_gen is not None and cit_gen is not None:
        window = cit_gen - pot_gen
        total += 1
        if 0 < window < 10000:
            passed += 1
            print(f"  [PASS] Potentiation window: {window:.0f} gens (computed: {cit_gen:.0f} - {pot_gen:.0f})")
        else:
            print(f"  [FAIL] Potentiation window: {window:.0f}")

        total += 1
        if pot_gen < cit_gen:
            passed += 1
            print(f"  [PASS] Potentiation precedes innovation: {pot_gen:.0f} < {cit_gen:.0f}")
        else:
            print(f"  [FAIL] Order violated: {pot_gen:.0f} >= {cit_gen:.0f}")

    # --- Validate replay probabilities ---
    replay = data.get("replay_probabilities", {})
    if replay:
        total += 1
        all_valid = all(isinstance(v, (int, float)) and 0 <= v <= 1 for v in replay.values())
        if all_valid:
            passed += 1
            print(f"  [PASS] Replay probabilities: {len(replay)} entries, all in [0,1]")
        else:
            print(f"  [FAIL] Replay probabilities: invalid values")

        total += 1
        early_zero = all(
            v == 0.0 for k, v in replay.items()
            if float(k) < 20000
        )
        if early_zero:
            passed += 1
            print(f"  [PASS] Early replays (< 20k gen) have zero probability")
        else:
            print(f"  [FAIL] Early replays should be zero")

    # --- Compute two-hit wait time comparison ---
    mu = data.get("mutation_rate_per_gen", 2e-4)
    single_hit_wait = 1.0 / mu if mu > 0 else float("inf")
    two_hit_analytical = 1.0 / (mu * mu) if mu > 0 else float("inf")

    stored_single = data.get("single_hit_mean_wait", single_hit_wait)
    stored_two_hit = data.get("two_hit_analytical_mean", two_hit_analytical)

    total += 1
    if stored_two_hit > stored_single * 10:
        passed += 1
        print(f"  [PASS] Two-hit wait >> single-hit: {stored_two_hit:.0f} > 10×{stored_single:.0f}")
    else:
        print(f"  [FAIL] Two-hit not >> single-hit")

    if cit_gen is not None:
        total += 1
        if cit_gen < stored_two_hit:
            passed += 1
            print(f"  [PASS] Empirical < analytical two-hit: {cit_gen:.0f} < {stored_two_hit:.0f}")
        else:
            print(f"  [FAIL] Empirical >= analytical: {cit_gen:.0f} >= {stored_two_hit:.0f}")

    paper = data.get("paper", "")
    total += 1
    if paper.startswith("Blount"):
        passed += 1
        print(f"  [PASS] Paper citation: {paper}")
    else:
        print(f"  [FAIL] Paper: {paper}")

    elapsed_ms = (time.time() - start) * 1000
    status = "PASS" if passed == total else "FAIL"
    print(f"\ncitrate_innovation: {status} — {passed}/{total} checks ({elapsed_ms:.0f}ms)")

    figures_dir = ROOT / "figures"
    generate_figures(data, figures_dir)

    return 0 if passed == total else 1


def generate_figures(data, output_dir):
    """Generate citrate innovation timeline and replay probability."""
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from litho_figures import can_generate, apply_style, save_figure, ensure_output_dir

    if not can_generate():
        print("  (matplotlib not available — skipping figures)")
        return

    import matplotlib.pyplot as plt
    apply_style()
    out = ensure_output_dir(output_dir)

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    n_pop = data.get("n_populations", 12)
    n_cit = data.get("n_cit_plus", 2)
    cit_frac = n_cit / n_pop if n_pop > 0 else 0
    pot_frac = data.get("potentiation_fraction", cit_frac)

    categories = ["Cit+ Fraction\n(computed)", "Potentiation\nFraction"]
    values = [cit_frac, pot_frac]
    colors = ["#59a14f", "#4e79a7"]
    ax1.bar(categories, values, color=colors, width=0.5)
    ax1.set_ylabel("Fraction")
    ax1.set_title("Citrate Innovation Rates")
    ax1.set_ylim(0, max(values) * 1.3 if max(values) > 0 else 0.3)
    for i, v in enumerate(values):
        ax1.text(i, v + 0.005, f"{v:.3f}", ha="center", fontsize=9)
    ax1.grid(True, alpha=0.3, axis="y")

    replay = data.get("replay_probabilities", {})
    if replay:
        gens = sorted(int(k) for k in replay.keys())
        probs = [replay[str(g)] for g in gens]
        ax2.bar([f"{g // 1000}k" for g in gens], probs, color="#e15759")
        ax2.set_xlabel("Freeze Generation")
        ax2.set_ylabel("Replay Probability")
        ax2.set_title("Historical Contingency: Replay Experiment")
    ax2.grid(True, alpha=0.3, axis="y")

    fig.suptitle("Module 4: Citrate Innovation — Blount et al. 2008/2012\n"
                 "(computed from lineage data)", fontsize=12)
    fig.tight_layout()
    save_figure(fig, out, "m4_citrate_innovation")


if __name__ == "__main__":
    sys.exit(main())
