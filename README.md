# lithoSpore

The ecosystem's first **Targeted GuideStone** — a self-contained, USB-deployable
artifact that reproduces Long-Term Evolution Experiment (LTEE) papers and generates
new predictions using the Anderson disorder framework.

**Organization**: sporeGarden (products built on ecoPrimals)
**Subsystem of**: projectNUCLEUS
**Target**: Barrick Lab, UT Austin (LTEE continuation)
**License**: AGPL-3.0-or-later (code), CC-BY-SA 4.0 (docs)
**Standard**: `TARGETED_GUIDESTONE_STANDARD.md` (wateringHole)

## What This Is

lithoSpore builds a portable validation artifact — a ~3GB USB drive that:

- Runs `./ltee validate` on any Linux machine (no install, no internet, no dependencies)
- Runs Python notebooks on any machine with Python 3.10+
- Carries all data on the drive (BLAKE3-hashed, licensed, with source URIs for refresh)
- Produces structured JSON with PASS/FAIL per module and named tolerances
- Tracks its own deployment history via `liveSpore.json`
- Can be extended via `./ltee refresh` when internet is available

## Three-Tier Architecture

| Tier | What Runs | Requirements |
|------|-----------|-------------|
| **1 (Python)** | Pre-rendered HTML notebooks, Python analysis scripts | Python 3.10+ (or browser for HTML) |
| **2 (Rust)** | musl-static ecoBin binaries — full validation at native speed | Linux x86_64 or aarch64 |
| **3 (Primal)** | NUCLEUS composition with provenance trio | NUCLEUS running + plasmidBin |

No containers. ecoBin/genomeBin handles platform detection. Primals self-container
via genomeBin if needed for Tier 3.

## Seven Science Modules

| # | Module | Paper | Spring Sources |
|---|--------|-------|----------------|
| 1 | `ltee-fitness` | Wiser 2013 (B2) — power-law fitness | groundSpring, wetSpring |
| 2 | `ltee-mutations` | Barrick 2009 (B1) — mutation accumulation | groundSpring, neuralSpring |
| 3 | `ltee-alleles` | Good 2017 (B3) — allele trajectories | neuralSpring, groundSpring |
| 4 | `ltee-citrate` | Blount 2008/2012 (B4) — citrate innovation | neuralSpring, groundSpring |
| 5 | `ltee-biobricks` | Burden 2024 (B6) — BioBrick burden | neuralSpring, groundSpring |
| 6 | `ltee-breseq` | Tenaillon 2016 (B7) — 264 genomes | wetSpring, groundSpring |
| 7 | `ltee-anderson` | Anderson-QS (new) — disorder predictions | hotSpring, groundSpring |

## Repository Structure

```
lithoSpore/
├── Cargo.toml                    # Workspace: 7 modules + core + CLI
├── crates/
│   ├── litho-core/               # Shared types: validation, tolerance, provenance, liveSpore
│   ├── ltee-fitness/             # Module 1: power-law fitness
│   ├── ltee-mutations/           # Module 2: mutation accumulation
│   ├── ltee-alleles/             # Module 3: allele trajectories
│   ├── ltee-citrate/             # Module 4: citrate innovation
│   ├── ltee-biobricks/           # Module 5: BioBrick burden
│   ├── ltee-breseq/              # Module 6: 264-genome comparison
│   ├── ltee-anderson/            # Module 7: Anderson-QS predictions
│   └── ltee-cli/                 # Unified CLI: validate/refresh/status/spore
│
├── artifact/                     # The deployable artifact (USB layout)
│   ├── ltee                      # Entry point script
│   ├── scope.toml                # Scope graph (birth certificate)
│   ├── data.toml                 # Data manifest (source URIs + BLAKE3)
│   ├── tolerances.toml           # Named tolerances with justification
│   ├── liveSpore.json            # Deployment tracking (append-only)
│   ├── bin/{arch}/static/        # musl-static ecoBin binaries
│   ├── data/                     # Datasets (fetched, hashed)
│   ├── notebooks/html/           # Pre-rendered HTML notebooks
│   ├── validation/expected/      # Reference outputs
│   └── deploy/                   # Tier 3 deploy graphs
│
├── data/
│   ├── sources/                  # Data source manifests (foundation pattern)
│   └── targets/                  # Validation targets (quantitative claims)
│
├── notebooks/                    # Python Tier 1 baselines (7 modules)
├── validation/                   # Validation harness + scenarios
├── graphs/                       # Tier 3 deploy graphs
├── workloads/                    # projectNUCLEUS workload TOMLs
├── lineage/                      # Foundation thread linkage
├── deploy/                       # Deployment scripts
├── scripts/                      # Build + utility scripts
├── specs/                        # Specifications
└── docs/                         # Architecture + gap analysis
```

## Building

```bash
# Build all modules (native)
cargo build --release

# Build artifact (cross-compile musl-static)
./scripts/build-artifact.sh

# Run validation (scaffold mode — modules report SKIP until spring reproductions land)
cargo run --bin litho -- validate --json
```

## Current Status

**Phase 1: Architecture + Queue Seeding** — COMPLETE

- Cargo workspace scaffolded with 7 module crates + shared lib + CLI
- Artifact structure defined (scope graph, data manifest, tolerances, liveSpore)
- Python Tier 1 baselines scaffolded for all 7 modules
- Data source manifests and validation targets defined
- Foundation thread linkage established
- 36 paper-spring assignments seeded across 6 upstream springs

**Phase 2: Spring Reproductions** — AWAITING UPSTREAM

All 7 modules currently report SKIP — they are scaffolded and wired to the
validation harness but await upstream spring teams to complete their LTEE
paper queue items. See `docs/UPSTREAM_GAPS.md` for the full gap analysis.

## Upstream Dependencies

| Spring | Papers | Module(s) | Status |
|--------|--------|-----------|--------|
| wetSpring | B1-B8, E1, E5 | fitness, breseq | Queued |
| neuralSpring | B1-B4, B6-B9, E2-E5 | mutations, alleles, citrate, biobricks | Queued |
| groundSpring | B1-B4, B6-B9 | ALL 7 modules | Queued |
| hotSpring | B2, B9 | anderson | Queued |
| healthSpring | B5, E2, E4 | (future) | Queued |
| airSpring | E3 | (future) | Queued |

## Foundation Thread Linkage

| Thread | Relevance | Modules |
|--------|-----------|---------|
| Thread 4 (Environmental Genomics) | LTEE metagenomic data, NCBI accessions | 1-4, 6 |
| Thread 7 (Anderson Framework) | Disorder analogy, DFE/RMT | 1, 7 |
| Thread 2 (Plasma/QCD) | RMT eigenvalue statistics | 7 |
| Thread 1 (Whole-Cell Modeling) | Metabolic context for citrate | 4 |
