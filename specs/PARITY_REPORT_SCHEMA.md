# Cross-Tier Parity Report — Ecosystem Standard

**Version:** 1.0
**Origin:** lithoSpore (`litho parity`)
**Adopted by:** airSpring (3 validators), primalSpring `VALIDATION_TIERS.md`
**Status:** Published for ecosystem consumption

---

## Purpose

A `ParityReport` proves mathematical stability between two independent
implementations of the same computation. lithoSpore uses it to validate
Python (Tier 1) against Rust (Tier 2) for all 7 LTEE modules. Other
products (projectFOUNDATION barraCuda benchmarks, airSpring validators)
can adopt the same format.

## JSON Schema

```json
{
  "artifact": "ltee-guidestone",
  "version": "0.1.0",
  "modules": [
    {
      "module": "ltee-fitness",
      "tier1_status": "Pass",
      "tier2_status": "Pass",
      "tier1_checks": 8,
      "tier2_checks": 8,
      "tier1_passed": 8,
      "tier2_passed": 8,
      "parity": "MATCH"
    },
    {
      "module": "ltee-mutations",
      "tier1_status": "Pass",
      "tier2_status": "Pass",
      "tier1_checks": 7,
      "tier2_checks": 7,
      "tier1_passed": 7,
      "tier2_passed": 7,
      "parity": "MATCH"
    }
  ],
  "parity_pass": true
}
```

## Field Definitions

### Top Level

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `artifact` | string | YES | Scope name from `scope.toml` |
| `version` | string | YES | Artifact/package version |
| `modules` | array | YES | Per-module parity results |
| `parity_pass` | boolean | YES | `true` if no modules have DIVERGENCE |

### Per-Module (`modules[]`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `module` | string | YES | Module name (e.g., `ltee-fitness`) |
| `tier1_status` | enum | YES | `Pass`, `Fail`, or `Skip` |
| `tier2_status` | enum | YES | `Pass`, `Fail`, or `Skip` |
| `tier1_checks` | u32 | YES | Total checks in Tier 1 |
| `tier2_checks` | u32 | YES | Total checks in Tier 2 |
| `tier1_passed` | u32 | YES | Passed checks in Tier 1 |
| `tier2_passed` | u32 | YES | Passed checks in Tier 2 |
| `parity` | enum | YES | `MATCH`, `DIVERGENCE`, or `SKIPPED` |

### Parity Status Rules

| Condition | Result |
|-----------|--------|
| Either tier is `Skip` | `SKIPPED` |
| Both tiers `Pass` or both `Fail` with same status | `MATCH` |
| Tier 1 and Tier 2 disagree on pass/fail | `DIVERGENCE` |

## Rust Types

Defined in `litho-core::validation`:

```rust
#[derive(Serialize, Deserialize)]
pub struct ParityReport {
    pub artifact: String,
    pub version: String,
    pub modules: Vec<ParityResult>,
    pub parity_pass: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ParityResult {
    pub module: String,
    pub tier1_status: ValidationStatus,
    pub tier2_status: ValidationStatus,
    pub tier1_checks: u32,
    pub tier2_checks: u32,
    pub tier1_passed: u32,
    pub tier2_passed: u32,
    pub parity: ParityStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParityStatus {
    Match,
    Divergence,
    Skipped,
}
```

## Usage

```bash
# Human-readable output
litho parity

# JSON output (pipe to downstream)
litho parity --json
```

Exit code: 0 = all MATCH, 1 = any DIVERGENCE.

## Adoption Guide

To produce a `ParityReport` for your product:

1. Run your Tier 1 (Python/baseline) and Tier 2 (Rust/compiled) implementations
2. Compare per-module: same pass/fail? same check counts?
3. Emit the JSON schema above
4. Use `parity_pass` as a CI gate

The format is product-agnostic — replace `ltee-*` module names with your
own. The `artifact` field identifies the source product.
