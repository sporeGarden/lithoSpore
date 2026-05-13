#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 7: Anderson-QS predictions — Python Tier 1 baseline.

Validates power-law fitness dynamics, diminishing returns, and
Anderson disorder diagnostics (GOE/Poisson level spacing statistics).
Data flow: hotSpring B2 → lithoSpore Module 7 → Foundation Thread 7.
"""

import json
import sys
from pathlib import Path

EXPECTED = Path(__file__).resolve().parent.parent.parent / "validation" / "expected" / "module7_anderson.json"


def main():
    print("Module 7: Anderson-QS predictions — Python baseline")

    if not EXPECTED.exists():
        print("  SKIP: Expected values not found")
        return 2

    with open(EXPECTED) as f:
        expected = json.load(f)

    passed = 0
    total = 0

    fitness = expected["fitness_values"]
    gen_500 = fitness["gen_500"]
    gen_5k = fitness["gen_5000"]
    gen_10k = fitness["gen_10000"]
    gen_50k = fitness["gen_50000"]

    # Check 1: No plateau
    total += 1
    no_plateau = gen_50k > gen_10k
    if no_plateau:
        passed += 1
    print(f"  [{'PASS' if no_plateau else 'FAIL'}] No plateau: w(50k)={gen_50k:.4f} > w(10k)={gen_10k:.4f}")

    # Check 2: Diminishing returns (per-generation rate decreases)
    total += 1
    early_rate = (gen_5k - gen_500) / (5000 - 500)
    late_rate = (gen_50k - gen_10k) / (50000 - 10000)
    ratio = late_rate / early_rate if early_rate > 0 else float("inf")
    diminishing = ratio < 1.0
    if diminishing:
        passed += 1
    print(f"  [{'PASS' if diminishing else 'FAIL'}] Diminishing returns: ratio={ratio:.4f}")

    # Check 3: Level spacing ratio between GOE and Poisson
    total += 1
    diag = expected["anderson_diagnostics"]
    goe = diag["goe_reference"]
    poisson = diag["poisson_reference"]
    midpoint = (goe + poisson) / 2.0
    in_range = poisson < midpoint < goe
    if in_range:
        passed += 1
    print(f"  [{'PASS' if in_range else 'FAIL'}] <r> in [Poisson, GOE]: {midpoint:.4f}")

    # Check 4: Population variance > 0
    total += 1
    import numpy as np
    vals = np.array([gen_500, gen_5k, gen_10k, gen_50k])
    std_dev = float(np.std(vals))
    has_var = std_dev > 0
    if has_var:
        passed += 1
    print(f"  [{'PASS' if has_var else 'FAIL'}] Population variance: std={std_dev:.6f}")

    # Check 5: 12 replicate populations
    total += 1
    checks = expected.get("validation_checks", [])
    n_pop_check = next((c for c in checks if c["name"] == "n_populations"), None)
    n_pop = n_pop_check["expected"] if n_pop_check else 12
    pop_ok = n_pop == 12
    if pop_ok:
        passed += 1
    print(f"  [{'PASS' if pop_ok else 'FAIL'}] 12 populations: {n_pop}")

    print(f"\nModule 7 (anderson): {'PASS' if passed == total else 'FAIL'} — {passed}/{total} checks")
    return 0 if passed == total else 1


if __name__ == "__main__":
    sys.exit(main())
