# lithoSpore Module Specifications

## Module Registry

| # | Crate | Binary | Paper | Tier 2 Spring Sources | Tier 1 Python |
|---|-------|--------|-------|----------------------|---------------|
| 1 | `ltee-fitness` | `ltee-fitness` | Wiser 2013 (B2) | groundSpring, wetSpring | `notebooks/module1_fitness/` |
| 2 | `ltee-mutations` | `ltee-mutations` | Barrick 2009 (B1) | groundSpring, neuralSpring | `notebooks/module2_mutations/` |
| 3 | `ltee-alleles` | `ltee-alleles` | Good 2017 (B3) | neuralSpring, groundSpring | `notebooks/module3_alleles/` |
| 4 | `ltee-citrate` | `ltee-citrate` | Blount 2008/2012 (B4) | neuralSpring, groundSpring | `notebooks/module4_citrate/` |
| 5 | `ltee-biobricks` | `ltee-biobricks` | Burden 2024 (B6) | neuralSpring, groundSpring | `notebooks/module5_biobricks/` |
| 6 | `ltee-breseq` | `ltee-breseq` | Tenaillon 2016 (B7) | wetSpring, groundSpring | `notebooks/module6_breseq/` |
| 7 | `ltee-anderson` | `ltee-anderson` | Anderson-QS (new) | hotSpring, groundSpring | `notebooks/module7_anderson/` |

## Shared Infrastructure

- `litho-core`: validation types, tolerance framework, provenance chain, liveSpore,
  capability-based primal discovery, shared statistics (`pearson_r`), validation
  harness (`skip`, `load_expected`, `dispatch_python`, `output_and_exit`), and
  visualization adapters (`viz` — DataBinding for all 7 modules + 7 Barrick baselines)
- `ltee-cli`: unified `litho` binary with subcommands (validate/refresh/status/spore/visualize)

## Per-Module Contract

Every module binary MUST:
1. Accept `--data-dir`, `--expected`, and `--json` flags
2. Return `ModuleResult` JSON when `--json` is set
3. Exit 0 (pass), 1 (fail), or 2 (skip)
4. Use scientifically justified tolerances — either embedded in expected-values
   JSON (modules 6, 7) or as named constants matching `artifact/tolerances.toml`
5. Reference source data by dataset ID from `data.toml`
6. Be statically linked (musl) with zero runtime dependencies

`artifact/tolerances.toml` is the centralized reference for all tolerance values
and their scientific justifications. Module binaries may read tolerances from
expected-values JSON or define them as compile-time constants; either way, the
values MUST match the TOML reference and include justification in comments.

## Validation Targets

Quantitative claims are defined per-module in the expected-values JSON files
under `validation/expected/`. Each file documents the paper targets, tolerance
bands, and upstream spring provenance for that module's checks.
