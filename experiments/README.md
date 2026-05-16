<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Experiments Index

Chronological log of lithoSpore experiments. Each experiment has a unique
numeric prefix for ordering and a `_TOPIC_DESCRIPTOR` suffix.

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
| 005 | Chaos/fault injection | md | Infrastructure | Complete — 15 scenarios |
| 006 | Cross-platform binary validation | md | Infrastructure | Complete — x86_64, aarch64, Windows |
| 007 | Scope-driven chassis abstraction | md | Architecture | Complete — scope.toml + data.toml |

## Planned Experiments

| # | Name | Type | Domain | Blocked By |
|---|------|------|--------|------------|
| 008 | BLAKE3 hash backfill | toml | Data integrity | Network access to source repos |
| 009 | neuralSpring ML surrogate integration | md | Modules 3, 4 | neuralSpring B3/B4 models |
| 010 | Tier 3 NUCLEUS composition | md | Primal integration | NUCLEUS runtime + primals |
| 011 | Signal dispatch collapse | md | Architecture | biomeOS signal routing |
| 012 | FIDO2 hardware attestation | md | Security | BearDog CTAP2 library |

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

15 fault injection scenarios via `litho chaos-test`:
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
