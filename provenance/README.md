# Provenance Directory

Upstream computation braids from spring teams are stored in `braids/`.

## Wire Format

Each file is `{dataset_id}.json` per the ferment transcript contract
(`infra/wateringHole/handoffs/LITHOSPORE_FERMENT_TRANSCRIPT_BRAID_HANDOFF_MAY17_2026.md`):

```json
{
  "dataset_id": "barrick_2009_mutations",
  "spring": "wetSpring",
  "spring_version": "0.1.0",
  "braid_id": "<from sweetGrass>",
  "dag_session_id": "<from rhizoCrypt>",
  "dag_merkle_root": "<BLAKE3>",
  "spine_id": "<from loamSpine>",
  "computation": {
    "tool": "breseq",
    "tool_version": "0.40.1",
    "input_accession": "SRP001569",
    "node_count": 7,
    "wall_time_seconds": 3793
  },
  "summary_blake3": "529e34ee..."
}
```

## Current State

Awaiting first braid from wetSpring Exp381 (Barrick 2009 — 3/7 clones done).
When received, the braid ID flows into `artifact/data.toml` as `upstream_braid`.

Standalone braids (trio IDs empty because sweetGrass/loamSpine were not running)
are structurally valid — the computation provenance (tool, accession, merkle)
is the important content. Full trio IDs come when NUCLEUS is deployed.
