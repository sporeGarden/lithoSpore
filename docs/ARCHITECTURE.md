# lithoSpore Architecture

## Targeted GuideStone Pattern

lithoSpore implements the Targeted GuideStone standard defined in
`ecoPrimals/infra/wateringHole/TARGETED_GUIDESTONE_STANDARD.md`. A Targeted GuideStone
is a **frozen composition snapshot** — binaries + data + validation, all
self-contained — that buds from the ecosystem into a portable artifact.

A guideStone-grade artifact satisfies five properties (per primals.eco):

| Property | lithoSpore Status |
|----------|-------------------|
| **1. Deterministic Output** | 7/7 modules produce identical results across USB, container, dev. Cross-arch (aarch64) validation pending. |
| **2. Reference-Traceable** | 16 papers in `papers/registry.toml`, all with DOIs. 14 validation targets (T01–T14). Expected JSONs carry `doi` + `source_figures`. |
| **3. Self-Verifying** | BLAKE3 hashes in `data.toml`, `litho verify`, `litho validate`, `litho self-test`. |
| **4. Environment-Agnostic** | Tier 2: pure Rust, musl-static, zero runtime dependencies. Tier 1: requires Python (bundled on USB). |
| **5. Tolerance-Documented** | `artifact/tolerances.toml` — named tolerances with scientific justification per module. |

## Five Components

1. **Scope Graph** (`artifact/scope.toml`) — which springs, primals, and
   foundation threads contribute. The artifact's birth certificate.

2. **Data Manifest** (`artifact/data.toml`) — every dataset with source URI,
   BLAKE3 hash, license, and refresh command.

3. **Binary Bundle** (`artifact/bin/{arch}/static/`) — musl-static ecoBin
   binaries. No containers. genomeBin handles platform detection.

4. **Validation Harness** (`validation/`, `artifact/validation/`) — expected
   values, named tolerances, structured JSON output.

5. **Provenance Chain** (`artifact/CHECKSUMS`, `graphs/ltee_guidestone.toml`)
   — BLAKE3 hashes for all files. Tier 3 adds rhizoCrypt DAG + loamSpine
   certificate + sweetGrass braid via JSON-RPC.

## Crate Architecture

```
litho-core          ← shared library (CHASSIS — 100% domain-agnostic, 11 modules)
  ├── validation/     ModuleResult, ValidationReport, Tier3Session, ParityReport
  ├── tolerance/      named tolerances with scientific justification
  ├── provenance/     ProvenanceChain + JSON-RPC client for trio (dag/spine/braid)
  ├── discovery/      capability-based primal resolution (env → UDS → TURN → standalone)
  ├── spore/          liveSpore tracking, BLAKE3 anchoring, hostname hashing
  ├── scope/          ScopeManifest + ScopeModule parser (scope.toml → module table)
  ├── braid/          upstream ferment transcript braid ingestion + validation
  ├── manifest/       DataManifest (data.toml → BLAKE3 verification)
  ├── stats/          shared statistics (pearson_r)
  ├── harness/        module skip/load/dispatch helpers
  └── graph_checks/   deploy graph validation (registry alignment, Dark Forest invariants)
  ↑
  ├── ltee-fitness   ← Module 1: power-law fitness (INSTANCE)
  ├── ltee-mutations ← Module 2: mutation accumulation
  ├── ltee-alleles   ← Module 3: allele trajectories
  ├── ltee-citrate   ← Module 4: citrate innovation
  ├── ltee-biobricks ← Module 5: BioBrick burden
  ├── ltee-breseq    ← Module 6: 264 genomes
  ├── ltee-anderson  ← Module 7: Anderson-QS predictions
  └── ltee-cli       ← Unified CLI (INSTANCE + CHASSIS GLUE)
      ├── main.rs         thin wiring (arg parse + argv[0] dispatch)
      ├── registry.rs     scope-driven module registry (replaces LTEE_MODULES constants)
      ├── validate.rs     litho validate — in-process module execution + Tier 3 branch
      ├── parity.rs       litho parity — cross-tier numerical parity check
      ├── verify.rs       litho verify — BLAKE3 integrity
      ├── fetch.rs        litho fetch — data pipeline (ureq + blake3)
      ├── assemble.rs     litho assemble — USB artifact assembly + .biomeos-spore generation
      ├── grow.rs         litho grow — self-bootstrap from USB
      ├── visualize.rs    litho visualize — petalTongue IPC
      ├── chaos.rs        litho chaos-test — 10 fault injection tests
      ├── deploy_test.rs  litho deploy-test — local deployment cycle
      ├── ops.rs          refresh / status / spore / self-test / deploy-report / tier
      └── viz/            petalTongue DataBinding adapters (INSTANCE — LTEE-specific)
          ├── modules.rs    m1–m7 LTEE module bindings
          └── baselines.rs  Barrick Lab baseline tool bindings
```

Each module crate exposes `lib.rs::run_validation()` for in-process execution.
The `ltee-cli` crate provides the unified `litho` binary that calls each
module's `run_validation()` directly (no subprocesses) and produces a
combined `ValidationReport`.

The `litho` binary also supports argv[0] symlink detection: when invoked as
`validate`, `verify`, `refresh`, `spore`, `parity`, or `grow`, it dispatches
to the corresponding subcommand without requiring explicit `litho <subcommand>` syntax.

## Chassis vs Instance

lithoSpore separates the domain-agnostic chassis from the LTEE instance.
`litho-core` is 100% chassis — zero LTEE-specific code. The LTEE is the
first instance; creating a new guideStone (e.g., Chuna, Bazavov) requires
only `scope.toml` + `data.toml` + module crates. No changes to `litho-core`.

| Layer | What | Current Files | Agnostic? |
|-------|------|---------------|-----------|
| **Chassis** | Validation pipeline, data integrity, provenance, discovery, deployment, pseudoSpore | `litho-core` (12 modules), `scope.toml`, `data.toml`, `tolerances.toml`, `liveSpore.json` | **Yes** |
| **Registry** | Scope-driven module resolution, dispatch, name mapping | `ltee-cli/registry.rs` — reads `[[module]]` from scope.toml, falls back to compiled LTEE defaults | **Yes** (data-driven from scope.toml) |
| **Instance** | Science modules, expected values, datasets, viz, papers | `crates/ltee-*`, `ltee-cli/viz/`, `validation/expected/`, `artifact/data/`, `papers/` | LTEE-specific |

### Chassis evolution roadmap

1. **DONE**: Scope-driven module registry. `[[module]]` entries in `scope.toml`
   carry name, binary, data_dir, expected, and tier1_notebook. All 6 consumer
   files (validate, parity, ops, chaos, deploy_test, visualize) import from
   `registry.rs`. `.biomeos-spore` generated from scope.toml during assembly.
   Braid accessions derived from `data.toml`. Target coverage and graph paths
   parameterized. viz/ moved from `litho-core` to `ltee-cli` instance layer.
   `litho-core` is 100% chassis.
2. **DONE**: pseudoSpore standard. `litho-core/src/pseudospore.rs` provides
   parsing, validation, and checksum verification for lightweight braid-first
   artifacts. `litho ingest-pseudospore` and `litho emit-pseudospore` CLI
   subcommands handle the full lifecycle. `pseudospores/registry.toml` tracks
   ingested artifacts. See `specs/PSEUDOSPORE_STANDARD.md`.
3. **Next**: Rename `ltee-cli` to `litho-cli`. Feature flags per instance.
   Dynamic module loading or plugin architecture.
4. **Target**: Any guideStone instance is a set of workspace member crates +
   `scope.toml` + `data.toml` + `papers/registry.toml`. `litho-core` and
   `litho-cli` are unchanged. `litho parity` and `litho validate` work
   against whatever modules `scope.toml` declares.

## pseudoSpore: Lightweight Braid-First Deployments

A pseudoSpore sits between LiveSpore and full lithoSpore in the spore taxonomy.
It carries braids, compute receipts, data outputs, and reproducibility configs —
but no runtime or raw input data. It proves a computation happened without
carrying the tools to re-execute it.

```
ColdSpore → LiveSpore → pseudoSpore → lithoSpore (full)
                          ↑
                     braids + receipts + outputs
                     (no binaries, no runtime, no raw data)
```

**Structure**: `scope.toml` + `validation.json` + `receipts/` + `provenance/` + optional `outputs/` and `configs/`.

**CLI**: `litho emit-pseudospore` (create) and `litho ingest-pseudospore` (consume).

**Promotion**: A pseudoSpore that gains Python + Rust implementations becomes a full lithoSpore module candidate.

**Use case**: Any computation-heavy spring producing quantitative results can ship a pseudoSpore instead of waiting for full lithoSpore module integration. The braid carries the provenance, the receipts carry the proof, the configs carry reproducibility.

**Chassis support**: `litho_core::pseudospore` (12th module in litho-core) — `PseudoSporeManifest`, `load_pseudospore()`, `verify_checksums()`, `check_completeness()`, `compute_checksums()`.

See `specs/PSEUDOSPORE_STANDARD.md` for the complete specification.

## Three-Tier Validation

| Tier | Runtime | What Runs | Provenance |
|------|---------|-----------|------------|
| **1 (Python)** | Python notebooks | Baseline scripts with numpy/scipy | — |
| **2 (Rust)** | musl-static binaries | In-process `run_validation()` | BLAKE3 on inputs/outputs |
| **3 (Primal)** | NUCLEUS composition | Tier 2 science + provenance trio | DAG + spine + braid via JSON-RPC |

### Cross-tier parity

`litho parity` runs both Tier 1 and Tier 2 for all modules and compares
results. If both tiers agree (same PASS/FAIL, compatible check counts),
the module is MATCH. Any disagreement is DIVERGENCE. This validates that
the math is stable between implementation languages.

### Tier 3 provenance

When `--max-tier 3`, after Tier 2 science:
1. `announce_self()` announces lithoSpore to biomeOS via `primal.announce`
2. `discover()` resolves rhizoCrypt (DAG), loamSpine (spine), sweetGrass (braid)
3. `try_record_tier3()` executes the provenance sequence:
   - `dag.session.create` → `dag.event.append` × N → `dag.session.complete`
   - `spine.create` → `entry.append` (validation summary)
   - `braid.create` (attribution record)
4. If trio unavailable, stays at Tier 2 with diagnostic

The 3-call sequence maps to `nest.store` — when biomeOS supports signal
dispatch, it collapses to `ctx.dispatch("nest.store", ...)`.

### Provenance directory (projectFOUNDATION Thread 10)

`litho validate --provenance-dir <dir>` writes:
- `results.json` — full `ValidationReport` including optional `Tier3Session`
- `provenance.toml` — summary metadata (artifact, version, timestamp, tier, counts)

## Discovery Chain

litho-core's `discovery.rs` implements the capability-based primal
discovery chain. The chain determines the operating mode:

```
$CAPABILITY_PORT env var → UDS discovery.sock → $RELAY_SERVER → None
         ↓                       ↓                    ↓             ↓
    DiscoveryPath::Env     DiscoveryPath::Uds   DiscoveryPath::Turn  ::Standalone
     (LAN mode)             (LAN mode)         (Geo-delocalized)   (Standalone)
```

`probe_operating_mode()` is called before validation and the result is
recorded in `liveSpore.json` as `discovery_path` + optional `turn_relay`.

`announce_self()` announces lithoSpore to biomeOS via `primal.announce`
with capabilities `["validation"]` and methods `["validate.run", "validate.parity", "validate.verify"]`.

`query_capabilities()` parses the Wave 20 canonical envelope
`{ "capabilities": [...], "count": N }`.

## Two-Tier Data Model

The spore ships **small** and validates **deep** when connected:

| Tier | What Ships | Size | Validates | Mode |
|------|-----------|------|-----------|------|
| **Summary** | Published parameters, expected values, summary statistics | ~KB per module | Against published claims within tolerance bands | Airgapped |
| **Complete** | Full dataset (e.g., BioBricks CSVs) | ~MB | Full analysis from raw data | Airgapped |
| **Full** (upstream) | Raw sequencing reads, complete archives | 10s–100s GB | Re-pipeline from raw reads, de novo mutation calling | Online |

Each dataset in `data.toml` declares its `data_tier` (`summary`, `complete`,
`internal`), its `full_data_size`, the `full_data_tool` needed to pull
upstream data, and the `full_data_checks` unlocked by having the full dataset.

```bash
# Summary mode (default) — uses shipped data, works airgapped
litho fetch --all

# Full mode — pulls raw upstream data (SRA reads, full archives)
litho fetch --all --full
```

When full data is present, future module evolution will run deeper checks
(e.g., breseq re-pipeline on 264 genomes instead of validating published
spectrum fractions).

### Upstream braid handoff

The two-tier data model has a natural extension: **upstream springs do
the heavy computation on NUCLEUS, then hand the braid to lithoSpore**.

The lithoSpore doesn't carry the mountain — it carries the receipt that
the mountain was climbed. Concretely:

```
wetSpring on NUCLEUS
    ↓ runs breseq on 200GB of raw reads (PRJNA294072)
    ↓ produces mutation calls, spectrum, accumulation curves
    ↓ records provenance: rhizoCrypt DAG + loamSpine cert + sweetGrass braid
    ↓
sweetGrass braid (portable, ~KB)
    ↓ handed off to lithoSpore
    ↓
lithoSpore
    ↓ ships the braid + summary statistics (~KB)
    ↓ validates summary against published claims (airgapped)
    ↓ braid proves the full computation was done upstream
    ↓ anyone can audit: follow the braid → spine → DAG → raw data
```

This is the **ferment transcript** pattern: the spring does the
fermentation (processing raw data into validated results), and
lithoSpore carries the transcript — the ingredients, the maps,
and the paths taken, allowing full audit and reproduction.

Each dataset in `data.toml` can reference upstream braids:

```toml
[[dataset]]
id = "barrick_2009_mutations"
data_tier = "summary"
upstream_braid = "braid-sovereign-barrick2009"
upstream_spring = "wetSpring"
upstream_dag_session = "dag-wetspring-sovereign-1779053983605"
upstream_braids = ["provenance/braids/barrick_2009_sovereign.json",
                   "provenance/braids/barrick_2009_breseq.json"]
```

The `braid` module in `litho-core` loads all `*.json` from `provenance/braids/`,
parses both sovereign (full `computation` block) and baseline (flat breseq)
wire formats, and validates SRA accessions automatically during `litho validate`.

When a braid is present, `litho verify` can validate the chain:
summary stats → braid → spine → DAG → raw data. The spore is
self-stable airgapped (validates from summary), but in a live
environment the provenance chain extends all the way back to the
raw reads on NUCLEUS.

This pattern is what makes lithoSpore a **novel use case for the
provenance trio**: the braid is not just an audit trail of
lithoSpore's own validation — it's a portable receipt from
upstream computation. The guideStone artifact becomes a **notarized
summary** of work done across the ecosystem.

## Data Flow

```
Foundation threads (4, 7, 2, 1)
    ↓ (source URIs, SRA accessions)
data/sources/ltee_barrick.toml
    ↓ (litho fetch — pure Rust, ureq + blake3)
    ↓ (litho fetch --full — SRA toolkit for raw reads)
artifact/data/{dataset}/
    ↓ (BLAKE3 hashed)
artifact/data.toml (manifest with data_tier + upstream_braid metadata)
    ↓
Module binaries (7x) — in-process via run_validation()
    ↓ (compare against expected/)
validation/expected/
    ↓ (within tolerance)
ValidationReport JSON
    ↓
liveSpore.json (append — discovery_path + turn_relay + hostname_hash)
    ↓ (Tier 3 only)
Provenance trio: rhizoCrypt DAG → loamSpine cert → sweetGrass braid
    ↓
sporePrint pipeline (primals.eco)

Upstream braid handoff (when available):

Springs on NUCLEUS (wetSpring, groundSpring, hotSpring)
    ↓ process raw data (10s–100s GB)
    ↓ record provenance via trio
    ↓
sweetGrass braid (~KB)
    ↓ handed off to lithoSpore
    ↓
lithoSpore data.toml (upstream_braid, upstream_dag_session)
    ↓ validates summary stats (airgapped)
    ↓ braid proves full upstream computation
    ↓ auditable: braid → spine → DAG → raw data
```

## Cross-Platform

| Platform | Tier 1 | Tier 2 | Tier 3 |
|----------|--------|--------|--------|
| Linux x86_64 | Python | musl-static (5.1 MB) | NUCLEUS |
| Linux aarch64 | Python | musl-static | NUCLEUS |
| Container (any OS) | Python | Containerfile | — |
| Windows x86_64 | Python | litho.exe (7.9 MB, mingw-w64) | — |
| macOS | Python | genomeBin | plasmidBin |

No containers in the artifact. Primals self-container via genomeBin if needed.
Cross-OS deployment via `Containerfile` + `litho grow --container`.

## Deployment Architecture (Hypogeal Cotyledon)

The USB artifact uses a 3-zone structure with a 5-layer cross-platform model:

```
Root (5-8 items max)
├── README.md          Layer 1 (Browse)
├── validate           Layer 2 (Launch) — shell shim for exFAT
├── ltee.bat           Layer 2 (Launch) — Windows entry
├── MANIFEST.toml      Machine-readable artifact map
├── science/           Layer 1 (Browse) + Layer 4 (Validate)
│   ├── index.html     Pre-rendered module gallery
│   ├── modules/       Per-module HTML + Python scripts
│   ├── artifact/      scope.toml, data.toml, data/
│   ├── provenance/    Upstream braids
│   └── validation/    Expected values
├── docs/              Full documentation
├── runtime/           Layer 3 (Execute)
│   ├── bin/litho      musl-static binary (6.3 MB)
│   ├── python/        CPython runtime
│   ├── container/     OCI image tar
│   ├── vm/            Cloud image
│   └── source/        Full source tree (seedable)
└── data/              Live project data (SRA reads, experiment results)
```

| Layer | What | Works On |
|-------|------|----------|
| 0 — Media | exFAT filesystem | Every OS mounts |
| 1 — Browse | HTML, docs, figures | Every OS reads |
| 2 — Launch | validate / ltee.bat | Platform shim |
| 3 — Execute | tmpdir + chmod (exFAT workaround) | Linux/macOS |
| 4 — Validate | Rust binary, 73 checks, <100ms | Full output |

The shim pattern handles exFAT's lack of Unix permissions:
`validate` copies `runtime/bin/litho` to `/tmp`, `chmod +x`, executes,
cleans up. Windows uses WSL2 or opens `science/index.html`.

## projectNUCLEUS Integration

lithoSpore is a projectNUCLEUS subsystem:
- `workloads/litho-validate-tier2.toml` — dispatched by NUCLEUS for Tier 2
- `workloads/litho-validate-tier3.toml` — dispatched via composition graph
- `graphs/ltee_guidestone.toml` — Tier 3 deploy graph (provenance trio)

The artifact can run independently (Tier 1/2) or under NUCLEUS (Tier 3).
