# lithoSpore Module Specifications

## Module Registry

| # | Crate | Binary | Paper | Tier 2 Spring Sources | Tier 1 Python | Tier 1 Status |
|---|-------|--------|-------|----------------------|---------------|---------------|
| 1 | `ltee-fitness` | `ltee-fitness` | Wiser 2013 (B2) | groundSpring, wetSpring | `notebooks/module1_fitness/` | Complete |
| 2 | `ltee-mutations` | `ltee-mutations` | Barrick 2009 (B1) | groundSpring, neuralSpring | `notebooks/module2_mutations/` | Complete |
| 3 | `ltee-alleles` | `ltee-alleles` | Good 2017 (B3) | neuralSpring, groundSpring | `notebooks/module3_alleles/` | Complete |
| 4 | `ltee-citrate` | `ltee-citrate` | Blount 2008/2012 (B4) | neuralSpring, groundSpring | `notebooks/module4_citrate/` | Complete |
| 5 | `ltee-biobricks` | `ltee-biobricks` | Burden 2024 (B6) | neuralSpring, groundSpring | `notebooks/module5_biobricks/` | Complete |
| 6 | `ltee-breseq` | `ltee-breseq` | Tenaillon 2016 (B7) | wetSpring, groundSpring | `notebooks/module6_breseq/` | Complete |
| 7 | `ltee-anderson` | `ltee-anderson` | Anderson-QS (new) | hotSpring, groundSpring | `notebooks/module7_anderson/` | Complete |

## Per-Module Tier Support Matrix

| # | Module | Tier 0 (structural) | Tier 1 (Python) | Tier 2 (Rust) | Tier 3 (Primal) | `max_tier` honored? | Parity testable? |
|---|--------|---------------------|-----------------|---------------|-----------------|---------------------|------------------|
| 1 | fitness | PASS (`tier0_structural`) | PASS | PASS (8/8) | Via `try_record_tier3` | Yes | Yes |
| 2 | mutations | PASS (`tier0_structural`) | PASS | PASS (7/7) | Via `try_record_tier3` | Yes | Yes |
| 3 | alleles | PASS (`tier0_structural`) | PASS | PASS (20/20) | Via `try_record_tier3` | Yes | Yes |
| 4 | citrate | PASS (`tier0_structural`) | PASS | PASS (11/11) | Via `try_record_tier3` | Yes | Yes |
| 5 | biobricks | PASS (`tier0_structural`) | PASS | PASS (6/6) | Via `try_record_tier3` | Yes | Yes |
| 6 | breseq | PASS (`tier0_structural`) | PASS | PASS (16/16) | Via `try_record_tier3` | Yes | Yes |
| 7 | anderson | PASS (`tier0_structural`) | PASS | PASS (7/7) | Via `try_record_tier3` | Yes | Yes |

**Total**: 7/7 Tier 0 PASS, 7/7 Tier 1 PASS, 7/7 Tier 2 PASS, 75/75 checks.
Cross-tier parity testable for all 7 modules.

## Shared Infrastructure

- **`litho-core`** (chassis — domain-agnostic, 12 modules):
  validation types (`ModuleResult`, `ValidationReport`, `Tier3Session`, `ParityReport`),
  tolerance framework, provenance chain + JSON-RPC client for trio,
  braid ingestion with dual wire format support,
  liveSpore tracking, capability-based primal discovery (env → UDS → TURN → standalone),
  `announce_self()` + `query_capabilities()` for Wave 20,
  shared statistics (`pearson_r`), validation harness (`skip`, `load_expected`, `dispatch_python`),
  scope parser (`ScopeManifest`, `ScopeModule` with `[[module]]` registry), data manifest,
  typed errors (`LithoError` via `error.rs`), env var constants

- **`ltee-cli`** (instance wiring + chassis glue):
  unified `litho` binary with 20 subcommands,
  scope-driven module registry (`registry.rs` — `load_module_table()`, `dispatch_module()`,
  `module_name_matches()`), visualization adapters (`viz/` — DataBinding for all 7 LTEE
  modules + 7 Barrick baselines). `.biomeos-spore` generated from scope.toml during assembly.

## Chassis vs Instance Coupling Inventory

Coupling status after the Chassis Abstraction Evolution (May 17, 2026):

| Coupling Point | Status | Resolution |
|----------------|--------|------------|
| `MODULE_DISPATCH` | **RESOLVED** | Centralized in `registry.rs` — compiled fallback, scope.toml `[[module]]` is primary |
| `LTEE_MODULES` | **RESOLVED** | Moved to `registry.rs` as fallback; `load_module_table()` reads scope.toml first |
| `LTEE_NOTEBOOKS` | **RESOLVED** | Moved to `registry.rs`; scope.toml `[[module]]` `tier1_notebook` field is primary |
| `MODULE_DISPATCH` (parity) | **RESOLVED** | `parity.rs` imports from `registry.rs` — no duplication |
| `LTEE_MODULES` usage across 6 files | **RESOLVED** | validate, parity, ops, chaos, deploy_test, visualize all import from `registry.rs` |
| `module_name_matches()` | **RESOLVED** | `registry::module_name_matches()` does registry lookup, not hardcoded match |
| `.biomeos-spore` template | **RESOLVED** | Generated from scope.toml during `litho assemble` |
| `viz/` in litho-core | **RESOLVED** | Moved to `ltee-cli/src/viz/` (instance layer) |
| Graph/target paths | **RESOLVED** | `guidestone.graph_file` and `guidestone.targets_file` in scope.toml |
| Braid accessions | **RESOLVED** | Derived from `data.toml` `sra_accession` fields |
| `strip_prefix("ltee-")` | **RESOLVED** | `derive_logical_name()` handles `ltee-`, `milc-`, `lattice-` prefixes |
| `ltee-*` Cargo deps | **Remaining** | Compile-time module crate imports — feature flags per instance (future) |
| `ltee-cli` naming | **Remaining** | Crate named for LTEE — cosmetic rename to `litho-cli` (future) |

## Per-Module Contract

Every module binary MUST:
1. Accept `--data-dir`, `--expected`, and `--json` flags
2. Expose `lib.rs::run_validation(data_dir, expected, max_tier) -> ModuleResult`
3. Return `ModuleResult` JSON when `--json` is set
4. Exit 0 (pass), 1 (fail), or 2 (skip)
5. Use scientifically justified tolerances — either embedded in expected-values
   JSON (modules 6, 7) or as named constants matching `artifact/tolerances.toml`
6. Reference source data by dataset ID from `data.toml`
7. Be statically linked (musl) with zero runtime dependencies
8. Honor `max_tier` parameter. All modules honor `max_tier`.

`artifact/tolerances.toml` is the centralized reference for all tolerance values
and their scientific justifications. Module binaries may read tolerances from
expected-values JSON or define them as compile-time constants; either way, the
values MUST match the TOML reference and include justification in comments.

## Validation Targets

14 quantitative targets (T01–T14) map published claims to tolerance bands.
13 active, 1 pending upstream (T06 — HMM/ESN classifier ≥95%).
Defined in `data/targets/ltee_validation_targets.toml`.

Expected values per module in `validation/expected/module{N}_{name}.json`.
Each carries `doi`, `source_figures`, and tolerance specifications.

## Test Coverage

192 tests across 10 crates:
- `litho-core`: 47 unit tests (discovery, provenance, validation, braid, spore, scope, harness, env_vars, tolerance, manifest, stats)
- `pseudospore-core`: 45 unit tests (manifest, validation, tarball, braid envelope, receipts, scope, domain profile, envelope load+validate, error types)
- `ltee-cli`: 58 unit + integration tests (lib 31, cli_integration 20, integration 7)
- Module crates: 42 combined (fitness 9, mutations 9, anderson 7, biobricks 5, breseq 4, alleles 4, citrate 4)
