// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::viz::{bar_owned, heatmap_owned, scatter};
use serde_json::Value;

pub fn rna_mi(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(matrix) = data.get("mi_matrix") {
        let size = matrix
            .get("size")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize;
        let cols = matrix.get("columns_shown").and_then(|v| v.as_array());
        let vals = matrix.get("values").and_then(|v| v.as_array());
        if let (Some(c), Some(v)) = (cols, vals) {
            let labels: Vec<String> = c
                .iter()
                .filter_map(|x| x.as_u64().map(|n| format!("Col {n}")))
                .collect();
            let mut flat = Vec::with_capacity(size * size);
            for row in v.iter().filter_map(|r| r.as_array()) {
                for val in row {
                    flat.push(val.as_f64().unwrap_or(0.0));
                }
            }
            b.push(heatmap_owned(
                "bl_rna_mi_matrix",
                "RNA MI: Mutual Information Matrix (SAM-II)",
                labels.clone(),
                labels,
                flat,
                "MI (bits)",
            ));
        }
    }

    if let Some(pairs) = data.get("significant_pairs").and_then(|v| v.as_array()) {
        b.push(scatter(
            "bl_rna_mi_significant_pairs",
            "RNA MI: Significant Base Pairs",
            pairs
                .iter()
                .filter_map(|p| p.get("col_i").and_then(serde_json::Value::as_f64))
                .collect(),
            pairs
                .iter()
                .filter_map(|p| p.get("col_j").and_then(serde_json::Value::as_f64))
                .collect(),
            "Column i",
            "Column j",
            pairs
                .iter()
                .filter_map(|p| p.get("pairing").and_then(|v| v.as_str()).map(String::from))
                .collect(),
            "column index",
        ));
    }

    if let Some(entropy) = data.get("column_entropy") {
        let cols = entropy.get("columns").and_then(|v| v.as_array());
        let bits = entropy.get("entropy_bits").and_then(|v| v.as_array());
        if let (Some(c), Some(bv)) = (cols, bits) {
            b.push(bar_owned(
                "bl_rna_mi_entropy",
                "RNA MI: Per-Column Shannon Entropy",
                c.iter()
                    .filter_map(|v| v.as_u64().map(|n| n.to_string()))
                    .collect(),
                bv.iter().filter_map(serde_json::Value::as_f64).collect(),
                "bits",
            ));
        }
    }

    b
}
