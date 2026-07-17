// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::viz::{bar_owned, distribution_owned};
use serde_json::Value;

pub fn ostir(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(preds) = data
        .get("rbs_strength_predictions")
        .and_then(|v| v.as_array())
    {
        let cats: Vec<String> = preds
            .iter()
            .filter_map(|p| p.get("name").and_then(|v| v.as_str()).map(String::from))
            .collect();
        let vals: Vec<f64> = preds
            .iter()
            .filter_map(|p| p.get("tir").and_then(serde_json::Value::as_f64))
            .collect();
        b.push(bar_owned(
            "bl_ostir_rbs_strength",
            "OSTIR: RBS Strength Predictions",
            cats,
            vals,
            "TIR (au)",
        ));

        for pred in preds {
            let name = pred.get("name").and_then(|v| v.as_str()).unwrap_or("RBS");
            if let Some(dg) = pred.get("dg_decomposition").and_then(|v| v.as_object()) {
                let terms = [
                    "dG_mRNA_rRNA",
                    "dG_spacing",
                    "dG_start_codon",
                    "dG_mRNA_unfold",
                    "dG_standby",
                    "dG_total",
                ];
                let (term_cats, term_vals): (Vec<String>, Vec<f64>) = terms
                    .iter()
                    .filter_map(|t| {
                        dg.get(*t)
                            .and_then(serde_json::Value::as_f64)
                            .map(|v| (t.to_string(), v))
                    })
                    .unzip();
                if !term_cats.is_empty() {
                    b.push(bar_owned(
                        &format!("bl_ostir_dg_{}", name.to_lowercase()),
                        &format!("OSTIR: dG Decomposition — {name}"),
                        term_cats,
                        term_vals,
                        "kcal/mol",
                    ));
                }
            }
        }
    }

    if let Some(dist) = data.get("rate_distribution")
        && let Some(v) = dist.get("values").and_then(|v| v.as_array())
    {
        b.push(distribution_owned(
            "bl_ostir_rate_distribution",
            "OSTIR: Translation Initiation Rate Distribution",
            v.iter().filter_map(serde_json::Value::as_f64).collect(),
            dist.get("mean")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            dist.get("std")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            "TIR (au)",
        ));
    }

    if let Some(codons) = data
        .get("start_codon_comparison")
        .and_then(|v| v.as_object())
    {
        let cats: Vec<String> = codons.keys().cloned().collect();
        let vals: Vec<f64> = codons
            .values()
            .filter_map(|v| v.get("mean_tir").and_then(serde_json::Value::as_f64))
            .collect();
        b.push(bar_owned(
            "bl_ostir_start_codons",
            "OSTIR: Start Codon Mean TIR",
            cats,
            vals,
            "TIR (au)",
        ));
    }

    b
}
