# Validation Data Stream Standard

**Version**: 1.0.0
**Authority**: lithoSpore team (pseudoSpore pipeline owner)
**Consumers**: All science spring teams producing pseudoSpores
**Date**: Jul 21, 2026

---

## Purpose

This standard defines the data format that spring teams must produce to
populate their pseudoSpore's `validation.json`. Following this contract
enables the automated promotion pipeline:

```
scope.toml → init-validation → populate-validation → audit → promote-spore → pack
```

## Module Results JSON Schema

Spring team validators must produce a JSON file containing an array of
module results. This file is consumed by `litho populate-validation`.

### Schema

```json
[
  {
    "name": "module_name",
    "status": "PASS",
    "checks_total": 10,
    "checks_passed": 10,
    "checks": [
      {
        "name": "check_name",
        "expected": 5.57,
        "observed": 5.52,
        "tolerance": 0.1,
        "unit": "kJ/mol",
        "status": "PASS"
      }
    ],
    "errata": []
  }
]
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | **yes** | Must match a `[[module]]` name in scope.toml |
| `status` | string | **yes** | `PASS`, `FAIL`, `SKIP`, or `PENDING` |
| `checks_total` | u32 | recommended | Total number of checks run for this module |
| `checks_passed` | u32 | recommended | Number of checks that passed |
| `checks` | array | recommended | Individual check records (see below) |
| `errata` | array | optional | Known discrepancies or caveats |

### Check Record Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | **yes** | Descriptive check name |
| `status` | string | **yes** | `PASS` or `FAIL` |
| `expected` | number/string | recommended | Expected value (from literature/control) |
| `observed` | number/string | recommended | Computed value from spring validator |
| `tolerance` | number | recommended | Acceptable deviation |
| `unit` | string | recommended | Physical unit or dimensionless label |
| `note` | string | optional | Additional context |

### Status Values

| Status | Meaning | Promotion eligible? |
|--------|---------|-------------------|
| `PASS` | All checks within tolerance | Yes |
| `FAIL` | One or more checks outside tolerance | No |
| `SKIP` | Module not applicable to this spring | No (remove from scope) |
| `PENDING` | Not yet validated | No |

## Scope Contract

Each spring declares its modules in `scope.toml` at the spring root:

```toml
[artifact]
name = "springName-Domain-Artifact"
version = "1.0.0"
type = "pseudoSpore"

[[modules]]
name = "module_name"
entity_group = "entity_group_name"
computation = ["pipeline_a", "pipeline_b"]
```

### Rules

1. **Module names are stable identifiers.** Once declared in scope.toml and
   emitted as a pseudoSpore, module names must not change without version bump.

2. **Every declared module must be validated.** The `promote-spore` command
   will reject promotion if any module lacks a `PASS` status.

3. **Entity groups map to data directories.** The `entity_group` field should
   correspond to a subdirectory in `data/` containing the module's source data.

4. **Computations are informational.** The `computation` array describes the
   pipelines within a module but is not enforced by the promotion pipeline.

## Spring Team Workflow

### 1. Declare modules (once)

Edit your spring's root `scope.toml` to declare all modules with their
entity groups and computation pipelines.

### 2. Run your validators

Execute your spring's test/validation suite. Collect results into a JSON
file matching the schema above. Example:

```bash
# Spring team runs their validator
cargo test --lib -- --format json > raw_results.json

# Or a custom validator script
./validate_modules.sh --output results.json
```

### 3. Populate validation

```bash
# From results file
litho populate-validation /path/to/pseudospore --results results.json

# Or inline for quick updates
litho populate-validation /path/to/pseudospore \
  --module et0_reference=PASS \
  --module soil_physics=PASS \
  --module water_balance=FAIL
```

### 4. Promote

```bash
# Verify all modules pass
litho audit --path /path/to/pseudospore --verbose

# Promote to COMPLETE
litho promote-spore /path/to/pseudospore --artifact-root /path/to/lithoSpore

# Pack for redistribution
litho pack-pseudospore /path/to/pseudospore
```

## Versioning

When a spring team updates their science (new data, corrected algorithms,
additional checks), they should:

1. Bump `version` in scope.toml
2. Re-emit the pseudoSpore
3. Re-run validators
4. Re-populate and re-promote

The registry tracks version history. Previous versions remain in the
fossil record.

## Tolerances

Numerical checks should use named tolerances with scientific justification.
See `tolerances.toml` in any emitted pseudoSpore for the pattern:

```toml
[[tolerance]]
name = "fes_rmsd"
value = 2.0
unit = "kJ/mol"
justification = "Standard convergence threshold for metadynamics FES reconstruction"
```

Inline magic numbers in check records are acceptable for initial population
but should evolve to reference named tolerances.

---

**Reference implementation**: See `crates/pseudospore-core/src/validation.rs`
for the `ValidationDoc`, `ValidationModule`, and `ValidationSummary` types.
