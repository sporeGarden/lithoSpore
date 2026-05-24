# pseudoSpore Examples

## Canonical Example: hotSpring CAZyme FEL (v0.6.0)

The first pseudoSpore deployed in the ecosystem. Produced by hotSpring Experiment 220
(CAZyme conformational free energy landscapes via well-tempered metadynamics).

**Location:** `ecoPrimals/springs/hotSpring/control/gromacs_fel/lithoSpore_handoff/`

### Mapping to Standard

| Standard Path | Existing File | Notes |
|---------------|--------------|-------|
| `scope.toml` | `scope.toml` | Already uses `type = "pseudo-lithoSpore"` — upgrade to `"pseudoSpore"` |
| `validation.json` | `validation.json` | Conforms to spec (modules array with checks + errata) |
| `receipts/environment.toml` | `provenance/environment.toml` | Move to `receipts/` |
| `receipts/checksums.blake3` | — | Generate with `litho emit-pseudospore` |
| `receipts/compute_log.toml` | — | Optional — compute parameters are in environment.toml |
| `provenance/ferment_transcript.json` | `provenance/ferment_transcript.json` | Conforms (has dataset_id, spring, braid_id, dag, spine) |
| `provenance/dag.json` | `provenance/dag.json` | Conforms (11-event DAG with Merkle root) |
| `provenance/spine.json` | `provenance/spine.json` | Conforms (3 ledger entries) |
| `provenance/braids/` | `provenance/braids/` | Conforms (live_braid.json, cazyme_fel_v0.6.0.json, provo_export.jsonld) |
| `outputs/` | `modules/` | Rename to `outputs/` for standard alignment |
| `configs/` | (within modules/) | Extract .mdp and plumed.dat to `configs/` |
| `README.md` | `README.md` | Conforms |
| `AUDIT.md` | `AUDIT.md` | Optional — present and conformant |
| `RELEASE.md` | `RELEASE.md` | Optional — present and conformant |

### Migration Steps

To align the existing CAZyme handoff with the pseudoSpore standard:

1. Rename `[artifact]` type from `"pseudo-lithoSpore"` to `"pseudoSpore"` in scope.toml
2. Move `provenance/environment.toml` to `receipts/environment.toml`
3. Generate `receipts/checksums.blake3` via `litho emit-pseudospore` or manually
4. Rename `modules/` to `outputs/`
5. Extract config files (.mdp, plumed.dat) from outputs into `configs/`

Or generate a fresh pseudoSpore using the CLI:

```bash
litho emit-pseudospore \
  --name hotSpring-CAZyme-FEL \
  --version 0.7.0 \
  --origin ecoPrimals/springs/hotSpring \
  --output ~/Desktop/ \
  --outputs control/gromacs_fel/lithoSpore_handoff/modules \
  --braids control/gromacs_fel/lithoSpore_handoff/provenance/braids
```

### Key Design Patterns Demonstrated

1. **Three-module validation ladder**: benchmark (alanine dipeptide) → substrate (free xylose) → target system (enzyme-bound). Each module produces independent FES outputs.

2. **Errata as first-class data**: The `errata` field in both scope.toml `[[module]]` entries and validation.json module entries captures known limitations honestly.

3. **IN_FLIGHT modules**: Module 3 (enzyme-bound) is marked IN_FLIGHT — the pseudoSpore ships before all modules complete. This is valid: the existing passes prove the pipeline works, the in-flight module is honestly flagged.

4. **Live sweetGrass braid**: The provenance includes a live IPC braid (not just pseudo). This demonstrates the sweetGrass → pseudoSpore integration path.

5. **Tiered evolution**: scope.toml `[evolution]` section documents the Python and Rust implementations that exist alongside the GROMACS control — showing the promotion path to full lithoSpore module.

---

## Template: Minimal pseudoSpore

For a new spring creating its first pseudoSpore:

```bash
litho emit-pseudospore \
  --name mySpring-experiment-name \
  --version 0.1.0 \
  --origin ecoPrimals/springs/mySpring \
  --output ./

# Then:
# 1. Edit scope.toml — add [target] paper info and [[module]] entries
# 2. Copy result files to outputs/<module>/
# 3. Copy input configs to configs/<module>/
# 4. Replace the ferment transcript stub with real braid data
# 5. Populate validation.json with actual results
# 6. Regenerate checksums: update receipts/checksums.blake3
# 7. Validate: litho ingest-pseudospore pseudoSpore_mySpring-experiment-name_v0.1.0/
```
