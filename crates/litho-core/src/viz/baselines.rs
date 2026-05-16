// SPDX-License-Identifier: AGPL-3.0-or-later

//! Barrick Lab baseline tool DataBinding adapters.
//!
//! Each adapter reads a tool's `reference_data.json` and produces
//! petalTongue-compatible `DataBinding` JSON values reproducing
//! the tool's key visualization patterns.

use super::{bar_from_object, bar, gauge, timeseries, scatter, heatmap, distribution, genome_track, track_segment};
use serde_json::{json, Value};

pub(crate) fn breseq(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(ev) = data.get("evidence_type_distribution").and_then(|v| v.as_object()) {
        b.push(bar_from_object("bl_breseq_evidence_types", "breseq: Evidence Type Distribution", ev, "count"));
    }
    if let Some(spec) = data.get("mutation_spectrum").and_then(|v| v.as_object()) {
        b.push(bar_from_object("bl_breseq_mutation_spectrum", "breseq: 6-Class Mutation Spectrum", spec, "fraction"));
    }

    if let Some(curve) = data.get("mutation_accumulation_curve") {
        let gens = curve.get("generations").and_then(|v| v.as_array());
        let muts = curve.get("expected_mutations_nonmutator").and_then(|v| v.as_array());
        if let (Some(g), Some(m)) = (gens, muts) {
            b.push(timeseries(
                "bl_breseq_accumulation", "breseq: Mutation Accumulation Curve",
                "Generation", "Expected Mutations", "mutations",
                g.iter().filter_map(|v| v.as_f64()).collect(),
                m.iter().filter_map(|v| v.as_f64()).collect(),
            ));
        }
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let genome_len = data.get("genome_length").and_then(|v| v.as_f64()).unwrap_or(4_629_812.0);
        let segments: Vec<Value> = features.iter().filter_map(|feat| {
            let ftype = feat.get("type").and_then(|v| v.as_str())?;
            match ftype {
                "SNP" => {
                    let pos = feat.get("position").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let gene = feat.get("gene").and_then(|v| v.as_str()).unwrap_or("");
                    Some(track_segment("SNP", pos, pos + genome_len * 0.003, "+", gene))
                }
                "IS_insertion" => {
                    let pos = feat.get("position").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let elem = feat.get("element").and_then(|v| v.as_str()).unwrap_or("IS");
                    Some(track_segment("IS Element", pos, pos + genome_len * 0.005, "+", elem))
                }
                "large_deletion" => {
                    let s = feat.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let e = feat.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let genes = feat.get("genes_affected").and_then(|v| v.as_u64()).unwrap_or(0);
                    Some(track_segment("Large Deletion", s, e, "-", &format!("{genes} genes")))
                }
                _ => None,
            }
        }).collect();

        b.push(genome_track(
            "bl_breseq_genome_overview", "breseq: Genome Overview (REL606)",
            genome_len,
            vec!["SNP".into(), "IS Element".into(), "Large Deletion".into()],
            segments, "bp",
        ));
    }

    if let Some(stats) = data.get("summary_statistics") {
        let total = stats.get("total_predicted_mutations").and_then(|v| v.as_f64()).unwrap_or(0.0);
        b.push(gauge("bl_breseq_total_mutations", "breseq: Total Predicted Mutations", total, 0.0, 200.0, "mutations", [0.0, 50.0], [50.0, 150.0]));
    }

    b
}

pub(crate) fn plannotate(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(dist) = data.get("feature_distribution").and_then(|v| v.as_object()) {
        b.push(bar_from_object("bl_plannotate_features", "pLannotate: Feature Category Distribution", dist, "count"));
    }
    if let Some(cov) = data.get("annotation_coverage").and_then(|v| v.as_f64()) {
        b.push(gauge("bl_plannotate_coverage", "pLannotate: Annotation Coverage", cov, 0.0, 1.0, "fraction", [0.8, 1.0], [0.5, 0.8]));
    }

    let plasmid_size = data.get("plasmid_size_bp").and_then(|v| v.as_f64()).unwrap_or(2686.0);
    let plasmid_name = data.get("plasmid_name").and_then(|v| v.as_str()).unwrap_or("plasmid");

    if let Some(arcs) = data.get("circular_map_data").and_then(|v| v.get("arcs")).and_then(|v| v.as_array()) {
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

pub(crate) fn ostir(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(preds) = data.get("rbs_strength_predictions").and_then(|v| v.as_array()) {
        let cats: Vec<String> = preds.iter().filter_map(|p| p.get("name").and_then(|v| v.as_str()).map(String::from)).collect();
        let vals: Vec<f64> = preds.iter().filter_map(|p| p.get("tir").and_then(|v| v.as_f64())).collect();
        b.push(bar("bl_ostir_rbs_strength", "OSTIR: RBS Strength Predictions", cats, vals, "TIR (au)"));

        for pred in preds {
            let name = pred.get("name").and_then(|v| v.as_str()).unwrap_or("RBS");
            if let Some(dg) = pred.get("dg_decomposition").and_then(|v| v.as_object()) {
                let terms = ["dG_mRNA_rRNA", "dG_spacing", "dG_start_codon", "dG_mRNA_unfold", "dG_standby", "dG_total"];
                let (term_cats, term_vals): (Vec<String>, Vec<f64>) = terms.iter()
                    .filter_map(|t| dg.get(*t).and_then(|v| v.as_f64()).map(|v| (t.to_string(), v)))
                    .unzip();
                if !term_cats.is_empty() {
                    b.push(bar(
                        &format!("bl_ostir_dg_{}", name.to_lowercase()),
                        &format!("OSTIR: dG Decomposition — {name}"),
                        term_cats, term_vals, "kcal/mol",
                    ));
                }
            }
        }
    }

    if let Some(dist) = data.get("rate_distribution") {
        if let Some(v) = dist.get("values").and_then(|v| v.as_array()) {
            b.push(distribution(
                "bl_ostir_rate_distribution", "OSTIR: Translation Initiation Rate Distribution",
                v.iter().filter_map(|x| x.as_f64()).collect(),
                dist.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0),
                dist.get("std").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "TIR (au)",
            ));
        }
    }

    if let Some(codons) = data.get("start_codon_comparison").and_then(|v| v.as_object()) {
        let cats: Vec<String> = codons.keys().cloned().collect();
        let vals: Vec<f64> = codons.values().filter_map(|v| v.get("mean_tir").and_then(|t| t.as_f64())).collect();
        b.push(bar("bl_ostir_start_codons", "OSTIR: Start Codon Mean TIR", cats, vals, "TIR (au)"));
    }

    b
}

pub(crate) fn cryptkeeper(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(hm) = data.get("burden_heatmap") {
        let regions = hm.get("regions").and_then(|v| v.as_array());
        let fwd = hm.get("forward_burden").and_then(|v| v.as_array());
        let rev = hm.get("reverse_burden").and_then(|v| v.as_array());
        if let (Some(r), Some(f), Some(rv)) = (regions, fwd, rev) {
            let x_labels: Vec<String> = r.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            let mut values: Vec<f64> = f.iter().filter_map(|v| v.as_f64()).collect();
            values.extend(rv.iter().filter_map(|v| v.as_f64()));
            b.push(heatmap(
                "bl_cryptkeeper_burden", "CryptKeeper: Cryptic Expression Burden",
                x_labels, vec!["Forward".into(), "Reverse".into()], values, "au",
            ));
        }
    }

    if let Some(dist) = data.get("promoter_distribution") {
        if let Some(v) = dist.get("values").and_then(|v| v.as_array()) {
            b.push(distribution(
                "bl_cryptkeeper_promoter_dist", "CryptKeeper: Promoter Strength Distribution",
                v.iter().filter_map(|x| x.as_f64()).collect(),
                dist.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0),
                dist.get("std").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "au",
            ));
        }
    }

    if let Some(burden) = data.get("total_burden") {
        let total = burden.get("combined_total").and_then(|v| v.as_f64()).unwrap_or(0.0);
        b.push(gauge("bl_cryptkeeper_total_burden", "CryptKeeper: Total Expression Burden", total, 0.0, 5000.0, "au", [0.0, 500.0], [500.0, 2000.0]));
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let construct_len = data.get("construct_length_bp").and_then(|v| v.as_f64()).unwrap_or(4200.0);
        let construct_name = data.get("construct_name").and_then(|v| v.as_str()).unwrap_or("construct");
        let segments: Vec<Value> = features.iter().map(|feat| {
            let ftype = feat.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let track = if ftype == "cryptic_promoter" { "Cryptic Promoter" } else { "ORF/Feature" };
            track_segment(
                track,
                feat.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0),
                feat.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0),
                feat.get("strand").and_then(|v| v.as_str()).unwrap_or("+"),
                feat.get("name").and_then(|v| v.as_str()).unwrap_or(""),
            )
        }).collect();

        b.push(genome_track(
            "bl_cryptkeeper_genome_track",
            &format!("CryptKeeper: {construct_name} Multi-Track View"),
            construct_len,
            vec!["ORF/Feature".into(), "Cryptic Promoter".into()],
            segments, "bp",
        ));
    }

    b
}

pub(crate) fn efm(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(counts) = data.get("site_counts").and_then(|v| v.as_object()) {
        b.push(bar_from_object("bl_efm_site_counts", "EFM: Hypermutable Site Counts", counts, "sites"));
    }
    if let Some(rates) = data.get("rate_distribution").and_then(|v| v.as_object()) {
        b.push(bar_from_object("bl_efm_rate_distribution", "EFM: Mutation Rate Distribution", rates, "sites"));
    }

    if let Some(cov) = data.get("region_coverage") {
        let frac = cov.get("fraction").and_then(|v| v.as_f64()).unwrap_or(0.0);
        b.push(gauge("bl_efm_region_coverage", "EFM: Affected Region Fraction", frac, 0.0, 1.0, "fraction", [0.0, 0.3], [0.3, 0.7]));
    }

    if let Some(rate) = data.get("total_evolutionary_failure_rate").and_then(|v| v.as_f64()) {
        let half_life = data.get("half_life_generations").and_then(|v| v.as_f64()).unwrap_or(0.0);
        b.push(bar(
            "bl_efm_summary", "EFM: Evolutionary Stability Summary",
            vec!["Failure Rate (×10⁶)".into(), "Half-life (×10³ gen)".into()],
            vec![rate * 1e6, half_life / 1000.0], "mixed",
        ));
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let seq_len = data.get("sequence_length_bp").and_then(|v| v.as_f64()).unwrap_or(1252.0);
        let seq_name = data.get("sequence_name").and_then(|v| v.as_str()).unwrap_or("sequence");
        let segments: Vec<Value> = features.iter().map(|feat| {
            let ftype = feat.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let track = match ftype {
                "CDS" | "terminator" | "rep_origin" | "promoter" => "Feature",
                "IS_target" => "IS Target",
                "repeat_indel" => "Repeat Indel",
                "base_sub_hotspot" => "Base Sub Hotspot",
                _ => "Feature",
            };
            track_segment(
                track,
                feat.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0),
                feat.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0),
                feat.get("strand").and_then(|v| v.as_str()).unwrap_or("+"),
                feat.get("name").and_then(|v| v.as_str())
                    .or_else(|| feat.get("element").and_then(|v| v.as_str()))
                    .unwrap_or(""),
            )
        }).collect();

        b.push(genome_track(
            "bl_efm_genome_track",
            &format!("EFM: {seq_name} Rate-Colored Track"),
            seq_len,
            vec!["Feature".into(), "IS Target".into(), "Repeat Indel".into(), "Base Sub Hotspot".into()],
            segments, "bp",
        ));
    }

    b
}

pub(crate) fn marker_divergence(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(curves) = data.get("divergence_curves") {
        let transfers = curves.get("transfers").and_then(|v| v.as_array());
        let series = curves.get("series").and_then(|v| v.as_object());
        if let (Some(t), Some(s)) = (transfers, series) {
            let x: Vec<f64> = t.iter().filter_map(|v| v.as_f64()).collect();
            for (name, vals) in s {
                if let Some(arr) = vals.as_array() {
                    b.push(timeseries(
                        &format!("bl_md_divergence_{name}"),
                        &format!("Marker Divergence: {name}"),
                        "Transfer", "Marker Ratio", "ratio",
                        x.clone(), arr.iter().filter_map(|v| v.as_f64()).collect(),
                    ));
                }
            }
        }
    }

    if let Some(params) = data.get("fitted_parameters").and_then(|v| v.as_object()) {
        let (mut x, mut y, mut labels) = (Vec::new(), Vec::new(), Vec::new());
        for (name, fit) in params {
            x.push(fit.get("alpha").and_then(|v| v.as_f64()).unwrap_or(0.0));
            y.push(fit.get("tau").and_then(|v| v.as_f64()).unwrap_or(0.0));
            labels.push(name.clone());
        }
        b.push(scatter("bl_md_parameter_scatter", "Marker Divergence: Fitted (α, τ) Parameters", x, y, "α", "τ", labels, "dimensionless"));
    }

    if let Some(contour) = data.get("confidence_contour") {
        if let Some(ml_val) = contour.get("maximum_likelihood") {
            let mu = ml_val.get("log10_mu").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let s = ml_val.get("s").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let (mut x_pts, mut y_pts, mut labels) = (vec![mu], vec![s], vec!["Maximum Likelihood".to_string()]);

            if let Some(verts) = contour.get("contour_95_vertices").and_then(|v| v.as_array()) {
                for (i, vert) in verts.iter().enumerate() {
                    if let Some(arr) = vert.as_array() {
                        x_pts.push(arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0));
                        y_pts.push(arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0));
                        labels.push(format!("95% contour {}", i + 1));
                    }
                }
            }

            b.push(scatter("bl_md_confidence_contour", "Marker Divergence: ML + 95% Confidence Contour (μ, s)", x_pts, y_pts, "log₁₀(μ)", "s (selection coefficient)", labels, "dimensionless"));
        }
    }

    b
}

pub(crate) fn rna_mi(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(matrix) = data.get("mi_matrix") {
        let size = matrix.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let cols = matrix.get("columns_shown").and_then(|v| v.as_array());
        let vals = matrix.get("values").and_then(|v| v.as_array());
        if let (Some(c), Some(v)) = (cols, vals) {
            let labels: Vec<String> = c.iter().filter_map(|x| x.as_u64().map(|n| format!("Col {n}"))).collect();
            let mut flat = Vec::with_capacity(size * size);
            for row in v.iter().filter_map(|r| r.as_array()) {
                for val in row {
                    flat.push(val.as_f64().unwrap_or(0.0));
                }
            }
            b.push(heatmap("bl_rna_mi_matrix", "RNA MI: Mutual Information Matrix (SAM-II)", labels.clone(), labels, flat, "MI (bits)"));
        }
    }

    if let Some(pairs) = data.get("significant_pairs").and_then(|v| v.as_array()) {
        b.push(scatter(
            "bl_rna_mi_significant_pairs", "RNA MI: Significant Base Pairs",
            pairs.iter().filter_map(|p| p.get("col_i").and_then(|v| v.as_f64())).collect(),
            pairs.iter().filter_map(|p| p.get("col_j").and_then(|v| v.as_f64())).collect(),
            "Column i", "Column j",
            pairs.iter().filter_map(|p| p.get("pairing").and_then(|v| v.as_str()).map(String::from)).collect(),
            "column index",
        ));
    }

    if let Some(entropy) = data.get("column_entropy") {
        let cols = entropy.get("columns").and_then(|v| v.as_array());
        let bits = entropy.get("entropy_bits").and_then(|v| v.as_array());
        if let (Some(c), Some(bv)) = (cols, bits) {
            b.push(bar(
                "bl_rna_mi_entropy", "RNA MI: Per-Column Shannon Entropy",
                c.iter().filter_map(|v| v.as_u64().map(|n| n.to_string())).collect(),
                bv.iter().filter_map(|v| v.as_f64()).collect(),
                "bits",
            ));
        }
    }

    b
}
