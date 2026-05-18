# lithoSpore — Prototype Layer

**Author:** Kevin Mok (BS Microbiology, MSU 2018; MS Data Science, MSU 2025)
**Date:** May 17, 2026
**License:** AGPL-3.0-or-later
**Website:** [primals.eco](https://primals.eco)

---

## What This Document Is

This artifact is a working prototype. The science is real, the code runs,
and every claim traces to a published paper. But the system around it is
still evolving. This document names what's finished, what's in progress,
and what's planned — honestly, so you can evaluate both the current state
and the trajectory.

---

## What's Solid

### The Science (Modules 1–6)

These six modules reproduce published LTEE results. Every one runs,
passes, and produces values within named tolerances justified by
measurement science:

| Module | Paper | Status |
|--------|-------|--------|
| 1. Power-law fitness | Wiser 2013, *Science* | Reproduced — AIC/BIC model comparison, jackknife SE |
| 2. Mutation accumulation | Barrick 2009, *Nature* | Reproduced — Kimura fixation, Poisson accumulation |
| 3. Allele trajectories | Good 2017, *Nature* | Reproduced — clonal interference, fixation probability |
| 4. Citrate innovation | Blount 2008/2012 | Validated — potentiation window, replay probability |
| 5. BioBrick burden | Barrick 2024, *Nat Comms* | Reproduced — complete dataset shipped (3.3 MB, 69 files) |
| 6. 264-genome evolution | Tenaillon 2016, *Nature* | Reproduced — accumulation curves, Ts/Tv, spectrum |

The Python baselines (`notebooks/`) are readable, commented, and use
standard numpy/scipy. The Rust implementations (`bin/litho`) produce
identical results. Cross-tier parity is verified.

### The Validation Infrastructure

- **`./validate`** — runs all 7 modules, reports PASS/FAIL with tolerances
- **`./verify`** — BLAKE3 integrity check on all bundled data
- **14 validation targets** (T01–T14) with named tolerances and justifications
- **`liveSpore.json`** — append-only deployment provenance log
- **`data_manifest.toml`** — BLAKE3 inventory of every file

### The Paper Trail

- **16 papers** registered in `papers/registry.toml` with DOIs, specific
  figures reproduced, specific tables validated
- **`READING_ORDER.md`** — guided reading path through the LTEE literature
- **`SCIENCE.md`** — narrative connecting all 7 modules

---

## What's In Progress

### Module 7: Anderson-QS Predictions

Module 7 applies P.W. Anderson's 1958 disorder theory (condensed matter
physics) to LTEE fitness landscapes. **This is not a reproduction of
published LTEE work — it's a new theoretical prediction.**

The core idea: the diminishing-returns pattern in LTEE fitness gains maps
to Anderson's disorder parameter W/V. The distribution of fitness effects
(DFE) connects to random matrix theory (GOE eigenvalue statistics). The
prediction is that the LTEE sits near the Anderson localization transition,
explaining both the absence of a fitness plateau and the decreasing rate
of adaptation.

**Current state:**
- The computation runs and passes its internal checks
- The expected values are self-generated (all 4 targets marked "Internal")
- The GOE/Poisson level-spacing diagnostic is implemented correctly
- The Anderson framework has 3,700+ validation checks across the broader
  ecosystem (baseCamp Paper 01, [primals.eco/science/01-anderson-qs](https://primals.eco/science/01-anderson-qs/))

**What's needed to move this from prediction to validation:**
- Real LTEE fitness data at higher resolution (per-replicate, per-generation)
- DFE measurements from the recent Baym/Tenaillon/Couce 2024 *Science* paper
- Bench experiments: biofilm vs planktonic growth with sdiA reporter assays
  using existing LTEE strains (see baseCamp Paper 02, Section 1.3)

The broader Anderson-QS framework — applying condensed matter localization
theory to microbial quorum sensing — is documented in 28 baseCamp papers
at [primals.eco/science](https://primals.eco/science/) and has been
reviewed by faculty in physics (Murillo, Chuna — MSU CMSE), mathematics
(Kachkovskiy — MSU), and microbiology (Waters — MSU MMG).

### Sovereign Variant Calling Pipeline

The `provenance/braids/` directory contains two provenance records from
wetSpring's sovereign variant calling pipeline applied to Barrick 2009
sequencing data. This pipeline (FM-index, Smith-Waterman GPU alignment,
SNP calling) is in early development:

- **159 sovereign variants vs 569 breseq variants, 0 position matches**
- This reflects coordinate convention differences and pipeline immaturity,
  not a challenge to breseq
- The braid is included for provenance transparency — documenting what was
  attempted, not claiming correctness
- Aligning sovereign output to breseq coordinate conventions is an active
  development target (wetSpring Track 1)

breseq is the gold standard for LTEE mutation calling. The sovereign
pipeline's goal is to provide a second independent implementation for
cross-validation, not replacement.

### Data Completeness

The artifact ships summary statistics for 6 of 7 datasets. Full upstream
data requires SRA pulls ranging from 5 MB to 200 GB:

| Dataset | Shipped | Full Upstream | Status |
|---------|---------|---------------|--------|
| BioBricks 2024 | **Complete** (3.3 MB) | Same | All checks run on shipped data |
| Wiser 2013 | 12-row summary | ~5 MB Dryad | `litho fetch` ready |
| Barrick 2009 | Published params | ~15 GB SRA | Pipeline exists |
| Good 2017 | Simulation tallies | ~50 GB SRA | Pipeline designed |
| Blount 2012 | Replay summary | ~30 GB SRA | Pipeline designed |
| Tenaillon 2016 | Published stats | ~200 GB SRA | Pipeline designed |
| Anderson-QS | Internal predictions | n/a | Needs real LTEE data |

The two-tier data model (summary for airgapped validation, full for deep
re-analysis) is intentional. `litho fetch --full` will pull raw upstream
data when implemented. Each dataset in `artifact/data.toml` documents
what deeper checks become possible with full data.

### Binary Portability

The `bin/litho` binary is currently dynamically linked against glibc.
The documentation references musl-static linking, which is the target:

- **Current:** x86_64 ELF, dynamically linked (works on standard Linux)
- **Target:** musl-static binary (runs on any Linux, Alpine, containers)
- The Containerfile provides a cross-OS fallback via Docker/Podman

---

## The Ecosystem Around This

This USB is one artifact from a larger system. The parts relevant to
LTEE work:

### Springs (Science Validation)

| Spring | What It Does | LTEE Relevance |
|--------|-------------|----------------|
| [wetSpring](https://github.com/syntheticChemistry/wetSpring) | 16S/WGS pipelines, diversity metrics, sovereign NCBI | LTEE population genomics, 264-genome ingest |
| [groundSpring](https://github.com/syntheticChemistry/groundSpring) | Statistical measurement science, uncertainty | Jackknife SE, drift vs selection, model comparison |
| [neuralSpring](https://github.com/syntheticChemistry/neuralSpring) | ML primitives — LSTM, ESN, HMM | Trajectory prediction, regime classification |
| [hotSpring](https://github.com/syntheticChemistry/hotSpring) | Physics — Anderson localization, lattice QCD, GPU | DFE/RMT connection, disorder theory |

These are public repositories. 175+ papers reproduced across 8 domains,
20,695+ validation checks, 112K+ tests. See
[primals.eco/science](https://primals.eco/science/) for the full catalog.

### Infrastructure (Primals)

The USB references "primals" — these are Rust microservices that compose
into a system called NUCLEUS. They handle cryptography, provenance,
storage, visualization, and compute dispatch. On this USB, none of them
are required — Tier 1 and Tier 2 validation run standalone. Tier 3
(provenance trio) requires a NUCLEUS deployment.

The ecosystem vocabulary (primals, springs, biomeOS, spores) comes from
a mycological metaphor. The [Glossary](https://primals.eco/glossary/)
at primals.eco has plain-language definitions for every term.

### baseCamp Papers

Two baseCamp papers are directly relevant to LTEE research:

- **Paper 01** — [Anderson Localization as QS Null Hypothesis](https://primals.eco/science/01-anderson-qs/):
  The core framework. 3D communities sustain QS signaling; 2D/1D suppress
  it. W_c = 16.26 validated. 3,700+ checks.

- **Paper 02** — [LTEE Extensions](https://primals.eco/science/02-ltee-extensions/):
  Specific, falsifiable predictions for LTEE populations. sdiA biofilm
  vs planktonic experiments. Potentiation as Anderson near-criticality.
  Testable with existing strains and standard reporter assays.

---

## Why This Matters to the Barrick/Lenski Lab

### For LTEE Operations

The daily LTEE involves serial transfers, frozen fossil archiving, and
population characterization. This artifact demonstrates:

- **Computational fluency with the LTEE literature** — every major LTEE
  paper reproduced with correct expected values, proper tolerances, and
  traceable provenance
- **Bench-to-computation bridge** — the same person who understands
  aseptic technique and fermentation (5 years MSUBI BSL2) also wrote
  the curve fitting, model comparison, and statistical validation code
- **Data management patterns** — BLAKE3 integrity, structured manifests,
  and provenance tracking that could strengthen the frozen fossil record's
  computational audit trail

### For breseq and Computational Genomics

- The sovereign variant calling pipeline is complementary, not competitive.
  An independent Rust implementation that can cross-validate breseq calls
  is useful for the same reason Tier 1/Tier 2 parity is useful: if two
  independent implementations agree, the result is more trustworthy
- The Python baselines use standard numpy/scipy — graduate students can
  read and extend them immediately
- The Rust layer adds speed and reproducibility without changing the math

### For the Anderson-QS Predictions

This is the novel scientific contribution. No prior work applies Anderson
localization to quorum sensing (confirmed via literature search, February
2026). The predictions are specific and testable:

- sdiA expression should differ in biofilm (3D) vs planktonic (shaken flask)
  for LTEE strains — Anderson predicts geometry-dependent response
- The DFE evolution documented in Baym/Tenaillon/Couce 2024 (*Science*)
  should connect to random matrix theory eigenvalue statistics
- Potentiation for the citrate innovation may correspond to near-critical
  Anderson disorder — a testable structural prediction

These experiments use existing LTEE strains, standard reporter assays,
and plate reader data. The computational infrastructure to analyze the
results is already built.

### For Insect Symbiont Engineering

The Anderson framework makes specific predictions for engineered
symbionts in structured host environments:

- Bee gut (3D biofilm) → QS predicted active → coordinated engineered
  gene expression should work
- Aphid bacteriome (near-monoculture) → low disorder → deep extended
  regime → different coordination dynamics
- baseCamp Paper 05 (Cross-Species Signaling) develops these predictions
  in detail

### For the Lab Team

- **Reading order and paper registry** — useful for onboarding new
  lab members into the LTEE literature
- **Executable reproductions** — graduate students can modify the Python
  baselines, re-run `./validate`, and immediately see if their changes
  break the published results
- **Everything is open source** (AGPL-3.0) and runs on commodity hardware

---

## How This Was Built

This artifact — and the broader ecosystem it comes from — was built using
AI-assisted development. Not as a novelty, but as a methodology taken
seriously enough to name, formalize, and document publicly.

### K-NOME: Knowledge-Numeric Observed & Mentored Evolutionary Programming

K-NOME is the operational methodology behind ecoPrimals. The full
writeup is at [primals.eco/methodology/k-nome-programming](https://primals.eco/methodology/k-nome-programming/).

The short version: one human with domain expertise (microbiology,
fermentation, data science) mentors an AI (Cursor IDE) through
iterative evolutionary cycles. The Rust compiler is the fitness
function — code either compiles and passes tests, or it doesn't. Every
generation is validated against published scientific results.

**This is not vibecoding.** The distinction matters:

| | Vibecoding | K-NOME |
|--|-----------|--------|
| Human role | Prompt loosely, accept output | Mentor with domain expertise, observe, correct |
| AI role | Generate code from vague instructions | Knowledgeable collaborator under selective pressure |
| Validation | "It seems to work" | 20,695+ checks against published papers |
| Observation | Minimal — accept/reject | Bidirectional — human deepens understanding as project grows |
| Inheritance | None — each prompt is independent | Lamarckian — patterns propagate across springs and primals |

### The Numbers

| Metric | Value |
|--------|-------|
| Tool | Cursor IDE (only — no multi-agent frameworks) |
| Agent invocations | 69,000+ |
| Tokens processed | 51 billion |
| Consecutive days | 185 |
| Duration | ~10 months |
| Rust lines | 3.3 million (zero C dependencies) |
| Test functions | 112,000+ |
| Papers reproduced | 175+ from peer-reviewed literature |
| Validation checks | 20,695+ (exit 0 on pass) |
| WGSL GPU shaders | 952 (74K lines, cross-domain) |
| Developer | One person |

### Why This Matters for a Lab

AI-assisted development is coming to every lab. The question is whether
it arrives as unvetted generated code or as a methodology with:

- **Reproducibility guarantees** — every claim validated against a published result
- **Compiler-enforced correctness** — Rust's type system rejects unsound code
- **Full provenance** — every generation traceable through commit history
- **Domain expertise as the constraint** — the AI generates candidates,
  the scientist's knowledge selects

The 7 modules on this USB were built this way. The Python baselines are
human-readable. The Rust implementations are compiler-verified. The
expected values come from the published literature. The methodology
produces real science — not generated text that looks like science.

K-NOME is documented publicly because AI development methodology
should be transparent, especially when the output is used for
scientific work.

---

## How to Explore

```bash
# Validate all 7 modules (works airgapped, ~5 seconds)
./validate

# Verify data integrity (BLAKE3 checksums)
./verify

# Read the science narrative
less SCIENCE.md

# Read the guided paper trail
less papers/READING_ORDER.md

# Look at a Python baseline
less notebooks/module1_fitness/power_law_fitness.py

# Run in a container (any OS with Docker/Podman)
docker build -f Containerfile -t litho-spore . && docker run litho-spore
```

For the broader ecosystem: [primals.eco](https://primals.eco)

---

## Contact

Kevin Mok — mokkevin@msu.edu
GitHub: [github.com/syntheticChemistry](https://github.com/syntheticChemistry) (springs)
       [github.com/ecoPrimals](https://github.com/ecoPrimals) (infrastructure)
       [github.com/sporeGarden](https://github.com/sporeGarden) (products, including lithoSpore)
