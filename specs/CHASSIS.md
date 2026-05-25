# lithoSpore Chassis Standard v0.1

**Date:** May 25, 2026  
**Status:** Draft  
**Scope:** Defines the lithoSpore deployment chassis — full self-contained reproducibility.

---

## Overview

A **lithoSpore** is a self-contained deployment artifact that carries everything
needed to reproduce a computation from scratch — binaries, runtime, data, proof,
and tooling. It extends the pseudoSpore (proof layer) with a runtime layer.

```
pseudoSpore = proof (configs + data + outputs + provenance)
lithoSpore  = proof + deployment (runtime + binaries + force fields)
```

The chassis pattern: a lithoSpore **wraps** a pseudoSpore as its proof core,
then adds the deployment layer on top. Promotion is additive — nothing in the
pseudoSpore changes when it becomes a lithoSpore module.

---

## Chassis Layout

```
lithoSpore_<name>_v<X.Y.Z>/
├── guidestone.toml              # REQUIRED — lithoSpore identity + module registry
├── proof/                       # REQUIRED — embedded pseudoSpore (proof layer)
│   ├── scope.toml
│   ├── index_map.toml
│   ├── TRANSLATE.md
│   ├── validation.json
│   ├── receipts/
│   │   ├── environment.toml
│   │   └── checksums.blake3
│   ├── provenance/
│   │   └── ferment_transcript.json
│   ├── configs/
│   │   └── <module>/
│   ├── data/
│   │   └── <module>/
│   ├── outputs/
│   │   └── <module>/
│   └── README.md
├── runtime/                     # REQUIRED — deployment layer
│   ├── bin/                     # Pre-compiled tools
│   │   ├── litho               # lithoSpore CLI (self-verification)
│   │   ├── plumed              # Domain tool (if applicable)
│   │   └── cazyme-fel          # Tier 2 Rust validator
│   ├── env/                    # Python environment snapshot
│   │   └── requirements.txt    # Pinned versions for Tier 1
│   ├── forcefields/            # Domain-specific data files
│   │   └── charmm36m/
│   └── scripts/                # Automation
│       ├── validate.sh         # Run full validation chain
│       ├── reproduce.sh        # Re-run from raw inputs
│       └── translate.sh        # Generate domain-frame configs
├── expected/                    # REQUIRED — validation targets
│   └── <module>/
│       └── expected.json       # Named tolerance + expected values
├── tolerances.toml              # REQUIRED — acceptance criteria
└── README.md                    # REQUIRED — deployment guide
```

---

## Key Differences from pseudoSpore

| Aspect | pseudoSpore | lithoSpore |
|--------|------------|------------|
| Purpose | Prove computation | Deploy + prove computation |
| Self-sufficient | No (needs external tools) | Yes (carries all tools) |
| Size | KB–MB | MB–GB (target: USB stick) |
| Verification | Manual (user runs commands) | Automated (`./litho validate`) |
| Translation | `litho translate-config` (external) | Built-in (`./runtime/scripts/translate.sh`) |
| Reproducibility | Recipe in TRANSLATE.md | One-command: `./runtime/scripts/reproduce.sh` |

---

## guidestone.toml Schema

```toml
[identity]
name = "CAZyme-FEL"
version = "1.0.0"
type = "lithoSpore"
date = "2026-05-25"
origin = "ecoPrimals/springs/hotSpring"
pseudospore_version = "0.9.0"

[deployment]
target_size_gb = 16
platform = ["linux-x86_64"]
gpu_required = false
min_ram_gb = 8

[[module]]
name = "xylose-puckering-fel"
tier_0_tool = "GROMACS 2026.0 + PLUMED 2.9.2"
tier_1_notebook = "runtime/env/puckering_fel.py"
tier_2_binary = "runtime/bin/cazyme-fel"
expected = "expected/xylose-puckering-fel/expected.json"
tolerance = "parity_rmsd_kj < 2.0"

[source]
repo = "git@github.com:sporeGarden/hotSpring.git"
commit = ""
branch = "main"
proof_path = "proof/"
```

---

## Promotion: pseudoSpore → lithoSpore

```
pseudoSpore_hotSpring-CAZyme-FEL_v0.9.0/
  ↓ litho promote --add-runtime
lithoSpore_CAZyme-FEL_v1.0.0/
  ├── proof/   ← pseudoSpore contents moved here verbatim
  ├── runtime/ ← binaries compiled, env captured
  └── expected/ ← validation targets generated
```

The `litho promote` command:
1. Copies pseudoSpore contents into `proof/`
2. Compiles Tier 2 binaries (`cargo build --release`) → `runtime/bin/`
3. Snapshots Python environment → `runtime/env/`
4. Copies force field files → `runtime/forcefields/`
5. Generates `expected/*.json` from `proof/outputs/`
6. Computes tolerances from parity results
7. Generates `guidestone.toml` from `proof/scope.toml`
8. Writes automation scripts
9. Final BLAKE3 seal of entire artifact

---

## Verification Tiers (inside lithoSpore)

```bash
# Self-test: verify artifact integrity
./runtime/bin/litho verify --artifact-root .

# Tier 1: Python validation (no compilation needed)
./runtime/scripts/validate.sh --tier 1

# Tier 2: Rust validation (pre-compiled)
./runtime/scripts/validate.sh --tier 2

# Full: Tier 0 + 1 + 2 (requires GROMACS installed externally)
./runtime/scripts/validate.sh --tier 0
```

---

## Translation in Chassis

The lithoSpore chassis handles index translation automatically:

```bash
# Generate domain-frame plumed.dat for expert review
./runtime/bin/litho translate-config \
  --index-map proof/index_map.toml \
  --config proof/configs/enzyme-bound-puckering/plumed.dat \
  --frame domain

# Generate computation-frame for runtime (default in proof/configs/)
./runtime/bin/litho translate-config \
  --index-map proof/index_map.toml \
  --config proof/configs/enzyme-bound-puckering/plumed.dat \
  --frame computation
```

The `translate.sh` script wraps this for all configs:

```bash
./runtime/scripts/translate.sh --frame domain --output domain-configs/
```

---

## Design Principles

1. **Proof is immutable.** The `proof/` directory is the verbatim pseudoSpore — never modified after promotion.
2. **Runtime is replaceable.** Binaries can be rebuilt from source; the `proof/` layer is the truth.
3. **USB-deployable.** Target: fits on a 16 GB USB stick, runs on any Linux x86_64 machine with a GPU.
4. **Self-verifying.** The embedded `litho` binary can verify the entire artifact without external tools.
5. **Domain-first.** All human-facing output uses domain-standard numbering. Computation indices are an internal implementation detail.
6. **Additive promotion.** Going from pseudoSpore to lithoSpore only adds files — never modifies the proof layer.
