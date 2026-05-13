// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared statistical routines used across lithoSpore modules.

/// Pearson correlation coefficient between two equal-length slices.
///
/// Returns 0.0 when either series has zero variance rather than NaN.
#[must_use]
pub fn pearson_r(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "pearson_r: mismatched lengths");
    let n = x.len() as f64;
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let (mut sxy, mut sxx, mut syy) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (&xi, &yi) in x.iter().zip(y) {
        let dx = xi - mx;
        let dy = yi - my;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    let denom = (sxx * syy).sqrt();
    if denom == 0.0 { 0.0 } else { sxy / denom }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_linear() {
        let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&v| 3.0 * v + 7.0).collect();
        assert!((pearson_r(&x, &y) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn uncorrelated_near_zero() {
        let x: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&v| (v * 17.3).sin()).collect();
        assert!(pearson_r(&x, &y).abs() < 0.3);
    }

    #[test]
    fn zero_variance_returns_zero() {
        let x = vec![5.0; 10];
        let y: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        assert_eq!(pearson_r(&x, &y), 0.0);
    }
}
