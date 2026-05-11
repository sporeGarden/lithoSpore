# lithoSpore — Upstream & River Delta Gap Analysis

**Last Updated**: May 11, 2026 (interstadial exit tagging + foundation thread coverage)
**Purpose**: Track what lithoSpore needs from upstream springs and the river delta
to complete each module. This is the gap inventory that drives evolution cycles.
**Interstadial exit criteria**: `infra/wateringHole/INTERSTADIAL_EXIT_CRITERIA.md` (Pillar 4)

---

## Gap Summary

| Module | Ready | Blocked By | Severity | Phase |
|--------|-------|-----------|----------|-------|
| 1. ltee-fitness | Scaffold | groundSpring B2, wetSpring B2 | **BLOCKER** | Interstadial |
| 2. ltee-mutations | Scaffold | groundSpring B1, neuralSpring B1 | **BLOCKER** | Interstadial |
| 3. ltee-alleles | Scaffold | neuralSpring B3, groundSpring B3 | Medium | Interstadial |
| 4. ltee-citrate | Scaffold | neuralSpring B4, groundSpring B4 | Medium | Interstadial |
| 5. ltee-biobricks | Scaffold | neuralSpring B6, groundSpring B6 | Medium | Stadial |
| 6. ltee-breseq | Scaffold | wetSpring B7, groundSpring B7 | Medium | Stadial |
| 7. ltee-anderson | Scaffold | hotSpring B2+B9, groundSpring B9 | Medium | Stadial |

**All 7 modules are scaffold-only** — awaiting upstream spring paper queue
reproductions before implementation can begin.

**Interstadial exit gate (Pillar 4)**: At least 2 modules PASS at Tier 1
(Python) with real data fetched from Dryad/NCBI. Modules 1 + 2 are the
interstadial target (groundSpring is the critical path — contributes to all 7).
Modules 5–7 are stadial targets.

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

**Status**: All 8 papers QUEUED in groundSpring `specs/PAPER_REVIEW_QUEUE.md`.
groundSpring already has relevant infrastructure: jackknife (Paper 19),
drift vs selection (Paper 20), rare biosphere (Paper 21). The LTEE papers
extend these existing capabilities to new data.

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
| Data fetching scripts | lithoSpore | **INTERSTADIAL** | `scripts/fetch_*.sh` for each dataset — need actual URLs and formats |
| Expected values (modules 1+2) | Springs → lithoSpore | **INTERSTADIAL** | Modules 1+2 need reference outputs for exit gate |
| Expected values (modules 3–7) | Springs → lithoSpore | Stadial | Remaining modules need reference outputs |
| musl cross-compilation | lithoSpore | Interstadial | Need `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` targets |
| Python baseline implementations (1+2) | lithoSpore | **INTERSTADIAL** | Modules 1+2 need actual analysis code for exit gate |
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

The scaffold is complete. The gap is upstream spring reproductions.
