<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Experiments Index

Chronological log of lithoSpore experiments. Each experiment has a unique
numeric prefix for ordering and a `_TOPIC_DESCRIPTOR` suffix.

Experiment narratives are documented **inline in this file** (see
[Experiment Log](#experiment-log-may-2026) below), not as separate
per-experiment markdown files under `experiments/`.

## Naming Convention

```
NNN_DESCRIPTOR.{rs,md,json,toml}
```

- `rs` — runnable experiment code (standalone or cargo test)
- `md` — analysis write-ups, validation investigations
- `json` — benchmark results and structured output
- `toml` — experiment configuration

## LTEE Validation Experiments

| # | Name | Type | Domain | Status |
|---|------|------|--------|--------|
| 001 | Python baseline establishment | md | All 7 modules | Complete |
| 002 | Rust Tier 2 elevation | md | All 7 modules | Complete — 75/75 checks |
| 003 | Tolerance calibration | md | Cross-module | Complete — tolerances.toml finalized |
| 004 | USB deployment validation | md | Infrastructure | Complete — VM + container + local |
| 005 | Chaos/fault injection | md | Infrastructure | Complete — 10 scenarios |
| 006 | Cross-platform binary validation | md | Infrastructure | Complete — x86_64, aarch64, Windows |
| 007 | Scope-driven chassis abstraction | md | Architecture | Complete — scope.toml + data.toml |

| 008 | Cross-tier parity validation | md | Validation | Complete — `litho parity` all 7 modules |
| 009 | Tier 3 NUCLEUS provenance wiring | md | Primal integration | Complete — JSON-RPC trio client |
| 010 | Two-tier data model | md | Data strategy | Complete — `litho fetch --full`, upstream braids |
| 011 | Chassis abstraction evolution | md | Architecture | Complete — scope-driven registry, litho-core domain-agnostic chassis |

## Planned Experiments

| # | Name | Type | Domain | Blocked By |
|---|------|------|--------|------------|
| 013 | BLAKE3 hash backfill | toml | Data integrity | Network access to source repos |
| 014 | neuralSpring ML surrogate integration | md | Modules 3, 4 | neuralSpring B3/B4 models |
| 015 | Signal dispatch collapse | md | Architecture | biomeOS signal routing |
| 016 | FIDO2 hardware attestation | md | Security | BearDog CTAP2 library |
| 017 | Upstream braid handoff (wetSpring) | md | Provenance | wetSpring breseq pipeline |

## Experiment Log (May 2026)

### 001 — Python Baseline Establishment (May 11)

Ported 7 LTEE paper analyses from literature to Python scripts using
numpy, scipy, and matplotlib. Each module produces golden JSON values
saved to `validation/expected/`. Visual outputs (SVG figures) saved to
`figures/`.

### 002 — Rust Tier 2 Elevation (May 11–13)

Reimplemented all 7 modules in pure Rust. Each module's `lib.rs` exports
`run_validation(data_path, expected_path) -> Result<Value>`. Shared
utilities extracted to `litho-core` (harness, stats, tolerance, provenance).
Final validation: 75/75 checks passing against Python golden values.

### 003 — Tolerance Calibration (May 13)

Named tolerances in `artifact/tolerances.toml` calibrated against
numerical precision differences between Python (f64 via numpy C extensions)
and Rust (f64 native). Key finding: power-law exponent β requires
±0.01 tolerance due to Nelder-Mead optimizer path sensitivity.

### 004 — USB Deployment Validation (May 14)

Three deployment paths validated via agentReagents:
- **Local**: `litho deploy-test` — filesystem isolation, ~1s
- **Container**: OCI image, airgap-capable, Podman + Docker
- **VM**: libvirt with USB passthrough, full airgap simulation

### 005 — Chaos/Fault Injection (May 14)

10 fault injection scenarios via `litho chaos-test`:
- Data file corruption (bit flip, truncation, deletion)
- Expected value drift (tolerance boundary testing)
- Manifest corruption (TOML parse failure)
- liveSpore.json corruption + backup recovery
- Missing binary graceful degradation

### 006 — Cross-Platform Binary Validation (May 15)

- `x86_64-unknown-linux-musl`: 5.1 MB, tested on Alpine/Ubuntu/Fedora/Debian
- `x86_64-pc-windows-gnu`: 7.9 MB, tested via Wine 11
- `aarch64-unknown-linux-musl`: cross-compiled, tested on RPi4

### 007 — Scope-Driven Chassis Abstraction (May 16)

Abstracted LTEE-specific module tables to runtime-loaded `scope.toml` +
`data.toml`. Regression introduced (0/75 → fixed → 75/75). Root cause:
expected-file matching, empty-path guard, multi-dataset resolution.
4 integration tests added as regression guards.

### 008 — Cross-Tier Parity Validation (May 17)

`litho parity` runs Tier 1 (Python) and Tier 2 (Rust) side-by-side for
all 7 modules and reports MATCH/DIVERGENCE per module. Validates that
the math is implementation-independent. `ParityReport`, `ParityResult`,
`ParityStatus` types added to `litho-core::validation`.

### 009 — Tier 3 NUCLEUS Provenance Wiring (May 17)

`provenance.rs` evolved from data structs to a JSON-RPC client for the
provenance trio. `validate --max-tier 3` branches into `announce_self()`
+ `try_record_tier3()` — creates DAG session, spine entry, and braid
via discovered rhizoCrypt/loamSpine/sweetGrass endpoints. Falls back
gracefully when primals are unavailable.

### 010 — Two-Tier Data Model (May 17)

Formalized "ship small, validate deep" — `data.toml` gains `data_tier`,
`full_data_size`, `full_data_tool`, `full_data_checks`, `upstream_spring`,
`upstream_braid`, `upstream_dag_session` fields. `litho fetch --full`
pulls raw upstream data when online. Ferment transcript pattern defined:
upstream springs compute, record provenance, hand the braid to lithoSpore.

### 011 — Chassis Abstraction Evolution (May 17)

Four-phase systematic decoupling of LTEE instance from lithoSpore chassis:

**Phase 1**: `scope.toml` `[[module]]` entries (name, binary, data_dir,
expected, tier1_notebook). `ScopeModule` struct in litho-core. New
`registry.rs` in ltee-cli centralizes `load_module_table()`,
`dispatch_module()`, `module_name_matches()`. All 6 consumer files
migrated. `derive_logical_name()` handles arbitrary binary prefixes.

**Phase 2**: Braid accession expectations derived from `data.toml`
`sra_accession` fields. Target coverage path from `guidestone.targets_file`.
`module_name_matches()` uses registry lookup.

**Phase 3**: `.biomeos-spore` generated from scope.toml during assembly.
Graph and target staging paths parameterized from scope.toml.

**Phase 4**: `viz/` moved from litho-core to ltee-cli (instance layer).
LTEE test fixtures isolated in `crates/litho-core/tests/fixtures/`.
`litho-core` reaches domain-agnostic chassis — 12 modules, no LTEE science logic in source.
192 workspace tests pass (75/75 science checks at Tier 2).

### 012 — Hypogeal Deployment to Barrick Lab (May 18)

First live guideStone handoff to external scientists. 4 USB drives
deployed to the Barrick Lab (MSU) for the LTEE Research Assistant II
interview.

**Deployment cycle**:
1. ext4 deploy (4 USBs) — all pass 75/75 science checks
2. Field test: ext4 invisible on Windows → lesson learned
3. exFAT reformat (showcase USB, 58G) — universal mount
4. Shim pattern: copy-to-tmpdir + chmod for no-exec filesystems
5. Surface audit: 41 root items → 8 (3-zone restructure)
6. Layer 0-4 model codified
7. Pre-rendered HTML browse layer (science/index.html)
8. MANIFEST.toml for AI agent navigation
9. Data courier: 5.2G SRA reads for airgapped seeding

**Key metrics**: 75 science checks, 192 workspace tests, 7 modules, <100ms,
6.3MB binary, exFAT
cross-platform, 10+ validated liveSpore.json entries.

**Documented**: gen4/architecture/HYPOGEAL_DEPLOYMENT_EVOLUTION.md
