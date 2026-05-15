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

mod baselines;
mod modules;

use serde_json::{json, Value};

/// Convert a module's expected JSON into a vec of petalTongue DataBinding objects.
pub fn module_to_bindings(module_name: &str, expected: &Value) -> Vec<Value> {
    match module_name {
        "power_law_fitness" => modules::m1_fitness(expected),
        "mutation_accumulation" => modules::m2_mutations(expected),
        "allele_trajectories" => modules::m3_alleles(expected),
        "citrate_innovation" => modules::m4_citrate(expected),
        "biobrick_burden" => modules::m5_biobricks(expected),
        "breseq_264_genomes" => modules::m6_breseq(expected),
        "anderson_qs_predictions" => modules::m7_anderson(expected),
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

/// Convert a baseline tool's reference_data JSON into DataBinding objects.
pub fn baseline_to_bindings(tool_name: &str, data: &Value) -> Vec<Value> {
    match tool_name {
        "breseq" => baselines::breseq(data),
        "plannotate" => baselines::plannotate(data),
        "ostir" => baselines::ostir(data),
        "cryptkeeper" => baselines::cryptkeeper(data),
        "efm" => baselines::efm(data),
        "marker_divergence" => baselines::marker_divergence(data),
        "rna_mi" => baselines::rna_mi(data),
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
