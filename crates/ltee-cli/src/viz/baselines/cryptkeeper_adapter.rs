// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::viz::{distribution_owned, gauge, genome_track_owned, heatmap_owned, track_segment};
use serde_json::Value;

pub fn cryptkeeper(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(hm) = data.get("burden_heatmap") {
        let regions = hm.get("regions").and_then(|v| v.as_array());
        let fwd = hm.get("forward_burden").and_then(|v| v.as_array());
        let rev = hm.get("reverse_burden").and_then(|v| v.as_array());
        if let (Some(r), Some(f), Some(rv)) = (regions, fwd, rev) {
            let x_labels: Vec<String> = r
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            let mut values: Vec<f64> = f.iter().filter_map(serde_json::Value::as_f64).collect();
            values.extend(rv.iter().filter_map(serde_json::Value::as_f64));
            b.push(heatmap_owned(
                "bl_cryptkeeper_burden",
                "CryptKeeper: Cryptic Expression Burden",
                x_labels,
                vec!["Forward".into(), "Reverse".into()],
                values,
                "au",
            ));
        }
    }

    if let Some(dist) = data.get("promoter_distribution")
        && let Some(v) = dist.get("values").and_then(|v| v.as_array())
    {
        b.push(distribution_owned(
            "bl_cryptkeeper_promoter_dist",
            "CryptKeeper: Promoter Strength Distribution",
            v.iter().filter_map(serde_json::Value::as_f64).collect(),
            dist.get("mean")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            dist.get("std")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            "au",
        ));
    }

    if let Some(burden) = data.get("total_burden") {
        let total = burden
            .get("combined_total")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        b.push(gauge(
            "bl_cryptkeeper_total_burden",
            "CryptKeeper: Total Expression Burden",
            total,
            0.0,
            5000.0,
            "au",
            [0.0, 500.0],
            [500.0, 2000.0],
        ));
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let construct_len = data
            .get("construct_length_bp")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(4200.0);
        let construct_name = data
            .get("construct_name")
            .and_then(|v| v.as_str())
            .unwrap_or("construct");
        let segments: Vec<Value> = features
            .iter()
            .map(|feat| {
                let ftype = feat.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let track = if ftype == "cryptic_promoter" {
                    "Cryptic Promoter"
                } else {
                    "ORF/Feature"
                };
                track_segment(
                    track,
                    feat.get("start")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0),
                    feat.get("end")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0),
                    feat.get("strand").and_then(|v| v.as_str()).unwrap_or("+"),
                    feat.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                )
            })
            .collect();

        b.push(genome_track_owned(
            "bl_cryptkeeper_genome_track",
            &format!("CryptKeeper: {construct_name} Multi-Track View"),
            construct_len,
            vec!["ORF/Feature".into(), "Cryptic Promoter".into()],
            segments,
            "bp",
        ));
    }

    b
}
