# The Science of lithoSpore

## Overview

lithoSpore reproduces and validates seven strands of the Long-Term
Evolution Experiment (LTEE) — a continuous evolutionary experiment with
*Escherichia coli* that has run since February 24, 1988, through 75,000+
generations across 12 replicate populations.

This document connects the seven science modules into a coherent
narrative of evolutionary dynamics. The LTEE is the first **instance** of
the lithoSpore verification chassis — the same infrastructure (multi-tier
validation, BLAKE3 integrity, provenance, tolerances) will serve other
domains of science.

### Cross-Tier Parity

Every numerical claim is validated independently in two implementations:
Python (Tier 1) and Rust (Tier 2). `litho parity` runs both side-by-side
and confirms they agree — the math is stable between languages. This is
not a test of software correctness alone; it validates that the science
is implementation-independent.

## Module 1: Fitness Never Stops Increasing

**Wiser, Ribeck & Lenski 2013** (Science 342:1364–1367)

Does fitness plateau after tens of thousands of generations? No.
The LTEE shows fitness follows a power law w(t) ~ t^b, not an
asymptote. lithoSpore fits power-law, hyperbolic, and logarithmic
models to Dryad competition assay data and verifies the power law
wins by AIC/BIC.

Key result: power-law exponent b ≈ 0.66 (equivalently, Wiser's
daily-rate parametrization β ≈ 0.056), with no sign of leveling off.

## Module 2: Mutations Accumulate Like Clockwork

**Barrick et al. 2009** (Nature 461:1243–1247)

The molecular clock ticks at ~8.9×10⁻¹¹ mutations per bp per
generation. lithoSpore validates this rate using Kimura fixation
probability, Poisson accumulation, and Pearson correlation with
the expected linear accumulation curve.

Key result: ~45 mutations fixed per genome at 20,000 generations.

## Module 3: Competing Clades and Clonal Interference

**Good et al. 2017** (Nature 551:45–50)

Deep metagenomic sequencing at ~500-generation intervals reveals
multiple competing clades within each population. Beneficial
mutations arise frequently enough to interfere with each other's
fixation. lithoSpore computes fixation probabilities and
interference ratios across population sizes, confirming that
clonal interference suppresses fixation at large N.

Key result: Fixation probability drops below Haldane's 2s
prediction when N ≥ 10,000.

## Module 4: A Key Innovation — Citrate Use

**Blount et al. 2008** (PNAS 105:7899–7906)
**Blount et al. 2012** (Nature 489:513–518)

After ~31,500 generations, one of the 12 populations evolved the
ability to metabolize citrate — a defining negative trait of *E. coli*.
Replay experiments showed this required potentiating mutations present
~2,000 generations before the innovation. lithoSpore validates the
potentiation window, replay probabilities, and two-hit model.

Key result: Historical contingency — only 1/6 Ara⁻ lines evolved Cit⁺.

## Module 5: The Burden of Synthetic Biology

**Barrick et al. 2024** (Nature Communications 15:6242)

301 BioBrick plasmids impose varying metabolic burden on their
host. lithoSpore computes growth rates from plate reader CSVs,
normalizes burden relative to controls, and validates the
fat-tailed distribution reported in the paper.

Key result: Burden follows a leptokurtic (fat-tailed) distribution,
with most plasmids imposing small burden but a few imposing severe cost.

## Module 6: 264 Genomes Tell the Story

**Tenaillon et al. 2016** (Nature 536:165–170)

264 whole-genome sequences across all 12 populations at 50,000
generations reveal the tempo and mode of genome evolution.
lithoSpore validates mutation accumulation curves, the 6-class
mutation spectrum, Ts/Tv ratios, and the GC→AT mutational bias.

Key result: Near-linear mutation accumulation with a dominant
GC→AT transition bias (68% of point mutations).

## Module 7: Anderson Localization Meets Evolution

This module applies P.W. Anderson's 1958 disorder theory to LTEE
fitness landscapes. The diminishing-returns pattern in fitness
gains maps to the Anderson disorder parameter W/V, and the
distribution of fitness effects (DFE) connects to random matrix
theory (GOE eigenvalue statistics).

lithoSpore computes the level-spacing ratio from fitness data and
validates it against the Wigner surmise prediction for GOE systems.

Key prediction: The LTEE sits near the Anderson transition
(W/V ~ 2–4), explaining both the absence of a fitness plateau
and the decreasing rate of adaptation.

## The Thread That Connects Them

These seven modules trace a path from molecular to ecological:

```
Mutations accumulate (M2) → Some fix, some interfere (M3)
    → Rare innovations emerge (M4) → Fitness keeps climbing (M1)
        → Synthetic constructs impose burden (M5)
            → 264 genomes show the full picture (M6)
                → Disorder theory predicts the dynamics (M7)
```

Each module is independently validatable. Together, they reconstruct
the LTEE as a self-contained reproducible argument.
