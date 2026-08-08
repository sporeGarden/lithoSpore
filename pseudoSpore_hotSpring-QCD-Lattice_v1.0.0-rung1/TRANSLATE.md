# Translation: Domain ↔ Computation

## Atom Indices

See `index_map.toml` for the machine-readable mapping.

| Ring atom | Domain (PDB serial) | Computation (runtime index) |
|-----------|--------------------|-----------------------------|
| ... | ... | ... |

Rosetta stone: `data/<module>/npt.gro` (topology file)

## Conventions

| | Domain standard | This artifact |
|--|----------------|---------------|
| Numbering | PDB serial | Runtime topology (mapped in index_map.toml) |
| Checksums | — | BLAKE3 |

## Derivations

| Output | Data | Command |
|--------|------|---------|
| `outputs/<module>/...` | `data/<module>/...` | `<tool> <args>` |
