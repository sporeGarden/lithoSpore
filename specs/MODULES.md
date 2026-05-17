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

| # | Module | Tier 1 (Python) | Tier 2 (Rust) | Tier 3 (Primal) | `max_tier` honored? | Parity testable? |
|---|--------|-----------------|---------------|-----------------|---------------------|------------------|
| 1 | fitness | PASS | PASS (8/8) | Via `try_record_tier3` | Yes | Yes |
| 2 | mutations | PASS | PASS (7/7) | Via `try_record_tier3` | Yes | Yes |
| 3 | alleles | PASS | PASS (20/20) | Via `try_record_tier3` | **Ignored** | Yes |
| 4 | citrate | PASS | PASS (11/11) | Via `try_record_tier3` | **Ignored** | Yes |
| 5 | biobricks | PASS | PASS (6/6) | Via `try_record_tier3` | Yes | Yes |
| 6 | breseq | PASS | PASS (16/16) | Via `try_record_tier3` | Yes | Yes |
| 7 | anderson | PASS | PASS (7/7) | Via `try_record_tier3` | Yes | Yes |

**Total**: 7/7 Tier 2 PASS, 75/75 checks. 7/7 Tier 1 PASS.
Cross-tier parity testable for all 7 modules.

## Shared Infrastructure

- **`litho-core`** (chassis — domain-agnostic):
  validation types (`ModuleResult`, `ValidationReport`, `Tier3Session`, `ParityReport`),
  tolerance framework, provenance chain + JSON-RPC client for trio,
  liveSpore tracking, capability-based primal discovery (env → UDS → TURN → standalone),
  `announce_self()` + `query_capabilities()` for Wave 20,
  shared statistics (`pearson_r`), validation harness (`skip`, `load_expected`, `dispatch_python`),
  scope parser (`ScopeManifest`), data manifest, graph checks,
  visualization adapters (`viz` — DataBinding for all 7 modules + 7 Barrick baselines)

- **`ltee-cli`** (instance wiring — LTEE-specific):
  unified `litho` binary with 15 subcommands:
  `validate`, `parity`, `refresh`, `status`, `spore`, `verify`, `visualize`,
  `self-test`, `tier`, `assemble`, `fetch`, `chaos-test`, `deploy-test`,
  `deploy-report`, `grow`

## Chassis vs Instance Coupling Inventory

The following compile-time coupling points tie `ltee-cli` to the LTEE instance.
These are the specific items that must evolve for domain-agnostic support:

| Coupling Point | File | What It Does | Agnostic Path |
|----------------|------|-------------|---------------|
| `MODULE_DISPATCH` | `validate.rs:139-147` | Maps binary names → `run_validation()` entry points | Load from scope.toml or trait registry |
| `LTEE_MODULES` | `validate.rs:14-22` | Fallback module table (name, binary, data, expected) | Already superseded by scope.toml loader |
| `LTEE_NOTEBOOKS` | `validate.rs:24-32` | Maps module names → Python notebook paths | Move to scope.toml `[module.tier1]` |
| `MODULE_DISPATCH` (parity) | `parity.rs:11-20` | Duplicates dispatch table for parity | Share with validate.rs |
| `LTEE_MODULES` (parity) | `parity.rs:34` | Iterates hardcoded module list | Use scope-driven loader |
| `module_name_matches()` | `validate.rs:437-448` | LTEE-specific name mapping | Derive from scope.toml module metadata |
| `ltee-*` Cargo deps | `ltee-cli/Cargo.toml` | Compile-time module crate imports | Feature flags per instance |

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
8. Honor `max_tier` parameter (modules 3, 4 currently do not — known gap)

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

117 tests across 9 crates:
- `litho-core`: 54 unit tests (discovery, provenance, validation, spore, viz, graph, etc.)
- `ltee-cli`: 16 unit + 20 integration tests
- Module crates: 27 combined (fitness 6, mutations 6, anderson 5, biobricks 4, alleles 2, citrate 2, breseq 2)
