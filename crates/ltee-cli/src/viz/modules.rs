// SPDX-License-Identifier: AGPL-3.0-or-later

//! Per-module DataBinding adapters for LTEE science modules.

use super::{bar, bar_from_object, gauge, timeseries, scatter};
use serde_json::Value;

pub(crate) fn m1_fitness(exp: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    let gens = exp.get("generations").and_then(|v| v.as_array());
    let fitness = exp.get("mean_fitness").and_then(|v| v.as_array());
    if let (Some(g), Some(f)) = (gens, fitness) {
        b.push(timeseries(
            "m1_fitness_trajectory", "LTEE Mean Fitness (Wiser 2013)",
            "Generation", "Relative Fitness", "W",
            g.iter().filter_map(|v| v.as_f64()).collect(),
            f.iter().filter_map(|v| v.as_f64()).collect(),
        ));
    }

    if let Some(fits) = exp.get("model_fits") {
        let (cats, vals): (Vec<String>, Vec<f64>) = ["power_law", "hyperbolic", "logarithmic"]
            .iter()
            .filter_map(|name| {
                fits.get(name)?
                    .get("r_squared")
                    .and_then(|v| v.as_f64())
                    .map(|r2| (name.to_string(), r2))
            })
            .unzip();
        if !cats.is_empty() {
            b.push(bar("m1_model_comparison", "Model R² Comparison", cats, vals, "R²"));
        }
    }

    b
}

pub(crate) fn m2_mutations(exp: &Value) -> Vec<Value> {
    let pfix = exp.get("kimura_fixation_prob_neutral").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let clock = exp.get("molecular_clock_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let pearson = exp.get("molecular_clock_pearson_r").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let drift = exp.get("drift_dominance_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let mut b = vec![bar(
        "m2_mutation_parameters", "Module 2: Mutation Parameters (Barrick 2009)",
        vec!["P_fix(neutral)".into(), "Clock rate (μ)".into(), "Pearson r".into(), "Drift ratio".into()],
        vec![pfix, clock, pearson, drift], "mixed",
    )];

    if drift > 0.0 {
        b.push(gauge("m2_drift_ratio", "Drift Dominance Ratio", drift, 0.0, 5.0, "×", [1.0, 3.0], [0.0, 5.0]));
    }

    b
}

pub(crate) fn m3_alleles(exp: &Value) -> Vec<Value> {
    let results = match exp.get("results_by_size").and_then(|v| v.as_object()) {
        Some(r) => r,
        None => return vec![],
    };

    let mut sizes: Vec<(&String, &Value)> = results.iter().collect();
    sizes.sort_by_key(|(k, _)| k.parse::<u64>().unwrap_or(0));

    let x: Vec<f64> = sizes.iter().filter_map(|(k, _)| k.parse::<f64>().ok()).collect();

    let extract = |field: &str| -> Vec<f64> {
        sizes.iter().filter_map(|(_, v)| v.get(field).and_then(|p| p.as_f64())).collect()
    };

    vec![
        scatter("m3_fixation_probability", "Fixation Probability vs Population Size (Good 2017)",
            x.clone(), extract("fixation_probability"), "Population Size (N)", "Fixation Probability", vec![], "P"),
        scatter("m3_interference_ratio", "Clonal Interference Ratio vs Pop Size",
            x.clone(), extract("interference_ratio"), "Population Size (N)", "Interference Ratio", vec![], "ratio"),
        scatter("m3_adaptation_rate", "Adaptation Rate vs Pop Size",
            x, extract("adaptation_rate"), "Population Size (N)", "Adaptation Rate", vec![], "per gen"),
    ]
}

pub(crate) fn m4_citrate(exp: &Value) -> Vec<Value> {
    let cit_frac = exp.get("cit_plus_fraction").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let pot_frac = exp.get("potentiation_fraction").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let mut b = vec![bar(
        "m4_citrate_fractions", "Citrate Innovation Fractions (Blount 2008)",
        vec!["Cit+ Fraction".into(), "Potentiation Fraction".into()],
        vec![cit_frac, pot_frac], "fraction",
    )];

    if let Some(replay) = exp.get("replay_probabilities").and_then(|v| v.as_object()) {
        let mut gens: Vec<(i64, f64)> = replay.iter()
            .filter_map(|(k, v)| Some((k.parse::<i64>().ok()?, v.as_f64()?)))
            .collect();
        gens.sort_by_key(|(g, _)| *g);
        b.push(timeseries(
            "m4_replay_probability", "Replay Probability vs Freeze Generation",
            "Freeze Generation", "Replay Probability", "P",
            gens.iter().map(|(g, _)| *g as f64).collect(),
            gens.iter().map(|(_, p)| *p).collect(),
        ));
    }

    if let Some(pot_gen) = exp.get("mean_potentiation_gen").and_then(|v| v.as_f64()) {
        let cit_gen = exp.get("mean_cit_plus_gen").and_then(|v| v.as_f64()).unwrap_or(0.0);
        b.push(bar(
            "m4_timeline_markers", "Citrate Event Timeline",
            vec!["Potentiation Gen".into(), "Cit+ Gen".into()],
            vec![pot_gen, cit_gen], "generation",
        ));
    }

    b
}

pub(crate) fn m5_biobricks(exp: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(backbones) = exp.get("plasmid_backbones").and_then(|v| v.as_object()) {
        b.push(bar_from_object("m5_backbone_distribution", "Plasmid Backbone Distribution (Barrick 2024)", backbones, "count"));
    }
    if let Some(thresholds) = exp.get("burden_thresholds").and_then(|v| v.as_object()) {
        b.push(bar_from_object("m5_burden_thresholds", "Burden Severity Bins", thresholds, "parts"));
    }

    let sig = exp.get("significantly_burdensome_count").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total = exp.get("total_biobricks_tested").and_then(|v| v.as_f64()).unwrap_or(301.0);
    b.push(gauge("m5_burden_fraction", "Significantly Burdensome Fraction", sig, 0.0, total, "parts", [0.0, total * 0.25], [total * 0.25, total * 0.50]));

    b
}

pub(crate) fn m6_breseq(exp: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(curve) = exp.get("mutation_accumulation_curve") {
        let gens = curve.get("generations").and_then(|v| v.as_array());
        let muts = curve.get("expected_mutations_nonmutator").and_then(|v| v.as_array());
        if let (Some(g), Some(m)) = (gens, muts) {
            b.push(timeseries(
                "m6_mutation_accumulation", "Non-mutator Mutation Accumulation (Tenaillon 2016)",
                "Generation", "Expected Point Mutations", "mutations",
                g.iter().filter_map(|v| v.as_f64()).collect(),
                m.iter().filter_map(|v| v.as_f64()).collect(),
            ));
        }
    }

    if let Some(spectrum) = exp.get("targets")
        .and_then(|t| t.get("mutation_spectrum"))
        .and_then(|ms| ms.get("value"))
        .and_then(|v| v.as_object())
    {
        b.push(bar_from_object("m6_mutation_spectrum", "6-Class Point Mutation Spectrum", spectrum, "fraction"));
    }

    b
}

pub(crate) fn m7_anderson(exp: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(fitness) = exp.get("fitness_values").and_then(|v| v.as_object()) {
        let gen_map = [("gen_500", 500.0), ("gen_5000", 5000.0), ("gen_10000", 10000.0), ("gen_50000", 50000.0)];
        let (x, y): (Vec<f64>, Vec<f64>) = gen_map.iter()
            .filter_map(|(key, generation)| fitness.get(*key).and_then(|v| v.as_f64()).map(|v| (*generation, v)))
            .unzip();
        b.push(scatter("m7_fitness_sparse", "Fitness Values — Anderson/Wiser Model", x, y, "Generation", "Relative Fitness", vec![], "W"));
    }

    if let Some(diag) = exp.get("anderson_diagnostics") {
        let goe = diag.get("goe_reference").and_then(|v| v.as_f64()).unwrap_or(0.531);
        let poisson = diag.get("poisson_reference").and_then(|v| v.as_f64()).unwrap_or(0.3863);
        let midpoint = (goe + poisson) / 2.0;
        b.push(bar(
            "m7_anderson_diagnostics", "Anderson Disorder Diagnostics",
            vec!["Poisson".into(), "Midpoint ⟨r⟩".into(), "GOE".into()],
            vec![poisson, midpoint, goe], "⟨r⟩",
        ));
        b.push(gauge("m7_level_spacing", "Level Spacing Ratio ⟨r⟩", midpoint, 0.0, 1.0, "⟨r⟩", [poisson, goe], [0.0, poisson]));
    }

    b
}
