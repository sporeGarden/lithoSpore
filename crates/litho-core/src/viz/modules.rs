// SPDX-License-Identifier: AGPL-3.0-or-later

//! Per-module DataBinding adapters for LTEE science modules.

use serde_json::{json, Value};

pub(crate) fn m1_fitness(exp: &Value) -> Vec<Value> {
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

pub(crate) fn m2_mutations(exp: &Value) -> Vec<Value> {
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

pub(crate) fn m3_alleles(exp: &Value) -> Vec<Value> {
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

pub(crate) fn m4_citrate(exp: &Value) -> Vec<Value> {
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

pub(crate) fn m5_biobricks(exp: &Value) -> Vec<Value> {
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

pub(crate) fn m6_breseq(exp: &Value) -> Vec<Value> {
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

pub(crate) fn m7_anderson(exp: &Value) -> Vec<Value> {
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
