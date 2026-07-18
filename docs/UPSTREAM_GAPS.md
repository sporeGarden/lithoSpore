# lithoSpore Upstream Gap Registry

**Last Updated**: Jul 17, 2026 (75/75 checks, 7/7 modules, Tier 3 wired, 216 tests, 3 live braids)
**Phase**: Post-deployment — silicon atheism evolution pass complete
**Scope**: lithoSpore verification chassis (LTEE first instance; chassis domain-agnostic)
**Geo-delocalization**: Absorbed — discovery chain env → UDS → TURN → standalone
**Deployment**: exFAT cross-platform, 3-zone structure, Layer 0-4 model

---

## Resolved This Session

| Module | Ready | Blocked By | Severity | Phase |
|--------|-------|-----------|----------|-------|
| 1. ltee-fitness | **Tier 2 PASS (8/8)** | — | **RESOLVED** | Interstadial |
| 2. ltee-mutations | **Tier 2 PASS (7/7)** | — | **RESOLVED** | Interstadial |
| 3. ltee-alleles | **Tier 2 PASS (20/20)** | neuralSpring B3 ML additive | **RESOLVED** | Interstadial |
| 4. ltee-citrate | **Tier 2 PASS (11/11)** | neuralSpring B4 ML additive | **RESOLVED** | Interstadial |
| 5. ltee-biobricks | **Tier 2 PASS (6/6)** | DOI pending | **RESOLVED** | Interstadial |
| 6. ltee-breseq | **Tier 2 PASS (16/16)** | — | **RESOLVED** | Interstadial |
| 7. ltee-anderson | **Tier 2 PASS (7/7)** | — | **RESOLVED** | Interstadial |

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
| LS-6 | `thiserror` dep in litho-core | Now used for `LithoError` typed error hierarchy (`error.rs`) |
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
| LS-UNSAFE | No `#![forbid(unsafe_code)]` on any crate | `unsafe_code = "forbid"` at workspace lint level, all 10 crates inherit via `[lints] workspace = true` |
| LS-PANIC | `pearson_r` used `assert_eq!` (panics on mismatch) | Changed to `debug_assert_eq!` + early return 0.0 on length mismatch or empty |
| LS-DISC2 | `discovery.rs` hardcoded `127.0.0.1` fallback | Now checks `$PRIMAL_HOST` env var before falling back to localhost |
| LS-PORT | Port parsing via truncating `as u16` cast | Replaced with `u16::try_from().ok()?` — returns `None` on overflow |
| LS-CLIPPY | Many pedantic clippy warnings | Auto-fixed (`f64::midpoint`, `f64::from`, closures); scientific casts allowed at workspace level |
| FN-FETCH | `fetch_sources.sh` hardcoded `127.0.0.1` for NestGate | Replaced with `${PRIMAL_HOST:-127.0.0.1}` |
| FN-GRAPH | `foundation_validation.toml` stale method names | `store.put`→`storage.store`, `dag.session_start`→`dag.session.create`, etc. |
| FN-GPATH | `graphs/README.md` stale `sporeGarden/` paths | Fixed to `../../projectNUCLEUS/deploy` and direct graph paths |
| FN-DEAD | `fetch_from_manifest` 54 LOC dead code | Removed from `deploy/fetch_sources.sh` (replaced by `litho fetch` / projectFOUNDATION) |
| FN-WK | Workload TOMLs missing SPDX headers | Added `AGPL-3.0-or-later` to all 20 workload TOMLs |
| FN-CI | Shellcheck in CI was `|| true` (advisory) | Made blocking — removed `|| true` |

### Silicon Atheism Evolution Pass (Jul 17, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| SA-CFG | 18 `#[cfg]` gates scattered across 7 files | Created `Platform` trait in `litho-core::platform` — `UnixPlatform` + `FallbackPlatform`. Only 2 `#[cfg]` blocks remain (in `platform.rs::current()`) |
| SA-MAIN | `main.rs` at 730 lines — symlink dispatch embedded in main | Extracted `dispatch.rs` module — `try_symlink_dispatch()` + helpers. `main.rs` reduced to 648 lines |
| SA-DOMAIN | `domain_profile.rs` at 799 lines | Split to `domain_profile/mod.rs` (493 lines) + `domain_profile/parse.rs` (parsing helpers) |
| SA-AUDIT | `audit/domain.rs` at 797 lines | Extracted `audit/derivation.rs` (228 lines) — derivation contract checks + PLUMED discovery. `domain.rs` reduced to 585 lines |
| SA-COW | `provenance::endpoint_addr` cloned strings unnecessarily | Changed to `Cow<'_, str>` — zero-copy when address is already in env |
| SA-BRAID | `braid::format_braid_summary` used Vec<String> + join | Replaced with single String + `write!` — eliminates intermediate allocations |
| SA-VIZ | `visualize.rs` cloned candidate string unnecessarily | Direct return of owned `candidate` — avoids redundant clone |

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
| USB-ASM | No USB assembly | `litho assemble` — pure Rust 10-step orchestrator per `LITHOSPORE_USB_DEPLOYMENT.md` (replaces assemble-usb.sh) |
| USB-ROOT | Missing USB root entry points | argv[0] symlink detection — `validate`, `verify`, `refresh`, `spore` are symlinks to `bin/litho` |
| USB-TOWER | No biomeOS spore composition | `artifact/usb-root/biomeOS/tower.toml` + `graphs/lithoSpore_validation.toml` |
| USB-FLAT | Build only produced `bin/{arch}/static/` layout | `litho assemble` produces flat `bin/` layout directly |

### USB Pipeline Hardening (May 14, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| USB-BIN | `litho` CLI only checked `target/release/` for module binaries | Added `resolve_binary()` — checks `bin/` (USB) first, then `target/release/` (dev) |
| USB-SPORE | `liveSpore.json` written to `artifact/liveSpore.json` only | Added `resolve_livespore()` — detects USB layout via `.biomeos-spore` marker, writes to root |
| USB-EXPECTED | `assemble-usb.sh` did not stage `validation/expected/` | Added step 8: copies expected-value JSONs to USB |
| USB-DATA | Modules 3+4 data not fetched | Ran `fetch_good_2017.sh` + `fetch_blount_2012.sh` with `$ECOPRIMALS_ROOT` — 6/6 data bundles staged. Superseded by `litho fetch --all` |
| USB-VM | No VM validation of USB artifact | Built VM via agentReagents `lithoSpore-validation.yaml`, SSH'd USB, 6/7 PASS (51/51 checks) |

### Deep Evolution Pass (May 15, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| LS-VIZ | `viz.rs` monolith (1248 lines) | Refactored into `viz/mod.rs`, `viz/modules.rs`, `viz/baselines.rs` — grouped by data flow |
| LS-CLI | `main.rs` monolith (994 lines) | Refactored into `validate.rs`, `visualize.rs`, `verify.rs`, `ops.rs` — thin wiring in `main.rs` |
| LS-UDS | UDS RPC transport was stub (`None`) | Implemented `rpc_uds()` — `UnixStream` JSON-RPC matching TCP pattern |
| LS-ENVHC | Hardcoded IPs, env keys, socket paths | Evolved to `$PRIMAL_HOST`, `resolve_xdg_runtime`, `has_any_capability_env`, configurable connectivity hosts |
| LS-TEST | `ltee-cli` had zero tests | Added 13 unit + 8 integration tests with fixture-based harness |

### Tier 3 Wiring and Cross-Tier Parity (May 17, 2026)

| ID | Gap | Resolution |
|----|-----|-----------|
| T3-BRANCH | `validate.rs` treated `--max-tier 3` identically to `--max-tier 2` | Added Tier 3 branch: discovery probe → `announce_self()` → `try_record_tier3()` → upgrade `tier_reached` to 3 on success |
| T3-RPC | `provenance.rs` had data structs only, no RPC client | Evolved to JSON-RPC client: `dag.session.create` → `event.append` × N → `dag.session.complete` → `spine.create` → `entry.append` → `braid.create` |
| T3-SESSION | `ValidationReport` had no Tier 3 metadata | Added `Tier3Session` struct (dag_session_id, merkle_root, spine_id, braid_id, primals_reached) as optional field on report |
| T3-ANNOUNCE | `discovery.rs` never called from validation | Added `announce_self()` → `primal.announce` to biomeOS; `query_capabilities()` for Wave 20 canonical envelope |
| T3-SIGNAL | `nest.store` signal mapping undocumented in code | Added code-level documentation mapping 3-call provenance sequence to `nest.store` atomic signal for future biomeOS collapse |
| PARITY | No cross-tier numerical check | New `litho parity` subcommand: runs Tier 1 + Tier 2 side-by-side, reports MATCH/DIVERGENCE/SKIPPED per module |
| PARITY-TYPES | No parity report types | Added `ParityResult`, `ParityStatus`, `ParityReport` to `litho-core::validation` |
| PROV-DIR | No projectFOUNDATION Thread 10 output | Added `--provenance-dir` flag: writes `results.json` + `provenance.toml` to dated folder |

### Chassis Abstraction Evolution (May 17, 2026 — completed)

All 7 coupling points from the original inventory have been resolved:

| Coupling Point | Status | Resolution |
|----------------|--------|------------|
| `MODULE_DISPATCH` (validate/parity/chaos/deploy) | **RESOLVED** | Centralized in `registry.rs` — scope.toml `[[module]]` entries primary, compiled LTEE fallback |
| `LTEE_MODULES` constant (6 files) | **RESOLVED** | All 6 consumers (validate, parity, ops, chaos, deploy_test, visualize) import from `registry.rs` |
| `LTEE_NOTEBOOKS` constant | **RESOLVED** | scope.toml `[[module]]` `tier1_notebook` field primary; `registry.rs` LTEE fallback |
| `module_name_matches()` | **RESOLVED** | `registry::module_name_matches()` does registry lookup via `derive_logical_name()` |
| `strip_prefix("ltee-")` assumption | **RESOLVED** | `derive_logical_name()` handles `ltee-`, `milc-`, `lattice-` prefixes generically |
| `.biomeos-spore` hardcoded template | **RESOLVED** | `generate_biomeos_spore()` derives from scope.toml during `litho assemble` |
| `viz/` in litho-core | **RESOLVED** | Moved to `ltee-cli/src/viz/` (instance layer) — `litho-core` is domain-agnostic chassis |
| Hardcoded graph/target paths | **RESOLVED** | `guidestone.graph_file` + `guidestone.targets_file` in scope.toml |
| Hardcoded braid accessions | **RESOLVED** | Derived from `data.toml` `sra_accession` fields at runtime |
| `litho-core` | **Agnostic** | 12 modules, no LTEE science logic in source |
| `scope.toml` loader | **Agnostic** | `ScopeModule` struct + `[[module]]` table + `module_binaries()` |
| `data.toml` manifest | **Agnostic** | Dataset registry with BLAKE3, source URIs, licenses, SRA accessions |
| `discovery.rs` / `provenance.rs` | **Agnostic** | Capability strings, JSON-RPC to capability-discovered endpoints |
| `liveSpore.json` | **Agnostic** | Append-only, PII-hashed, platform-detected |

**Remaining (cosmetic / future)**:
- `ltee-cli` → `litho-cli` rename (cosmetic)
- `ltee-*` Cargo deps → feature flags per instance (future)
- Dynamic module loading / plugin architecture (future)
- `litho init` scaffolder for non-LTEE instances (future)

### guideStone Five-Property Audit (May 17, 2026)

Per primals.eco guideStone specification:

| Property | Status | Gap |
|----------|--------|-----|
| **1. Deterministic Output** | PARTIAL | 7/7 modules deterministic on x86_64. No cross-arch (aarch64) validation matrix yet. No bit-identical cross-substrate comparison table. |
| **2. Reference-Traceable** | **STRONG** | 16 papers, all DOIs. 14 targets. Expected JSONs carry `doi` + `source_figures`. |
| **3. Self-Verifying** | **STRONG** | BLAKE3 in `data.toml`, `litho verify`, `litho self-test` (23 checks), `litho validate` (75 checks). |
| **4. Environment-Agnostic** | PARTIAL | Tier 2 is pure Rust musl-static. Tier 1 depends on Python. Container deployment helps but isn't musl-pure. |
| **5. Tolerance-Documented** | PARTIAL | `tolerances.toml` exists with justifications. Not all modules derive tolerances from published quantities; some are empirical. |

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
| FN-1 | HIGH | All `data/sources/*.toml` have `blake3 = ""` and `retrieved = ""` | Run `litho fetch` / projectFOUNDATION (replaces `deploy/fetch_sources.sh --thread all`), capture hashes, backfill TOMLs |
| FN-5 | MEDIUM | Thread 1 WCM: all 24 targets `validated = false` despite existing logs | Review `validation/wcm-20260509/` results, flip validated where justified |
| FN-4 | MEDIUM | Thread 5 ML: `thread05_ml_surrogates.toml` has `accessions = []` everywhere | ML sources are internal (neuralSpring models) — document as `source_type = "internal"` |
| FN-WK2 | LOW | Anderson/enviro workloads embed synthetic actuals=expected | Wire to real spring output or mark `synthetic = true` |

## Ecosystem Gaps (Upstream / Cross-cutting)

| ID | Priority | Gap | Owner |
|----|----------|-----|-------|
| CC-1 | INFO | `SCYBORG_PROVENANCE_TRIO_GUIDANCE.md` only in external fossilRecord repo | infra team |
| CC-2 | RESOLVED | `LTEE_GUIDESTONE_SUBSYSTEM_HANDOFF_MAY11_2026.md` in `handoffs/archive/` | Archived as expected |
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

**Status**: All 12 papers QUEUED for lithoSpore integration. neuralSpring
reports B3 HMM/ESN classifier at 100% accuracy (T06 target met upstream);
ready for ingest when surrogate model is packaged. ML surrogates are additive
to modules 3+4 (groundSpring B3/B4 validation already works without ML).

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
| `fetch_*.sh` (7 scripts) | `litho fetch` | **DONE** — curl subprocess + serde_json + blake3 |
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
- `Platform` trait replaces all `#[cfg]` platform guards — trait-based OS abstraction (silicon atheism, Jul 2026)
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

## Wave 18 Signal Adoption Status

lithoSpore is a verification chassis (consumer), not a primal (provider).
Signal adoption applies to orchestration metadata and provenance wiring,
not to in-process `ctx.dispatch()` calls.

| Signal | Status | Notes |
|--------|--------|-------|
| `primal.announce` | **Registry absorbed** | Added to `capability_registry.toml`; lithoSpore does not self-register (CLI tool, not daemon) |
| `primal.info` | **Registry absorbed** | Available for querying ecosystem primals |
| `nest.store` | **Graph + workload annotated** | `ltee_guidestone.toml` signals field, Tier 3 workload signals field |
| `nest.commit` | **Graph + workload annotated** | Session finalization for provenance |
| `node.compute` | **Graph annotated** | toadStool compute dispatch in Tier 3 graph |
| `health.readiness` | **Registry absorbed** | Deployment Validation Standard triad |
| `health.check` | **Registry absorbed** | Deployment Validation Standard triad |
| `visualization.render` | **Code + registry** | `RPC_VIZ_RENDER` constant in `visualize.rs`, capability-based discovery |

lithoSpore's signal adoption path: When biomeOS supports signal dispatch
routing, the Tier 3 graph's provenance phase (rhizoCrypt → loamSpine →
sweetGrass) collapses to a single `nest.store` dispatch. The graph and
workload TOMLs are annotated to enable this. Code-level adoption requires
a `CompositionContext`-compatible runtime, which lithoSpore does not
currently embed (standalone CLI pattern).

## Changelog

- **2026-07-17**: Silicon Atheism Evolution Pass — `Platform` trait absorbs 18 `#[cfg]`
  gates across 7 files into 2 blocks in `litho-core::platform`. `main.rs` refactored:
  symlink dispatch extracted to `dispatch.rs` (730→648 lines). Large files refactored:
  `domain_profile.rs` split to `domain_profile/mod.rs` + `parse.rs` (799→493 lines),
  `audit/domain.rs` split to `audit/derivation.rs` (797→585 lines). Idiomatic Rust:
  `Cow<str>` in provenance, `write!`-based string building in braid, clone elimination
  in visualize. 16 new tests added. All deps confirmed pure Rust. Zero production mocks.
  216/216 tests, 0 clippy warnings, fmt clean, doc clean.
- **2026-05-17 PM**: Chassis Abstraction Evolution — all 7 coupling points resolved.
  `scope.toml` `[[module]]` entries carry name/binary/data_dir/expected/tier1_notebook.
  New `registry.rs` centralizes module resolution for all 6 consumer files. `.biomeos-spore`
  generated from scope.toml. Braid accessions from data.toml. `viz/` moved from litho-core
  to ltee-cli (instance layer). `litho-core` 12 modules, domain-agnostic chassis. Graph/target paths
  parameterized. Test fixtures isolated. 199 tests, zero clippy warnings. [updated: 216 tests post evolution pass]
- **2026-05-17 PM**: wetSpring braid ingestion — `litho-core::braid` module (4 tests),
  sovereign + breseq baseline braids parsed, accession validated (SRP001569 PASS),
  braids displayed in `litho validate` output. URI fixes: gzip/zip content-type
  ordering bug, Dryad API auth handled, tar.gz/zip unpacking. [historical: was 123 tests at that time].
- **2026-05-17 PM**: Wave 21 absorption — canonical `primal.list` / `capability.list`
  envelope types and `query_primal_list()` added. Method stability tiers annotated
  on all registry domains (stable/evolving/internal). `try_record_tier3()` evolved
  for partial provenance (DAG-only valid, spine/braid optional). `ParityReport`
  published as ecosystem standard (`specs/PARITY_REPORT_SCHEMA.md`). Per-primal
  degradation matrix documented (`docs/DEGRADATION_BEHAVIOR.md`). Braid ingestion
  path prepared (`provenance/braids/`). [historical: was 119 tests at that time] (2 new Wave 20 envelope tests).
  wateringHole handoff: `LITHOSPORE_WAVE21_ABSORPTION_HANDOFF_MAY17_2026.md`.
- **2026-05-17**: Root docs cleanup — README.md chaos count corrected (15→10),
  GETTING_STARTED.md paper count (18→16) and check count (73→75) fixed,
  specs/MODULES.md Tier 1 status for M6/M7 updated (No Tier 1 → Complete,
  parity No → Yes), experiments/README.md chaos count fixed and experiments
  008-010 added (parity, Tier 3, two-tier data). whitePaper/baseCamp/README.md
  updated with Tier 3, parity, and ferment transcript sections. main.rs module
  doc updated (9→20 subcommands). scripts/ description corrected in README tree.
  7 CATHEDRAL handoffs (May 13-15) archived per 48h rule. New wateringHole
  handoff: LITHOSPORE_PRIMAL_SPRING_EVOLUTION_HANDOFF_MAY17_2026.md — primal
  evolution requests, NUCLEUS composition, deployment patterns. [historical: was 117 tests at that time]
  pass, zero clippy errors.
- **2026-05-17**: Two-tier data model and ferment transcript pattern formalized.
  `data.toml` gains `data_tier`, `full_data_size`, `full_data_tool`, `full_data_checks`,
  `upstream_spring`, `upstream_braid`, `upstream_dag_session` fields. `litho fetch --full`
  flag implemented for deep data pulls. ARCHITECTURE.md and GETTING_STARTED.md updated
  with "ship small, validate deep" strategy. wateringHole handoff written:
  `LITHOSPORE_FERMENT_TRANSCRIPT_BRAID_HANDOFF_MAY17_2026.md` — defines the upstream
  contract for springs handing braids to guideStone artifacts. Cross-referenced from
  `PROVENANCE_TRIO_INTEGRATION_GUIDE.md`, `SWEETGRASS_SPRING_BRAID_PATTERNS.md`,
  and `LITHOSPORE_USB_DEPLOYMENT.md`.
- **2026-05-17**: Tier 3 and cross-tier parity implementation — `provenance.rs` evolved
  from data structs to JSON-RPC client for provenance trio (dag/spine/braid).
  `validate.rs` gains `--max-tier 3` branch with discovery probe, `announce_self()`,
  `try_record_tier3()`. New `litho parity` subcommand for cross-tier numerical
  comparison. `Tier3Session`, `ParityReport`, `ParityResult`, `ParityStatus` types
  added to `litho-core::validation`. `discovery.rs` gains `announce_self()` +
  `query_capabilities()` for Wave 20 canonical envelope. `--provenance-dir` flag
  for projectFOUNDATION Thread 10 compatibility. [historical: was 117 tests at that time], 20 subcommands.
  Specs updated: ARCHITECTURE.md (chassis evolution roadmap, guideStone five-property
  assessment), MODULES.md (tier support matrix, coupling inventory), UPSTREAM_GAPS.md
  (Tier 3, parity, chassis abstraction status, guideStone audit), README.md, SCIENCE.md.
- **2026-05-23**: Wave 46 audit absorption — reviewed primalSpring downstream
  pattern guide, confirmed: Dark Forest gate compliance (deploy graph has
  secure_by_default + btsp_enforced + uds_only + by_capability on all nodes),
  signal adoption annotations present (nest.store/nest.commit mapped in graph),
  TURN discovery wired (env → UDS → TURN → standalone chain operational),
  production unwrap() in `grow/` module replaced with graceful error handling.
  Updated all check counts to 75/75 (citrate at 11/11 checks).
  Upstream items: braid accession normalization (SRP→PRJNA), songbird-turn-client
  integration (enhancement over current raw TCP), aarch64 binary for Apple Silicon.
- **2026-05-18**: First live deployment — 4 USBs to Barrick Lab (MSU). exFAT
  evolution (ext4 invisible on Windows). 3-zone restructure (41→8 root items).
  Layer 0-4 model. Pre-rendered HTML browse layer. MANIFEST.toml. Data courier
  (5.2G SRA reads). Documented in gen4/architecture/HYPOGEAL_DEPLOYMENT_EVOLUTION.md.
- **2026-05-17**: Chassis abstraction complete — scope-driven module registry,
  viz moved to instance layer, litho-core 100% agnostic. Braid sync from wetSpring
  (3 braids). Documented as Exp 011 + 012.
- **2026-05-16**: Deep debt pass: viz/baselines.rs (637→376 LOC) and viz/modules.rs
  (367→178 LOC) refactored via 9 extracted DataBinding builder helpers. Discovery
  evolved to capability-generic env vars ($RELAY_SERVER, $VISUALIZATION_SOCKET) with
  legacy fallback. Rust 2024 reserved keyword fix (`gen` → `generation`). Root docs
  updated (README.md date/counts, GETTING_STARTED.md unified CLI). Created
  whitePaper/baseCamp/ and experiments/ per ecosystem conventions. Two upstream
  handoffs written to infra/wateringHole/. Stale barracuda scenario ref cleaned from
  module7_anderson.json. UPSTREAM_GAPS.md renamed from CATHEDRAL, module 6/7 check
  counts corrected (8→16, 5→7).
- **2026-05-16**: Chassis regression fixed — scope-driven module resolution
  bugs in `validate.rs` (expected-file matching, empty-path guard, multi-dataset
  resolution). 4 integration tests added. Deep debt pass: consolidated 6
  duplicate module tables, capability-based discovery wiring, `#[allow]`
  elimination, redundant dep removal, stale `scripts/fetch_*.sh` references
  cleaned from all 7 module crates. Wave 18 absorption: THREAD_INDEX.toml
  expanded (4→6 threads, Thread 5 LTEE + Thread 6 Agriculture added),
  capability_registry.toml aligned with health triad + primal.announce +
  signal tier annotations, deploy graph and workload TOMLs annotated with
  signal names, workload isolation hardened (None→Standard).
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
- **2026-05-14**: petalTongue scientific visualization — ltee-cli::viz module
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
