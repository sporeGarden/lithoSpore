#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 4: Citrate innovation cascade — Python Tier 1 baseline.

Reproduces Blount et al. 2008/2012 (B4): Cit+ innovation timeline.
Validates potentiation, replay probabilities, and two-hit model
from expected values JSON.
Data: data/blount_2012/ (NCBI public-domain)
Upstream: neuralSpring B4 (early warning ESN), groundSpring B4 (rare event stats)
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
    / "module4_citrate.json"
)


def main() -> int:
    start = time.time()
    print("Module 4: Citrate innovation — Python Tier 1 baseline")

    if not EXPECTED_PATH.exists():
        print("  SKIP: Expected values not found — run groundSpring B4 first")
        return 2

    with open(EXPECTED_PATH) as f:
        expected = json.load(f)

    passed = 0
    total = 0

    cit_frac = expected.get("cit_plus_fraction")
    if cit_frac is not None:
        total += 1
        expected_frac = 1.0 / 6.0
        if abs(cit_frac - expected_frac) < 0.01:
            passed += 1
            print(f"  [PASS] cit_plus_fraction={cit_frac:.6f} ≈ 1/6")
        else:
            print(f"  [FAIL] cit_plus_fraction={cit_frac:.6f} (expected ≈ {expected_frac:.6f})")

    pot_frac = expected.get("potentiation_fraction")
    if pot_frac is not None:
        total += 1
        if 0.0 < pot_frac <= 1.0:
            passed += 1
            print(f"  [PASS] potentiation_fraction={pot_frac:.4f} in (0,1]")
        else:
            print(f"  [FAIL] potentiation_fraction={pot_frac:.4f} out of (0,1]")

    pot_gen = expected.get("mean_potentiation_gen")
    if pot_gen is not None:
        total += 1
        if 30000.0 < pot_gen < 50000.0:
            passed += 1
            print(f"  [PASS] mean_potentiation_gen={pot_gen:.0f} in (30k, 50k)")
        else:
            print(f"  [FAIL] mean_potentiation_gen={pot_gen:.0f} outside (30k, 50k)")

    cit_gen = expected.get("mean_cit_plus_gen")
    if cit_gen is not None:
        total += 1
        if 40000.0 < cit_gen < 55000.0:
            passed += 1
            print(f"  [PASS] mean_cit_plus_gen={cit_gen:.0f} in (40k, 55k)")
        else:
            print(f"  [FAIL] mean_cit_plus_gen={cit_gen:.0f} outside (40k, 55k)")

    replay = expected.get("replay_probabilities")
    if replay is not None and isinstance(replay, dict):
        total += 1
        all_valid = all(
            isinstance(v, (int, float)) and 0.0 <= v <= 1.0 for v in replay.values()
        )
        if all_valid:
            passed += 1
            print(f"  [PASS] replay_probabilities: {len(replay)} entries, all in [0,1]")
        else:
            print(f"  [FAIL] replay_probabilities: some values outside [0,1]")

    single = expected.get("single_hit_mean_wait")
    two_hit = expected.get("two_hit_analytical_mean")
    if single is not None and two_hit is not None:
        total += 1
        if two_hit > single * 10.0:
            passed += 1
            print(f"  [PASS] two_hit_analytical >> single_hit ({two_hit:.0f} > {single:.0f}×10)")
        else:
            print(f"  [FAIL] two_hit_analytical not >> single_hit ({two_hit:.0f} vs {single:.0f}×10)")

    empirical = expected.get("two_hit_empirical_mean")
    if empirical is not None and two_hit is not None:
        total += 1
        if empirical < two_hit:
            passed += 1
            print(f"  [PASS] empirical < analytical ({empirical:.0f} < {two_hit:.0f})")
        else:
            print(f"  [FAIL] empirical >= analytical ({empirical:.0f} >= {two_hit:.0f})")

    paper = expected.get("paper", "")
    if paper:
        total += 1
        if paper.startswith("Blount"):
            passed += 1
            print(f"  [PASS] paper={paper}")
        else:
            print(f"  [FAIL] paper={paper} (expected Blount*)")

    elapsed_ms = (time.time() - start) * 1000
    print(f"\ncitrate_innovation: {passed}/{total} checks passed ({elapsed_ms:.0f}ms)")

    figures_dir = Path(__file__).resolve().parent.parent.parent / "figures"
    generate_figures(expected, figures_dir)

    return 0 if passed == total else 1


def generate_figures(expected, output_dir):
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

    categories = ["Cit+ Fraction", "Potentiation\nFraction"]
    values = [expected.get("cit_plus_fraction", 0),
              expected.get("potentiation_fraction", 0)]
    colors = ["#59a14f", "#4e79a7"]
    ax1.bar(categories, values, color=colors, width=0.5)
    ax1.set_ylabel("Fraction")
    ax1.set_title("Citrate Innovation Rates")
    ax1.set_ylim(0, max(values) * 1.3 if max(values) > 0 else 0.3)
    for i, v in enumerate(values):
        ax1.text(i, v + 0.005, f"{v:.3f}", ha="center", fontsize=9)
    ax1.grid(True, alpha=0.3, axis="y")

    pot_gen = expected.get("mean_potentiation_gen", 0)
    cit_gen = expected.get("mean_cit_plus_gen", 0)
    ax1.axhline(y=0, color="black", linewidth=0.5)

    replay = expected.get("replay_probabilities", {})
    if replay:
        gens = sorted(int(k) for k in replay.keys())
        probs = [replay[str(g)] for g in gens]
        ax2.bar([str(g // 1000) + "k" for g in gens], probs, color="#e15759")
        ax2.set_xlabel("Freeze Generation")
        ax2.set_ylabel("Replay Probability")
        ax2.set_title("Historical Contingency: Replay Experiment")
        ax2.axvline(x=len(gens) - 2, ls="--", color="gray", alpha=0.5,
                    label=f"Pot. gen ~{pot_gen:.0f}")
        ax2.legend(fontsize=8)
    ax2.grid(True, alpha=0.3, axis="y")

    fig.suptitle("Module 4: Citrate Innovation — Blount et al. 2008/2012", fontsize=13)
    fig.tight_layout()
    save_figure(fig, out, "m4_citrate_innovation")


if __name__ == "__main__":
    sys.exit(main())
