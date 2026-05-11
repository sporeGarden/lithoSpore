# lithoSpore — Upstream & River Delta Gap Analysis

**Last Updated**: May 11, 2026 (scaffold)
**Purpose**: Track what lithoSpore needs from upstream springs and the river delta
to complete each module. This is the gap inventory that drives evolution cycles.

---

## Gap Summary

| Module | Ready | Blocked By | Severity |
|--------|-------|-----------|----------|
| 1. ltee-fitness | Scaffold | groundSpring B2, wetSpring B2 | Medium |
| 2. ltee-mutations | Scaffold | groundSpring B1, neuralSpring B1 | Medium |
| 3. ltee-alleles | Scaffold | neuralSpring B3, groundSpring B3 | Medium |
| 4. ltee-citrate | Scaffold | neuralSpring B4, groundSpring B4 | Medium |
| 5. ltee-biobricks | Scaffold | neuralSpring B6, groundSpring B6 | Medium |
| 6. ltee-breseq | Scaffold | wetSpring B7, groundSpring B7 | Medium |
| 7. ltee-anderson | Scaffold | hotSpring B2+B9, groundSpring B9 | Medium |

**All 7 modules are scaffold-only** — awaiting upstream spring paper queue
reproductions before implementation can begin.

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

| Gap | Owner | Description |
|-----|-------|-------------|
| Data fetching scripts | lithoSpore | `scripts/fetch_*.sh` for each dataset — need actual URLs and formats |
| Expected values | Springs → lithoSpore | Each module needs reference outputs from spring reproductions |
| musl cross-compilation | lithoSpore | Need `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` targets |
| Python baseline implementations | lithoSpore | 7 notebooks need actual analysis code (not just scaffold) |
| HTML pre-rendering | lithoSpore | Convert Python notebooks to static HTML for zero-dep viewing |
| Foundation thread linkage validation | foundation | Verify thread 04/07 source TOMLs cover LTEE accessions |
| projectNUCLEUS workload integration | projectNUCLEUS | Add lithoSpore workloads to NUCLEUS dispatch |
| BioBrick paper DOI | External | B6 DOI placeholder — update when Nat Comms finalizes |
| DFE paper DOI | External | B9 DOI placeholder — update when Science finalizes |

---

## Evolution Cycle: How Gaps Close

```
1. Springs work LTEE paper queue items (L3)
2. Springs produce: Python baselines + Rust validators + expected values
3. lithoSpore absorbs: copy expected values, implement module logic
4. lithoSpore tests: cargo test + validate.sh + Python baselines
5. Build: scripts/build-artifact.sh produces musl-static binaries
6. projectNUCLEUS dispatches: workloads/litho-validate-tier2.toml
7. External deployment: USB to Barrick Lab (Phase 5)
```

The scaffold is complete. The gap is upstream spring reproductions.
