// SPDX-License-Identifier: AGPL-3.0-or-later

//! Per-module `DataBinding` adapters for LTEE science modules.

use super::{
    GaugeBinding, ScatterBinding, bar, bar_from_object, gauge_binding, scatter_binding, timeseries,
};
use serde_json::Value;

pub fn m1_fitness(exp: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    let gens = exp.get("generations").and_then(|v| v.as_array());
    let fitness = exp.get("mean_fitness").and_then(|v| v.as_array());
    if let (Some(g), Some(f)) = (gens, fitness) {
        let x: Vec<f64> = g.iter().filter_map(serde_json::Value::as_f64).collect();
        let y: Vec<f64> = f.iter().filter_map(serde_json::Value::as_f64).collect();
        b.push(timeseries(
            "m1_fitness_trajectory",
            "LTEE Mean Fitness (Wiser 2013)",
            "Generation",
            "Relative Fitness",
            "W",
            &x,
            &y,
        ));
    }

    if let Some(fits) = exp.get("model_fits") {
        let (cats, vals): (Vec<String>, Vec<f64>) = ["power_law", "hyperbolic", "logarithmic"]
            .iter()
            .filter_map(|name| {
                fits.get(name)?
                    .get("r_squared")
                    .and_then(serde_json::Value::as_f64)
                    .map(|r2| (name.to_string(), r2))
            })
            .unzip();
        if !cats.is_empty() {
            b.push(bar(
                "m1_model_comparison",
                "Model R² Comparison",
                &cats,
                &vals,
                "R²",
            ));
        }
    }

    b
}

pub fn m2_mutations(exp: &Value) -> Vec<Value> {
    let pfix = exp
        .get("kimura_fixation_prob_neutral")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let clock = exp
        .get("molecular_clock_rate")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let pearson = exp
        .get("molecular_clock_pearson_r")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let drift = exp
        .get("drift_dominance_ratio")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);

    let cats = [
        "P_fix(neutral)".to_string(),
        "Clock rate (μ)".to_string(),
        "Pearson r".to_string(),
        "Drift ratio".to_string(),
    ];
    let vals = [pfix, clock, pearson, drift];

    let mut b = vec![bar(
        "m2_mutation_parameters",
        "Module 2: Mutation Parameters (Barrick 2009)",
        &cats,
        &vals,
        "mixed",
    )];

    if drift > 0.0 {
        b.push(gauge_binding(&GaugeBinding {
            id: "m2_drift_ratio",
            label: "Drift Dominance Ratio",
            value: drift,
            min: 0.0,
            max: 5.0,
            unit: "×",
            normal: [1.0, 3.0],
            warning: [0.0, 5.0],
        }));
    }

    b
}

pub fn m3_alleles(exp: &Value) -> Vec<Value> {
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

    let extract = |field: &str| -> Vec<f64> {
        sizes
            .iter()
            .filter_map(|(_, v)| v.get(field).and_then(serde_json::Value::as_f64))
            .collect()
    };

    let fix_prob = extract("fixation_probability");
    let interference = extract("interference_ratio");
    let adaptation = extract("adaptation_rate");
    let empty_labels: [String; 0] = [];

    vec![
        scatter_binding(&ScatterBinding {
            id: "m3_fixation_probability",
            label: "Fixation Probability vs Population Size (Good 2017)",
            x: &x,
            y: &fix_prob,
            x_label: "Population Size (N)",
            y_label: "Fixation Probability",
            point_labels: &empty_labels,
            unit: "P",
        }),
        scatter_binding(&ScatterBinding {
            id: "m3_interference_ratio",
            label: "Clonal Interference Ratio vs Pop Size",
            x: &x,
            y: &interference,
            x_label: "Population Size (N)",
            y_label: "Interference Ratio",
            point_labels: &empty_labels,
            unit: "ratio",
        }),
        scatter_binding(&ScatterBinding {
            id: "m3_adaptation_rate",
            label: "Adaptation Rate vs Pop Size",
            x: &x,
            y: &adaptation,
            x_label: "Population Size (N)",
            y_label: "Adaptation Rate",
            point_labels: &empty_labels,
            unit: "per gen",
        }),
    ]
}

pub fn m4_citrate(exp: &Value) -> Vec<Value> {
    let cit_frac = exp
        .get("cit_plus_fraction")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let pot_frac = exp
        .get("potentiation_fraction")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);

    let cats = [
        "Cit+ Fraction".to_string(),
        "Potentiation Fraction".to_string(),
    ];
    let vals = [cit_frac, pot_frac];

    let mut b = vec![bar(
        "m4_citrate_fractions",
        "Citrate Innovation Fractions (Blount 2008)",
        &cats,
        &vals,
        "fraction",
    )];

    if let Some(replay) = exp.get("replay_probabilities").and_then(|v| v.as_object()) {
        let mut gens: Vec<(i64, f64)> = replay
            .iter()
            .filter_map(|(k, v)| Some((k.parse::<i64>().ok()?, v.as_f64()?)))
            .collect();
        gens.sort_by_key(|(g, _)| *g);
        let x: Vec<f64> = gens.iter().map(|(g, _)| *g as f64).collect();
        let y: Vec<f64> = gens.iter().map(|(_, p)| *p).collect();
        b.push(timeseries(
            "m4_replay_probability",
            "Replay Probability vs Freeze Generation",
            "Freeze Generation",
            "Replay Probability",
            "P",
            &x,
            &y,
        ));
    }

    if let Some(pot_gen) = exp
        .get("mean_potentiation_gen")
        .and_then(serde_json::Value::as_f64)
    {
        let cit_gen = exp
            .get("mean_cit_plus_gen")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let timeline_cats = ["Potentiation Gen".to_string(), "Cit+ Gen".to_string()];
        let timeline_vals = [pot_gen, cit_gen];
        b.push(bar(
            "m4_timeline_markers",
            "Citrate Event Timeline",
            &timeline_cats,
            &timeline_vals,
            "generation",
        ));
    }

    b
}

pub fn m5_biobricks(exp: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(backbones) = exp.get("plasmid_backbones").and_then(|v| v.as_object()) {
        b.push(bar_from_object(
            "m5_backbone_distribution",
            "Plasmid Backbone Distribution (Barrick 2024)",
            backbones,
            "count",
        ));
    }
    if let Some(thresholds) = exp.get("burden_thresholds").and_then(|v| v.as_object()) {
        b.push(bar_from_object(
            "m5_burden_thresholds",
            "Burden Severity Bins",
            thresholds,
            "parts",
        ));
    }

    let sig = exp
        .get("significantly_burdensome_count")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let total = exp
        .get("total_biobricks_tested")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(301.0);
    b.push(gauge_binding(&GaugeBinding {
        id: "m5_burden_fraction",
        label: "Significantly Burdensome Fraction",
        value: sig,
        min: 0.0,
        max: total,
        unit: "parts",
        normal: [0.0, total * 0.25],
        warning: [total * 0.25, total * 0.50],
    }));

    b
}

pub fn m6_breseq(exp: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(curve) = exp.get("mutation_accumulation_curve") {
        let gens = curve.get("generations").and_then(|v| v.as_array());
        let muts = curve
            .get("expected_mutations_nonmutator")
            .and_then(|v| v.as_array());
        if let (Some(g), Some(m)) = (gens, muts) {
            let x: Vec<f64> = g.iter().filter_map(serde_json::Value::as_f64).collect();
            let y: Vec<f64> = m.iter().filter_map(serde_json::Value::as_f64).collect();
            b.push(timeseries(
                "m6_mutation_accumulation",
                "Non-mutator Mutation Accumulation (Tenaillon 2016)",
                "Generation",
                "Expected Point Mutations",
                "mutations",
                &x,
                &y,
            ));
        }
    }

    if let Some(spectrum) = exp
        .get("targets")
        .and_then(|t| t.get("mutation_spectrum"))
        .and_then(|ms| ms.get("value"))
        .and_then(|v| v.as_object())
    {
        b.push(bar_from_object(
            "m6_mutation_spectrum",
            "6-Class Point Mutation Spectrum",
            spectrum,
            "fraction",
        ));
    }

    b
}

pub fn m7_anderson(exp: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(fitness) = exp.get("fitness_values").and_then(|v| v.as_object()) {
        let gen_map = [
            ("gen_500", 500.0),
            ("gen_5000", 5000.0),
            ("gen_10000", 10000.0),
            ("gen_50000", 50000.0),
        ];
        let (x, y): (Vec<f64>, Vec<f64>) = gen_map
            .iter()
            .filter_map(|(key, generation)| {
                fitness
                    .get(*key)
                    .and_then(serde_json::Value::as_f64)
                    .map(|v| (*generation, v))
            })
            .unzip();
        let empty_labels: [String; 0] = [];
        b.push(scatter_binding(&ScatterBinding {
            id: "m7_fitness_sparse",
            label: "Fitness Values — Anderson/Wiser Model",
            x: &x,
            y: &y,
            x_label: "Generation",
            y_label: "Relative Fitness",
            point_labels: &empty_labels,
            unit: "W",
        }));
    }

    if let Some(diag) = exp.get("anderson_diagnostics") {
        let goe = diag
            .get("goe_reference")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.531);
        let poisson = diag
            .get("poisson_reference")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.3863);
        let midpoint = f64::midpoint(goe, poisson);
        let cats = [
            "Poisson".to_string(),
            "Midpoint ⟨r⟩".to_string(),
            "GOE".to_string(),
        ];
        let vals = [poisson, midpoint, goe];
        b.push(bar(
            "m7_anderson_diagnostics",
            "Anderson Disorder Diagnostics",
            &cats,
            &vals,
            "⟨r⟩",
        ));
        b.push(gauge_binding(&GaugeBinding {
            id: "m7_level_spacing",
            label: "Level Spacing Ratio ⟨r⟩",
            value: midpoint,
            min: 0.0,
            max: 1.0,
            unit: "⟨r⟩",
            normal: [poisson, goe],
            warning: [0.0, poisson],
        }));
    }

    b
}
