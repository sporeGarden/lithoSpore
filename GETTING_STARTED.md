# lithoSpore — Getting Started

## What Is This?

lithoSpore is a **verification chassis** — a reusable pattern for building
portable, self-validating scientific artifacts. The chassis handles data
integrity (BLAKE3), multi-tier validation (Python/Rust/Primal), provenance
tracking, and USB deployment. You supply the science.

**This instance** targets the Long-Term Evolution Experiment (LTEE) with
*E. coli* — the work of Barrick, Lenski, and collaborators across 75,000+
generations of continuous evolution. It is the first **Targeted GuideStone**
built on the lithoSpore chassis. The same infrastructure would work for
any body of science with quantitative claims, source data, and expected values.

### Chassis vs Instance

Everything in this artifact separates cleanly into two layers:

| Layer | What | Files |
|-------|------|-------|
| **Chassis** (universal) | Validation pipeline, data integrity, provenance, deployment | `bin/litho`, `litho-core`, `liveSpore.json`, `data_manifest.toml` |
| **Instance** (LTEE-specific) | 7 science modules, expected values, datasets, tolerances | `crates/ltee-*`, `validation/expected/`, `artifact/data/`, `papers/` |

To build a lithoSpore for **your** science: keep the chassis, replace the instance.
See **Building Your Own lithoSpore** at the end of this document.

## Quick Start

```bash
# Validate all 7 science modules (USB)
./validate

# Or directly via cargo (development)
cargo run --bin litho -- validate --json

# Verify data integrity (BLAKE3 checksums)
./verify

# Refresh data from upstream sources
./refresh

# Check what tier is achievable on this machine
./spore tier
```

## Three Tiers of Validation

| Tier | Runtime | What It Does |
|------|---------|-------------|
| **1** | Python | Runs 7 baseline scripts with numpy/scipy/matplotlib |
| **2** | Rust | Runs 7 compiled modules — real computation, no dependencies |
| **3** | Primals | NUCLEUS composition — DAG provenance, certificates, braids |

The USB artifact attempts Tier 2 first (precompiled static binaries).
If binaries are unavailable, it falls back to Tier 1 (Python).
Tier 3 requires a projectNUCLEUS deployment.

## What's Inside

```
├── validate              Run all 7 modules
├── verify                BLAKE3 data integrity check
├── refresh               Update data from upstream
├── spore                 biomeOS integration hook (symlink → bin/litho)
│
├── papers/               Scholarly foundation
│   ├── registry.toml     Machine-readable bibliography (16 papers)
│   └── READING_ORDER.md  Guided reading path through the LTEE literature
│
├── bin/                  Compiled Rust binaries (musl-static)
│   └── litho             Unified CLI (all 7 modules via in-process dispatch)
│
├── artifact/
│   ├── data/             Fetched datasets (7 LTEE sources)
│   ├── scope.toml        What springs/primals contribute
│   ├── data.toml         Dataset manifest with BLAKE3 hashes
│   └── tolerances.toml   Named tolerances with justifications
│
├── validation/
│   └── expected/         Expected values (7 JSON files)
│
├── figures/              Pre-rendered SVG scientific figures
│   └── m1–m7_*.svg
│
├── notebooks/            Python baselines (Tier 1)
│   └── module1–7/*.py
│
├── liveSpore.json        Deployment/validation history
└── data_manifest.toml    BLAKE3 inventory of all bundled data
```

## The Seven Science Modules

1. **Power-law fitness** — Wiser et al. 2013, Science
2. **Mutation accumulation** — Barrick et al. 2009, Nature
3. **Allele trajectories** — Good et al. 2017, Nature
4. **Citrate innovation** — Blount et al. 2008/2012, PNAS/Nature
5. **BioBrick burden** — Barrick et al. 2024, Nature Communications
6. **264-genome evolution** — Tenaillon et al. 2016, Nature
7. **Anderson-QS predictions** — Anderson disorder analogy (new)

See `papers/READING_ORDER.md` for the full reading guide.

## Validation Targets

14 quantitative targets (T01–T14) map published claims to tolerance
bands. Run `litho validate --json` to see target coverage.

## Running on Any OS (Docker/Podman)

This artifact includes a `Containerfile` for cross-OS deployment.
On **any system** with Docker or Podman — Linux, macOS, or Windows:

```bash
# Build the OCI image from the USB root
docker build -f Containerfile -t litho-spore .

# Run full validation (Tier 1 + 2)
docker run litho-spore

# Run with airgap simulation (no network)
docker run --network=none litho-spore

# Interactive exploration
docker run -it --entrypoint /bin/bash litho-spore
```

Or use the integrated command:

```bash
./grow --container
```

This builds the image and runs validation automatically — no Rust
toolchain, no clone, no system dependencies. The musl-static binaries
and Python stack run inside the container.

## Growing Into a Full Development Environment

This artifact carries its own source code metadata. On any Linux machine
with internet:

```bash
# Germinate: clone source, install Rust, build, fetch data, validate
./grow --target ~/Development/lithoSpore

# Also clone the full ecoPrimals ecosystem
./grow --target ~/Development/lithoSpore --ecosystem

# Provision a benchScale VM for isolated validation
./grow --target ~/Development/lithoSpore --vm
```

## Data: Ship Small, Validate Deep

The spore ships summary statistics (~3.4 MB total) — enough to validate all
75 science checks airgapped. When connected to the internet, it can pull
full upstream datasets and re-validate at deeper granularity:

```bash
# Default: uses shipped summary data (works airgapped)
litho fetch --all

# Full: pulls raw upstream data (SRA reads, complete archives)
# Requires SRA toolkit for genomic datasets. Can be 10s–100s of GB.
litho fetch --all --full
```

| Dataset | Shipped | Full Upstream | Additional Checks |
|---------|---------|---------------|-------------------|
| Wiser 2013 | 12-row fitness CSV | ~5 MB Dryad archive | Per-replicate fits, jackknife |
| Barrick 2009 | Published parameters | ~15 GB (19 genomes) | breseq re-pipeline |
| Good 2017 | Simulation tallies | ~50 GB (metagenomic) | Allele frequency time-series |
| Blount 2012 | Replay summary | ~30 GB (replay seq) | Potentiation mutation ID |
| BioBricks 2024 | **Complete** (3.3 MB) | Same | All checks run on shipped data |
| Tenaillon 2016 | Published stats | ~200 GB (264 genomes) | Full breseq re-pipeline |
| Anderson-QS | Internal predictions | n/a | Requires Module 1 full data |

Every dataset carries its BLAKE3 hash, SRA accession (where applicable),
and the `full_data_checks` field in `data.toml` describing what deeper
analysis becomes possible with full data.

Datasets processed by upstream springs carry provenance braids — the
`upstream_spring`, `upstream_braid`, and `upstream_dag_session` fields in
`data.toml` record the computation chain. See `docs/ARCHITECTURE.md` for
the ferment transcript pattern.

## Adding Modules to This Instance

To add new LTEE papers or predictions:

1. Add a paper entry to `papers/registry.toml`
2. Add a dataset entry to `artifact/data.toml` (source URI + expected hash)
3. Add expected values to `validation/expected/`
4. Create a new module crate under `crates/` with `lib.rs` exposing `run_validation`
5. Add a module entry to `artifact/scope.toml` and wire into `LTEE_MODULES` in `crates/ltee-cli/src/validate.rs`
6. `litho fetch` will automatically handle data retrieval from the data.toml entry

## Building Your Own lithoSpore

The LTEE is just the first instance. The chassis works for any science:

**What you need:**
- Quantitative claims with tolerance bands (your `validation_targets.toml`)
- Source data with URIs and integrity hashes (your `data.toml`)
- Expected values from published results (your `validation/expected/*.json`)
- Computation that validates claims against data (your module crates)

**What the chassis gives you for free:**
- `litho validate` — scope-driven module dispatch with structured PASS/FAIL output
- `litho verify` — BLAKE3 data integrity verification
- `litho fetch` — HTTP/SRA data retrieval with hash verification
- `litho assemble` — USB artifact assembly with embedded Python runtime
- `litho grow` — self-bootstrap from USB to full development environment
- `liveSpore.json` — append-only deployment provenance trail
- Three-tier architecture — Python baseline / Rust native / Primal composition
- `scope.toml` — birth certificate declaring what contributes to this artifact
- `tolerances.toml` — named tolerances with scientific justification
- `data_manifest.toml` — BLAKE3 inventory of all bundled data

**The pattern:**
```
your-guidestone/
├── artifact/
│   ├── scope.toml        # YOUR springs, primals, foundation threads
│   ├── data.toml          # YOUR datasets with source URIs
│   └── tolerances.toml    # YOUR tolerances with justifications
├── validation/expected/   # YOUR expected values (JSON)
├── crates/
│   ├── litho-core/        # CHASSIS (unchanged)
│   └── your-module-*/     # YOUR science modules
└── papers/registry.toml   # YOUR bibliography
```

The math is real. The infrastructure is universal. The deployment is sovereign.

## Contact

This artifact targets the Barrick Lab, UT Austin.
See `artifact/scope.toml` for the full provenance chain.
