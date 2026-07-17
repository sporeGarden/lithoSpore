// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::viz::{bar_from_object, gauge, genome_track_owned, timeseries_owned, track_segment};
use serde_json::Value;

pub fn breseq(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(ev) = data
        .get("evidence_type_distribution")
        .and_then(|v| v.as_object())
    {
        b.push(bar_from_object(
            "bl_breseq_evidence_types",
            "breseq: Evidence Type Distribution",
            ev,
            "count",
        ));
    }
    if let Some(spec) = data.get("mutation_spectrum").and_then(|v| v.as_object()) {
        b.push(bar_from_object(
            "bl_breseq_mutation_spectrum",
            "breseq: 6-Class Mutation Spectrum",
            spec,
            "fraction",
        ));
    }

    if let Some(curve) = data.get("mutation_accumulation_curve") {
        let gens = curve.get("generations").and_then(|v| v.as_array());
        let muts = curve
            .get("expected_mutations_nonmutator")
            .and_then(|v| v.as_array());
        if let (Some(g), Some(m)) = (gens, muts) {
            b.push(timeseries_owned(
                "bl_breseq_accumulation",
                "breseq: Mutation Accumulation Curve",
                "Generation",
                "Expected Mutations",
                "mutations",
                g.iter().filter_map(serde_json::Value::as_f64).collect(),
                m.iter().filter_map(serde_json::Value::as_f64).collect(),
            ));
        }
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let genome_len = data
            .get("genome_length")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(ltee_breseq::E_COLI_K12_MG1655_BP);
        let segments: Vec<Value> = features
            .iter()
            .filter_map(|feat| {
                let ftype = feat.get("type").and_then(|v| v.as_str())?;
                match ftype {
                    "SNP" => {
                        let pos = feat
                            .get("position")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0);
                        let gene = feat.get("gene").and_then(|v| v.as_str()).unwrap_or("");
                        Some(track_segment(
                            "SNP",
                            pos,
                            genome_len.mul_add(0.003, pos),
                            "+",
                            gene,
                        ))
                    }
                    "IS_insertion" => {
                        let pos = feat
                            .get("position")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0);
                        let elem = feat.get("element").and_then(|v| v.as_str()).unwrap_or("IS");
                        Some(track_segment(
                            "IS Element",
                            pos,
                            genome_len.mul_add(0.005, pos),
                            "+",
                            elem,
                        ))
                    }
                    "large_deletion" => {
                        let s = feat
                            .get("start")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0);
                        let e = feat
                            .get("end")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0);
                        let genes = feat
                            .get("genes_affected")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0);
                        Some(track_segment(
                            "Large Deletion",
                            s,
                            e,
                            "-",
                            &format!("{genes} genes"),
                        ))
                    }
                    _ => None,
                }
            })
            .collect();

        b.push(genome_track_owned(
            "bl_breseq_genome_overview",
            "breseq: Genome Overview (REL606)",
            genome_len,
            vec!["SNP".into(), "IS Element".into(), "Large Deletion".into()],
            segments,
            "bp",
        ));
    }

    if let Some(stats) = data.get("summary_statistics") {
        let total = stats
            .get("total_predicted_mutations")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        b.push(gauge(
            "bl_breseq_total_mutations",
            "breseq: Total Predicted Mutations",
            total,
            0.0,
            200.0,
            "mutations",
            [0.0, 50.0],
            [50.0, 150.0],
        ));
    }

    b
}
