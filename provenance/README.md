# Provenance Directory

Upstream computation braids from spring teams are stored in `braids/`.

## Current Braids

| File | Spring | Tool | Status |
|------|--------|------|--------|
| `barrick_2009_sovereign.json` | wetSpring | Sovereign Rust+GPU pipeline | READY |
| `barrick_2009_breseq.json` | wetSpring | breseq 0.40.1 (C++ baseline) | READY |
| `tenaillon_2016_wetspring_tier2.json` | wetSpring | Exp380 Tier 2 expected values (10 targets, 27/27 PASS) | READY |
| `tenaillon_2016_sovereign.json` | wetSpring | Sovereign Rust+GPU pipeline | PENDING — file not yet present (42/312 accessions downloaded upstream) |

## Wire Formats

### Sovereign Pipeline Braid

Full provenance with computation metadata, accession validation, and substrate info:

```json
{
  "dataset_id": "barrick_2009_sovereign_resequencing",
  "spring": "wetSpring",
  "spring_version": "0.1.0",
  "braid_id": "braid-sovereign-barrick2009",
  "dag_session_id": "dag-wetspring-sovereign-...",
  "computation": {
    "tool": "wetspring-sovereign-pipeline",
    "substrate": "GPU+CPU hybrid",
    "pipeline": "FM-index → SmithWatermanGpu → Tensor::scan → SnpCallingF64",
    "input_accession": "SRP001569",
    "node_count": 7,
    "sovereign_variants": 159,
    "breseq_variants": 569
  }
}
```

### Baseline Braid (breseq)

Flat format with per-clone mutation counts (no `computation` block):

```json
{
  "dataset": "barrick_2009",
  "clones_processed": 7,
  "total_mutations": 6664,
  "mutation_counts": [{"clone": "REL1164M", "mutations": 579}, ...],
  "reference": "CP000819.1",
  "reference_length_bp": 4629812
}
```

## Ingestion

`litho validate` automatically loads all `*.json` files from `provenance/braids/`,
parses both wire formats, validates accessions against expected SRA entries,
and displays braid provenance alongside science validation results.

Braids flow into `artifact/data.toml` via the `upstream_braid` and
`upstream_dag_session` fields.

## Chain Model

```
NUCLEUS (4TB NVMe)                    Repo / USB / guideStone
├── 200 GB raw reads                  ├── braids/barrick_2009_sovereign.json  (~1 KB)
├── breseq output (GBs)         →     ├── braids/barrick_2009_breseq.json     (~1 KB)
├── braids/tenaillon_2016_wetspring_tier2.json (~2 KB)
├── sovereign pipeline output         └── braids/tenaillon_2016_sovereign.json (~1 KB, not yet present)
└── full provenance DAG sessions
```

The spore can't carry the mountain. But it proves the mountain was climbed.

## Related Documents

- `infra/wateringHole/handoffs/LITHOSPORE_FERMENT_TRANSCRIPT_BRAID_HANDOFF_MAY17_2026.md`
- `specs/PARITY_REPORT_SCHEMA.md`
- `docs/DEGRADATION_BEHAVIOR.md`
