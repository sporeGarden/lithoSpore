// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::viz::{bar_from_object, bar_owned, gauge, genome_track_owned, track_segment};
use serde_json::Value;

pub fn efm(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(counts) = data.get("site_counts").and_then(|v| v.as_object()) {
        b.push(bar_from_object(
            "bl_efm_site_counts",
            "EFM: Hypermutable Site Counts",
            counts,
            "sites",
        ));
    }
    if let Some(rates) = data.get("rate_distribution").and_then(|v| v.as_object()) {
        b.push(bar_from_object(
            "bl_efm_rate_distribution",
            "EFM: Mutation Rate Distribution",
            rates,
            "sites",
        ));
    }

    if let Some(cov) = data.get("region_coverage") {
        let frac = cov
            .get("fraction")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        b.push(gauge(
            "bl_efm_region_coverage",
            "EFM: Affected Region Fraction",
            frac,
            0.0,
            1.0,
            "fraction",
            [0.0, 0.3],
            [0.3, 0.7],
        ));
    }

    if let Some(rate) = data
        .get("total_evolutionary_failure_rate")
        .and_then(serde_json::Value::as_f64)
    {
        let half_life = data
            .get("half_life_generations")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        b.push(bar_owned(
            "bl_efm_summary",
            "EFM: Evolutionary Stability Summary",
            vec!["Failure Rate (×10⁶)".into(), "Half-life (×10³ gen)".into()],
            vec![rate * 1e6, half_life / 1000.0],
            "mixed",
        ));
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let seq_len = data
            .get("sequence_length_bp")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1252.0);
        let seq_name = data
            .get("sequence_name")
            .and_then(|v| v.as_str())
            .unwrap_or("sequence");
        let segments: Vec<Value> = features
            .iter()
            .map(|feat| {
                let ftype = feat.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let track = match ftype {
                    "IS_target" => "IS Target",
                    "repeat_indel" => "Repeat Indel",
                    "base_sub_hotspot" => "Base Sub Hotspot",
                    _ => "Feature",
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
                    feat.get("name")
                        .and_then(|v| v.as_str())
                        .or_else(|| feat.get("element").and_then(|v| v.as_str()))
                        .unwrap_or(""),
                )
            })
            .collect();

        b.push(genome_track_owned(
            "bl_efm_genome_track",
            &format!("EFM: {seq_name} Rate-Colored Track"),
            seq_len,
            vec![
                "Feature".into(),
                "IS Target".into(),
                "Repeat Indel".into(),
                "Base Sub Hotspot".into(),
            ],
            segments,
            "bp",
        ));
    }

    b
}
