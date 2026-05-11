#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 1: Power-law fitness trajectories — Python Tier 1 baseline.

Reproduces Wiser et al. 2013 (B2): fits power-law, hyperbolic, and
logarithmic models to LTEE fitness data. Compares via AIC/BIC.

Data: data/wiser_2013/ (Dryad CC0)
Upstream: groundSpring B2 (model selection), wetSpring B2 (Anderson-QS)
"""

import json
import sys
from pathlib import Path

ARTIFACT_ROOT = Path(__file__).resolve().parent.parent.parent / "artifact"
DATA_DIR = ARTIFACT_ROOT / "data" / "wiser_2013"
EXPECTED = ARTIFACT_ROOT / "validation" / "expected" / "module1_fitness.json"


def main():
    print("Module 1: Power-law fitness — Python baseline")
    print(f"  Data dir: {DATA_DIR}")
    print(f"  Expected: {EXPECTED}")

    if not DATA_DIR.exists():
        print("  SKIP: Data not yet fetched. Run ./ltee refresh first.")
        return 2

    # TODO: Implement when data is available:
    # 1. Load fitness time-series from Wiser 2013 supplemental
    # 2. Fit power-law: w(t) = (1 + α)^(t^β)
    # 3. Fit hyperbolic: w(t) = a*t / (b + t)
    # 4. Fit logarithmic: w(t) = a * ln(t) + b
    # 5. AIC/BIC model comparison
    # 6. Compare against expected values

    print("  SKIP: Awaiting upstream spring reproductions")
    return 2


if __name__ == "__main__":
    sys.exit(main())
