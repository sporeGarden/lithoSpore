# lithoSpore Degradation Behavior

**Ecosystem invariant**: Science is never gated behind primal availability.
All RPC calls return `Result`. No method panics on unreachable primals.

---

## Per-Primal Degradation Matrix

| Primal | Capability | When Reachable | When Unreachable | Impact |
|--------|-----------|----------------|------------------|--------|
| **rhizoCrypt** | `dag` | DAG session records all module events + merkle root | No Tier 3 provenance at all | Tier stays at 2 |
| **loamSpine** | `spine` | Spine entry created, validation summary committed | DAG recorded but no ledger entry | Partial provenance (DAG-only) — valid |
| **sweetGrass** | `braid` | Attribution braid links DAG + spine | DAG + spine recorded but no attribution | Partial provenance (DAG+spine) — valid |
| **bearDog** | `crypto` | `primals_reached` includes crypto endpoint | Skipped silently | No impact on science or provenance |
| **songBird** | `discovery` | UDS/TURN discovery resolves capabilities | Falls back to env vars or standalone | Tier 2 or 1 depending on env |
| **petalTongue** | `visualization` | Dashboard rendering via IPC | `litho visualize` reports "no visualization primal" | JSON stdout fallback |
| **nestGate** | `storage` | Persistent provenance storage | `liveSpore.json` local append | Local provenance only |
| **toadStool** | `compute` | GPU-accelerated module dispatch | CPU path (standard) | No performance enrichment |
| **biomeOS** | `orchestration` | `primal.announce` registers lithoSpore | Skipped silently | No registration, no impact |

## Discovery Chain Degradation

```
1. $CAPABILITY_PORT env var     → direct TCP connection
   FAIL → try next
2. UDS discovery.sock           → ipc.resolve JSON-RPC
   FAIL → try next
3. $RELAY_SERVER (TURN)         → geo-delocalized relay
   FAIL → try next
4. Standalone                   → None returned
   → Caller degrades gracefully (Tier 2 or Tier 1)
```

## Provenance Trio Partial Completion

Per `PROVENANCE_TRIO_INTEGRATION_GUIDE.md` (v2.0):

| State | DAG | Spine | Braid | Valid? | lithoSpore Behavior |
|-------|:---:|:-----:|:-----:|:------:|---------------------|
| Full | YES | YES | YES | YES | `tier_reached = 3`, all IDs populated |
| DAG + spine | YES | YES | no | YES | `tier_reached = 3`, `braid_id = ""` |
| DAG only | YES | no | no | YES | `tier_reached = 3`, `spine_id = ""`, `braid_id = ""` |
| None | no | no | no | YES | `tier_reached = 2`, standalone mode |

No rollback on partial. Consumer (`validate.rs`) accepts any non-error
`Tier3Session` as success. `primals_reached` tracks exactly which
primals responded.

## Code References

- `discovery::discover()` → returns `Option<PrimalEndpoint>` (never panics)
- `provenance::try_record_tier3()` → `Result<Tier3Session, String>` (Err only if DAG unreachable)
- `discovery::rpc_call()` → `Option<Value>` (timeout/parse failure → None)
- `validate::run_with_provenance()` → falls back to Tier 2 on any `Err`
