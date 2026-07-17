// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::viz::{scatter, timeseries};
use serde_json::Value;

pub fn marker_divergence(data: &Value) -> Vec<Value> {
    let mut b = Vec::new();

    if let Some(curves) = data.get("divergence_curves") {
        let transfers = curves.get("transfers").and_then(|v| v.as_array());
        let series = curves.get("series").and_then(|v| v.as_object());
        if let (Some(t), Some(s)) = (transfers, series) {
            let x: Vec<f64> = t.iter().filter_map(serde_json::Value::as_f64).collect();
            for (name, vals) in s {
                if let Some(arr) = vals.as_array() {
                    let y: Vec<f64> = arr.iter().filter_map(serde_json::Value::as_f64).collect();
                    b.push(timeseries(
                        &format!("bl_md_divergence_{name}"),
                        &format!("Marker Divergence: {name}"),
                        "Transfer",
                        "Marker Ratio",
                        "ratio",
                        &x,
                        &y,
                    ));
                }
            }
        }
    }

    if let Some(params) = data.get("fitted_parameters").and_then(|v| v.as_object()) {
        let (mut x, mut y, mut labels) = (Vec::new(), Vec::new(), Vec::new());
        for (name, fit) in params {
            x.push(
                fit.get("alpha")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0),
            );
            y.push(
                fit.get("tau")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0),
            );
            labels.push(name.clone());
        }
        b.push(scatter(
            "bl_md_parameter_scatter",
            "Marker Divergence: Fitted (α, τ) Parameters",
            x,
            y,
            "α",
            "τ",
            labels,
            "dimensionless",
        ));
    }

    if let Some(contour) = data.get("confidence_contour")
        && let Some(ml_val) = contour.get("maximum_likelihood")
    {
        let mu = ml_val
            .get("log10_mu")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let s = ml_val
            .get("s")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let (mut x_pts, mut y_pts, mut labels) =
            (vec![mu], vec![s], vec!["Maximum Likelihood".to_string()]);

        if let Some(verts) = contour
            .get("contour_95_vertices")
            .and_then(|v| v.as_array())
        {
            for (i, vert) in verts.iter().enumerate() {
                if let Some(arr) = vert.as_array() {
                    x_pts.push(
                        arr.first()
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0),
                    );
                    y_pts.push(
                        arr.get(1)
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0),
                    );
                    labels.push(format!("95% contour {}", i + 1));
                }
            }
        }

        b.push(scatter(
            "bl_md_confidence_contour",
            "Marker Divergence: ML + 95% Confidence Contour (μ, s)",
            x_pts,
            y_pts,
            "log₁₀(μ)",
            "s (selection coefficient)",
            labels,
            "dimensionless",
        ));
    }

    b
}
