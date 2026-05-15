// SPDX-License-Identifier: AGPL-3.0-or-later

//! DataBinding adapter for petalTongue visualization.
//!
//! Maps each lithoSpore module's expected JSON into petalTongue-compatible
//! `DataBinding` JSON arrays. Pure data transformation — no compile-time
//! dependency on petalTongue crates. Output conforms to:
//!   `petal-tongue-types::DataBinding` (channel_type-tagged enum)
//!
//! Supported channel types: timeseries, bar, scatter, gauge, distribution,
//! heatmap, genome_track, circular_map.
//!
//! Baseline adapters read `baselines/<tool>/reference_data.json` and produce
//! DataBindings that reproduce each Barrick Lab tool's key visualizations.

use serde_json::{json, Value};

/// Convert a module's expected JSON into a vec of petalTongue DataBinding objects.
pub fn module_to_bindings(module_name: &str, expected: &Value) -> Vec<Value> {
    match module_name {
        "power_law_fitness" => m1_fitness(expected),
        "mutation_accumulation" => m2_mutations(expected),
        "allele_trajectories" => m3_alleles(expected),
        "citrate_innovation" => m4_citrate(expected),
        "biobrick_burden" => m5_biobricks(expected),
        "breseq_264_genomes" => m6_breseq(expected),
        "anderson_qs_predictions" => m7_anderson(expected),
        _ => vec![],
    }
}

/// Build a lithoSpore dashboard payload with all module bindings.
pub fn build_dashboard(modules: &[(&str, &Value)]) -> Value {
    let mut bindings = Vec::new();
    for (name, expected) in modules {
        bindings.extend(module_to_bindings(name, expected));
    }
    json!({
        "session_id": "lithoSpore-dashboard",
        "title": "lithoSpore LTEE Scientific Visualization",
        "domain": "ecology",
        "bindings": bindings,
    })
}

// ── Per-module mapping functions ────────────────────────────────────────

fn m1_fitness(exp: &Value) -> Vec<Value> {
    let gens = exp.get("generations").and_then(|v| v.as_array());
    let fitness = exp.get("mean_fitness").and_then(|v| v.as_array());

    let mut bindings = Vec::new();

    if let (Some(g), Some(f)) = (gens, fitness) {
        let x: Vec<f64> = g.iter().filter_map(|v| v.as_f64()).collect();
        let y: Vec<f64> = f.iter().filter_map(|v| v.as_f64()).collect();

        bindings.push(json!({
            "channel_type": "timeseries",
            "id": "m1_fitness_trajectory",
            "label": "LTEE Mean Fitness (Wiser 2013)",
            "x_label": "Generation",
            "y_label": "Relative Fitness",
            "unit": "W",
            "x_values": x,
            "y_values": y,
        }));
    }

    if let Some(fits) = exp.get("model_fits") {
        let mut categories = Vec::new();
        let mut values = Vec::new();
        for name in &["power_law", "hyperbolic", "logarithmic"] {
            if let Some(model) = fits.get(name) {
                categories.push(name.to_string());
                values.push(model.get("r_squared").and_then(|v| v.as_f64()).unwrap_or(0.0));
            }
        }
        if !categories.is_empty() {
            bindings.push(json!({
                "channel_type": "bar",
                "id": "m1_model_comparison",
                "label": "Model R² Comparison",
                "categories": categories,
                "values": values,
                "unit": "R²",
            }));
        }
    }

    bindings
}

fn m2_mutations(exp: &Value) -> Vec<Value> {
    let pfix = exp.get("kimura_fixation_prob_neutral").and_then(|v| v.as_f64());
    let clock = exp.get("molecular_clock_rate").and_then(|v| v.as_f64());
    let pearson = exp.get("molecular_clock_pearson_r").and_then(|v| v.as_f64());
    let drift = exp.get("drift_dominance_ratio").and_then(|v| v.as_f64());

    let mut bindings = Vec::new();

    let cats: Vec<String> = ["P_fix(neutral)", "Clock rate (μ)", "Pearson r", "Drift ratio"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let vals: Vec<f64> = [pfix, clock, pearson, drift]
        .iter()
        .map(|v| v.unwrap_or(0.0))
        .collect();

    bindings.push(json!({
        "channel_type": "bar",
        "id": "m2_mutation_parameters",
        "label": "Module 2: Mutation Parameters (Barrick 2009)",
        "categories": cats,
        "values": vals,
        "unit": "mixed",
    }));

    if let Some(drift_v) = drift {
        bindings.push(json!({
            "channel_type": "gauge",
            "id": "m2_drift_ratio",
            "label": "Drift Dominance Ratio",
            "value": drift_v,
            "min": 0.0,
            "max": 5.0,
            "unit": "×",
            "normal_range": [1.0, 3.0],
            "warning_range": [0.0, 5.0],
        }));
    }

    bindings
}

fn m3_alleles(exp: &Value) -> Vec<Value> {
    let results = match exp.get("results_by_size").and_then(|v| v.as_object()) {
        Some(r) => r,
        None => return vec![],
    };

    let mut sizes: Vec<(&String, &Value)> = results.iter().collect();
    sizes.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(0));

    let x: Vec<f64> = sizes
        .iter()
        .filter_map(|(k, _)| k.parse::<f64>().ok())
        .collect();

    let fix_prob: Vec<f64> = sizes
        .iter()
        .filter_map(|(_, v)| v.get("fixation_probability").and_then(|p| p.as_f64()))
        .collect();

    let interference: Vec<f64> = sizes
        .iter()
        .filter_map(|(_, v)| v.get("interference_ratio").and_then(|p| p.as_f64()))
        .collect();

    let adaptation: Vec<f64> = sizes
        .iter()
        .filter_map(|(_, v)| v.get("adaptation_rate").and_then(|p| p.as_f64()))
        .collect();

    vec![
        json!({
            "channel_type": "scatter",
            "id": "m3_fixation_probability",
            "label": "Fixation Probability vs Population Size (Good 2017)",
            "x": x,
            "y": fix_prob,
            "x_label": "Population Size (N)",
            "y_label": "Fixation Probability",
            "unit": "P",
        }),
        json!({
            "channel_type": "scatter",
            "id": "m3_interference_ratio",
            "label": "Clonal Interference Ratio vs Pop Size",
            "x": x,
            "y": interference,
            "x_label": "Population Size (N)",
            "y_label": "Interference Ratio",
            "unit": "ratio",
        }),
        json!({
            "channel_type": "scatter",
            "id": "m3_adaptation_rate",
            "label": "Adaptation Rate vs Pop Size",
            "x": x,
            "y": adaptation,
            "x_label": "Population Size (N)",
            "y_label": "Adaptation Rate",
            "unit": "per gen",
        }),
    ]
}

fn m4_citrate(exp: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    let cit_frac = exp.get("cit_plus_fraction").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let pot_frac = exp.get("potentiation_fraction").and_then(|v| v.as_f64()).unwrap_or(0.0);

    bindings.push(json!({
        "channel_type": "bar",
        "id": "m4_citrate_fractions",
        "label": "Citrate Innovation Fractions (Blount 2008)",
        "categories": ["Cit+ Fraction", "Potentiation Fraction"],
        "values": [cit_frac, pot_frac],
        "unit": "fraction",
    }));

    if let Some(replay) = exp.get("replay_probabilities").and_then(|v| v.as_object()) {
        let mut gens: Vec<(i64, f64)> = replay
            .iter()
            .filter_map(|(k, v)| {
                let generation = k.parse::<i64>().ok()?;
                let prob = v.as_f64()?;
                Some((generation, prob))
            })
            .collect();
        gens.sort_by_key(|(g, _)| *g);

        let x: Vec<f64> = gens.iter().map(|(g, _)| *g as f64).collect();
        let y: Vec<f64> = gens.iter().map(|(_, p)| *p).collect();

        bindings.push(json!({
            "channel_type": "timeseries",
            "id": "m4_replay_probability",
            "label": "Replay Probability vs Freeze Generation",
            "x_label": "Freeze Generation",
            "y_label": "Replay Probability",
            "unit": "P",
            "x_values": x,
            "y_values": y,
        }));
    }

    if let Some(pot_gen) = exp.get("mean_potentiation_gen").and_then(|v| v.as_f64()) {
        let cit_gen = exp.get("mean_cit_plus_gen").and_then(|v| v.as_f64()).unwrap_or(0.0);
        bindings.push(json!({
            "channel_type": "bar",
            "id": "m4_timeline_markers",
            "label": "Citrate Event Timeline",
            "categories": ["Potentiation Gen", "Cit+ Gen"],
            "values": [pot_gen, cit_gen],
            "unit": "generation",
        }));
    }

    bindings
}

fn m5_biobricks(exp: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(backbones) = exp.get("plasmid_backbones").and_then(|v| v.as_object()) {
        let mut cats = Vec::new();
        let mut vals = Vec::new();
        for (name, count) in backbones {
            cats.push(name.clone());
            vals.push(count.as_f64().unwrap_or(0.0));
        }
        bindings.push(json!({
            "channel_type": "bar",
            "id": "m5_backbone_distribution",
            "label": "Plasmid Backbone Distribution (Barrick 2024)",
            "categories": cats,
            "values": vals,
            "unit": "count",
        }));
    }

    if let Some(thresholds) = exp.get("burden_thresholds").and_then(|v| v.as_object()) {
        let cats: Vec<String> = thresholds.keys().cloned().collect();
        let vals: Vec<f64> = thresholds.values().filter_map(|v| v.as_f64()).collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "m5_burden_thresholds",
            "label": "Burden Severity Bins",
            "categories": cats,
            "values": vals,
            "unit": "parts",
        }));
    }

    let sig_count = exp.get("significantly_burdensome_count").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total = exp.get("total_biobricks_tested").and_then(|v| v.as_f64()).unwrap_or(301.0);
    bindings.push(json!({
        "channel_type": "gauge",
        "id": "m5_burden_fraction",
        "label": "Significantly Burdensome Fraction",
        "value": sig_count,
        "min": 0.0,
        "max": total,
        "unit": "parts",
        "normal_range": [0.0, total * 0.25],
        "warning_range": [total * 0.25, total * 0.50],
    }));

    bindings
}

fn m6_breseq(exp: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(curve) = exp.get("mutation_accumulation_curve") {
        let gens = curve.get("generations").and_then(|v| v.as_array());
        let muts = curve.get("expected_mutations_nonmutator").and_then(|v| v.as_array());
        if let (Some(g), Some(m)) = (gens, muts) {
            let x: Vec<f64> = g.iter().filter_map(|v| v.as_f64()).collect();
            let y: Vec<f64> = m.iter().filter_map(|v| v.as_f64()).collect();
            bindings.push(json!({
                "channel_type": "timeseries",
                "id": "m6_mutation_accumulation",
                "label": "Non-mutator Mutation Accumulation (Tenaillon 2016)",
                "x_label": "Generation",
                "y_label": "Expected Point Mutations",
                "unit": "mutations",
                "x_values": x,
                "y_values": y,
            }));
        }
    }

    if let Some(spectrum) = exp
        .get("targets")
        .and_then(|t| t.get("mutation_spectrum"))
        .and_then(|ms| ms.get("value"))
        .and_then(|v| v.as_object())
    {
        let cats: Vec<String> = spectrum.keys().cloned().collect();
        let vals: Vec<f64> = spectrum.values().filter_map(|v| v.as_f64()).collect();
        bindings.push(json!({
            "channel_type": "bar",
            "id": "m6_mutation_spectrum",
            "label": "6-Class Point Mutation Spectrum",
            "categories": cats,
            "values": vals,
            "unit": "fraction",
        }));
    }

    bindings
}

fn m7_anderson(exp: &Value) -> Vec<Value> {
    let mut bindings = Vec::new();

    if let Some(fitness) = exp.get("fitness_values").and_then(|v| v.as_object()) {
        let generation_map = [
            ("gen_500", 500.0),
            ("gen_5000", 5000.0),
            ("gen_10000", 10000.0),
            ("gen_50000", 50000.0),
        ];
        let mut x = Vec::new();
        let mut y = Vec::new();
        for (key, generation) in &generation_map {
            if let Some(val) = fitness.get(*key).and_then(|v| v.as_f64()) {
                x.push(*generation);
                y.push(val);
            }
        }
        bindings.push(json!({
            "channel_type": "scatter",
            "id": "m7_fitness_sparse",
            "label": "Fitness Values — Anderson/Wiser Model",
            "x": x,
            "y": y,
            "x_label": "Generation",
            "y_label": "Relative Fitness",
            "unit": "W",
        }));
    }

    if let Some(diag) = exp.get("anderson_diagnostics") {
        let goe = diag.get("goe_reference").and_then(|v| v.as_f64()).unwrap_or(0.531);
        let poisson = diag.get("poisson_reference").and_then(|v| v.as_f64()).unwrap_or(0.3863);
        let midpoint = (goe + poisson) / 2.0;

        bindings.push(json!({
            "channel_type": "bar",
            "id": "m7_anderson_diagnostics",
            "label": "Anderson Disorder Diagnostics",
            "categories": ["Poisson", "Midpoint ⟨r⟩", "GOE"],
            "values": [poisson, midpoint, goe],
            "unit": "⟨r⟩",
        }));

        bindings.push(json!({
            "channel_type": "gauge",
            "id": "m7_level_spacing",
            "label": "Level Spacing Ratio ⟨r⟩",
            "value": midpoint,
            "min": 0.0,
            "max": 1.0,
            "unit": "⟨r⟩",
            "normal_range": [poisson, goe],
            "warning_range": [0.0, poisson],
        }));
    }

    bindings
}

// ── Barrick Lab baseline adapters ──────────────────────────────────────

/// Convert a baseline tool's reference_data JSON into DataBinding objects.
pub fn baseline_to_bindings(tool_name: &str, data: &Value) -> Vec<Value> {
    match tool_name {
        "breseq" => baseline_breseq(data),
        "plannotate" => baseline_plannotate(data),
        "ostir" => baseline_ostir(data),
        "cryptkeeper" => baseline_cryptkeeper(data),
        "efm" => baseline_efm(data),
        "marker_divergence" => baseline_marker_divergence(data),
        "rna_mi" => baseline_rna_mi(data),
        _ => vec![],
    }
}

/// Build a baseline dashboard payload combining all 7 Barrick Lab tool
/// reference visualizations.
pub fn build_baseline_dashboard(tools: &[(&str, &Value)]) -> Value {
    let mut bindings = Vec::new();
    for (name, data) in tools {
        bindings.extend(baseline_to_bindings(name, data));
    }
    json!({
        "session_id": "barrick-baselines",
        "title": "Barrick Lab Software Baselines",
        "domain": "ecology",
        "bindings": bindings,
    })
}

fn baseline_breseq(data: &Value) -> Vec<Value> {
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

    // Genome overview as proper genome_track (not heatmap fallback)
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

    // Summary statistics gauge
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

fn baseline_plannotate(data: &Value) -> Vec<Value> {
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

    // Proper circular_map DataBinding
    let plasmid_size = data.get("plasmid_size_bp").and_then(|v| v.as_f64()).unwrap_or(2686.0);
    let plasmid_name = data.get("plasmid_name").and_then(|v| v.as_str()).unwrap_or("plasmid");

    if let Some(arcs) = data.get("circular_map_data").and_then(|v| v.get("arcs")).and_then(|v| v.as_array()) {
        // Collect ring names for the CircularMap type
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

fn baseline_ostir(data: &Value) -> Vec<Value> {
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

        // dG energy decomposition waterfall per RBS (when data available)
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
                let mut cats = Vec::new();
                let mut vals = Vec::new();
                for term in &terms {
                    if let Some(val) = dg.get(*term).and_then(|v| v.as_f64()) {
                        cats.push(term.to_string());
                        vals.push(val);
                    }
                }
                if !cats.is_empty() {
                    bindings.push(json!({
                        "channel_type": "bar",
                        "id": format!("bl_ostir_dg_{}", name.to_lowercase()),
                        "label": format!("OSTIR: dG Decomposition — {name}"),
                        "categories": cats,
                        "values": vals,
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

fn baseline_cryptkeeper(data: &Value) -> Vec<Value> {
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

    // Multi-track genome view with ORFs and cryptic promoters
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

fn baseline_efm(data: &Value) -> Vec<Value> {
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

    // Rate-colored genome track (matching SCRIBL canvas layout)
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

fn baseline_marker_divergence(data: &Value) -> Vec<Value> {
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

            // ML point + 95% confidence contour vertices
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

fn baseline_rna_mi(data: &Value) -> Vec<Value> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m1_produces_bindings() {
        let expected: Value = serde_json::from_str(
            r#"{
                "generations": [0, 500, 1000, 5000, 10000, 50000],
                "mean_fitness": [1.0, 1.147, 1.274, 2.042, 2.721, 5.759],
                "model_fits": {
                    "power_law": {"r_squared": 0.998},
                    "hyperbolic": {"r_squared": 0.997},
                    "logarithmic": {"r_squared": 0.901}
                }
            }"#,
        )
        .unwrap();
        let bindings = module_to_bindings("power_law_fitness", &expected);
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0]["channel_type"], "timeseries");
        assert_eq!(bindings[1]["channel_type"], "bar");
    }

    #[test]
    fn m6_produces_curve_and_spectrum() {
        let expected: Value = serde_json::from_str(
            r#"{
                "mutation_accumulation_curve": {
                    "generations": [0, 2000, 5000],
                    "expected_mutations_nonmutator": [0.0, 0.8, 2.1]
                },
                "targets": {
                    "mutation_spectrum": {
                        "value": {"GC_to_AT": 0.68, "AT_to_GC": 0.08}
                    }
                }
            }"#,
        )
        .unwrap();
        let bindings = module_to_bindings("breseq_264_genomes", &expected);
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn unknown_module_returns_empty() {
        let bindings = module_to_bindings("nonexistent", &json!({}));
        assert!(bindings.is_empty());
    }

    #[test]
    fn dashboard_aggregates() {
        let m1: Value = serde_json::from_str(r#"{"generations": [0, 1000], "mean_fitness": [1.0, 1.3]}"#).unwrap();
        let m2: Value = serde_json::from_str(r#"{"drift_dominance_ratio": 2.3}"#).unwrap();
        let dashboard = build_dashboard(&[
            ("power_law_fitness", &m1),
            ("mutation_accumulation", &m2),
        ]);
        assert_eq!(dashboard["session_id"], "lithoSpore-dashboard");
        let bindings = dashboard["bindings"].as_array().unwrap();
        assert!(bindings.len() >= 2);
    }

    #[test]
    fn baseline_breseq_produces_bindings() {
        let data: Value = serde_json::from_str(r#"{
            "genome_length": 4629812,
            "evidence_type_distribution": {"RA": 56, "JC": 30, "MC": 8},
            "mutation_spectrum": {"GC_to_AT": 0.68, "AT_to_GC": 0.08},
            "mutation_accumulation_curve": {
                "generations": [0, 10000, 50000],
                "expected_mutations_nonmutator": [0.0, 8.9, 44.5]
            },
            "genome_track_features": [
                {"type": "SNP", "position": 2450, "gene": "thrA"},
                {"type": "IS_insertion", "position": 183262, "element": "IS150"},
                {"type": "large_deletion", "start": 547700, "end": 555825, "genes_affected": 9}
            ],
            "summary_statistics": {"total_predicted_mutations": 94}
        }"#).unwrap();
        let bindings = baseline_to_bindings("breseq", &data);
        assert!(bindings.len() >= 5);
        assert!(bindings.iter().any(|b| b["id"] == "bl_breseq_evidence_types"));
        assert!(bindings.iter().any(|b| b["id"] == "bl_breseq_mutation_spectrum"));
        assert!(bindings.iter().any(|b| b["id"] == "bl_breseq_accumulation"));
        assert!(bindings.iter().any(|b| b["id"] == "bl_breseq_genome_overview"));
        assert!(bindings.iter().any(|b| b["channel_type"] == "genome_track"));
    }

    #[test]
    fn baseline_plannotate_produces_bindings() {
        let data: Value = serde_json::from_str(r#"{
            "plasmid_name": "pUC19",
            "plasmid_size_bp": 2686,
            "feature_distribution": {"CDS": 2, "promoter": 2},
            "annotation_coverage": 0.92,
            "circular_map_data": {
                "arcs": [
                    {"start_angle": 53.05, "end_angle": 60.55, "ring": 0, "category": "CDS", "label": "lacZ-alpha"},
                    {"start_angle": 218.22, "end_angle": 333.62, "ring": 1, "category": "CDS", "label": "AmpR"}
                ]
            }
        }"#).unwrap();
        let bindings = baseline_to_bindings("plannotate", &data);
        assert!(bindings.len() >= 3);
        assert!(bindings.iter().any(|b| b["id"] == "bl_plannotate_features"));
        assert!(bindings.iter().any(|b| b["id"] == "bl_plannotate_coverage"));
        assert!(bindings.iter().any(|b| b["channel_type"] == "circular_map"));
    }

    #[test]
    fn baseline_ostir_produces_bindings() {
        let data: Value = serde_json::from_str(r#"{
            "rbs_strength_predictions": [
                {"name": "BBa_B0034", "tir": 48217.3},
                {"name": "BBa_B0032", "tir": 12543.8}
            ],
            "rate_distribution": {"values": [42.8, 127.5], "mean": 85.15, "std": 59.9}
        }"#).unwrap();
        let bindings = baseline_to_bindings("ostir", &data);
        assert!(bindings.len() >= 2);
    }

    #[test]
    fn baseline_marker_divergence_produces_bindings() {
        let data: Value = serde_json::from_str(r#"{
            "divergence_curves": {
                "transfers": [0, 10, 20],
                "series": {"exp_1": [0.5, 0.6, 0.7]}
            },
            "fitted_parameters": {"exp_1": {"alpha": 2.15, "tau": 42.3}}
        }"#).unwrap();
        let bindings = baseline_to_bindings("marker_divergence", &data);
        assert!(bindings.len() >= 2);
        assert!(bindings.iter().any(|b| b["id"] == "bl_md_divergence_exp_1"));
    }

    #[test]
    fn baseline_rna_mi_produces_bindings() {
        let data: Value = serde_json::from_str(r#"{
            "significant_pairs": [
                {"col_i": 3, "col_j": 47, "pairing": "Watson-Crick"}
            ],
            "column_entropy": {
                "columns": [1, 2, 3],
                "entropy_bits": [0.12, 0.45, 1.52]
            }
        }"#).unwrap();
        let bindings = baseline_to_bindings("rna_mi", &data);
        assert!(bindings.len() >= 2);
        assert!(bindings.iter().any(|b| b["id"] == "bl_rna_mi_entropy"));
    }

    #[test]
    fn baseline_unknown_returns_empty() {
        let bindings = baseline_to_bindings("nonexistent", &json!({}));
        assert!(bindings.is_empty());
    }

    #[test]
    fn baseline_dashboard_aggregates() {
        let breseq: Value = serde_json::from_str(r#"{
            "evidence_type_distribution": {"RA": 10}
        }"#).unwrap();
        let efm: Value = serde_json::from_str(r#"{
            "site_counts": {"IS_insertion": 8}
        }"#).unwrap();
        let dashboard = build_baseline_dashboard(&[("breseq", &breseq), ("efm", &efm)]);
        assert_eq!(dashboard["session_id"], "barrick-baselines");
        let bindings = dashboard["bindings"].as_array().unwrap();
        assert!(bindings.len() >= 2);
    }
}
