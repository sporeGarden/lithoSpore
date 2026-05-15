// SPDX-License-Identifier: AGPL-3.0-or-later

//! Barrick Lab baseline tool DataBinding adapters.
//!
//! Each adapter reads a tool's `reference_data.json` and produces
//! petalTongue-compatible `DataBinding` JSON values reproducing
//! the tool's key visualization patterns.

use serde_json::{json, Value};

pub(crate) fn breseq(data: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(ev) = data.get("evidence_type_distribution").and_then(|v| v.as_object()) {
        let cats: Vec<String> = ev.keys().cloned().collect();
        let vals: Vec<f64> = ev.values().filter_map(|v| v.as_f64()).collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "bl_breseq_evidence_types",
            "label": "breseq: Evidence Type Distribution",
            "categories": cats,
            "values": vals,
            "unit": "count",
        }));
    }

    if let Some(spec) = data.get("mutation_spectrum").and_then(|v| v.as_object()) {
        let cats: Vec<String> = spec.keys().cloned().collect();
        let vals: Vec<f64> = spec.values().filter_map(|v| v.as_f64()).collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "bl_breseq_mutation_spectrum",
            "label": "breseq: 6-Class Mutation Spectrum",
            "categories": cats,
            "values": vals,
            "unit": "fraction",
        }));
    }

    if let Some(curve) = data.get("mutation_accumulation_curve") {
        let gens = curve.get("generations").and_then(|v| v.as_array());
        let muts = curve.get("expected_mutations_nonmutator").and_then(|v| v.as_array());
        if let (Some(g), Some(m)) = (gens, muts) {
            let x: Vec<f64> = g.iter().filter_map(|v| v.as_f64()).collect();
            let y: Vec<f64> = m.iter().filter_map(|v| v.as_f64()).collect();
            bindings.push(json!({
                "channel_type": "timeseries",
                "id": "bl_breseq_accumulation",
                "label": "breseq: Mutation Accumulation Curve",
                "x_label": "Generation",
                "y_label": "Expected Mutations",
                "unit": "mutations",
                "x_values": x,
                "y_values": y,
            }));
        }
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let genome_len = data.get("genome_length").and_then(|v| v.as_f64()).unwrap_or(4_629_812.0);
        let tracks = vec![
            "SNP".to_string(),
            "IS Element".to_string(),
            "Large Deletion".to_string(),
        ];

        let mut segments = Vec::new();
        for feat in features {
            let ftype = feat.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let (track, start, end, strand, label) = match ftype {
                "SNP" => {
                    let pos = feat.get("position").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let gene = feat.get("gene").and_then(|v| v.as_str()).unwrap_or("");
                    ("SNP", pos, pos + genome_len * 0.003, "+", gene.to_string())
                }
                "IS_insertion" => {
                    let pos = feat.get("position").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let elem = feat.get("element").and_then(|v| v.as_str()).unwrap_or("IS");
                    ("IS Element", pos, pos + genome_len * 0.005, "+", elem.to_string())
                }
                "large_deletion" => {
                    let s = feat.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let e = feat.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let genes = feat.get("genes_affected").and_then(|v| v.as_u64()).unwrap_or(0);
                    ("Large Deletion", s, e, "-", format!("{genes} genes"))
                }
                _ => continue,
            };
            segments.push(json!({
                "track": track,
                "start": start,
                "end": end,
                "strand": strand,
                "label": label,
            }));
        }

        bindings.push(json!({
            "channel_type": "genome_track",
            "id": "bl_breseq_genome_overview",
            "label": "breseq: Genome Overview (REL606)",
            "sequence_length": genome_len,
            "tracks": tracks,
            "segments": segments,
            "unit": "bp",
        }));
    }

    if let Some(stats) = data.get("summary_statistics") {
        let total = stats.get("total_predicted_mutations").and_then(|v| v.as_f64()).unwrap_or(0.0);
        bindings.push(json!({
            "channel_type": "gauge",
            "id": "bl_breseq_total_mutations",
            "label": "breseq: Total Predicted Mutations",
            "value": total,
            "min": 0.0,
            "max": 200.0,
            "unit": "mutations",
            "normal_range": [0.0, 50.0],
            "warning_range": [50.0, 150.0],
        }));
    }

    bindings
}

pub(crate) fn plannotate(data: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(dist) = data.get("feature_distribution").and_then(|v| v.as_object()) {
        let cats: Vec<String> = dist.keys().cloned().collect();
        let vals: Vec<f64> = dist.values().filter_map(|v| v.as_f64()).collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "bl_plannotate_features",
            "label": "pLannotate: Feature Category Distribution",
            "categories": cats,
            "values": vals,
            "unit": "count",
        }));
    }

    if let Some(cov) = data.get("annotation_coverage").and_then(|v| v.as_f64()) {
        bindings.push(json!({
            "channel_type": "gauge",
            "id": "bl_plannotate_coverage",
            "label": "pLannotate: Annotation Coverage",
            "value": cov,
            "min": 0.0,
            "max": 1.0,
            "unit": "fraction",
            "normal_range": [0.8, 1.0],
            "warning_range": [0.5, 0.8],
        }));
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

        bindings.push(json!({
            "channel_type": "circular_map",
            "id": "bl_plannotate_circular_map",
            "label": format!("pLannotate: {plasmid_name} Circular Map"),
            "sequence_length": plasmid_size,
            "rings": rings,
            "arcs": arcs,
            "unit": "bp",
        }));
    }

    bindings
}

pub(crate) fn ostir(data: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(preds) = data.get("rbs_strength_predictions").and_then(|v| v.as_array()) {
        let cats: Vec<String> = preds.iter()
            .filter_map(|p| p.get("name").and_then(|v| v.as_str()).map(String::from))
            .collect();
        let vals: Vec<f64> = preds.iter()
            .filter_map(|p| p.get("tir").and_then(|v| v.as_f64()))
            .collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "bl_ostir_rbs_strength",
            "label": "OSTIR: RBS Strength Predictions",
            "categories": cats,
            "values": vals,
            "unit": "TIR (au)",
        }));

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
                let mut term_cats = Vec::new();
                let mut term_vals = Vec::new();
                for term in &terms {
                    if let Some(val) = dg.get(*term).and_then(|v| v.as_f64()) {
                        term_cats.push(term.to_string());
                        term_vals.push(val);
                    }
                }
                if !term_cats.is_empty() {
                    bindings.push(json!({
                        "channel_type": "bar",
                        "id": format!("bl_ostir_dg_{}", name.to_lowercase()),
                        "label": format!("OSTIR: dG Decomposition — {name}"),
                        "categories": term_cats,
                        "values": term_vals,
                        "unit": "kcal/mol",
                    }));
                }
            }
        }
    }

    if let Some(dist) = data.get("rate_distribution") {
        let vals = dist.get("values").and_then(|v| v.as_array());
        if let Some(v) = vals {
            let values: Vec<f64> = v.iter().filter_map(|x| x.as_f64()).collect();
            let mean = dist.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let std = dist.get("std").and_then(|v| v.as_f64()).unwrap_or(0.0);
            bindings.push(json!({
                "channel_type": "distribution",
                "id": "bl_ostir_rate_distribution",
                "label": "OSTIR: Translation Initiation Rate Distribution",
                "values": values,
                "mean": mean,
                "std": std,
                "unit": "TIR (au)",
            }));
        }
    }

    if let Some(codons) = data.get("start_codon_comparison").and_then(|v| v.as_object()) {
        let cats: Vec<String> = codons.keys().cloned().collect();
        let vals: Vec<f64> = codons.values()
            .filter_map(|v| v.get("mean_tir").and_then(|t| t.as_f64()))
            .collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "bl_ostir_start_codons",
            "label": "OSTIR: Start Codon Mean TIR",
            "categories": cats,
            "values": vals,
            "unit": "TIR (au)",
        }));
    }

    bindings
}

pub(crate) fn cryptkeeper(data: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(hm) = data.get("burden_heatmap") {
        let regions = hm.get("regions").and_then(|v| v.as_array());
        let fwd = hm.get("forward_burden").and_then(|v| v.as_array());
        let rev = hm.get("reverse_burden").and_then(|v| v.as_array());
        if let (Some(r), Some(f), Some(rv)) = (regions, fwd, rev) {
            let x_labels: Vec<String> = r.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            let y_labels = vec!["Forward".to_string(), "Reverse".to_string()];
            let mut values: Vec<f64> = f.iter().filter_map(|v| v.as_f64()).collect();
            values.extend(rv.iter().filter_map(|v| v.as_f64()));
            bindings.push(json!({
                "channel_type": "heatmap",
                "id": "bl_cryptkeeper_burden",
                "label": "CryptKeeper: Cryptic Expression Burden",
                "x_labels": x_labels,
                "y_labels": y_labels,
                "values": values,
                "unit": "au",
            }));
        }
    }

    if let Some(dist) = data.get("promoter_distribution") {
        let vals = dist.get("values").and_then(|v| v.as_array());
        if let Some(v) = vals {
            let values: Vec<f64> = v.iter().filter_map(|x| x.as_f64()).collect();
            let mean = dist.get("mean").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let std = dist.get("std").and_then(|v| v.as_f64()).unwrap_or(0.0);
            bindings.push(json!({
                "channel_type": "distribution",
                "id": "bl_cryptkeeper_promoter_dist",
                "label": "CryptKeeper: Promoter Strength Distribution",
                "values": values,
                "mean": mean,
                "std": std,
                "unit": "au",
            }));
        }
    }

    if let Some(burden) = data.get("total_burden") {
        let total = burden.get("combined_total").and_then(|v| v.as_f64()).unwrap_or(0.0);
        bindings.push(json!({
            "channel_type": "gauge",
            "id": "bl_cryptkeeper_total_burden",
            "label": "CryptKeeper: Total Expression Burden",
            "value": total,
            "min": 0.0,
            "max": 5000.0,
            "unit": "au",
            "normal_range": [0.0, 500.0],
            "warning_range": [500.0, 2000.0],
        }));
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let construct_len = data.get("construct_length_bp").and_then(|v| v.as_f64()).unwrap_or(4200.0);
        let construct_name = data.get("construct_name").and_then(|v| v.as_str()).unwrap_or("construct");
        let tracks = vec![
            "ORF/Feature".to_string(),
            "Cryptic Promoter".to_string(),
        ];

        let mut segments = Vec::new();
        for feat in features {
            let ftype = feat.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let start = feat.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let end = feat.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let strand = feat.get("strand").and_then(|v| v.as_str()).unwrap_or("+");
            let name = feat.get("name").and_then(|v| v.as_str()).unwrap_or("");

            let track = if ftype == "cryptic_promoter" {
                "Cryptic Promoter"
            } else {
                "ORF/Feature"
            };

            segments.push(json!({
                "track": track,
                "start": start,
                "end": end,
                "strand": strand,
                "label": name,
            }));
        }

        bindings.push(json!({
            "channel_type": "genome_track",
            "id": "bl_cryptkeeper_genome_track",
            "label": format!("CryptKeeper: {construct_name} Multi-Track View"),
            "sequence_length": construct_len,
            "tracks": tracks,
            "segments": segments,
            "unit": "bp",
        }));
    }

    bindings
}

pub(crate) fn efm(data: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(counts) = data.get("site_counts").and_then(|v| v.as_object()) {
        let cats: Vec<String> = counts.keys().cloned().collect();
        let vals: Vec<f64> = counts.values().filter_map(|v| v.as_f64()).collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "bl_efm_site_counts",
            "label": "EFM: Hypermutable Site Counts",
            "categories": cats,
            "values": vals,
            "unit": "sites",
        }));
    }

    if let Some(rates) = data.get("rate_distribution").and_then(|v| v.as_object()) {
        let cats: Vec<String> = rates.keys().cloned().collect();
        let vals: Vec<f64> = rates.values().filter_map(|v| v.as_f64()).collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "bl_efm_rate_distribution",
            "label": "EFM: Mutation Rate Distribution",
            "categories": cats,
            "values": vals,
            "unit": "sites",
        }));
    }

    if let Some(cov) = data.get("region_coverage") {
        let frac = cov.get("fraction").and_then(|v| v.as_f64()).unwrap_or(0.0);
        bindings.push(json!({
            "channel_type": "gauge",
            "id": "bl_efm_region_coverage",
            "label": "EFM: Affected Region Fraction",
            "value": frac,
            "min": 0.0,
            "max": 1.0,
            "unit": "fraction",
            "normal_range": [0.0, 0.3],
            "warning_range": [0.3, 0.7],
        }));
    }

    if let Some(rate) = data.get("total_evolutionary_failure_rate").and_then(|v| v.as_f64()) {
        let half_life = data.get("half_life_generations").and_then(|v| v.as_f64()).unwrap_or(0.0);
        bindings.push(json!({
            "channel_type": "bar",
            "id": "bl_efm_summary",
            "label": "EFM: Evolutionary Stability Summary",
            "categories": ["Failure Rate (×10⁶)", "Half-life (×10³ gen)"],
            "values": [rate * 1e6, half_life / 1000.0],
            "unit": "mixed",
        }));
    }

    if let Some(features) = data.get("genome_track_features").and_then(|v| v.as_array()) {
        let seq_len = data.get("sequence_length_bp").and_then(|v| v.as_f64()).unwrap_or(1252.0);
        let seq_name = data.get("sequence_name").and_then(|v| v.as_str()).unwrap_or("sequence");
        let tracks = vec![
            "Feature".to_string(),
            "IS Target".to_string(),
            "Repeat Indel".to_string(),
            "Base Sub Hotspot".to_string(),
        ];

        let mut segments = Vec::new();
        for feat in features {
            let ftype = feat.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let start = feat.get("start").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let end = feat.get("end").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let strand = feat.get("strand").and_then(|v| v.as_str()).unwrap_or("+");
            let name = feat.get("name").and_then(|v| v.as_str())
                .or_else(|| feat.get("element").and_then(|v| v.as_str()))
                .unwrap_or("");

            let track = match ftype {
                "CDS" | "terminator" | "rep_origin" | "promoter" => "Feature",
                "IS_target" => "IS Target",
                "repeat_indel" => "Repeat Indel",
                "base_sub_hotspot" => "Base Sub Hotspot",
                _ => "Feature",
            };

            segments.push(json!({
                "track": track,
                "start": start,
                "end": end,
                "strand": strand,
                "label": name,
            }));
        }

        bindings.push(json!({
            "channel_type": "genome_track",
            "id": "bl_efm_genome_track",
            "label": format!("EFM: {seq_name} Rate-Colored Track"),
            "sequence_length": seq_len,
            "tracks": tracks,
            "segments": segments,
            "unit": "bp",
        }));
    }

    bindings
}

pub(crate) fn marker_divergence(data: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(curves) = data.get("divergence_curves") {
        let transfers = curves.get("transfers").and_then(|v| v.as_array());
        let series = curves.get("series").and_then(|v| v.as_object());
        if let (Some(t), Some(s)) = (transfers, series) {
            let x: Vec<f64> = t.iter().filter_map(|v| v.as_f64()).collect();
            for (name, vals) in s {
                if let Some(arr) = vals.as_array() {
                    let y: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();
                    bindings.push(json!({
                        "channel_type": "timeseries",
                        "id": format!("bl_md_divergence_{name}"),
                        "label": format!("Marker Divergence: {name}"),
                        "x_label": "Transfer",
                        "y_label": "Marker Ratio",
                        "unit": "ratio",
                        "x_values": x,
                        "y_values": y,
                    }));
                }
            }
        }
    }

    if let Some(params) = data.get("fitted_parameters").and_then(|v| v.as_object()) {
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut labels = Vec::new();
        for (name, fit) in params {
            let alpha = fit.get("alpha").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let tau = fit.get("tau").and_then(|v| v.as_f64()).unwrap_or(0.0);
            x.push(alpha);
            y.push(tau);
            labels.push(name.clone());
        }
        bindings.push(json!({
            "channel_type": "scatter",
            "id": "bl_md_parameter_scatter",
            "label": "Marker Divergence: Fitted (α, τ) Parameters",
            "x": x,
            "y": y,
            "x_label": "α",
            "y_label": "τ",
            "point_labels": labels,
            "unit": "dimensionless",
        }));
    }

    if let Some(contour) = data.get("confidence_contour") {
        let ml = contour.get("maximum_likelihood");
        if let Some(ml_val) = ml {
            let mu = ml_val.get("log10_mu").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let s = ml_val.get("s").and_then(|v| v.as_f64()).unwrap_or(0.0);

            let mut x_pts = vec![mu];
            let mut y_pts = vec![s];
            let mut labels = vec!["Maximum Likelihood".to_string()];

            if let Some(verts) = contour.get("contour_95_vertices").and_then(|v| v.as_array()) {
                for (i, vert) in verts.iter().enumerate() {
                    if let Some(arr) = vert.as_array() {
                        let vx = arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let vy = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        x_pts.push(vx);
                        y_pts.push(vy);
                        labels.push(format!("95% contour {}", i + 1));
                    }
                }
            }

            bindings.push(json!({
                "channel_type": "scatter",
                "id": "bl_md_confidence_contour",
                "label": "Marker Divergence: ML + 95% Confidence Contour (μ, s)",
                "x": x_pts,
                "y": y_pts,
                "x_label": "log₁₀(μ)",
                "y_label": "s (selection coefficient)",
                "point_labels": labels,
                "unit": "dimensionless",
            }));
        }
    }

    bindings
}

pub(crate) fn rna_mi(data: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(matrix) = data.get("mi_matrix") {
        let size = matrix.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let cols = matrix.get("columns_shown").and_then(|v| v.as_array());
        let vals = matrix.get("values").and_then(|v| v.as_array());
        if let (Some(c), Some(v)) = (cols, vals) {
            let x_labels: Vec<String> = c.iter()
                .filter_map(|x| x.as_u64().map(|n| format!("Col {n}")))
                .collect();
            let y_labels = x_labels.clone();
            let mut flat_vals = Vec::with_capacity(size * size);
            for row in v.iter().filter_map(|r| r.as_array()) {
                for val in row {
                    flat_vals.push(val.as_f64().unwrap_or(0.0));
                }
            }
            bindings.push(json!({
                "channel_type": "heatmap",
                "id": "bl_rna_mi_matrix",
                "label": "RNA MI: Mutual Information Matrix (SAM-II)",
                "x_labels": x_labels,
                "y_labels": y_labels,
                "values": flat_vals,
                "unit": "MI (bits)",
            }));
        }
    }

    if let Some(pairs) = data.get("significant_pairs").and_then(|v| v.as_array()) {
        let x: Vec<f64> = pairs.iter().filter_map(|p| p.get("col_i").and_then(|v| v.as_f64())).collect();
        let y: Vec<f64> = pairs.iter().filter_map(|p| p.get("col_j").and_then(|v| v.as_f64())).collect();
        let labels: Vec<String> = pairs.iter()
            .filter_map(|p| p.get("pairing").and_then(|v| v.as_str()).map(String::from))
            .collect();
        bindings.push(json!({
            "channel_type": "scatter",
            "id": "bl_rna_mi_significant_pairs",
            "label": "RNA MI: Significant Base Pairs",
            "x": x,
            "y": y,
            "x_label": "Column i",
            "y_label": "Column j",
            "point_labels": labels,
            "unit": "column index",
        }));
    }

    if let Some(entropy) = data.get("column_entropy") {
        let cols = entropy.get("columns").and_then(|v| v.as_array());
        let bits = entropy.get("entropy_bits").and_then(|v| v.as_array());
        if let (Some(c), Some(b)) = (cols, bits) {
            let cats: Vec<String> = c.iter().filter_map(|v| v.as_u64().map(|n| n.to_string())).collect();
            let vals: Vec<f64> = b.iter().filter_map(|v| v.as_f64()).collect();
            bindings.push(json!({
                "channel_type": "bar",
                "id": "bl_rna_mi_entropy",
                "label": "RNA MI: Per-Column Shannon Entropy",
                "categories": cats,
                "values": vals,
                "unit": "bits",
            }));
        }
    }

    bindings
}
