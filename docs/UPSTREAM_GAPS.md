# lithoSpore — Upstream & River Delta Gap Analysis

**Last Updated**: May 13, 2026 (4/7 modules PASS Tier 2 — Pillar 4 GATE EXCEEDED)
**Purpose**: Track what lithoSpore needs from upstream springs and the river delta
to complete each module. This is the gap inventory that drives evolution cycles.
**Interstadial exit criteria**: `infra/wateringHole/INTERSTADIAL_EXIT_CRITERIA.md` (Pillar 4)

---

## Gap Summary

| Module | Ready | Blocked By | Severity | Phase |
|--------|-------|-----------|----------|-------|
| 1. ltee-fitness | **Tier 2 PASS (8/8)** | — | **RESOLVED** | Interstadial |
| 2. ltee-mutations | **Tier 2 PASS (7/7)** | — | **RESOLVED** | Interstadial |
| 3. ltee-alleles | **Tier 2 LIVE** | groundSpring B3 INGESTED, neuralSpring B3 ML pending | Low | Interstadial |
| 4. ltee-citrate | **Tier 2 LIVE** | groundSpring B4 INGESTED, neuralSpring B4 ML pending | Low | Interstadial |
| 5. ltee-biobricks | Scaffold | neuralSpring B6, groundSpring B6 | Medium | Stadial |
| 6. ltee-breseq | **Tier 2 PASS (8/8)** | — | **RESOLVED** | Interstadial |
| 7. ltee-anderson | **Tier 2 PASS (5/5)** | — | **RESOLVED** | Interstadial |

**6/7 modules live** — Modules 1, 2, 3, 4, 6, 7 have Rust Tier 2 validation
implementations. Modules 3+4 promoted from scaffold via groundSpring V140 ingestion
(B3 Good 2017 + B4 Blount 2008/2012). Module 5 (biobricks) remains scaffold —
DOI pending. Full ML surrogate enrichment (neuralSpring B3/B4) is additive, not blocking.

**Interstadial exit gate (Pillar 4)**: **EXCEEDED** — 6 modules wired at Tier 2
(Rust). BLAKE3 provenance on fetched data. Fetch scripts created for all modules
with upstream data (B1–B4, B7).

---

## Per-Spring Gap Detail

### groundSpring (8 papers — contributes to ALL 7 modules)

groundSpring is the **critical path**. Every module depends on statistical
methods from groundSpring.

| Paper | Gap | What lithoSpore Needs |
|-------|-----|----------------------|
| B1 | Drift vs selection null model | Neutral mutation rate estimator, fixation probability calculator |
| B2 | Jackknife + AIC/BIC model selection | Model fitting API: power-law, hyperbolic, logarithmic with AIC/BIC |
| B3 | Clonal interference statistics | Multi-beneficial-mutation fixation probability, interference model |
| B4 | Rare event statistics | Probability framework for potentiating mutation cascades |
| B6 | Anderson Wc analogy | Burden → disorder potential mapping, statistical distribution fitting |
| B7 | Epistasis quantification | Parallel evolution significance tests across 264 genomes |
| B8 | Bet-hedging statistics | Phase variation rate estimation, stochastic switching models |
| B9 | DFE fitting | Gamma/exponential/lognormal parameter estimation for DFE |

**Status**: B1 and B2 **COMPLETE** — Python 9/9 + Rust 10/10 (B2 Wiser), Python
8/8 + Rust 8/8 (B1 Barrick). Expected values ported to lithoSpore
`validation/expected/`. Remaining 6 papers QUEUED.

### neuralSpring (12 papers — contributes to modules 2, 3, 4, 5)

| Paper | Gap | What lithoSpore Needs |
|-------|-----|----------------------|
| B1 | LSTM time-series prediction | Mutation accumulation curve predictor |
| B2 | LSTM prediction + ESN regime detection | Fitness trajectory prediction, inflection detection |
| B3 | LSTM+HMM+ESN allele classification | Clade state detection, regime classification |
| B4 | Early warning ESN | Pre-citrate trajectory anomaly detection |
| B6 | ML burden prediction | Sequence-to-burden predictor (GC%, codon usage, promoter) |
| B7 | Parallel evolution ML | Transfer learning for gene-level convergence detection |
| B8 | Contingency loci prediction | Sequence feature → contingency locus classifier |
| B9 | DFE regime shifts | LSTM DFE parameter prediction, ESN regime shift detection |
| E2 | HOLIgraph GNN | Protein-ligand binding prediction (GPU accelerated) |
| E3 | Glycosylation ML | Glycosylation → binding affinity predictor |
| E4 | Macrocyclic ranking ML | Binder ranking from sequence features |
| E5 | Antibody pairing ML | VH/VL pairing prediction from single-cell data |

**Status**: All 12 papers QUEUED. neuralSpring has LSTM, ESN, HMM infrastructure
(Phase B AlphaFold2 primitives, compute.dispatch). Gap is domain-specific training
data and model tuning for LTEE-specific tasks.

### wetSpring (10 papers — contributes to modules 1, 6)

| Paper | Gap | What lithoSpore Needs |
|-------|-----|----------------------|
| B1 | Diversity metrics on OTU-like data | Shannon/Pielou calculator on genome-level data |
| B2 | Anderson-QS for biofilm | QS geometry predictions for LTEE biofilm populations |
| B3 | Clade diversity metrics | Pielou evenness of clade frequencies |
| B4 | Timeline analysis | Potentiating mutation cascade provenance tracking |
| B5 | Bee gut engineering | Anderson-QS geometry prediction for 3D biofilm |
| B6 | Burden diversity | Diversity metrics on burden value distribution |
| B7 | Sovereign genomics pipeline | Download 264 genomes from NCBI, mutation accumulation |
| B8 | HMM contingency loci | HMM identification across phage genomes |
| E1 | Sitewise diversity | Anderson disorder mapping per lattice site |
| E5 | scFab pipeline | Single-cell sequencing analysis (sovereign, Rust-based) |

**Status**: All 10 papers QUEUED. wetSpring has sovereign 16S pipeline, diversity
metrics (63 papers reproduced). Gap is applying existing pipeline to LTEE-specific
data formats (whole-genome rather than 16S amplicon).

### hotSpring (2 papers — contributes to module 7)

| Paper | Gap | What lithoSpore Needs |
|-------|-----|----------------------|
| B2 | Anderson disorder analogy | Map fitness landscape as disordered potential |
| B9 | DFE ↔ RMT connection | Eigenvalue distribution analysis of DFE |

**Status**: Both papers QUEUED. hotSpring has full Anderson localization
infrastructure (Papers 15-18 complete, all GPU primitives). Gap is applying
existing Anderson framework to biological fitness data (new domain application,
not new math).

### healthSpring (3 papers — future modules)

| Paper | Gap | What lithoSpore Needs |
|-------|-----|----------------------|
| B5 | Symbiont PK/PD | Pharmacokinetic modeling for engineered gut bacteria |
| E2 | OATP PK/PD bridge | Protein-ligand binding for drug transporters |
| E4 | Cyclic peptide screening | Macrocyclic peptide → ADDRC application |

**Status**: All 3 papers QUEUED. These don't map to current modules but may
seed future modules or extend existing ones.

### airSpring (1 paper — future module)

| Paper | Gap | What lithoSpore Needs |
|-------|-----|----------------------|
| E3 | FLS2 plant immunity | Glycosylation → receptor binding as environmental sensor |

**Status**: QUEUED. May seed a plant immunity sentinel module in a future version.

---

## Cross-Cutting Gaps

| Gap | Owner | Phase | Description |
|-----|-------|-------|-------------|
| Data fetching scripts | lithoSpore | **DONE (1–4, 6)** | `fetch_wiser_2013.sh`, `fetch_barrick_2009.sh`, `fetch_good_2017.sh`, `fetch_blount_2012.sh`, `fetch_tenaillon_2016.sh` |
| Expected values (modules 1–4) | Springs → lithoSpore | **DONE** | `module1_fitness.json` through `module4_citrate.json` ported from groundSpring B1–B4 |
| Expected values (modules 5–7) | Springs → lithoSpore | **PARTIAL** | Module 6+7 golden JSON exist; module 5 (biobricks) DOI pending |
| musl cross-compilation | lithoSpore | Interstadial | Need `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` targets |
| Python baseline implementations (1+2) | lithoSpore | **DONE** | `notebooks/module1_fitness/power_law_fitness.py` (8/8), `module2_mutations/mutation_accumulation.py` (7/7) |
| Python baseline implementations (3–7) | lithoSpore | Stadial | Remaining notebooks need analysis code |
| HTML pre-rendering | lithoSpore | Stadial | Convert Python notebooks to static HTML for zero-dep viewing |
| Foundation thread linkage validation | foundation | **INTERSTADIAL** | Verify thread 04/05/07 source TOMLs cover LTEE accessions |
| projectNUCLEUS workload integration | projectNUCLEUS | Interstadial | lithoSpore workloads in NUCLEUS dispatch (after Phase 2) |
| BioBrick paper DOI | External | Stadial | B6 DOI placeholder — update when Nat Comms finalizes |
| DFE paper DOI | External | Stadial | B9 DOI placeholder — update when Science finalizes |

---

## Foundation Thread Coverage

lithoSpore depends on `sporeGarden/foundation` threads for data anchoring and
provenance. Current coverage:

| Thread | Name | lithoSpore Relevance | Status |
|--------|------|---------------------|--------|
| 01 | Whole-Cell Modeling | Karr 2012 metabolic context for LTEE growth conditions | Active (ABG WCM) |
| 02 | Plasma Physics | Anderson disorder analogy (modules 5, 7) | Seeded (hotSpring) |
| 03 | Immunology | Not directly relevant | Not seeded |
| 04 | Environmental Genomics | LTEE genomic data (264 genomes, NCBI BioProject) | **CRITICAL** — needs LTEE accessions |
| 05 | Evolutionary Biology | LTEE paper anchoring (Barrick/Lenski corpus) | **CRITICAL** — needs sources/targets |
| 06 | Ecology & Environment | Population dynamics context | Active (airSpring) |
| 07 | Computational Science | Algorithm validation (stats/ML methods) | **HIGH** — needs LTEE benchmarks |
| 08 | Network Science | Population network topology | Not seeded |
| 09 | Material Science | Not directly relevant | Not seeded |
| 10 | Data Science | Data pipeline provenance | Not seeded |

**Interstadial target**: Threads 04 and 05 must have LTEE-specific source TOMLs
with NCBI/Dryad accessions. Thread 07 should have algorithm validation targets.

---

## Evolution Cycle: How Gaps Close

```
INTERSTADIAL (current):
  1. Springs work LTEE paper queue items — B1, B2 priority (L3)
  2. Springs produce: Python baselines + Rust validators + expected values
  3. lithoSpore absorbs: implement modules 1+2 with real data
  4. lithoSpore tests: cargo test + validate.sh + Python baselines
  5. Foundation: seed Threads 04+05 with LTEE accessions
  → EXIT GATE: 2+ modules PASS at Tier 1 (Python) with real data

STADIAL (next):
  6. Complete modules 3–7 as spring reproductions land
  7. Build: scripts/build-artifact.sh produces musl-static ecoBin binaries
  8. projectNUCLEUS dispatches: litho-validate-tier2.toml, tier3.toml
  9. External deployment: USB to Barrick Lab (Phase 5)
```

Modules 1, 2, 6, 7 are live and passing at Tier 2. The interstadial exit gate
is exceeded. Remaining gap is upstream neuralSpring reproductions for modules 3–5.

### Changelog

- **2026-05-13**: Modules 3+4 promoted from scaffold: groundSpring B3 (Good 2017 clonal
  interference) and B4 (Blount 2008/2012 citrate innovation) ingested. `fetch_good_2017.sh`,
  `fetch_blount_2012.sh`, `fetch_tenaillon_2016.sh` created. `ltee-alleles` and `ltee-citrate`
  Rust crates evolved from scaffold SKIP to live validation. `ltee-cli` updated to dispatch
  6 live modules (1–4, 6–7) with only module 5 (biobricks) as scaffold. `data.toml` updated.
  6/7 modules wired at Tier 2. Cross-cutting gap table updated.
- **2026-05-13**: Gap summary updated: 4/7 modules PASS Tier 2 (28/28 checks).
  Modules 6+7 marked RESOLVED (wetSpring B7, hotSpring B2). Gate status EXCEEDED.
- **2026-05-12**: Modules 6+7 integrated — wetSpring B7 Tenaillon (8/8 PASS),
  hotSpring B2 Anderson disorder (5/5 PASS). Pillar 4 gate exceeded (4/7 > 2+).
- **2026-05-11**: Modules 1+2 Tier 1 PASS — groundSpring B2/B1 integrated,
  fetch scripts created, Python baselines ported, expected values cross-validated.
  Rust crates updated to dispatch to Python Tier 1. `ltee-cli validate` now
  dispatches live modules. Interstadial exit gate (Pillar 4) MET.
