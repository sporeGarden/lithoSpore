<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# baseCamp — Python → Rust (uniBin) → Primal (NUCLEUS Composition)

**Date:** May 16, 2026
**Status:** 7/7 modules validated at Tier 2 (Rust). Python baselines complete.
           Tier 3 composition graph annotated, pending NUCLEUS runtime.

---

## What baseCamp Is

In ecoPrimals, baseCamp is where the technology meets the science domain.
For lithoSpore, the domain is **reproducible evolutionary biology** — so
baseCamp documents the three-tier validation pipeline that takes peer-reviewed
LTEE papers from Python notebooks through compiled Rust ecoBins to sovereign
primal composition:

- Which papers have been reproduced (the 7 science modules)
- How Python baselines become Rust implementations (the elevation path)
- How standalone Rust binaries compose into NUCLEUS workflows (the primal path)
- What the data provenance chain looks like at each tier

The pattern mirrors other ecoPrimals repos:
- hotSpring's baseCamp documents physics papers and what was reproduced
- wetSpring's baseCamp documents biology domains and what was validated
- rustChip's baseCamp documents model architectures and what runs on silicon
- **lithoSpore's baseCamp documents LTEE papers and the validation pipeline**

---

## The Three-Tier Pipeline

```
Tier 1 (Python)           Tier 2 (Rust/uniBin)         Tier 3 (Primal/NUCLEUS)
─────────────────────────────────────────────────────────────────────────────────

notebooks/module*/        crates/ltee-*/src/lib.rs      graphs/ltee_guidestone.toml
  *.py scripts              run_validation() → JSON       DAG: beardog → rhizocrypt
  numpy + scipy              litho-core::harness           → loamspine → sweetgrass
  matplotlib figs            litho-core::stats              → toadstool (optional)
  ↓                          litho-core::viz
  HTML reports               ↓                            workloads/litho-validate-
  visual inspection        JSON with PASS/FAIL              tier3.toml
                           tolerance matching               signals: nest.store,
validation/expected/       BLAKE3 provenance                        nest.commit
  golden JSON files        liveSpore.json append            ↓
                             ↓                            NUCLEUS composition runtime
                           musl-static ecoBin             provenance trio:
                           5.1 MB (x86_64)                  rhizoCrypt (DAG chain)
                           7.9 MB (Windows)                 loamSpine (lineage spine)
                           argv[0] symlink shims            sweetGrass (attribution)
                                                          biomeOS signal dispatch
```

---

## Paper → Module Mapping

| # | Paper | Python Baseline | Rust Module | Key Computation |
|---|-------|-----------------|-------------|-----------------|
| 1 | Wiser 2013 (Science) | `module1_fitness/` | `ltee-fitness` | Nelder-Mead curve fitting, AIC/BIC model selection |
| 2 | Barrick 2009 (Nature) | `module2_mutations/` | `ltee-mutations` | Kimura fixation, Poisson accumulation, Pearson molecular clock |
| 3 | Good 2017 (Nature) | `module3_alleles/` | `ltee-alleles` | Clonal interference dynamics, fixation probability |
| 4 | Blount 2008/2012 (PNAS/Nature) | `module4_citrate/` | `ltee-citrate` | Citrate innovation cascade, two-hit model |
| 5 | Burden 2024 (Nat Comms) | `module5_biobricks/` | `ltee-biobricks` | BioBrick metabolic burden validation |
| 6 | Tenaillon 2016 (Nature) | `module6_breseq/` | `ltee-breseq` | 264-genome comparison, parallel evolution significance |
| 7 | Anderson-QS (new) | `module7_anderson/` | `ltee-anderson` | Disorder mapping, GOE/Poisson eigenvalue statistics |

---

## Elevation Path: Python → Rust

Each module follows the same elevation pattern:

1. **Python baseline** (`notebooks/moduleN/`) reproduces the paper's
   key claims using numpy/scipy/matplotlib. Output is visual (plots)
   and numerical (JSON golden values saved to `validation/expected/`).

2. **Rust implementation** (`crates/ltee-{name}/src/lib.rs`) reimplements
   the computation in pure Rust using `litho-core` shared utilities:
   - `harness::skip_if_data_missing()` — graceful degradation
   - `harness::load_expected()` — golden value loading
   - `stats::pearson_r()` — correlation coefficient
   - `tolerance::check()` — named tolerance matching
   - `provenance::stamp()` — BLAKE3 data hash

3. **Validation** (`litho validate --json`) runs all modules in-process,
   compares Rust output against Python golden values within named
   tolerances defined in `artifact/tolerances.toml`.

4. **Tier 3 annotation** — workload and graph TOMLs declare primal
   dependencies and signal adoption for future NUCLEUS composition.

### What Changes Between Tiers

| Aspect | Tier 1 (Python) | Tier 2 (Rust) | Tier 3 (Primal) |
|--------|-----------------|---------------|-----------------|
| Runtime | Python 3.10+ | musl-static binary | NUCLEUS + primals |
| Dependencies | numpy, scipy, matplotlib | None (static linked) | biomeOS, primals |
| Provenance | None | liveSpore.json append | DAG chain + braid |
| Data integrity | Manual | BLAKE3 hash verification | nest.store signal |
| Network | Optional (fetch) | Optional (fetch) | TURN relay capable |
| Tolerance | Visual inspection | Named tolerances | Named + certified |
| Output | HTML + plots | Structured JSON | Signed JSON + braid |

---

## Primal Integration Architecture

lithoSpore is a **consumer** (verification chassis), not a primal (service
provider). It discovers primals at runtime via capability strings:

| Capability | Primal | Usage | Required? |
|------------|--------|-------|-----------|
| `visualization` | petalTongue | Dashboard rendering via IPC | No — degrades to JSON stdout |
| `discovery` | songBird | Primal discovery + TURN relay | No — degrades to standalone |
| `compute` | toadStool | GPU dispatch for accelerated validation | No — Tier 3 only |
| `storage` | nestGate | Persistent provenance storage | No — liveSpore.json local |
| `dag` | rhizoCrypt | DAG chain for provenance trio | No — Tier 3 only |
| `spine` | loamSpine | Lineage spine linking | No — Tier 3 only |
| `braid` | sweetGrass | Attribution braid integrity | No — Tier 3 only |

### Discovery Chain

```
1. Environment: $CAPABILITY_PORT (e.g. $VISUALIZATION_PORT=9500)
2. UDS socket:  $XDG_RUNTIME_DIR/ecoPrimals/discovery.sock
3. TURN relay:  $RELAY_SERVER (fallback: $SONGBIRD_TURN_SERVER)
4. Standalone:  No primals — graceful degradation
```

Every discovery result is recorded in `liveSpore.json` with
`discovery_path` (env/uds/turn/standalone) and `turn_relay` (if used).

---

## NUCLEUS Composition Pattern

When NUCLEUS is available, lithoSpore's Tier 3 graph
(`graphs/ltee_guidestone.toml`) composes the full provenance trio:

```toml
[graph.metadata]
signals = ["nest.store", "nest.commit"]

[[graph.nodes]]
name = "rhizocrypt"
by_capability = "dag"

[[graph.nodes]]
name = "loamspine"
by_capability = "spine"

[[graph.nodes]]
name = "sweetgrass"
by_capability = "braid"
```

**Signal adoption path**: When biomeOS supports signal dispatch routing,
the provenance phase (rhizoCrypt → loamSpine → sweetGrass) collapses to
a single `nest.store` dispatch. The graph and workload TOMLs are annotated
to enable this transition without code changes.

**Atomic instantiation via neuralAPI**: The deployment matrix cell
`lithospore-x86-vm-uds` validates the USB artifact in a VM with UDS
primal connectivity. biomeOS can instantiate this atomically — the
lithoSpore spore is a self-sufficient unit that composes into the
NUCLEUS topology without per-module wiring.

---

## Data Provenance at Each Tier

| Stage | Hash | Source | Persistence |
|-------|------|--------|-------------|
| Fetch | BLAKE3 of downloaded file | `artifact/data.toml` expected hash | `artifact/data/` |
| Verify | BLAKE3 of all bundled data | `litho verify` | CHECKSUMS file |
| Validate | Module output hash | `litho validate` → liveSpore.json | Append-only JSON |
| Compose | DAG chain hash | rhizoCrypt `dag.session.create` | Primal storage |
| Attest | Braid integrity hash | sweetGrass `braid.sign` | Attribution chain |

---

## Reading Order

**Starting from scratch:**
1. This README — the pipeline overview
2. `../../SCIENCE.md` — the narrative connecting all 7 modules
3. `../../papers/READING_ORDER.md` — the LTEE literature guide

**Understanding the Rust elevation:**
1. `../../specs/MODULES.md` — module contracts and interfaces
2. `../../docs/ARCHITECTURE.md` — crate diagram and data flow
3. Any `crates/ltee-*/src/lib.rs` — the actual validation code

**Understanding primal composition:**
1. `../../config/capability_registry.toml` — consumed capabilities
2. `../../graphs/ltee_guidestone.toml` — Tier 3 deploy graph
3. `../../docs/UPSTREAM_GAPS.md` — what's resolved, what's blocked

**Deployment and USB:**
1. `../../GETTING_STARTED.md` — quick start
2. `../../artifact/scope.toml` — scope graph (birth certificate)
3. `../../Containerfile` — OCI container alternative
