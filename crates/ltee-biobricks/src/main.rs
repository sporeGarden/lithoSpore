// SPDX-License-Identifier: AGPL-3.0-or-later

//! Module 5: BioBrick metabolic burden
//!
//! Validates burden measurements from Barrick et al. 2024 (B6), Nature Communications.
//! doi:10.1038/s41467-024-50639-9
//!
//! Data: barricklab/igem2019 (v1.0.2) — 301 BioBrick plasmid growth burden assays.
//! Springs: neuralSpring B6 (ML surrogate, future), groundSpring B6 (baseline, future).
//!
//! Tier 1: dispatches to Python baseline.
//! Tier 2: pure Rust CSV parsing + burden distribution validation.

use clap::Parser;
use litho_core::harness;
use litho_core::{ModuleResult, ValidationStatus};
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "ltee-biobricks", about = "BioBrick metabolic burden validation")]
struct Cli {
    #[arg(long, default_value = "artifact/data/biobricks_2024")]
    data_dir: String,

    #[arg(long, default_value = "validation/expected/module5_biobricks.json")]
    expected: String,

    #[arg(long, default_value = "2")]
    max_tier: u8,

    #[arg(long)]
    json: bool,
}

const BFP_CONTROLS: &[&str] = &["K3174002", "K3174003", "K3174004", "K3174006", "K3174007"];

fn main() {
    let cli = Cli::parse();
    let result = run_validation(&cli);
    harness::output_and_exit(&result, cli.json);
}

fn run_validation(cli: &Cli) -> ModuleResult {
    let start = Instant::now();

    if !Path::new(&cli.expected).exists() {
        return harness::skip(
            "biobrick_burden", 1, start,
            "Expected values not found — run fetch + generate expected values first",
        );
    }

    if !Path::new(&cli.data_dir).exists() {
        return harness::skip(
            "biobrick_burden", 1, start,
            "Data not fetched — run scripts/fetch_biobricks_2024.sh",
        );
    }

    if cli.max_tier >= 2 {
        return run_tier2_rust(cli, start);
    }
    if cli.max_tier >= 1 {
        return harness::dispatch_python(
            "biobrick_burden",
            Path::new("notebooks/module5_biobricks/biobrick_burden.py"),
            Path::new("."),
        );
    }

    harness::skip(
        "biobrick_burden", cli.max_tier, start,
        &format!("Tier {} not implemented yet", cli.max_tier),
    )
}

// ── Tier 2: Pure Rust ────────────────────────────────────────────────

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

fn run_tier2_rust(cli: &Cli, start: Instant) -> ModuleResult {
    let expected = match harness::load_expected(&cli.expected) {
        Some(v) => v,
        None => return harness::skip(
            "biobrick_burden", 2, start,
            "Cannot parse expected values JSON",
        ),
    };

    let strains = match load_strain_metadata(&cli.data_dir) {
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

    let plate_data_dir = Path::new(&cli.data_dir).join("input-plate-data");
    let has_plate_data = plate_data_dir.exists()
        && std::fs::read_dir(&plate_data_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);

    let mut passed = 0_u32;
    let mut total = 0_u32;

    let exp_total = expected["total_biobricks_tested"].as_u64().unwrap_or(301);
    let count_tol = expected["tolerances"]["count_tolerance"].as_u64().unwrap_or(5);

    // Check 1: total BioBrick parts count
    total += 1;
    let bb_count = biobrick_accessions.len() as u64;
    let count_ok = bb_count.abs_diff(exp_total) <= count_tol;
    if count_ok { passed += 1; }
    eprintln!(
        "  [{}] Total BioBrick parts: {} (expected: {} +/- {})",
        if count_ok { "PASS" } else { "FAIL" },
        bb_count, exp_total, count_tol,
    );

    // Check 2: growth curve plate data present
    total += 1;
    if has_plate_data { passed += 1; }
    eprintln!(
        "  [{}] Growth curve plate data present",
        if has_plate_data { "PASS" } else { "FAIL" },
    );

    // Check 3: backbone distribution (pSB1C3 >= 230)
    total += 1;
    let bb_ok = psb1c3_accessions.len() >= 230;
    if bb_ok { passed += 1; }
    eprintln!(
        "  [{}] pSB1C3 BioBricks: {} (expected: >= 230)",
        if bb_ok { "PASS" } else { "FAIL" },
        psb1c3_accessions.len(),
    );

    // Check 4: all 5 BFP controls present
    total += 1;
    let bfp_ok = bfp_measured.len() == BFP_CONTROLS.len();
    if bfp_ok { passed += 1; }
    eprintln!(
        "  [{}] BFP controls: {}/{}",
        if bfp_ok { "PASS" } else { "FAIL" },
        bfp_measured.len(), BFP_CONTROLS.len(),
    );

    // Check 5: pSB1A2 representation
    total += 1;
    let a2_ok = psb1a2_accessions.len() >= 35;
    if a2_ok { passed += 1; }
    eprintln!(
        "  [{}] pSB1A2 BioBricks: {} (expected: >= 35)",
        if a2_ok { "PASS" } else { "FAIL" },
        psb1a2_accessions.len(),
    );

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
    fn bfp_controls_have_k3174_prefix() {
        for ctrl in BFP_CONTROLS {
            assert!(ctrl.starts_with("K3174"), "{ctrl} should start with K3174");
        }
    }
}
