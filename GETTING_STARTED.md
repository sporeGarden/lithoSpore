# lithoSpore — Getting Started

## What Is This?

lithoSpore is a **Targeted GuideStone** — a portable, self-validating
scientific artifact that reproduces key results from the Long-Term
Evolution Experiment (LTEE) with *E. coli*. It targets the work of
Barrick, Lenski, and collaborators across 75,000+ generations of
continuous evolution.

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
│   ├── registry.toml     Machine-readable bibliography (18 papers)
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

## Extending

To add new LTEE papers or predictions:

1. Add a paper entry to `papers/registry.toml`
2. Add a dataset entry to `artifact/data.toml` (source URI + expected hash)
3. Add expected values to `validation/expected/`
4. Create a new module crate under `crates/` with `lib.rs` exposing `run_validation`
5. Add a module entry to `artifact/scope.toml` and wire into `LTEE_MODULES` in `crates/ltee-cli/src/validate.rs`
6. `litho fetch` will automatically handle data retrieval from the data.toml entry

## Contact

This artifact targets the Barrick Lab, UT Austin.
See `artifact/scope.toml` for the full provenance chain.
