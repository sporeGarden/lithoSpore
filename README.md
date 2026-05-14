# lithoSpore

The ecosystem's first **Targeted GuideStone** — a self-contained, USB-deployable
artifact that reproduces Long-Term Evolution Experiment (LTEE) papers and generates
new predictions using the Anderson disorder framework.

**Organization**: sporeGarden (products built on ecoPrimals)
**Subsystem of**: projectNUCLEUS
**Target**: Barrick Lab, UT Austin (LTEE continuation)
**License**: AGPL-3.0-or-later (code), CC-BY-SA 4.0 (docs)
**Standard**: `TARGETED_GUIDESTONE_STANDARD.md` (wateringHole)

## Spore Taxonomy

lithoSpore is a **hypogeal cotyledon** — a self-sufficient spore that
carries its own food supply. The botanical metaphor: a seed leaf that
stays underground, nourishing the seedling until it can photosynthesize.

| Class | What | Self-Sufficient? |
|-------|------|-----------------|
| ColdSpore | Static artifact, `.biomeos-spore` marker, frozen data | No (needs host) |
| LiveSpore | + `liveSpore.json` provenance + `./refresh` self-update | Partially |
| **lithoSpore** (Hypogeal Cotyledon) | + Python runtime + 7 LTEE data bundles + litho-core Rust ecoBins | **Yes** |

See `wateringHole/LITHOSPORE_USB_DEPLOYMENT.md` for the full standard.

## What This Is

lithoSpore builds a portable validation artifact — a ~16GB USB that:

- Runs `./validate` on any Linux machine (no install, no internet, no dependencies)
- Runs Python notebooks on any machine with Python 3.10+
- Carries all data on the drive (BLAKE3-hashed, licensed, with source URIs for refresh)
- Produces structured JSON with PASS/FAIL per module and named tolerances
- Tracks its own deployment history via `liveSpore.json` (append-only, publishable)
- Can be extended via `./refresh` when internet is available
- Records operating mode in provenance (`standalone`, `env`, `uds`, `turn`)

## Three Operating Modes

| Mode | Network | Discovery | Tier |
|------|---------|-----------|------|
| **Standalone** | None | No primals — Python-only against bundled data | 1 |
| **LAN** | Local network | env vars / UDS socket — Rust + primal IPC | 2 |
| **Geo-delocalized** | Remote gate via cellMembrane | Songbird TURN relay — Tier 2 via relay | 2 |

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
│   ├── litho-core/               # Shared library: validation, tolerance, provenance, discovery, stats, harness
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
├── validation/                   # Validation harness (validate.sh) + scenarios
├── graphs/                       # Tier 3 deploy graphs
├── workloads/                    # projectNUCLEUS workload TOMLs
├── lineage/                      # Foundation thread linkage
├── scripts/                      # Build, fetch, and utility scripts
├── specs/                        # Specifications
└── docs/                         # Architecture + gap analysis
```

## Building

```bash
# Build all modules (native)
cargo build --release

# Build artifact (cross-compile musl-static)
./scripts/build-artifact.sh

# Run validation (6/7 modules LIVE at Tier 2, 1 scaffold SKIP)
cargo run --bin litho -- validate --json
```

## Current Status — 6/7 Modules LIVE (May 13, 2026)

**Pillar 4 EXIT GATE: EXCEEDED** — 6 modules wired at Tier 2, gate required 2+.

| Module | Status | Checks | Source |
|--------|--------|--------|--------|
| 1. ltee-fitness | **PASS** Tier 2 | 8/8 | groundSpring B2 Wiser 2013 |
| 2. ltee-mutations | **PASS** Tier 2 | 7/7 | groundSpring B1 Barrick 2009 |
| 3. ltee-alleles | **LIVE** Tier 2 | — | groundSpring B3 Good 2017 (ingested) |
| 4. ltee-citrate | **LIVE** Tier 2 | — | groundSpring B4 Blount 2008/2012 (ingested) |
| 5. ltee-biobricks | SKIP | — | DOI pending (Nat Comms) |
| 6. ltee-breseq | **PASS** Tier 2 | 8/8 | wetSpring B7 Tenaillon 2016 |
| 7. ltee-anderson | **PASS** Tier 2 | 5/5 | hotSpring B2 Anderson disorder |

**Tier 2 Rust implementations**:
- **Module 1**: Nelder-Mead curve fitting (power-law/hyperbolic/logarithmic) + AIC/BIC model selection
- **Module 2**: Kimura fixation probability, Poisson neutral accumulation, Pearson molecular clock
- **Module 3**: Clonal interference dynamics — fixation probability, interference ratio, adaptation rate validation
- **Module 4**: Citrate innovation cascade — Cit+ fraction, potentiation, replay probabilities, two-hit model
- **Module 6**: breseq 264-genome comparison, mutation accumulation analysis, parallel evolution significance
- **Module 7**: Anderson disorder mapping, GOE/Poisson eigenvalue statistics

**Infrastructure**: `litho-core` crate with 6 modules (validation, provenance, tolerance,
spore tracking, capability-based discovery, shared stats + harness), 33 unit tests,
CI wired, zero clippy warnings, `#![forbid(unsafe_code)]` workspace-wide,
pure Rust BLAKE3 (ecoBin compliant), `liveSpore.json` operational.

**Architecture** (May 14 geo-delocalization absorption):
- `unsafe_code = "forbid"` enforced at workspace lint level — all 9 crates inherit
- Shared harness (`litho_core::harness`) eliminates ~200 LOC of duplicated skip/load/dispatch
- Shared stats (`litho_core::stats`) deduplicates `pearson_r` with safe length checks
- Capability-based discovery (`litho_core::discovery`) — primal resolution via
  env → UDS → Songbird TURN → standalone. No hardcoded primal names.
  `DiscoveryPath` + `turn_relay` recorded in `liveSpore.json` for provenance.
- `probe_operating_mode()` detects standalone/LAN/geo-delocalized before validation
- Clippy pedantic clean — scientific casts allowed; all other pedantic lints enforced
- `cmd_refresh` real `data.toml`-driven fetch pipeline (5 fetch scripts: B1–B4, B7)

See `docs/UPSTREAM_GAPS.md` for remaining module gap (module 5 only).

## Upstream Dependencies

| Spring | Papers | Module(s) | Status |
|--------|--------|-----------|--------|
| groundSpring | B1-B4, B6-B9 | ALL 7 modules | **B1–B4 COMPLETE** |
| wetSpring | B1-B8, E1, E5 | fitness, breseq | **B7 INTEGRATED** (Module 6) |
| neuralSpring | B1-B4, B6-B9, E2-E5 | mutations, alleles, citrate, biobricks | B1 active, ML surrogates additive |
| hotSpring | B2, B9 | anderson | **B2 INTEGRATED** (Module 7) |
| healthSpring | B5, E2, E4 | (future) | Queued |
| airSpring | E3 | (future) | Queued |

## Foundation Thread Linkage

| Thread | Relevance | Modules |
|--------|-----------|---------|
| Thread 4 (Environmental Genomics) | LTEE metagenomic data, NCBI accessions | 1-4, 6 |
| Thread 7 (Anderson Framework) | Disorder analogy, DFE/RMT | 1, 7 |
| Thread 2 (Plasma/QCD) | RMT eigenvalue statistics | 7 |
| Thread 1 (Whole-Cell Modeling) | Metabolic context for citrate | 4 |
