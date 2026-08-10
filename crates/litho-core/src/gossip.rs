// SPDX-License-Identifier: AGPL-3.0-or-later

//! Gossip injection — fire-and-forget event publication via swarmVine.
//!
//! When swarmVine is discoverable on the mesh (`discover("gossip")`),
//! lithoSpore publishes lifecycle events so other primals and gates can
//! observe validation activity without polling.
//!
//! All injection is non-fatal: if swarmVine is absent or unreachable,
//! events are silently dropped. lithoSpore never depends on gossip for
//! correctness — this is a best-effort notification layer.

use crate::discovery;

/// Gossip event types that lithoSpore can inject into the mesh.
///
/// Each variant maps to a `gossip.spread` RPC call with a typed payload.
/// See `config/capability_registry.toml` `[gossip_injection]` for the
/// canonical event list and payload specifications.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "event", content = "payload")]
pub enum GossipEvent {
    /// A full validation run completed (any exit code).
    #[serde(rename = "validation.complete")]
    ValidationComplete {
        artifact: String,
        version: String,
        tier_reached: u8,
        exit_code: i32,
        modules_total: usize,
        modules_passed: usize,
    },

    /// A new pseudoSpore was emitted via `litho emit-pseudospore`.
    #[serde(rename = "pseudospore.emitted")]
    PseudosporeEmitted {
        spore_name: String,
        modules: usize,
    },

    /// A pseudoSpore was promoted from PENDING to COMPLETE.
    #[serde(rename = "pseudospore.promoted")]
    PseudosporePromoted {
        spore_name: String,
        modules_passing: usize,
        modules_total: usize,
    },

    /// A cross-tier parity check completed.
    #[serde(rename = "parity.checked")]
    ParityChecked {
        artifact: String,
        version: String,
        parity_pass: bool,
        divergence_count: usize,
    },
}

/// Attempt to publish a gossip event to swarmVine.
///
/// Returns `true` if the event was successfully sent, `false` if
/// swarmVine was unreachable or the RPC failed. Callers should not
/// branch on the return value — it exists only for logging/testing.
#[allow(clippy::must_use_candidate)]
pub fn inject(event: &GossipEvent) -> bool {
    let Some(endpoint) = discovery::discover("gossip") else {
        return false;
    };

    let payload = match serde_json::to_value(event) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "gossip.spread",
        "params": {
            "origin": "lithospore",
            "topic": "validation",
            "data": payload,
        },
        "id": 1,
    });

    let request_str = match serde_json::to_string(&request) {
        Ok(s) => s,
        Err(_) => return false,
    };

    discovery::rpc_call(&endpoint, &request_str).is_some()
}

/// Build a `ValidationComplete` event from a finished report.
#[must_use]
pub fn from_validation_report(report: &crate::ValidationReport) -> GossipEvent {
    let modules_passed = report
        .modules
        .iter()
        .filter(|m| m.status == crate::ValidationStatus::Pass)
        .count();

    GossipEvent::ValidationComplete {
        artifact: report.artifact.clone(),
        version: report.version.clone(),
        tier_reached: report.tier_reached,
        exit_code: report.exit_code(),
        modules_total: report.modules.len(),
        modules_passed,
    }
}

/// Build a `ParityChecked` event from a finished parity report.
#[must_use]
pub fn from_parity_report(report: &crate::ParityReport) -> GossipEvent {
    let divergence_count = report
        .modules
        .iter()
        .filter(|m| m.parity == crate::ParityStatus::Divergence)
        .count();

    GossipEvent::ParityChecked {
        artifact: report.artifact.clone(),
        version: report.version.clone(),
        parity_pass: report.parity_pass,
        divergence_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_without_swarmvine_returns_false() {
        let event = GossipEvent::ValidationComplete {
            artifact: "test".into(),
            version: "0.0.0".into(),
            tier_reached: 2,
            exit_code: 0,
            modules_total: 5,
            modules_passed: 5,
        };
        assert!(!inject(&event));
    }

    #[test]
    fn event_serializes_with_tag() {
        let event = GossipEvent::PseudosporeEmitted {
            spore_name: "hotSpring-QCD".into(),
            modules: 3,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("pseudospore.emitted"));
        assert!(json.contains("hotSpring-QCD"));
    }

    #[test]
    fn from_validation_report_counts_correctly() {
        let mut report = crate::ValidationReport::new("test-artifact", "1.0.0");
        report.add_module(crate::ModuleResult {
            name: "mod_a".into(),
            status: crate::ValidationStatus::Pass,
            tier: 2,
            checks: 10,
            checks_passed: 10,
            runtime_ms: 50,
            error: None,
        });
        report.add_module(crate::ModuleResult {
            name: "mod_b".into(),
            status: crate::ValidationStatus::Fail,
            tier: 2,
            checks: 5,
            checks_passed: 3,
            runtime_ms: 30,
            error: Some("diverged".into()),
        });

        let event = from_validation_report(&report);
        match event {
            GossipEvent::ValidationComplete {
                modules_total,
                modules_passed,
                exit_code,
                tier_reached,
                ..
            } => {
                assert_eq!(modules_total, 2);
                assert_eq!(modules_passed, 1);
                assert_eq!(exit_code, 1);
                assert_eq!(tier_reached, 2);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn from_parity_report_counts_divergences() {
        let report = crate::ParityReport {
            artifact: "test".into(),
            version: "1.0.0".into(),
            modules: vec![
                crate::ParityResult {
                    module: "a".into(),
                    tier1_status: crate::ValidationStatus::Pass,
                    tier2_status: crate::ValidationStatus::Pass,
                    tier1_checks: 5,
                    tier2_checks: 5,
                    tier1_passed: 5,
                    tier2_passed: 5,
                    parity: crate::ParityStatus::Match,
                },
                crate::ParityResult {
                    module: "b".into(),
                    tier1_status: crate::ValidationStatus::Pass,
                    tier2_status: crate::ValidationStatus::Fail,
                    tier1_checks: 3,
                    tier2_checks: 3,
                    tier1_passed: 3,
                    tier2_passed: 1,
                    parity: crate::ParityStatus::Divergence,
                },
            ],
            parity_pass: false,
        };

        let event = from_parity_report(&report);
        match event {
            GossipEvent::ParityChecked {
                parity_pass,
                divergence_count,
                ..
            } => {
                assert!(!parity_pass);
                assert_eq!(divergence_count, 1);
            }
            _ => panic!("wrong variant"),
        }
    }
}
