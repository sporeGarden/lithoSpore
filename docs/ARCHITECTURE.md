# lithoSpore Architecture

## Targeted GuideStone Pattern

lithoSpore implements the Targeted GuideStone standard defined in
`ecoPrimals/infra/wateringHole/TARGETED_GUIDESTONE_STANDARD.md`. A Targeted GuideStone
is a **frozen composition snapshot** — binaries + data + validation, all
self-contained — that buds from the ecosystem into a portable artifact.

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
   certificate + sweetGrass braid.

## Crate Architecture

```
litho-core          ← shared library
  ├── validation/     tolerance framework, named tolerances
  ├── provenance/     liveSpore tracking, BLAKE3 anchoring
  ├── discovery/      capability-based primal resolution (env → UDS → TURN → standalone)
  ├── stats/          shared statistics (pearson_r, etc.)
  ├── harness/        module skip/load/dispatch helpers
  └── viz/            petalTongue DataBinding adapters
      ├── modules.rs    m1–m7 LTEE module bindings
      └── baselines.rs  Barrick Lab baseline tool bindings
  ↑
  ├── ltee-fitness   ← Module 1: power-law fitness (groundSpring + wetSpring)
  ├── ltee-mutations ← Module 2: mutation accumulation (groundSpring + neuralSpring)
  ├── ltee-alleles   ← Module 3: allele trajectories (neuralSpring + groundSpring)
  ├── ltee-citrate   ← Module 4: citrate innovation (neuralSpring + groundSpring)
  ├── ltee-biobricks ← Module 5: BioBrick burden (neuralSpring + groundSpring)
  ├── ltee-breseq    ← Module 6: 264 genomes (wetSpring + groundSpring)
  ├── ltee-anderson  ← Module 7: Anderson-QS predictions (hotSpring + groundSpring)
  └── ltee-cli       ← Unified CLI (13 subcommands)
      ├── main.rs         thin wiring (arg parse + argv[0] dispatch)
      ├── validate.rs     litho validate — in-process module execution
      ├── verify.rs       litho verify — BLAKE3 integrity
      ├── fetch.rs        litho fetch — data pipeline (ureq + blake3)
      ├── assemble.rs     litho assemble — USB artifact assembly
      ├── visualize.rs    litho visualize — petalTongue IPC
      ├── chaos.rs        litho chaos-test — 10 fault injection tests
      ├── deploy_test.rs  litho deploy-test — local deployment cycle
      └── ops.rs          refresh / status / spore / self-test / deploy-report / tier
```

Each module crate exposes `lib.rs::run_validation()` for in-process execution.
The `ltee-cli` crate provides the unified `litho` binary that calls each
module's `run_validation()` directly (no subprocesses) and produces a
combined `ValidationReport`.

The `litho` binary also supports argv[0] symlink detection: when invoked as
`validate`, `verify`, `refresh`, or `spore`, it dispatches to the corresponding
subcommand without requiring explicit `litho <subcommand>` syntax.

## Discovery Chain

litho-core's `discovery.rs` implements the capability-based primal
discovery chain. The chain determines the operating mode:

```
$CAPABILITY_PORT env var → UDS discovery.sock → $SONGBIRD_TURN_SERVER → None
         ↓                       ↓                       ↓                ↓
    DiscoveryPath::Env     DiscoveryPath::Uds    DiscoveryPath::Turn   ::Standalone
     (LAN mode)             (LAN mode)         (Geo-delocalized)    (Standalone)
```

`probe_operating_mode()` is called before validation and the result is
recorded in `liveSpore.json` as `discovery_path` + optional `turn_relay`.

## Data Flow

```
Foundation threads (4, 7, 2, 1)
    ↓ (source URIs, accessions)
data/sources/ltee_barrick.toml
    ↓ (litho fetch — pure Rust, ureq + blake3)
artifact/data/{dataset}/
    ↓ (BLAKE3 hashed)
artifact/data.toml (manifest)
    ↓
Module binaries (7x)
    ↓ (compare against expected/)
artifact/validation/expected/
    ↓ (within tolerance)
ValidationReport JSON
    ↓
liveSpore.json (append — discovery_path + turn_relay + hostname_hash)
    ↓
sporePrint pipeline (primals.eco)
```

## Cross-Platform

The `litho` binary (via argv[0] symlink detection) supports multiple platforms:

| Platform | Tier 1 | Tier 2 | Tier 3 |
|----------|--------|--------|--------|
| Linux x86_64 | Python | musl-static (5.1 MB) | NUCLEUS |
| Linux aarch64 | Python | musl-static | NUCLEUS |
| Windows x86_64 | Python | litho.exe (7.9 MB, mingw-w64) | — |
| macOS | Python | genomeBin | plasmidBin |

No containers in the artifact. Primals self-container via genomeBin if needed.

## projectNUCLEUS Integration

lithoSpore is a projectNUCLEUS subsystem:
- `workloads/litho-validate-tier2.toml` — dispatched by NUCLEUS for Tier 2
- `workloads/litho-validate-tier3.toml` — dispatched via composition graph
- `graphs/ltee_guidestone.toml` — Tier 3 deploy graph (provenance trio)

The artifact can run independently (Tier 1/2) or under NUCLEUS (Tier 3).
