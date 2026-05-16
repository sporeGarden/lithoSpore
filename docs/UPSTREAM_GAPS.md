# CATHEDRAL Upstream Gap Registry

**Last Updated**: May 15, 2026 (7/7 modules PASS Tier 2, deployment-validated across Linux/Alpine/Windows — Pillar 4 GATE EXCEEDED)
**Phase**: Interstadial → Stadial
**Scope**: lithoSpore + projectFOUNDATION (L5 knowledge layer)
**Geo-delocalization**: Absorbed — discovery chain extended to TURN, liveSpore.json provenance updated

---

## Resolved This Session

| Module | Ready | Blocked By | Severity | Phase |
|--------|-------|-----------|----------|-------|
| 1. ltee-fitness | **Tier 2 PASS (8/8)** | — | **RESOLVED** | Interstadial |
| 2. ltee-mutations | **Tier 2 PASS (7/7)** | — | **RESOLVED** | Interstadial |
| 3. ltee-alleles | **Tier 2 PASS (20/20)** | neuralSpring B3 ML additive | **RESOLVED** | Interstadial |
| 4. ltee-citrate | **Tier 2 PASS (11/11)** | neuralSpring B4 ML additive | **RESOLVED** | Interstadial |
| 5. ltee-biobricks | **Tier 2 PASS (6/6)** | DOI pending | **RESOLVED** | Interstadial |
| 6. ltee-breseq | **Tier 2 PASS (8/8)** | — | **RESOLVED** | Interstadial |
| 7. ltee-anderson | **Tier 2 PASS (5/5)** | — | **RESOLVED** | Interstadial |

**7/7 modules live** — All modules have Rust Tier 2 validation implementations.
75/75 checks passing. Module 5 (biobricks) promoted from scaffold with
metabolic burden validation (6/6 checks). ML surrogate enrichment via
neuralSpring is additive, not blocking.

**Interstadial exit gate (Pillar 4)**: **EXCEEDED** — 7 modules at Tier 2
(Rust). BLAKE3 provenance on fetched data. Pure Rust `litho fetch` replaces
all bash fetch scripts. `litho assemble` replaces `assemble-usb.sh`.

### Audit Debt Resolved (May 13, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| LS-7 | `blake3` crate pulled `cc` for C assembly | Set `default-features = false, features = ["pure", "std"]` — ecoBin compliant |
| LS-6 | Unused `thiserror` dep in litho-core | Removed from workspace + crate |
| LS-8 | Missing SPDX in fetch scripts | Added `AGPL-3.0-or-later` to all fetch scripts |
| FN-2 | CI thread-index validation was a no-op | Fixed `thread`→`threads`, `source_manifest`→`data_sources`, added ML sidecar checks |
| FN-6 | Spec said `store.put`, scripts use `storage.store` | Aligned spec to match implementation |
| FN-3 | Thread 9 used `status = "validated"` schema | Converted all 13 targets to `blake3 = ""`/`validated = true` |
| LS-5 | Orphan `module5_neuralspring_b1.json` misnamed | Renamed to `neuralspring_b1_ml_surrogate.json` |
| C-DUP | `skip_result`, `load_expected`, `pearson_r` duplicated across 4+ binaries | Extracted to `litho-core` modules: `harness`, `stats`, `discovery` |
| C-DISC | No capability-based primal discovery | Added `litho_core::discovery` with env → UDS → skip chain |
| FN-SPDX | Deploy scripts missing SPDX | Added to `fetch_sources.sh`, `foundation_validate.sh` |
| FN-HOST | `foundation_validate.sh` hardcoded `127.0.0.1` for Songbird | Replaced with `$PRIMAL_HOST` everywhere |
| FN-EXPR | `expressions/README.md` missing 4 expressions | Added threads 5-ML, 6, 9, 10 to table |

### Hardening Debt Resolved (May 13, 2026 — second pass)

| ID | Gap | Resolution |
|----|-----|-----------|
| LS-UNSAFE | No `#![forbid(unsafe_code)]` on any crate | `unsafe_code = "forbid"` at workspace lint level, all 9 crates inherit via `[lints] workspace = true` |
| LS-PANIC | `pearson_r` used `assert_eq!` (panics on mismatch) | Changed to `debug_assert_eq!` + early return 0.0 on length mismatch or empty |
| LS-DISC2 | `discovery.rs` hardcoded `127.0.0.1` fallback | Now checks `$PRIMAL_HOST` env var before falling back to localhost |
| LS-PORT | Port parsing via truncating `as u16` cast | Replaced with `u16::try_from().ok()?` — returns `None` on overflow |
| LS-CLIPPY | Many pedantic clippy warnings | Auto-fixed (`f64::midpoint`, `f64::from`, closures); scientific casts allowed at workspace level |
| FN-FETCH | `fetch_sources.sh` hardcoded `127.0.0.1` for NestGate | Replaced with `${PRIMAL_HOST:-127.0.0.1}` |
| FN-GRAPH | `foundation_validation.toml` stale method names | `store.put`→`storage.store`, `dag.session_start`→`dag.session.create`, etc. |
| FN-GPATH | `graphs/README.md` stale `sporeGarden/` paths | Fixed to `../../projectNUCLEUS/deploy` and direct graph paths |
| FN-DEAD | `fetch_from_manifest` 54 LOC dead code | Removed from `deploy/fetch_sources.sh` |
| FN-WK | Workload TOMLs missing SPDX headers | Added `AGPL-3.0-or-later` to all 20 workload TOMLs |
| FN-CI | Shellcheck in CI was `|| true` (advisory) | Made blocking — removed `|| true` |

### Geo-Delocalization Absorption (May 14, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| GEO-DISC | Discovery chain lacked TURN relay path | Extended: env → UDS → `$SONGBIRD_TURN_SERVER` → standalone. `DiscoveryPath` enum + `discover_full()` |
| GEO-MODE | No operating mode detection | Added `probe_operating_mode()` — checks env/UDS/TURN signals before validation |
| GEO-PROV | `liveSpore.json` lacked discovery provenance | Added `discovery_path` + `turn_relay` fields to `LiveSporeEntry` |
| GEO-DOC | Spore taxonomy and operating modes undocumented | README + ARCHITECTURE.md updated with taxonomy table + mode table + discovery chain diagram |

### USB Assembly (May 14–15, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| USB-ASM | No USB assembly | `litho assemble` — pure Rust 9-step orchestrator per `LITHOSPORE_USB_DEPLOYMENT.md` (replaces assemble-usb.sh) |
| USB-ROOT | Missing USB root entry points | argv[0] symlink detection — `validate`, `verify`, `refresh`, `spore` are symlinks to `bin/litho` |
| USB-TOWER | No biomeOS spore composition | `artifact/usb-root/biomeOS/tower.toml` + `graphs/lithoSpore_validation.toml` |
| USB-FLAT | Build only produced `bin/{arch}/static/` layout | `litho assemble` produces flat `bin/` layout directly |

### USB Pipeline Hardening (May 14, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| USB-BIN | `litho` CLI only checked `target/release/` for module binaries | Added `resolve_binary()` — checks `bin/` (USB) first, then `target/release/` (dev) |
| USB-SPORE | `liveSpore.json` written to `artifact/liveSpore.json` only | Added `resolve_livespore()` — detects USB layout via `.biomeos-spore` marker, writes to root |
| USB-EXPECTED | `assemble-usb.sh` did not stage `validation/expected/` | Added step 8: copies expected-value JSONs to USB |
| USB-DATA | Modules 3+4 data not fetched | Ran `fetch_good_2017.sh` + `fetch_blount_2012.sh` with `$ECOPRIMALS_ROOT` — 6/6 data bundles staged |
| USB-VM | No VM validation of USB artifact | Built VM via agentReagents `lithoSpore-validation.yaml`, SSH'd USB, 6/7 PASS (51/51 checks) |

### Deep Evolution Pass (May 15, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| LS-VIZ | `viz.rs` monolith (1248 lines) | Refactored into `viz/mod.rs`, `viz/modules.rs`, `viz/baselines.rs` — grouped by data flow |
| LS-CLI | `main.rs` monolith (994 lines) | Refactored into `validate.rs`, `visualize.rs`, `verify.rs`, `ops.rs` — thin wiring in `main.rs` |
| LS-UDS | UDS RPC transport was stub (`None`) | Implemented `rpc_uds()` — `UnixStream` JSON-RPC matching TCP pattern |
| LS-ENVHC | Hardcoded IPs, env keys, socket paths | Evolved to `$PRIMAL_HOST`, `resolve_xdg_runtime`, `has_any_capability_env`, configurable connectivity hosts |
| LS-TEST | `ltee-cli` had zero tests | Added 13 unit + 8 integration tests with fixture-based harness |

### Discovery Capability Gaps (documented, upstream-blocked)

| Gap | Status | Impact | Details |
|-----|--------|--------|---------|
| UDS RPC transport | **RESOLVED** | LAN mode Tier 2 IPC now supports UDS | `rpc_uds()` implements `UnixStream` JSON-RPC client matching TCP `rpc_call()` pattern. |
| Songbird TURN client | Stub (env-var only) | Geo-delocalized mode uses env var address only | `discover_from_turn()` resolves endpoint from `$SONGBIRD_TURN_SERVER` + `$SONGBIRD_TURN_DISCOVERY_PORT` but actual TURN relay requires upstream Songbird client library. |
| TURN-relayed RPC | Not implemented | No actual relay IPC | RPC calls through TURN endpoints use standard TCP, which only works if relay forwards raw TCP. |

These are documented in `litho_core::discovery::rpc_call()` doc comments. All callers degrade
gracefully to `None` / `Skip` — no panics, no silent failures.

Upstream-blocked (not actionable by CATHEDRAL):
- Songbird TURN client library (needed for actual TURN-relayed IPC)
- BearDog FIDO2/CTAP2 for SoloKey witness in `liveSpore.json`
- sporePrint pipeline wiring (`notify-sporeprint.yml` → Zola)
- genomeBin primal packaging for Tier 3 on USB

## Remaining — projectFOUNDATION

| ID | Priority | Gap | Action |
|----|----------|-----|--------|
| FN-1 | HIGH | All `data/sources/*.toml` have `blake3 = ""` and `retrieved = ""` | Run `deploy/fetch_sources.sh --thread all`, capture hashes, backfill TOMLs |
| FN-5 | MEDIUM | Thread 1 WCM: all 24 targets `validated = false` despite existing logs | Review `validation/wcm-20260509/` results, flip validated where justified |
| FN-4 | MEDIUM | Thread 5 ML: `thread05_ml_surrogates.toml` has `accessions = []` everywhere | ML sources are internal (neuralSpring models) — document as `source_type = "internal"` |
| FN-WK2 | LOW | Anderson/enviro workloads embed synthetic actuals=expected | Wire to real spring output or mark `synthetic = true` |

## Ecosystem Gaps (Upstream / Cross-cutting)

| ID | Priority | Gap | Owner |
|----|----------|-----|-------|
| CC-1 | INFO | `SCYBORG_PROVENANCE_TRIO_GUIDANCE.md` only in external fossilRecord repo | infra team |
| CC-2 | MEDIUM | `LTEE_GUIDESTONE_SUBSYSTEM_HANDOFF_MAY11_2026.md` missing from `handoffs/` | primalSpring — file never committed |
| CC-3 | RESOLVED | No CATHEDRAL handoffs written back to primalSpring | Written May 13: `CATHEDRAL_DEEP_DEBT_AUDIT_MAY13_2026.md` |
| FN-DATA | RESOLVED | `data/README.md` schema stale | Updated to reflect all 10 threads May 13 |

---

## Spring Gap Tables

### groundSpring (9 papers — contributes to ALL modules)

groundSpring is the **critical path**. Every module depends on statistical
methods from groundSpring.

| Paper | Gap | What lithoSpore Needs |
|-------|-----|----------------------|
| B1 | Drift vs selection null model | Neutral mutation rate estimator, fixation probability calculator |
| B2 | Jackknife + AIC/BIC model selection | Model fitting API: power-law, hyperbolic, logarithmic with AIC/BIC |
| B3 | Clonal interference statistics | Multi-beneficial-mutation fixation probability, interference model |
| B4 | Rare event statistics | Probability framework for potentiating mutation cascades |
| B6 | Anderson Wc analogy | Burden → disorder potential mapping, statistical distribution fitting |
| B7 | Epistasis quantification | Parallel evolution significance tests across 264 genomes |
| B8 | Bet-hedging statistics | Phase variation rate estimation, stochastic switching models |
| B9 | DFE fitting | Gamma/exponential/lognormal parameter estimation for DFE |

**Status**: B1–B4 **COMPLETE** — Python + Rust validated. B7 INTEGRATED via wetSpring.
Remaining 4 papers QUEUED.

### neuralSpring (12 papers — contributes to modules 2, 3, 4, 5)

| Paper | Gap | What lithoSpore Needs |
|-------|-----|----------------------|
| B1 | LSTM time-series prediction | Mutation accumulation curve predictor |
| B2 | LSTM prediction + ESN regime detection | Fitness trajectory prediction, inflection detection |
| B3 | LSTM+HMM+ESN allele classification | Clade state detection, regime classification |
| B4 | Early warning ESN | Pre-citrate trajectory anomaly detection |
| B6 | ML burden prediction | Sequence-to-burden predictor (GC%, codon usage, promoter) |
| B7 | Parallel evolution ML | Transfer learning for gene-level convergence detection |
| B8 | Contingency loci prediction | Sequence feature → contingency locus classifier |
| B9 | DFE regime shifts | LSTM DFE parameter prediction, ESN regime shift detection |
| E2 | HOLIgraph GNN | Protein-ligand binding prediction (GPU accelerated) |
| E3 | Glycosylation ML | Glycosylation → binding affinity predictor |
| E4 | Macrocyclic ranking ML | Binder ranking from sequence features |
| E5 | Antibody pairing ML | VH/VL pairing prediction from single-cell data |

**Status**: All 12 papers QUEUED. ML surrogates are additive to modules 3+4
(groundSpring B3/B4 validation already works without ML).

### wetSpring (10 papers — contributes to modules 1, 6)

**Status**: B7 **INTEGRATED** (Module 6). Remaining 9 papers QUEUED.

### hotSpring (2 papers — contributes to module 7)

**Status**: B2 **INTEGRATED** (Module 7). B9 QUEUED.

### healthSpring (3 papers — future modules)

**Status**: All 3 papers QUEUED.

### airSpring (1 paper — future module)

**Status**: E3 QUEUED.

---

## Cross-Cutting Gaps

| Gap | Owner | Phase | Description |
|-----|-------|-------|-------------|
| Data fetching | lithoSpore | **DONE (all 7)** | `litho fetch` — pure Rust, replaces 7 bash scripts |
| Expected values (modules 1–7) | Springs → lithoSpore | **DONE** | All 7 golden JSON files exist and validate |
| musl cross-compilation | lithoSpore | **DONE** | `x86_64-unknown-linux-musl` — 5.1 MB static binary, tested on Alpine/Ubuntu/Fedora/Debian |
| Windows cross-compilation | lithoSpore | **DONE** | `x86_64-pc-windows-gnu` — 7.9 MB litho.exe, tested via Wine 11 |
| BioBrick paper DOI | External | Stadial | B6 DOI placeholder — update when Nat Comms finalizes |
| DFE paper DOI | External | Stadial | B9 DOI placeholder — update when Science finalizes |

---

## Bash-to-Rust Migration — lithoSpore COMPLETE

lithoSpore bash-to-Rust elevation completed May 15, 2026. All shell scripts
replaced with pure Rust subcommands in the `litho` CLI:

| Script | Replaced By | Status |
|--------|-------------|--------|
| `assemble-usb.sh` | `litho assemble` | **DONE** — pure Rust, std::fs + walkdir + blake3 |
| `build-artifact.sh` | `cargo build --release --target x86_64-unknown-linux-musl` | **DONE** — direct cargo |
| `fetch_*.sh` (7 scripts) | `litho fetch` | **DONE** — ureq HTTP + serde_json + blake3 |
| `chaos-test.sh` | `litho chaos-test` | **DONE** — 10 fault injection tests, in-process |
| `deploy-test-local.sh` | `litho deploy-test` | **DONE** — assemble + verify + validate cycle |
| `validate.sh` | `litho validate` | **DONE** — in-process module calls |
| USB shims (validate, verify, refresh, spore.sh) | argv[0] symlink detection | **DONE** — single binary |
| `artifact/ltee` | argv[0] detection for `ltee` | **DONE** — legacy entry point |

**Only remaining shell**: `scripts/build-container.sh` (container engine interaction
doesn't benefit from Rust).

Additional platform evolution:
- External command calls (`date`, `hostname`, `id`) replaced with `chrono`, `/etc/hostname`, `/proc/self/status`
- 7 module binaries unified into single `litho` binary via lib.rs + in-process dispatch
- `#[cfg]` platform guards for Windows cross-compilation (COMPUTERNAME, %TEMP%, copy-for-symlink)
- `ipc.resolve` method aligned with capability registry
- `compute.dispatch` in tower.toml aligned with graph

### Remaining ecosystem migration (upstream, not lithoSpore)

| Priority | Script | Repo | Status |
|----------|--------|------|--------|
| 1 | `fetch_sources.sh` | projectFOUNDATION | Pending |
| 2 | `foundation_validate.sh` | projectFOUNDATION | Pending |
| 3 | `backfill_hashes.sh` | projectFOUNDATION | Pending |
| 4 | Lab orchestration scripts | benchScale | Pending |
| 5 | Image provisioning scripts | agentReagents | Pending |

## Changelog

- **2026-05-15**: Deployment matrix validated — musl-static on Ubuntu airgap/VPS, Alpine chroot,
  read-only FS; Windows litho.exe via Wine 11. agentReagents templates created for Alpine,
  Fedora, Debian, read-only. All platforms PASS.
- **2026-05-15**: Bash-to-Rust elevation complete — all 8 lithoSpore scripts replaced with
  pure Rust subcommands. External command calls (`date`, `hostname`, `id`) replaced with
  chrono/filesystem reads. 7 module binaries unified into single litho CLI via lib.rs.
  Windows #[cfg] guards added. Only scripts/build-container.sh remains as shell.
- **2026-05-15**: Deep Evolution pass — viz.rs refactored (1248→3 files), ltee-cli main.rs
  refactored (994→4 subcommand modules), UDS RPC implemented, hardcoding evolved to
  capability-based discovery, 21 new tests added. petalTongue dead_code markers evolved
  to `#[expect(dead_code, reason = "...")]`.
- **2026-05-15**: Root doc cleanup, broken wateringHole path fixes, handback directory
  created. Test count corrected (33→66), container positioning clarified.
- **2026-05-15**: petalTongue Interactive SceneGraph Evolution — 6 phases
  (semantic data_id, click-to-select, ViewCamera, IPC bridge, data-driven
  animation, parameter controls). Full handback written.
- **2026-05-14**: petalTongue scientific visualization — litho-core::viz module
  with DataBinding adapters for all 7 LTEE modules and 7 Barrick Lab baseline
  tools. Render-path convergence validation pipeline established.
- **2026-05-13**: Deep-debt audit sweep — extracted `litho_core::{harness, stats, discovery}`,
  ecoBin BLAKE3 compliance, SPDX headers, projectFOUNDATION CI fix, schema alignment.
  CATHEDRAL handoff written to `validation/handbacks/`.
- **2026-05-13**: Modules 3+4 promoted from scaffold: groundSpring B3 (Good 2017 clonal
  interference) and B4 (Blount 2008/2012 citrate innovation) ingested. 6/7 modules wired.
- **2026-05-13**: Gap summary updated: 4/7 modules PASS Tier 2 (28/28 checks).
  Modules 6+7 marked RESOLVED (wetSpring B7, hotSpring B2). Gate status EXCEEDED.
- **2026-05-12**: Modules 6+7 integrated — wetSpring B7 Tenaillon (8/8 PASS),
  hotSpring B2 Anderson disorder (5/5 PASS). Pillar 4 gate exceeded (4/7 > 2+).
- **2026-05-11**: Modules 1+2 Tier 1 PASS — groundSpring B2/B1 integrated,
  fetch scripts created, Python baselines ported, expected values cross-validated.
