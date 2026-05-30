# lithoSpore

The ecosystem's first **Targeted GuideStone** — a self-contained, USB-deployable
verification chassis that reproduces published scientific results and proves its
own correctness. The LTEE (Long-Term Evolution Experiment) is the first instance;
the chassis is evolving toward domain-agnostic support for any body of science
with quantitative claims, source data, and expected values.

**Organization**: sporeGarden (products built on ecoPrimals)
**Subsystem of**: projectNUCLEUS
**First Instance**: Barrick Lab, MSU (LTEE — 82,500+ generations)
**License**: AGPL-3.0-or-later (code), CC-BY-SA 4.0 (docs)
**Standard**: `TARGETED_GUIDESTONE_STANDARD.md` (ecoPrimals/infra/wateringHole)
**Repo**: [github.com/sporeGarden/lithoSpore](https://github.com/sporeGarden/lithoSpore)

## Spore Taxonomy

lithoSpore is a **hypogeal cotyledon** — a self-sufficient spore that
carries its own food supply. The botanical metaphor: a seed leaf that
stays underground, nourishing the seedling until it can photosynthesize.

| Class | What | Self-Sufficient? | Size |
|-------|------|-----------------|------|
| ColdSpore | Static artifact, `.biomeos-spore` marker, frozen data | No (needs host) | < 1 KB |
| LiveSpore | + `liveSpore.json` provenance + `./refresh` self-update | Partially | ~ KB |
| **pseudoSpore** | + braids + compute receipts + data outputs + reproducibility configs | No (proves, doesn't execute) | KB–MB |
| **lithoSpore** (Hypogeal Cotyledon) | + Python runtime + 7 LTEE data bundles + litho-core Rust ecoBins | **Yes** | MB–GB |

**pseudoSpore** is a lightweight braid-first deployment for computation-heavy springs.
It proves the mountain was climbed and shows the view from the top — without carrying
the mountain. See `specs/PSEUDOSPORE_STANDARD.md` for the format specification and
`litho ingest-pseudospore` / `litho emit-pseudospore` for CLI tooling.

See `ecoPrimals/infra/wateringHole/LITHOSPORE_USB_DEPLOYMENT.md` for the full deployment standard.

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

| Tier | What Runs | Requirements | Provenance |
|------|-----------|-------------|------------|
| **1 (Python)** | Python analysis scripts with numpy/scipy | Python 3.10+ | — |
| **2 (Rust)** | musl-static ecoBin binaries — full validation at native speed | Linux x86_64 or aarch64 | BLAKE3 on inputs/outputs |
| **3 (Primal)** | Tier 2 science + NUCLEUS provenance trio | NUCLEUS running + plasmidBin | DAG + spine + braid via JSON-RPC |

`litho validate --max-tier 3` runs Tier 2 science then attempts provenance recording
via rhizoCrypt (DAG), loamSpine (certificates), sweetGrass (attribution braids).
Falls back to Tier 2 gracefully when primals are unavailable.

`litho parity` runs both Tier 1 and Tier 2 side-by-side for all modules and reports
MATCH/DIVERGENCE — proving the math is stable between implementation languages.

The deployed artifact requires no containers — ecoBin/genomeBin handles platform
detection via musl-static binaries. An optional OCI image (`Containerfile`) exists
for cross-OS deployment. Primals self-container via genomeBin if needed for Tier 3.

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
├── Cargo.toml                    # Workspace: 10 workspace crates
├── deny.toml                     # cargo-deny policy (licenses, advisories)
├── crates/
│   ├── pseudospore-core/         # pseudoSpore format: BLAKE3 manifest, braid envelope, validation
│   ├── litho-core/               # Shared library (chassis): validation, tolerance, provenance, discovery, stats, harness, braid, scope, error
│   ├── ltee-fitness/             # Module 1: power-law fitness (lib.rs + thin main.rs)
│   ├── ltee-mutations/           # Module 2: mutation accumulation
│   ├── ltee-alleles/             # Module 3: allele trajectories
│   ├── ltee-citrate/             # Module 4: citrate innovation
│   ├── ltee-biobricks/           # Module 5: BioBrick burden
│   ├── ltee-breseq/              # Module 6: 264-genome comparison
│   ├── ltee-anderson/            # Module 7: Anderson-QS predictions
│   └── ltee-cli/                 # Unified CLI: 21 subcommands + module registry + viz (instance layer)
│
├── artifact/                     # The deployable artifact
│   ├── usb-root/                 # USB root templates (.biomeos-spore, biomeOS/)
│   │   └── biomeOS/              # biomeOS tower.toml + validation graph
│   ├── scope.toml                # Scope graph (birth certificate)
│   ├── data.toml                 # Data manifest (source URIs + BLAKE3)
│   ├── tolerances.toml           # Named tolerances with justification
│   └── data/                     # Datasets (fetched, hashed)
│
├── data/
│   ├── sources/                  # Data source manifests (foundation pattern)
│   └── targets/                  # Validation targets (quantitative claims)
│
├── validation/expected/          # Reference outputs (7 modules)
├── notebooks/                    # Python Tier 1 baselines (7 modules)
├── graphs/                       # Tier 3 deploy graphs
├── workloads/                    # projectNUCLEUS workload TOMLs
├── config/                       # Capability registry
├── baselines/                    # Barrick Lab tool baseline reproductions
├── lineage/                      # Foundation thread linkage
├── papers/                       # Paper registry (16 DOIs) + READING_ORDER.md
├── figures/                      # Publication-quality SVG figures (7 modules)
├── profiles/                     # Domain profiles (e.g. md-metadynamics-carbohydrate.toml)
├── pseudospores/                 # Ingested pseudoSpore registry (lightweight braid-first artifacts)
├── provenance/braids/            # Upstream ferment transcript braids (from springs)
├── whitePaper/baseCamp/          # Python → Rust → Primal pipeline docs
├── experiments/                  # Experiment index (chronological log)
├── scripts/                      # Container build, VM cloud-init, Python wrapper
├── specs/                        # Specifications + ParityReport schema
└── docs/                         # Architecture, gap analysis, degradation behavior
```

## Building

```bash
# Build all modules (native)
cargo build --release

# Run validation (7/7 modules at Tier 2)
cargo run --bin litho -- validate --json
```

## Building the USB (Hypogeal Cotyledon)

```bash
# Assemble complete USB to ./usb-staging/ (fetches data, builds binaries, embeds Python)
cargo run --bin litho -- assemble

# Assemble to a mounted USB drive
cargo run --bin litho -- assemble --target /media/lithoSpore

# Skip steps for iterative development
cargo run --bin litho -- assemble --skip-python --skip-fetch --skip-build

# Preview what would be assembled
cargo run --bin litho -- assemble --dry-run
```

The assembled USB is a self-sufficient hypogeal cotyledon: plug it into
any Linux machine and run `./validate`. See `artifact/usb-root/` for
the root file templates.

## Current Status — 7/7 PASS, Live Deployment (May 18, 2026)

**Pillar 4 EXIT GATE: EXCEEDED** — 7 modules PASS at Tier 2, gate required 2+.
**First live handoff**: 4 USB drives deployed to Barrick Lab (MSU) for LTEE RA II interview. exFAT cross-platform, 3-zone structure, pre-rendered HTML browse layer.
**Deployment-validated**: USB pipeline tested on 4+ machines — 75/75 science checks, 199 unit/integration tests, 10 chaos/fault-injection tests.
**Tier 3**: Provenance trio wired via JSON-RPC with partial completion support (requires NUCLEUS primals at runtime).
**Cross-tier parity**: `litho parity` validates math stability between Python and Rust.
**Upstream braids**: 3 live braids from wetSpring (sovereign GPU pipeline + breseq baseline + Tenaillon expected values). Known: braid accession normalization (SRP001569 vs PRJNA29543) evolving upstream.

| Module | Status | Checks | Source |
|--------|--------|--------|--------|
| 1. ltee-fitness | **PASS** Tier 2 | 8/8 | groundSpring B2 Wiser 2013 |
| 2. ltee-mutations | **PASS** Tier 2 | 7/7 | groundSpring B1 Barrick 2009 |
| 3. ltee-alleles | **PASS** Tier 2 | 20/20 | groundSpring B3 Good 2017 |
| 4. ltee-citrate | **PASS** Tier 2 | 11/11 | groundSpring B4 Blount 2008/2012 |
| 5. ltee-biobricks | **PASS** Tier 2 | 6/6 | Burden 2024 BioBrick metabolic burden |
| 6. ltee-breseq | **PASS** Tier 2 | 16/16 | wetSpring B7 Tenaillon 2016 |
| 7. ltee-anderson | **PASS** Tier 2 | 7/7 | hotSpring B2 Anderson disorder |

**Tier 2 Rust implementations**:
- **Module 1**: Nelder-Mead curve fitting (power-law/hyperbolic/logarithmic) + AIC/BIC model selection
- **Module 2**: Kimura fixation probability, Poisson neutral accumulation, Pearson molecular clock
- **Module 3**: Clonal interference dynamics — fixation probability, interference ratio, adaptation rate validation
- **Module 4**: Citrate innovation cascade — Cit+ fraction, potentiation, replay probabilities, two-hit model
- **Module 6**: breseq 264-genome comparison, mutation accumulation analysis, parallel evolution significance
- **Module 7**: Anderson disorder mapping, GOE/Poisson eigenvalue statistics

**Infrastructure**: `litho-core` crate with 12 modules (chassis — domain-agnostic;
validation types including `Tier3Session`/`ParityReport`, provenance JSON-RPC client
for trio, braid ingestion with dual wire format support, tolerance framework, spore
tracking, capability-based discovery with `announce_self()`/`query_capabilities()`,
scope parser with `[[module]]` registry, data manifest, shared stats, env var
constants, harness). `pseudospore-core` is the canonical crate for pseudoSpore
parsing, validation, checksums, and domain profiles. `ltee-cli` adds instance layer: unified module registry (`registry.rs`),
viz DataBinding adapters, 21 subcommands. pseudoSpore pipeline: `emit-pseudospore`
(auto-figures, PDB serial extraction, BLAKE3 sealing), `ingest-pseudospore`,
`fetch-pseudospore` (remote download + validate + optional ingest),
`audit` (10-check pre-handoff validation — BLAKE3 integrity, config↔data, topology
cross-ref, derivation contract, version consistency, provenance, visual evidence),
`promote` (pseudoSpore → lithoSpore chassis with stripped binaries, auto RELEASE.md),
`translate-config` (domain↔computation index translation).
199 unit/integration tests + 10
chaos/fault-injection tests, zero clippy warnings, `#![forbid(unsafe_code)]`
workspace-wide, pure Rust BLAKE3 (ecoBin compliant), `liveSpore.json` operational
with corruption resilience and backup.

**Architecture** (May 14 geo-delocalization absorption):
- `unsafe_code = "forbid"` enforced at workspace lint level — all 10 crates inherit
- Shared harness (`litho_core::harness`) eliminates ~200 LOC of duplicated skip/load/dispatch
- Shared stats (`litho_core::stats`) deduplicates `pearson_r` with safe length checks
- Capability-based discovery (`litho_core::discovery`) — primal resolution via
  env → UDS → TURN relay → standalone. No hardcoded primal names; env vars are
  capability-generic (`$RELAY_SERVER`, `$VISUALIZATION_SOCKET`) with legacy fallback.
  `DiscoveryPath` + `turn_relay` recorded in `liveSpore.json` for provenance.
- `probe_operating_mode()` detects standalone/LAN/geo-delocalized before validation
- Clippy pedantic clean — scientific casts allowed; all other pedantic lints enforced
- `litho fetch` pure Rust data pipeline (ureq HTTP (**rustls** TLS) + BLAKE3 hashing, replaces 7 bash scripts)
- `litho assemble` pure Rust USB assembly (replaces assemble-usb.sh)
- `litho deploy-report` structured TOML output for deployment validation
- `litho self-test` artifact integrity check (23 checks)
- In-process module execution — all 7 modules call lib::run_validation directly, no subprocesses
- `litho verify` hardened: exits non-zero on MISSING, ERROR, and corrupt manifests
- Cross-platform: musl-static Linux (5.1 MB), Windows x86_64 (7.9 MB), argv[0] symlink detection

**Visualization Integration**: `ltee-cli::viz` (instance layer) provides `DataBinding`
adapters for all 7 LTEE modules and 7 Barrick Lab baseline tools via 9 generic builder
helpers. `litho visualize` pushes live dashboards to visualization primals via
capability-based IPC discovery.

**Chassis Abstraction**: Scope-driven module registry (`scope.toml` `[[module]]` entries).
`registry::load_modules()` reads scope.toml first; compiled `LTEE_MODULES`/`LTEE_NOTEBOOKS`/`MODULE_DISPATCH` serve as offline fallback.
Unified `registry.rs` serves validate, parity, ops, chaos, deploy-test, and visualize.
`.biomeos-spore` generated from scope.toml during assembly. Braid accession expectations
derived from `data.toml`. Target coverage and graph paths parameterized. `litho-core`
is domain-agnostic chassis — no LTEE science logic in source.

**Deployment testing** (3 paths):
- Local: `litho deploy-test` — filesystem isolation, ~1s
- Container: `agentReagents/scripts/validate-lithoSpore-container.sh` — Docker, airgap-capable
- VM: `agentReagents/scripts/validate-lithoSpore.sh` — libvirt, full airgap simulation
- Chaos: `litho chaos-test` — fault injection (10 tests: drift, corruption, missing files)

**Ecosystem posture** (Wave 21): primalSpring registry at 445 methods (stable),
delta spring tests 9,539+, all 8 springs at zero debt, wetSpring producing first
ferment transcript braids (Barrick 2009, 3/7 clones done). Method stability tiers
annotated in `config/capability_registry.toml`. Partial provenance supported
(DAG without braid is valid). `ParityReport` published as ecosystem standard
(`specs/PARITY_REPORT_SCHEMA.md`).

See `docs/UPSTREAM_GAPS.md` for upstream integration status.

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
