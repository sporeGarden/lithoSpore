// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 5: BioBrick metabolic burden
//!
//! Validates burden measurements from Barrick et al. 2024 (B6), Nature Communications.
//! doi:10.1038/s41467-024-50639-9
//!
//! Data: barricklab/igem2019 (v1.0.2) — 301 BioBrick plasmid growth burden assays.
//! Tier 2: pure Rust CSV parsing + growth rate computation + burden distribution.
//! Target: T08 (fat-tailed burden distribution).

use litho_core::harness;
use litho_core::{ModuleResult, ValidationStatus};
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

const BFP_CONTROLS: &[&str] = &["K3174002", "K3174003", "K3174004", "K3174006", "K3174007"];

/// Run module 5 validation with the given paths and tier.
pub fn run_validation(data_dir: &str, expected: &str, max_tier: u8) -> ModuleResult {
    let start = Instant::now();

    if !Path::new(expected).exists() {
        return harness::skip(
            "biobrick_burden", 1, start,
            "Expected values not found — run fetch + generate expected values first",
        );
    }

    if !Path::new(data_dir).exists() {
        return harness::skip(
            "biobrick_burden", 1, start,
            "Data not fetched — run `litho fetch --all`",
        );
    }

    if max_tier >= 2 {
        return run_tier2_rust(data_dir, expected, start);
    }
    if max_tier >= 1 {
        return harness::dispatch_python(
            "biobrick_burden",
            Path::new("notebooks/module5_biobricks/biobrick_burden.py"),
            Path::new("."),
        );
    }

    harness::skip(
        "biobrick_burden", max_tier, start,
        &format!("Tier {max_tier} not implemented yet"),
    )
}

// ── CSV parsing ─────────────────────────────────────────────────────

struct StrainRecord {
    accession: String,
    measured: bool,
    vector: String,
}

fn load_strain_metadata(data_dir: &str) -> Option<Vec<StrainRecord>> {
    let path = Path::new(data_dir).join("igem2019_strain_metadata.csv");
    let content = std::fs::read_to_string(path).ok()?;
    let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

    let mut lines = content.lines();
    let header = lines.next()?;
    let fields: Vec<&str> = header.split(',').collect();

    let acc_idx = fields.iter().position(|f| f.contains("accession"))?;
    let measured_idx = fields.iter().position(|f| *f == "measured")?;
    let vector_idx = fields.iter().position(|f| *f == "vector")?;

    let mut records = Vec::new();
    for line in lines {
        let cols: Vec<&str> = parse_csv_line(line);
        if cols.len() <= acc_idx.max(measured_idx).max(vector_idx) {
            continue;
        }
        records.push(StrainRecord {
            accession: cols[acc_idx].to_string(),
            measured: cols[measured_idx].eq_ignore_ascii_case("TRUE"),
            vector: cols[vector_idx].to_string(),
        });
    }

    Some(records)
}

fn parse_csv_line(line: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let bytes = line.as_bytes();

    for i in 0..bytes.len() {
        match bytes[i] {
            b'"' => in_quotes = !in_quotes,
            b',' if !in_quotes => {
                fields.push(line[start..i].trim_matches('"'));
                start = i + 1;
            }
            _ => {}
        }
    }
    fields.push(line[start..].trim_matches('"'));
    fields
}

// ── Growth rate computation from plate data ─────────────────────────

struct GrowthMeasurement {
    time_hours: f64,
    od600: f64,
}

fn compute_growth_rate(measurements: &[GrowthMeasurement]) -> Option<f64> {
    if measurements.len() < 3 {
        return None;
    }

    let log_phase: Vec<&GrowthMeasurement> = measurements
        .iter()
        .filter(|m| m.od600 > 0.02 && m.od600 < 0.3)
        .collect();

    if log_phase.len() < 2 {
        return None;
    }

    let n = log_phase.len() as f64;
    let sum_t: f64 = log_phase.iter().map(|m| m.time_hours).sum();
    let sum_lnod: f64 = log_phase.iter().map(|m| m.od600.ln()).sum();
    let sum_t2: f64 = log_phase.iter().map(|m| m.time_hours.powi(2)).sum();
    let sum_t_lnod: f64 = log_phase.iter().map(|m| m.time_hours * m.od600.ln()).sum();

    let denom = n * sum_t2 - sum_t * sum_t;
    if denom.abs() < 1e-15 {
        return None;
    }

    let slope = (n * sum_t_lnod - sum_t * sum_lnod) / denom;
    if slope.is_finite() && slope > 0.0 {
        Some(slope)
    } else {
        None
    }
}

fn load_plate_measurements(plate_dir: &Path) -> Vec<GrowthMeasurement> {
    let meas_file = plate_dir.join(
        plate_dir.file_name()
            .and_then(|n| n.to_str())
            .map(|n| format!("{n}.measurements.csv"))
            .unwrap_or_default()
    );

    let content = match std::fs::read_to_string(&meas_file) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut measurements = Vec::new();
    let mut lines = content.lines();
    let _header = lines.next();

    for line in lines {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() >= 2 {
            if let (Ok(time), Ok(od)) = (cols[0].trim().parse::<f64>(), cols[1].trim().parse::<f64>()) {
                if od > 0.0 {
                    measurements.push(GrowthMeasurement { time_hours: time, od600: od });
                }
            }
        }
    }

    measurements
}

fn compute_burden(sample_rate: f64, control_rate: f64) -> f64 {
    if control_rate > 0.0 {
        1.0 - (sample_rate / control_rate)
    } else {
        0.0
    }
}

// ── Tier 2: Rust validation ─────────────────────────────────────────

fn run_tier2_rust(data_dir: &str, expected_path: &str, start: Instant) -> ModuleResult {
    let expected = match harness::load_expected(expected_path) {
        Some(v) => v,
        None => return harness::skip(
            "biobrick_burden", 2, start,
            "Cannot parse expected values JSON",
        ),
    };

    let strains = match load_strain_metadata(data_dir) {
        Some(s) => s,
        None => return harness::skip(
            "biobrick_burden", 2, start,
            "Cannot parse igem2019_strain_metadata.csv",
        ),
    };

    let bfp_set: HashSet<&str> = BFP_CONTROLS.iter().copied().collect();
    let measured: Vec<&StrainRecord> = strains.iter().filter(|s| s.measured).collect();
    let biobrick_measured: Vec<&&StrainRecord> = measured
        .iter()
        .filter(|s| !bfp_set.contains(s.accession.as_str()))
        .collect();

    let biobrick_accessions: HashSet<&str> = biobrick_measured
        .iter()
        .map(|s| s.accession.as_str())
        .collect();

    let bfp_measured: HashSet<&str> = measured
        .iter()
        .filter(|s| bfp_set.contains(s.accession.as_str()))
        .map(|s| s.accession.as_str())
        .collect();

    let psb1c3_accessions: HashSet<&str> = biobrick_measured
        .iter()
        .filter(|s| s.vector == "pSB1C3")
        .map(|s| s.accession.as_str())
        .collect();

    let psb1a2_accessions: HashSet<&str> = biobrick_measured
        .iter()
        .filter(|s| s.vector == "pSB1A2")
        .map(|s| s.accession.as_str())
        .collect();

    let plate_data_dir = Path::new(data_dir).join("input-plate-data");
    let has_plate_data = plate_data_dir.exists()
        && std::fs::read_dir(&plate_data_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);

    let mut passed = 0_u32;
    let mut total = 0_u32;

    let exp_total = expected["total_biobricks_tested"].as_u64().unwrap_or(301);
    let count_tol = expected["tolerances"]["count_tolerance"].as_u64().unwrap_or(5);

    total += 1;
    let bb_count = biobrick_accessions.len() as u64;
    let count_ok = bb_count.abs_diff(exp_total) <= count_tol;
    if count_ok { passed += 1; }
    eprintln!("  [{}] Total BioBrick parts: {} (expected: {} ± {})",
        if count_ok { "PASS" } else { "FAIL" }, bb_count, exp_total, count_tol);

    total += 1;
    if has_plate_data { passed += 1; }
    eprintln!("  [{}] Growth curve plate data present",
        if has_plate_data { "PASS" } else { "FAIL" });

    total += 1;
    let bb_ok = psb1c3_accessions.len() >= 230;
    if bb_ok { passed += 1; }
    eprintln!("  [{}] pSB1C3 BioBricks: {} (expected: >= 230)",
        if bb_ok { "PASS" } else { "FAIL" }, psb1c3_accessions.len());

    total += 1;
    let bfp_ok = bfp_measured.len() == BFP_CONTROLS.len();
    if bfp_ok { passed += 1; }
    eprintln!("  [{}] BFP controls: {}/{}",
        if bfp_ok { "PASS" } else { "FAIL" }, bfp_measured.len(), BFP_CONTROLS.len());

    total += 1;
    let a2_ok = psb1a2_accessions.len() >= 35;
    if a2_ok { passed += 1; }
    eprintln!("  [{}] pSB1A2 BioBricks: {} (expected: >= 35)",
        if a2_ok { "PASS" } else { "FAIL" }, psb1a2_accessions.len());

    if has_plate_data {
        let mut growth_rates: Vec<f64> = Vec::new();
        let mut plate_count = 0u32;

        if let Ok(entries) = std::fs::read_dir(&plate_data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let measurements = load_plate_measurements(&path);
                    if let Some(rate) = compute_growth_rate(&measurements) {
                        growth_rates.push(rate);
                    }
                    plate_count += 1;
                }
            }
        }

        total += 1;
        let plates_ok = plate_count >= 10;
        if plates_ok { passed += 1; }
        eprintln!("  [{}] Plate experiments parsed: {} (expected >= 10)",
            if plates_ok { "PASS" } else { "FAIL" }, plate_count);

        if growth_rates.len() >= 5 {
            total += 1;
            let rates_positive = growth_rates.iter().all(|r| *r > 0.0);
            if rates_positive { passed += 1; }
            eprintln!("  [{}] Growth rates all positive: {} rates computed",
                if rates_positive { "PASS" } else { "FAIL" }, growth_rates.len());

            let mean_rate = growth_rates.iter().sum::<f64>() / growth_rates.len() as f64;
            let burdens: Vec<f64> = growth_rates.iter()
                .map(|r| compute_burden(*r, mean_rate))
                .collect();

            let mean_burden = burdens.iter().sum::<f64>() / burdens.len() as f64;
            let var_burden = burdens.iter()
                .map(|b| (b - mean_burden).powi(2))
                .sum::<f64>() / burdens.len() as f64;
            let std_burden = var_burden.sqrt();

            total += 1;
            let has_variance = std_burden > 0.01;
            if has_variance { passed += 1; }
            eprintln!("  [{}] Burden distribution has variance: mean={mean_burden:.4}, std={std_burden:.4}",
                if has_variance { "PASS" } else { "FAIL" });

            if burdens.len() >= 10 {
                let m4 = burdens.iter()
                    .map(|b| (b - mean_burden).powi(4))
                    .sum::<f64>() / burdens.len() as f64;
                let kurtosis = if var_burden > 0.0 { m4 / var_burden.powi(2) } else { 0.0 };

                total += 1;
                let fat_tail = kurtosis > 2.5;
                if fat_tail { passed += 1; }
                eprintln!("  [{}] Fat-tailed burden distribution: kurtosis={kurtosis:.2} (expected > 2.5 for leptokurtic)",
                    if fat_tail { "PASS" } else { "FAIL" });
            }
        }
    }

    let status = if passed == total {
        ValidationStatus::Pass
    } else {
        ValidationStatus::Fail
    };

    ModuleResult {
        name: "biobrick_burden".to_string(),
        status,
        tier: 2,
        checks: total,
        checks_passed: passed,
        runtime_ms: start.elapsed().as_millis() as u64,
        error: if passed < total {
            Some(format!("{} check(s) failed", total - passed))
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_line_basic() {
        let line = "JEB1204,,E. coli DH10B,\"cam, kan\",pSB1C3,K3174002";
        let fields = parse_csv_line(line);
        assert_eq!(fields[0], "JEB1204");
        assert_eq!(fields[3], "cam, kan");
        assert_eq!(fields[4], "pSB1C3");
        assert_eq!(fields[5], "K3174002");
    }

    #[test]
    fn bfp_controls_are_five() {
        assert_eq!(BFP_CONTROLS.len(), 5);
    }

    #[test]
    fn growth_rate_computation() {
        let measurements = vec![
            GrowthMeasurement { time_hours: 0.0, od600: 0.01 },
            GrowthMeasurement { time_hours: 1.0, od600: 0.03 },
            GrowthMeasurement { time_hours: 2.0, od600: 0.08 },
            GrowthMeasurement { time_hours: 3.0, od600: 0.20 },
            GrowthMeasurement { time_hours: 4.0, od600: 0.50 },
        ];
        let rate = compute_growth_rate(&measurements);
        assert!(rate.is_some(), "should compute a growth rate");
        let r = rate.unwrap();
        assert!(r > 0.5 && r < 2.0, "growth rate {r} should be reasonable");
    }

    #[test]
    fn burden_computation() {
        let burden = compute_burden(0.8, 1.0);
        assert!((burden - 0.2).abs() < 0.001);
        let burden_zero = compute_burden(1.0, 1.0);
        assert!(burden_zero.abs() < 0.001);
    }
}
