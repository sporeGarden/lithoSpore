// SPDX-License-Identifier: AGPL-3.0-or-later

//! Graph TOML validation helpers — mirrors primalSpring `validation/helpers.rs`.
//!
//! Tier 1 structural checks for deploy graph TOMLs: parsing, Dark Forest
//! metadata, capability registry cross-referencing, and graph structure
//! validation. No IPC required.

/// Parse TOML content. Returns `Ok(parsed)` on success.
pub fn graph_parses(content: &str) -> Result<toml::Value, String> {
    toml::from_str::<toml::Value>(content).map_err(|e| format!("TOML parse error: {e}"))
}

/// Extract all `binary` field values from `[[graph.nodes]]`.
pub fn graph_binaries(parsed: &toml::Value) -> Vec<String> {
    graph_nodes(parsed)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|n| n.get("binary").and_then(|b| b.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Safe accessor for `graph.nodes` array.
pub fn graph_nodes(parsed: &toml::Value) -> Option<&Vec<toml::Value>> {
    parsed
        .get("graph")
        .and_then(|g| g.get("nodes"))
        .and_then(|n| n.as_array())
}

/// Safe accessor for `graph.metadata`.
pub fn graph_metadata(parsed: &toml::Value) -> Option<&toml::Value> {
    parsed.get("graph").and_then(|g| g.get("metadata"))
}

/// Check Dark Forest security invariants: `secure_by_default = true`,
/// `security_model = "btsp_enforced"`, `transport = "uds_only"`.
pub fn check_dark_forest(parsed: &toml::Value) -> Vec<(String, bool)> {
    let metadata = graph_metadata(parsed);
    let mut checks = Vec::new();

    let secure = metadata
        .and_then(|m| m.get("secure_by_default"))
        .and_then(|s| s.as_bool())
        .unwrap_or(false);
    checks.push(("secure_by_default".into(), secure));

    let btsp = metadata
        .and_then(|m| m.get("security_model"))
        .and_then(|s| s.as_str())
        .is_some_and(|s| s == "btsp_enforced");
    checks.push(("btsp_enforced".into(), btsp));

    let uds = metadata
        .and_then(|m| m.get("transport"))
        .and_then(|t| t.as_str())
        .is_some_and(|t| t == "uds_only");
    checks.push(("uds_only".into(), uds));

    checks
}

/// Check that every node has `by_capability` for capability routing.
pub fn check_node_capabilities(parsed: &toml::Value) -> Vec<(String, bool)> {
    let mut checks = Vec::new();
    let Some(nodes) = graph_nodes(parsed) else {
        return checks;
    };

    for node in nodes {
        let name = node
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        let has_by_cap = node
            .get("by_capability")
            .and_then(|b| b.as_str())
            .is_some();
        checks.push((format!("{name}:by_capability"), has_by_cap));
    }
    checks
}

/// Validate that graph has `id`, `name`, and `coordination` fields.
pub fn check_graph_envelope(parsed: &toml::Value) -> Vec<(String, bool)> {
    let graph = parsed.get("graph");
    let mut checks = Vec::new();

    let has_id = graph.and_then(|g| g.get("id")).and_then(|i| i.as_str()).is_some();
    checks.push(("has_id".into(), has_id));

    let has_name = graph.and_then(|g| g.get("name")).and_then(|n| n.as_str()).is_some();
    checks.push(("has_name".into(), has_name));

    let has_coordination = graph
        .and_then(|g| g.get("coordination"))
        .and_then(|c| c.as_str())
        .is_some();
    checks.push(("has_coordination".into(), has_coordination));

    let node_count = graph_nodes(parsed).map_or(0, Vec::len);
    checks.push(("has_nodes".into(), node_count > 0));

    checks
}

/// Parse capability methods from a registry TOML string, skipping
/// `test_fixtures`, `false_positives`, and `signals`.
pub fn parse_registry_capabilities(registry_toml: &str) -> Vec<String> {
    let parsed: toml::Value = match toml::from_str(registry_toml) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let skip = ["test_fixtures", "false_positives", "signals"];
    let mut caps = Vec::new();
    if let Some(table) = parsed.as_table() {
        for (section, value) in table {
            if skip.contains(&section.as_str()) {
                continue;
            }
            if let Some(methods) = value.get("methods").and_then(|m| m.as_array()) {
                for m in methods {
                    if let Some(s) = m.as_str() {
                        caps.push(s.to_owned());
                    }
                }
            }
        }
    }
    caps
}

/// Cross-check that all capabilities referenced in graph nodes are
/// present in the registry.
pub fn check_capabilities_registered(
    parsed: &toml::Value,
    registry_caps: &[String],
) -> Vec<(String, bool)> {
    let mut checks = Vec::new();
    let Some(nodes) = graph_nodes(parsed) else {
        return checks;
    };

    for node in nodes {
        let name = node
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        let caps = node
            .get("capabilities")
            .and_then(|c| c.as_array())
            .cloned()
            .unwrap_or_default();

        for cap in &caps {
            if let Some(cap_str) = cap.as_str() {
                let registered = registry_caps.iter().any(|r| r == cap_str);
                checks.push((format!("{name}:{cap_str}"), registered));
            }
        }
    }
    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRAPH_TOML: &str = include_str!("../../../graphs/ltee_guidestone.toml");
    const REGISTRY_TOML: &str = include_str!("../../../config/capability_registry.toml");

    #[test]
    fn graph_parses_valid() {
        let parsed = graph_parses(GRAPH_TOML);
        assert!(parsed.is_ok(), "ltee_guidestone.toml should parse");
    }

    #[test]
    fn graph_envelope_complete() {
        let parsed = graph_parses(GRAPH_TOML).expect("parse");
        let checks = check_graph_envelope(&parsed);
        for (name, ok) in &checks {
            assert!(ok, "envelope check failed: {name}");
        }
    }

    #[test]
    fn dark_forest_invariants() {
        let parsed = graph_parses(GRAPH_TOML).expect("parse");
        let checks = check_dark_forest(&parsed);
        for (name, ok) in &checks {
            assert!(ok, "Dark Forest check failed: {name}");
        }
    }

    #[test]
    fn all_nodes_have_by_capability() {
        let parsed = graph_parses(GRAPH_TOML).expect("parse");
        let checks = check_node_capabilities(&parsed);
        assert!(!checks.is_empty(), "should have nodes");
        for (name, ok) in &checks {
            assert!(ok, "by_capability missing: {name}");
        }
    }

    #[test]
    fn registry_parses() {
        let caps = parse_registry_capabilities(REGISTRY_TOML);
        assert!(caps.len() > 10, "registry should have methods");
    }

    #[test]
    fn graph_capabilities_in_registry() {
        let parsed = graph_parses(GRAPH_TOML).expect("parse");
        let registry_caps = parse_registry_capabilities(REGISTRY_TOML);
        let checks = check_capabilities_registered(&parsed, &registry_caps);
        for (name, ok) in &checks {
            assert!(ok, "capability not in registry: {name}");
        }
    }

    #[test]
    fn graph_has_expected_binaries() {
        let parsed = graph_parses(GRAPH_TOML).expect("parse");
        let bins = graph_binaries(&parsed);
        assert!(bins.contains(&"beardog".to_owned()));
        assert!(bins.contains(&"rhizocrypt".to_owned()));
        assert!(bins.contains(&"sweetgrass".to_owned()));
    }
}
