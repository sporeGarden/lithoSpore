# pseudoSpore Standard v1.0

**Date:** May 24, 2026  
**Status:** Active  
**Scope:** Defines the pseudoSpore deployment class — a lightweight, braid-first spore for computation-heavy springs.

---

## Overview

A **pseudoSpore** is a lightweight reproducibility artifact that proves a computation happened, what it produced, and how to reproduce it — without carrying the runtime or raw inputs needed to actually re-execute.

It sits between LiveSpore (has liveSpore.json + refresh) and full lithoSpore (has binaries + data + runtime):

| Class | Carries | Self-Sufficient? | Size |
|-------|---------|-----------------|------|
| ColdSpore | `.biomeos-spore` marker, frozen data | No | < 1 KB |
| LiveSpore | + `liveSpore.json` + `./refresh` | Partially | ~ KB |
| **pseudoSpore** | + braids + receipts + outputs + configs | No (proves, doesn't execute) | KB–MB |
| lithoSpore | + Python runtime + Rust binaries + full data | Yes | MB–GB |

The metaphor: **the spore can't carry the mountain, but it proves the mountain was climbed and shows the view from the top.**

---

## Required Structure

```
pseudoSpore_<name>_v<X.Y.Z>/
├── scope.toml                    # REQUIRED — birth certificate
├── validation.json               # REQUIRED — machine-readable results
├── receipts/                     # REQUIRED — compute provenance
│   ├── environment.toml          # REQUIRED — hardware, software, OS
│   └── checksums.blake3          # REQUIRED — BLAKE3 of all outputs
├── provenance/                   # REQUIRED — at minimum ferment_transcript
│   └── ferment_transcript.json   # REQUIRED — FermentBraid wire format
└── README.md                     # REQUIRED — human-readable summary
```

## Optional Structure

```
pseudoSpore_<name>_v<X.Y.Z>/
├── index_map.toml                # RECOMMENDED — domain ↔ computation translation
├── TRANSLATE.md                  # RECOMMENDED — human-readable legend
├── receipts/
│   └── compute_log.toml          # OPTIONAL — wall time, commands, GPU hours
├── provenance/
│   ├── dag.json                  # OPTIONAL — rhizoCrypt DAG (pseudo or live)
│   ├── spine.json                # OPTIONAL — loamSpine ledger (pseudo or live)
│   └── braids/                   # OPTIONAL — additional sweetGrass braids
│       └── <name>.json
├── outputs/                      # OPTIONAL — science results
│   └── <module_name>/
│       └── <result_files>
├── configs/                      # OPTIONAL — reproducibility chain
│   └── <module_name>/
│       └── <input_configs>
├── data/                         # RECOMMENDED — raw compute outputs (derivation proof)
│   └── <module_name>/
│       └── <raw_outputs>         # e.g. HILLS, trajectories, topology
├── figures/                       # RECOMMENDED — visual evidence layer
│   └── <name>.png                # Publication-quality plots (PNG, ≤ 500KB each)
├── AUDIT.md                      # OPTIONAL — verification audit trail
└── RELEASE.md                    # OPTIONAL — release notes / caveats
```

---

## Schema: scope.toml

Reuses the lithoSpore `scope.toml` schema with `type = "pseudoSpore"` in the artifact header:

```toml
[artifact]
name = "hotSpring-CAZyme-FEL"
version = "0.6.0"
type = "pseudoSpore"                 # Distinguishes from full lithoSpore
date = "2026-05-24"
origin = "ecoPrimals/springs/hotSpring"
experiment = 220
license = "AGPL-3.0-or-later"

[target]
paper_doi = "10.1039/C4SC02240H"
paper_title = "Title of the target paper"
paper_authors = "Author, List"
paper_year = 2015

[[module]]
name = "xylose-puckering-fel"
status = "PASS"                      # PASS | FAIL | IN_FLIGHT | SKIP
checks = 5
description = "Free β-D-xylopyranose ring puckering FEL"
# Optional fields inherited from lithoSpore:
# binary, data_dir, expected, tier1_notebook

[evolution]
tier_0 = "Industry control description"
tier_1 = "Python sovereign implementation"
tier_2 = "Rust sovereign implementation"
tier_3 = "NUCLEUS IPC composition (future)"

[source]
repo = "git@github.com:org/repo.git"
commit = "abc123"
branch = "main"
```

### Key differences from full lithoSpore scope.toml:
- `[artifact]` section replaces `[guidestone]`
- `type = "pseudoSpore"` is mandatory
- `[target]` replaces `target = "..."` (more structured)
- `[[module]]` entries have `status` and `checks` fields (self-reported results)
- `[source]` includes `commit` for exact reproducibility

---

## Schema: validation.json

Machine-readable validation results with inline errata:

```json
{
  "artifact": "hotSpring-CAZyme-FEL",
  "version": "0.6.0",
  "date": "2026-05-24",
  "modules": [
    {
      "name": "xylose-puckering-fel",
      "status": "PASS",
      "checks_total": 5,
      "checks_passed": 5,
      "checks": [
        {
          "name": "basin_count",
          "expected": 3,
          "observed": 3,
          "status": "PASS"
        }
      ],
      "errata": [
        {
          "severity": "medium",
          "finding": "Description of issue",
          "action": "Required follow-up"
        }
      ]
    }
  ],
  "summary": {
    "modules_total": 3,
    "modules_pass": 2,
    "modules_in_flight": 1
  }
}
```

---

## Schema: receipts/environment.toml

```toml
[hardware]
hostname = "strandGate"
cpu = "Intel i9-12900K"
ram_gb = 64
gpu = "RTX 3090 (GA102, 24GB GDDR6X)"

[software]
os = "Pop!_OS 22.04 (kernel 6.17.9)"
gromacs = "2026.0"
plumed = "2.9.2"
python = "3.13.1"
rust = "1.87.0"
conda_env = "gromacs-fel"

[timestamps]
started = "2026-05-24T08:00:00Z"
completed = "2026-05-24T14:00:00Z"
```

---

## Schema: receipts/checksums.blake3

One line per file, BLAKE3 hash followed by relative path (from pseudoSpore root):

```
a1b2c3d4...  outputs/xylose-puckering-fel/fes_theta.dat
e5f6a7b8...  outputs/ala-dipeptide-fel/fes_2d.dat
c9d0e1f2...  provenance/ferment_transcript.json
```

Format matches the output of `b3sum --no-names` with paths appended.

---

## Schema: receipts/compute_log.toml (optional)

```toml
[summary]
wall_time_hours = 6.5
gpu_hours = 4.2
total_commands = 12

[[command]]
step = 1
tool = "gmx grompp"
args = "-f md_meta.mdp -c npt.gro -p topol.top -o md_meta.tpr"
wall_seconds = 3
module = "xylose-puckering-fel"

[[command]]
step = 2
tool = "gmx mdrun"
args = "-deffnm md_meta -plumed plumed.dat -nsteps 5000000"
wall_seconds = 14400
gpu = true
module = "xylose-puckering-fel"
```

---

## Schema: provenance/ferment_transcript.json

FermentBraid wire format (already defined in lithoSpore ecosystem):

```json
{
  "dataset_id": "cazyme_fel_v0.6.0",
  "spring": "hotSpring",
  "spring_version": "0.6.32",
  "braid_id": "braid-hotspring-cazyme-fel-20260524",
  "dag_session_id": "dag-hotspring-cazyme-001",
  "dag_merkle_root": "blake3:...",
  "spine_id": "spine-hotspring-cazyme-001",
  "timestamp": "2026-05-24T14:00:00Z",
  "computation": {
    "tool": "GROMACS 2026.0 + PLUMED 2.9.2",
    "substrate": "GPU (RTX 3090)",
    "input_hashes": { "HILLS": "blake3:..." },
    "output_hashes": { "fes_theta.dat": "blake3:..." },
    "modules_complete": 2,
    "modules_in_flight": 1
  }
}
```

---

## Visual Evidence Layer (`figures/`)

A pseudoSpore SHOULD include a `figures/` directory with publication-quality
visualizations that make the science self-evident. Think of the pseudoSpore as
a **poster presentation** — the data proves it, the figures sell it.

**Requirements**:
- PNG format, 300 DPI, ≤ 500KB per image (keep tarball lean)
- SVG optional for vector reproduction
- Figures MUST be derivable from `outputs/` data (no external sources)
- Comparison panels preferred (e.g., free vs enzyme-bound side-by-side)

**Recommended figures for FEL studies**:
- 1D energy profile comparison (all systems overlaid)
- 2D heatmaps (one per system)
- Combined comparison panel (side-by-side with shared colorbar)

**Generation**: `litho emit-pseudospore` auto-generates figures from `outputs/`
data when matplotlib is available (see `emit_pseudospore/figures.rs`). For LTEE
module figures, re-run the Tier 1 notebooks in `notebooks/`.

---

## Validation Rules

A pseudoSpore is **VALID** if:

1. `scope.toml` exists and parses with `type = "pseudoSpore"`
2. `validation.json` exists and parses with at least one module
3. `receipts/environment.toml` exists and has `[hardware]` + `[software]`
4. `receipts/checksums.blake3` exists and all referenced files are present
5. `provenance/ferment_transcript.json` exists and has `dataset_id` + `spring`
6. `README.md` exists and is non-empty

A pseudoSpore is **VERIFIED** if additionally:

7. All BLAKE3 checksums in `receipts/checksums.blake3` match actual file hashes
8. All braids in `provenance/braids/` parse as valid FermentBraid JSON

A pseudoSpore is **COMPLETE** if additionally:

9. All modules in `scope.toml` have status `PASS` or `SKIP` in `validation.json`
10. No modules have status `IN_FLIGHT`

---

## Promotion Path: pseudoSpore to lithoSpore Module

A pseudoSpore that gains:
1. A **Python baseline** (Tier 1) producing numerical results from the same inputs
2. A **Rust implementation** (Tier 2) matching the Python baseline at tolerance
3. **Expected values JSON** compatible with lithoSpore's `validation/expected/` format
4. **Named tolerances** in `artifact/tolerances.toml` format

...becomes a candidate for full lithoSpore module integration:

```
pseudoSpore (proof + receipt)
  → Tier 1 Python implementation (algorithm validated)
    → Tier 2 Rust crate (staging/*)
      → lithoSpore [[module]] entry (full integration)
```

The `[evolution]` section in `scope.toml` tracks which tiers exist.

---

## CLI Integration

### `litho ingest-pseudospore <path>`

Validates a pseudoSpore and imports it:
- Checks VALID + VERIFIED status
- Copies braids to `provenance/braids/`
- Registers in `pseudospores/registry.toml`
- Reports status

### `litho emit-pseudospore --name <name> --version <ver> --output <dir>`

Assembles a pseudoSpore from current module state:
- Generates directory structure
- Computes BLAKE3 checksums
- Captures environment
- Generates README from scope metadata

---

## Schema: index_map.toml (recommended)

Machine-readable translation between domain-standard identifiers (what a domain expert uses)
and computation-layer identifiers (what the runtime requires). Eliminates "reindexing as
mental effort" — the artifact handles the translation, not the reader.

```toml
[meta]
pdb = "2D24"                           # Domain reference structure
ring_order = ["C1", "C2", "C3", "C4", "C5", "O5"]

[systems.<system_name>]
description = "Human-readable system description"
structure_source = "Where the structure came from"
total_atoms = 92745
rosetta_stone = "data/<module>/npt.gro"  # File that maps between layers

[systems.<system_name>.ring]
C1 = { domain = 6599, computation = 6278 }
C2 = { domain = 6600, computation = 6286 }
# ...
```

**Principle**: `configs/*/plumed.dat` uses computation indices (runtime requirement)
but carries inline `# INDEX TRANSLATION` comments with domain equivalents.
The `index_map.toml` is the machine-parseable version for tooling.

**Future (lithoSpore chassis)**: The `litho` CLI reads `index_map.toml` and can
generate configs in EITHER frame — domain-expert inputs PDB numbering,
lithoSpore handles the reindexing internally.

---

## Schema: data/ (recommended)

Raw compute outputs that prove derivation. Each file in `outputs/` should be
derivable from corresponding files in `data/` using the command in `TRANSLATE.md`.

```
data/
├── <module_name>/
│   ├── HILLS[_2d]        # Metadynamics: raw Gaussian depositions
│   └── npt.gro           # Topology: atom index Rosetta stone
```

**Verification contract**: `sum_hills(data/HILLS) == outputs/fes.dat`

Without `data/`, the pseudoSpore requires trust. With `data/`, anyone can
re-derive outputs independently. This is the difference between "trust me"
and "verify yourself."

---

## Schema: liveSpore.json (unified)

The liveSpore.json file uses a unified schema that combines emit-time metadata
(provenance of who created the spore) with an append-only validation journal
(where and when it was validated). Both lithoSpore and nest-validate write this
same shape.

```json
{
  "envelope": {
    "artifact": "hotSpring-CompChem-GuideStone",
    "version": "1.6.1",
    "emit_timestamp": "2026-05-27T11:42:17Z",
    "emit_host": "pop-os",
    "git_sha": "6c48a43",
    "tool": "litho emit-pseudospore",
    "tool_version": "2.3.0",
    "integrity": "BLAKE3 (data.toml)",
    "provenance_chain": {
      "parent": "pseudoSpore_hotSpring-CompChem-GuideStone_v1.6.0",
      "parent_merkle": "cbf908fb...",
      "evolution": "v1.5.0 → v1.6.0 → v1.6.1"
    },
    "software": {
      "gromacs": "2026.0",
      "plumed": "2.10"
    }
  },
  "validations": [
    {
      "timestamp": "2026-05-27T14:30:00Z",
      "hostname_hash": "blake3(hostname)",
      "arch": "x86_64",
      "os": "linux",
      "tier_reached": 2,
      "modules_passed": 7,
      "modules_total": 8,
      "runtime_ms": 1234
    }
  ]
}
```

### Rules

1. `envelope` is written once at emit time by the tool that creates the pseudoSpore
2. `validations` is initialized as `[]` at emit time
3. Each successful `./validate` or `litho audit` appends one entry to `validations`
4. The BLAKE3 hash in `data.toml` covers the initial state (envelope + empty validations)
5. Validation appends are expected to change the file hash — this is intentional
6. The `envelope.tool` field identifies which toolchain emitted the spore

### Migration from Legacy Schemas

- **lithoSpore legacy** (`[]`): treated as `{"envelope": {}, "validations": []}`
- **hotSpring legacy** (`{"liveSpore": {...}, ...}`): `liveSpore` maps to `envelope`,
  `provenance_chain` and `software` nest inside `envelope`

Tools MUST accept all three shapes on read and MUST write the unified shape on
new emissions.

---

## Design Principles

1. **Braids are the core truth.** The pseudoSpore is primarily a braid carrier — everything else supports the braid's claims.
2. **Receipts replace runtime.** Instead of carrying the tools to re-run, carry proof that it ran correctly.
3. **Replaceable by design.** A pseudoSpore can always be regenerated from `[source].repo` + `[source].commit` + `configs/`.
4. **Promotion is additive.** Nothing in the pseudoSpore format conflicts with full lithoSpore — promotion only adds files (binaries, notebooks, expected values).
5. **Trust the receipt.** When ingesting a pseudoSpore, `litho` trusts the `validation.json` results without re-running. The checksums verify data integrity, not computational correctness.
