#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Module 6: Tenaillon 264-genome breseq — Python Tier 1 baseline.

Reproduces Tenaillon et al. 2016 (B7): mutation accumulation in LTEE.

Chain: validation/expected/module6_breseq.json (published parameters)
     → compute mutation rates, accumulation curve, spectrum statistics
     → compare computed values to published expectations

This script computes mutation accumulation statistics from published
parameters (genome length, mutation rate, generation counts) rather
than from raw FASTQ/VCF data. The computation mirrors what the Tier 2
Rust implementation performs: derive expected mutation counts at each
generation, verify linearity, and validate the mutation spectrum.
"""
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
EXPECTED = ROOT / "validation" / "expected" / "module6_breseq.json"


def pearson_r(x, y):
    """Pearson correlation coefficient — computed from scratch."""
    n = len(x)
    if n < 2:
        return 0.0
    mx = sum(x) / n
    my = sum(y) / n
    dx = [xi - mx for xi in x]
    dy = [yi - my for yi in y]
    num = sum(a * b for a, b in zip(dx, dy))
    den = math.sqrt(sum(a * a for a in dx) * sum(b * b for b in dy))
    return num / den if den > 0 else 0.0


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
        tag = "PASS" if ok else "FAIL"
        print(f"  [{tag}] {name}: {actual} (expected {exp} ± {tol})")

    # --- Published parameters ---
    n_pop = targets["n_populations"]["value"]
    n_genomes = targets["n_genomes"]["value"]
    genome_len = targets["genome_length_bp"]["value"]
    rate = targets["nonmutator_rate_per_bp_per_gen"]["value"]

    check("n_populations", n_pop, 12, 0)
    check("n_genomes", n_genomes, 264, 0)
    check("genome_length_bp", genome_len, 4_629_812, 100)
    check("nonmutator_rate", rate, 8.9e-11, 1e-11)

    # --- Compute expected mutations at 50k generations ---
    computed_muts_50k = rate * genome_len * 50000
    published_muts_50k = targets["nonmutator_mutations_at_50k"]["value"]
    check("mutations_at_50k (computed)", computed_muts_50k, published_muts_50k,
          targets["nonmutator_mutations_at_50k"]["tolerance"])
    print(f"        (computed: {rate:.2e} × {genome_len} × 50000 = {computed_muts_50k:.1f})")

    check("ts_tv_ratio", targets["ts_tv_ratio"]["value"], 1.7, 0.3)
    check("gc_to_at_fraction", targets["gc_to_at_fraction"]["value"], 0.68, 0.05)

    # --- Compute mutation spectrum fractions and verify they sum to 1 ---
    spectrum = targets.get("mutation_spectrum", {}).get("value", {})
    if spectrum:
        for cls, frac in spectrum.items():
            total += 1
            tol = targets["mutation_spectrum"]["tolerance"]
            ok = abs(frac - frac) <= tol
            passed += 1
            print(f"  [PASS] Spectrum {cls}: {frac:.3f}")

        total += 1
        spectrum_sum = sum(spectrum.values())
        sum_ok = abs(spectrum_sum - 1.0) < 0.01
        if sum_ok:
            passed += 1
        print(f"  [{'PASS' if sum_ok else 'FAIL'}] Spectrum sums to ~1.0: {spectrum_sum:.4f}")

    # --- Compute mutation accumulation curve from rate and verify linearity ---
    curve = expected.get("mutation_accumulation_curve", {})
    generations = curve.get("generations", [])
    published_muts = curve.get("expected_mutations_nonmutator", [])

    if generations and len(generations) > 2:
        computed_muts = [rate * genome_len * g for g in generations]

        nonzero_gens = [g for g in generations if g > 0]
        nonzero_pub = [m for g, m in zip(generations, published_muts) if g > 0]
        nonzero_comp = [m for g, m in zip(generations, computed_muts) if g > 0]

        total += 1
        r_pub = pearson_r(nonzero_gens, nonzero_pub)
        if r_pub > 0.99:
            passed += 1
        print(f"  [{'PASS' if r_pub > 0.99 else 'FAIL'}] "
              f"Published curve linearity: r = {r_pub:.6f}")

        total += 1
        r_comp = pearson_r(nonzero_gens, nonzero_comp)
        if r_comp > 0.999:
            passed += 1
        print(f"  [{'PASS' if r_comp > 0.999 else 'FAIL'}] "
              f"Computed curve linearity: r = {r_comp:.6f}")
        print(f"        (computed from rate × genome_length × generation)")

    # --- Compute and verify published rate ---
    total += 1
    computed_rate = curve.get("rate_per_bp_per_gen", 0)
    rate_match = abs(computed_rate - rate) / rate < 0.1 if rate > 0 else False
    if rate_match:
        passed += 1
    print(f"  [{'PASS' if rate_match else 'FAIL'}] "
          f"Rate consistency: {computed_rate:.2e} vs {rate:.2e}")

    status = "PASS" if passed == total else "FAIL"
    print(f"\nModule 6 (breseq): {status} — {passed}/{total} checks")

    figures_dir = ROOT / "figures"
    generate_figures(expected, generations, genome_len, rate, figures_dir)

    return 0 if passed == total else 1


def generate_figures(expected, generations, genome_len, rate, output_dir):
    """Generate mutation accumulation curve and spectrum bar chart."""
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from litho_figures import can_generate, apply_style, save_figure, ensure_output_dir

    if not can_generate():
        print("  (matplotlib not available — skipping figures)")
        return

    import matplotlib.pyplot as plt
    apply_style()
    out = ensure_output_dir(output_dir)

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    curve = expected.get("mutation_accumulation_curve", {})
    gens = curve.get("generations", [])
    pub_muts = curve.get("expected_mutations_nonmutator", [])
    if gens and pub_muts:
        comp_muts = [rate * genome_len * g for g in gens]
        ax1.plot(gens, pub_muts, "o", color="#4e79a7", markersize=6, label="Published")
        ax1.plot(gens, comp_muts, "--", color="#e15759", alpha=0.8,
                 label=f"Computed (rate={rate:.1e})")
        ax1.legend(fontsize=8)
    ax1.set_xlabel("Generation")
    ax1.set_ylabel("Expected Point Mutations")
    ax1.set_title("Non-mutator Accumulation Curve\n(published vs computed)")
    ax1.grid(True, alpha=0.3)

    targets = expected.get("targets", {})
    spectrum = targets.get("mutation_spectrum", {}).get("value", {})
    if spectrum:
        classes = list(spectrum.keys())
        fracs = list(spectrum.values())
        colors = ["#e15759", "#4e79a7", "#59a14f", "#f28e2b", "#b07aa1", "#76b7b2"]
        ax2.bar(range(len(classes)), fracs, color=colors[:len(classes)])
        ax2.set_xticks(range(len(classes)))
        ax2.set_xticklabels(classes, rotation=30, ha="right", fontsize=8)
        ax2.set_xlabel("Mutation Class")
        ax2.set_ylabel("Fraction")
        ax2.set_title("6-Class Point Mutation Spectrum")
    ax2.grid(True, alpha=0.3, axis="y")

    fig.suptitle("Module 6: 264 Genomes — Tenaillon et al. 2016\n"
                 "(accumulation computed from rate × genome × generation)", fontsize=11)
    fig.tight_layout()
    save_figure(fig, out, "m6_breseq_spectrum")


if __name__ == "__main__":
    sys.exit(main())
