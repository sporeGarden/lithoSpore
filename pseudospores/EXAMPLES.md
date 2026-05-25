# pseudoSpore Examples

## Canonical Example: hotSpring CAZyme FEL (v0.9.0)

The first pseudoSpore deployed in the ecosystem. Produced by hotSpring Experiment 220
(CAZyme conformational free energy landscapes via well-tempered metadynamics).

**Tarball:** `~/Desktop/pseudoSpore_hotSpring-CAZyme-FEL_v0.9.0.tar.gz` (6.6 MB)  
**Source:** `ecoPrimals/springs/hotSpring/control/gromacs_fel/`

### Version History

| Version | Key Change |
|---------|-----------|
| v0.6.0 | Initial prototype (had lyxose/xylose error, missing PDBs) |
| v0.7.0 | Alistaire review: correct structure, complete Module 3, atom index docs |
| v0.8.0 | Overnight expansion: 2D FELs, raw data included, zero-trust derivation |
| v0.9.0 | Machine-readable `index_map.toml`, chassis-ready translation layer |

### Zero-Trust Features (v0.9.0)

```
index_map.toml — machine-readable domain ↔ computation translation
TRANSLATE.md   — human-readable derivation legend
configs/       — plumed.dat with inline INDEX TRANSLATION comments
data/          — raw HILLS + topology (re-derive outputs independently)
outputs/       — derived FES surfaces (verifiable from data/)
receipts/      — BLAKE3 checksums (integrity seal)
provenance/    — FermentBraid (who/what/when/how)
validation.json — 5 modules, all PASS, Tier 1+2 parity verified
```

### CLI Verification

```bash
# Ingest and verify
litho ingest-pseudospore pseudoSpore_hotSpring-CAZyme-FEL_v0.9.0/ --verify

# Translate configs to domain frame (PDB numbering)
litho translate-config \
  --index-map pseudoSpore_hotSpring-CAZyme-FEL_v0.9.0/index_map.toml \
  --config pseudoSpore_hotSpring-CAZyme-FEL_v0.9.0/configs/enzyme-bound-puckering/plumed.dat \
  --frame domain

# Manual derivation check
cd pseudoSpore_hotSpring-CAZyme-FEL_v0.9.0/
plumed sum_hills --hills data/xylose-puckering-fel/HILLS --mintozero --outfile /tmp/v.dat
diff outputs/xylose-puckering-fel/fes_theta.dat /tmp/v.dat
```

### Modules (5 total, all PASS)

| Module | CV | Duration | Tier 1 RMSD | Tier 2 RMSD |
|--------|-----|----------|-------------|-------------|
| ala-dipeptide-fel | φ/ψ 2D | 10 ns | 0.52 kJ/mol | — |
| xylose-puckering-fel | θ 1D | 10 ns | 0.73 kJ/mol | 0.79 kJ/mol |
| enzyme-bound-puckering | θ 1D | 10 ns | 0.76 kJ/mol | 0.77 kJ/mol |
| free-xylose-2d | qx,qy 2D | 20 ns | 1.71 kJ/mol | 1.71 kJ/mol |
| enzyme-bound-2d | qx,qy 2D | 20 ns | 1.72 kJ/mol | 1.72 kJ/mol |

### Promotion Path

```
pseudoSpore v0.9.0 (current — proof complete)
  → litho promote --add-runtime
lithoSpore v1.0.0 (deployment chassis)
  ├── proof/   ← pseudoSpore contents verbatim
  ├── runtime/ ← litho CLI + cazyme-fel binary + puckering_fel.py
  └── expected/ ← validation targets from proof/outputs/
```

See `specs/CHASSIS.md` for the full lithoSpore deployment layout.

### Key Design Patterns Demonstrated

1. **Translation as first-class data.** `index_map.toml` maps PDB serial (domain)
   to GROMACS topology (computation). The artifact handles reindexing — not the reader.

2. **Zero-trust derivation.** Every output file has a corresponding data file + command
   in TRANSLATE.md. Nothing requires trusting the producer.

3. **Three-module validation ladder.** Benchmark (alanine dipeptide) → substrate
   (free xylose) → target system (enzyme-bound). Each module independent.

4. **Multi-dimensional expansion.** 1D (θ) and 2D (qx,qy) FELs for both systems.
   Tier 1 (Python) and Tier 2 (Rust) parity for all.

5. **Errata-driven evolution.** v0.6.0 → v0.7.0 → v0.8.0 → v0.9.0 each addressed
   specific reviewer feedback (Alistaire). The `supersedes` chain in provenance braids
   makes this lineage machine-readable.

---

## Template: Minimal pseudoSpore

```bash
litho emit-pseudospore \
  --name mySpring-experiment-name \
  --version 0.1.0 \
  --origin ecoPrimals/springs/mySpring \
  --output ./ \
  --data ./raw-simulation-data/ \
  --configs ./input-configs/ \
  --outputs ./derived-results/

# Then:
# 1. Edit scope.toml — add [target] paper info and [[module]] entries
# 2. Review/edit auto-generated index_map.toml (domain indices need manual assignment)
# 3. Populate TRANSLATE.md with derivation commands
# 4. Replace ferment_transcript.json stub with real braid data
# 5. Populate validation.json with actual results
# 6. Regenerate checksums: find . -type f ! -name "checksums.blake3" | sort | xargs b3sum > receipts/checksums.blake3
# 7. Validate: litho ingest-pseudospore pseudoSpore_mySpring-experiment-name_v0.1.0/ --verify
```

## Template: Promote to lithoSpore (future)

```bash
litho promote \
  --pseudospore pseudoSpore_mySpring-experiment-name_v0.1.0/ \
  --tier2-crate staging/my-validator/ \
  --output lithoSpore_my-experiment_v1.0.0/

# Generates:
# - proof/ (verbatim pseudoSpore)
# - runtime/bin/ (compiled Tier 2 binary + litho CLI)
# - runtime/env/ (Python requirements)
# - expected/ (validation targets)
# - tolerances.toml (acceptance criteria)
# - guidestone.toml (lithoSpore identity)
```
