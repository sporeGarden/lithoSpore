// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::viz::{bar_from_object, gauge};
use serde_json::{Value, json};

pub fn plannotate(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(dist) = data.get("feature_distribution").and_then(|v| v.as_object()) {
        b.push(bar_from_object(
            "bl_plannotate_features",
            "pLannotate: Feature Category Distribution",
            dist,
            "count",
        ));
    }
    if let Some(cov) = data
        .get("annotation_coverage")
        .and_then(serde_json::Value::as_f64)
    {
        b.push(gauge(
            "bl_plannotate_coverage",
            "pLannotate: Annotation Coverage",
            cov,
            0.0,
            1.0,
            "fraction",
            [0.8, 1.0],
            [0.5, 0.8],
        ));
    }

    let plasmid_size = data
        .get("plasmid_size_bp")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(2686.0);
    let plasmid_name = data
        .get("plasmid_name")
        .and_then(|v| v.as_str())
        .unwrap_or("plasmid");

    if let Some(arcs) = data
        .get("circular_map_data")
        .and_then(|v| v.get("arcs"))
        .and_then(|v| v.as_array())
    {
        let mut ring_set = std::collections::BTreeSet::new();
        for arc in arcs {
            if let Some(cat) = arc.get("category").and_then(|v| v.as_str()) {
                ring_set.insert(cat.to_string());
            }
        }
        let rings: Vec<String> = ring_set.into_iter().collect();
        b.push(json!({
            "channel_type": "circular_map",
            "id": "bl_plannotate_circular_map",
            "label": format!("pLannotate: {plasmid_name} Circular Map"),
            "sequence_length": plasmid_size,
            "rings": rings, "arcs": arcs, "unit": "bp",
        }));
    }

    b
}
