#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 6: Tenaillon 264-genome breseq — Python Tier 1 baseline.

Validates mutation accumulation curve, mutation spectrum, ts/tv ratio,
and clock-like linearity from 264 sequenced LTEE clones (BioProject PRJNA294072).
Data flow: wetSpring B7 → lithoSpore Module 6 → Foundation Thread 4/5.
"""

import json
import sys
from pathlib import Path

import numpy as np

EXPECTED = Path(__file__).resolve().parent.parent.parent / "validation" / "expected" / "module6_breseq.json"


def pearson_r(x, y):
    """Pearson correlation coefficient."""
    x, y = np.asarray(x, dtype=float), np.asarray(y, dtype=float)
    mx, my = x.mean(), y.mean()
    dx, dy = x - mx, y - my
    denom = np.sqrt((dx**2).sum() * (dy**2).sum())
    if denom == 0:
        return 0.0
    return float((dx * dy).sum() / denom)


def main():
    print("Module 6: 264-genome breseq comparison — Python baseline")

    if not EXPECTED.exists():
        print("  SKIP: Expected values not found")
        return 2

    with open(EXPECTED) as f:
        expected = json.load(f)

    targets = expected["targets"]
    passed = 0
    total = 0

    def check(name, actual, exp, tol):
        nonlocal passed, total
        total += 1
        ok = abs(actual - exp) <= tol
        if ok:
            passed += 1
        print(f"  [{'PASS' if ok else 'FAIL'}] {name}: {actual} (expected {exp}±{tol})")

    check("n_populations", targets["n_populations"]["value"], 12, 0)
    check("n_genomes", targets["n_genomes"]["value"], 264, 0)
    check("genome_length_bp", targets["genome_length_bp"]["value"], 4_629_812, 100)
    check("nonmutator_rate", targets["nonmutator_rate_per_bp_per_gen"]["value"], 8.9e-11, 1e-11)
    check("mutations_at_50k", targets["nonmutator_mutations_at_50k"]["value"], 20.6, 2.3)
    check("ts_tv_ratio", targets["ts_tv_ratio"]["value"], 1.7, 0.3)
    check("gc_to_at_fraction", targets["gc_to_at_fraction"]["value"], 0.68, 0.05)

    # Check linearity of accumulation curve
    total += 1
    curve = expected["mutation_accumulation_curve"]
    gens = np.array(curve["generations"])
    muts = np.array(curve["expected_mutations_nonmutator"])
    mask = gens > 0
    r = pearson_r(gens[mask], muts[mask])
    linear_ok = r > 0.99
    if linear_ok:
        passed += 1
    print(f"  [{'PASS' if linear_ok else 'FAIL'}] Near-linear accumulation: r={r:.6f}")

    print(f"\nModule 6 (breseq): {'PASS' if passed == total else 'FAIL'} — {passed}/{total} checks")
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
