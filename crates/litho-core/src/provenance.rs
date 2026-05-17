// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance chain: tracks data lineage for every computation.
//!
//! When NUCLEUS primals are available (Tier 3), the provenance trio records
//! validation results into the DAG/spine/braid pipeline. The 3-call sequence
//! (`dag.*` -> `spine.*` -> `braid.*`) maps to the `nest.store` atomic signal;
//! when biomeOS supports signal dispatch, `ctx.dispatch("nest.store", ...)`
//! will collapse these into a single call.

use serde::{Deserialize, Serialize};

use crate::discovery::{self, PrimalEndpoint};
use crate::validation::{Tier3Session, ValidationReport};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub dataset_id: String,
    pub binary_version: String,
    pub tolerance_name: String,
    pub blake3_input: String,
    pub blake3_output: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    pub entries: Vec<ProvenanceEntry>,
}

impl ProvenanceChain {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, entry: ProvenanceEntry) {
        self.entries.push(entry);
    }
}

impl Default for ProvenanceChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Capability domain strings for the provenance trio (from `capability_registry.toml`).
const CAP_DAG: &str = "dag";
const CAP_SPINE: &str = "spine";
const CAP_BRAID: &str = "braid";

/// Attempt to record a Tier 3 provenance session via the primal trio.
///
/// Discovers rhizoCrypt (DAG), loamSpine (spine), and sweetGrass (braid)
/// by capability. If all three are reachable, records the validation results
/// as a provenance chain and returns the session metadata.
///
/// # Errors
///
/// Returns `Err` if any required primal is unreachable — caller stays at Tier 2.
pub fn try_record_tier3(report: &ValidationReport) -> Result<Tier3Session, String> {
    let dag_ep = discovery::discover(CAP_DAG)
        .ok_or("rhizoCrypt (dag) not reachable")?;
    let spine_ep = discovery::discover(CAP_SPINE)
        .ok_or("loamSpine (spine) not reachable")?;
    let braid_ep = discovery::discover(CAP_BRAID)
        .ok_or("sweetGrass (braid) not reachable")?;

    let mut primals_reached = vec![
        format!("rhizocrypt@{}", endpoint_addr(&dag_ep)),
        format!("loamspine@{}", endpoint_addr(&spine_ep)),
        format!("sweetgrass@{}", endpoint_addr(&braid_ep)),
    ];

    // Phase 1: DAG session — create, append module events, complete
    let create_params = serde_json::json!({"artifact": &report.artifact, "version": &report.version});
    let dag_session_id = rpc_call_extract(
        &dag_ep,
        "dag.session.create",
        &create_params,
        "session_id",
    )?;

    for module in &report.modules {
        let event = serde_json::json!({
            "session_id": &dag_session_id,
            "event_type": "module_validation",
            "module": &module.name,
            "status": format!("{:?}", module.status),
            "tier": module.tier,
            "checks": module.checks,
            "checks_passed": module.checks_passed,
        });
        let _ = rpc_call_result(&dag_ep, "dag.event.append", &event);
    }

    let complete_params = serde_json::json!({"session_id": &dag_session_id});
    let merkle_root = rpc_call_extract(
        &dag_ep,
        "dag.session.complete",
        &complete_params,
        "merkle_root",
    ).unwrap_or_else(|_| "pending".into());

    // Phase 2: Spine — create entry with validation summary
    let spine_params = serde_json::json!({"name": format!("{}-validation", report.artifact)});
    let spine_id = rpc_call_extract(
        &spine_ep,
        "spine.create",
        &spine_params,
        "spine_id",
    ).unwrap_or_else(|_| "pending".into());

    let entry_params = serde_json::json!({
        "spine_id": &spine_id,
        "entry_type": "validation_summary",
        "dag_session": &dag_session_id,
        "merkle_root": &merkle_root,
        "tier_reached": report.tier_reached,
        "modules_passed": report.modules.iter().filter(|m| m.status == crate::ValidationStatus::Pass).count(),
        "modules_total": report.modules.len(),
    });
    let _ = rpc_call_result(&spine_ep, "entry.append", &entry_params);

    // Phase 3: Braid — attribution record
    let braid_params = serde_json::json!({
        "artifact": &report.artifact,
        "dag_session": &dag_session_id,
        "spine_id": &spine_id,
        "attribution": "lithoSpore automated validation",
    });
    let braid_id = rpc_call_extract(
        &braid_ep,
        "braid.create",
        &braid_params,
        "braid_id",
    ).unwrap_or_else(|_| "pending".into());

    // Check for optional crypto primal
    if let Some(crypto_ep) = discovery::discover("crypto") {
        primals_reached.push(format!("beardog@{}", endpoint_addr(&crypto_ep)));
    }

    Ok(Tier3Session {
        dag_session_id,
        dag_merkle_root: merkle_root,
        spine_id,
        braid_id,
        primals_reached,
    })
}

fn endpoint_addr(ep: &PrimalEndpoint) -> String {
    if ep.port == 0 {
        ep.host.clone()
    } else {
        format!("{}:{}", ep.host, ep.port)
    }
}

fn rpc_call_extract(
    ep: &PrimalEndpoint,
    method: &str,
    params: &serde_json::Value,
    field: &str,
) -> Result<String, String> {
    let response = rpc_call_result(ep, method, params)?;
    response
        .get(field)
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| format!("{method}: missing '{field}' in response"))
}

fn rpc_call_result(
    ep: &PrimalEndpoint,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    });
    let request_str = serde_json::to_string(&request)
        .map_err(|e| format!("serialize {method}: {e}"))?;

    let response = discovery::rpc_call(ep, &request_str)
        .ok_or_else(|| format!("{method}: no response from {}", endpoint_addr(ep)))?;

    if let Some(err) = response.get("error") {
        return Err(format!("{method}: {err}"));
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| format!("{method}: no 'result' in response"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_append_and_default() {
        let mut chain = ProvenanceChain::default();
        assert!(chain.entries.is_empty());

        chain.append(ProvenanceEntry {
            dataset_id: "wiser_2013".into(),
            binary_version: "0.1.0".into(),
            tolerance_name: "fitness_relative_error".into(),
            blake3_input: "abc123".into(),
            blake3_output: "def456".into(),
            timestamp: "2026-05-12T00:00:00Z".into(),
        });
        assert_eq!(chain.entries.len(), 1);
        assert_eq!(chain.entries[0].dataset_id, "wiser_2013");
    }

    #[test]
    fn provenance_json_roundtrip() {
        let entry = ProvenanceEntry {
            dataset_id: "test".into(),
            binary_version: "0.1.0".into(),
            tolerance_name: "tol".into(),
            blake3_input: "aaa".into(),
            blake3_output: "bbb".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: ProvenanceEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.dataset_id, "test");
    }

    #[test]
    fn tier3_standalone_returns_err() {
        let report = ValidationReport::new("test", "0.1.0");
        let result = try_record_tier3(&report);
        assert!(result.is_err(), "should fail when no primals are available");
    }
}
