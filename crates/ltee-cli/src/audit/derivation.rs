// SPDX-License-Identifier: AGPL-3.0-or-later

//! Derivation contract verification — re-derive outputs from raw data and compare.
//!
//! Uses `plumed sum_hills` when available; falls back to HILLS line-count
//! sanity checks when plumed is not installed.

use std::{fs, path::Path};

use super::{Finding, Severity};

/// Verify derivation contract — outputs can be re-derived from data.
///
/// Uses plumed `sum_hills` internally if available, otherwise checks file sizes
/// and HILLS line counts as a proxy.
pub(super) fn check_derivation_contract(root: &Path, findings: &mut Vec<Finding>) {
    let data_dir = root.join("data");
    let outputs_dir = root.join("outputs");

    if !data_dir.exists() || !outputs_dir.exists() {
        return;
    }

    let plumed_bin = find_plumed();

    if let Ok(modules) = fs::read_dir(&data_dir) {
        for module in modules.flatten() {
            if !module.path().is_dir() {
                continue;
            }
            let mod_name = module.file_name().to_string_lossy().to_string();

            check_1d_hills(
                &module.path(),
                &outputs_dir,
                &mod_name,
                plumed_bin.as_deref(),
                findings,
            );
            check_2d_hills(
                &module.path(),
                &outputs_dir,
                &mod_name,
                plumed_bin.as_deref(),
                findings,
            );
        }
    }
}

fn check_1d_hills(
    module_dir: &Path,
    outputs_dir: &Path,
    mod_name: &str,
    plumed_bin: Option<&str>,
    findings: &mut Vec<Finding>,
) {
    let hills_path = module_dir.join("HILLS");
    let output_fes = outputs_dir.join(mod_name).join("fes_theta.dat");

    if !hills_path.exists() || !output_fes.exists() {
        return;
    }

    if let Some(plumed) = plumed_bin {
        let tmp_out = std::env::temp_dir().join(format!("litho_audit_derive_{mod_name}.dat"));
        let tmp_out_s = tmp_out.to_string_lossy();
        let result = std::process::Command::new(plumed)
            .args(["sum_hills", "--hills"])
            .arg(&hills_path)
            .args(["--mintozero", "--outfile", tmp_out_s.as_ref()])
            .output();

        if let Ok(o) = result
            && o.status.success()
        {
            compare_fes_files(&tmp_out, &output_fes, mod_name, "1D", findings);
        }
        fs::remove_file(&tmp_out).ok();
    } else {
        let hills_lines = fs::read_to_string(&hills_path).map_or(0, |c| {
            c.lines()
                .filter(|l| !l.starts_with('#') && !l.is_empty())
                .count()
        });
        if hills_lines < 100 {
            findings.push(Finding {
                id: format!("HILLS-SHORT-{mod_name}"),
                severity: Severity::Medium,
                category: "Derivation Contract",
                message: format!(
                    "{mod_name}: HILLS has only {hills_lines} depositions (< 100, likely incomplete)"
                ),
                fix: "Verify simulation completed or mark module as IN_FLIGHT".to_string(),
            });
        }
    }
}

fn check_2d_hills(
    module_dir: &Path,
    outputs_dir: &Path,
    mod_name: &str,
    plumed_bin: Option<&str>,
    findings: &mut Vec<Finding>,
) {
    let hills_2d_path = module_dir.join("HILLS_2d");
    let output_fes_2d = outputs_dir.join(mod_name).join("fes_2d.dat");

    if !hills_2d_path.exists() || !output_fes_2d.exists() {
        return;
    }

    let Some(plumed) = plumed_bin else { return };

    let tmp_out = std::env::temp_dir().join(format!("litho_audit_derive_2d_{mod_name}.dat"));
    let tmp_out_s = tmp_out.to_string_lossy();
    let result = std::process::Command::new(plumed)
        .args(["sum_hills", "--hills"])
        .arg(&hills_2d_path)
        .args([
            "--min",
            "-0.12,-0.12",
            "--max",
            "0.12,0.12",
            "--bin",
            "100,100",
        ])
        .args(["--mintozero", "--outfile", tmp_out_s.as_ref()])
        .output();

    if let Ok(o) = result
        && o.status.success()
    {
        compare_fes_files(&tmp_out, &output_fes_2d, mod_name, "2D", findings);
    }
    fs::remove_file(&tmp_out).ok();
}

/// Compare a derived FES file against the shipped output.
fn compare_fes_files(
    derived_path: &Path,
    expected_path: &Path,
    mod_name: &str,
    dimensionality: &str,
    findings: &mut Vec<Finding>,
) {
    let derived = fs::read_to_string(derived_path).unwrap_or_default();
    let expected = fs::read_to_string(expected_path).unwrap_or_default();
    if derived == expected {
        return;
    }

    let energy_col = if dimensionality == "2D" { 2 } else { 1 };

    let d_vals: Vec<f64> = derived
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(|l| l.split_whitespace().nth(energy_col))
        .filter_map(|s| s.parse().ok())
        .collect();
    let e_vals: Vec<f64> = expected
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(|l| l.split_whitespace().nth(energy_col))
        .filter_map(|s| s.parse().ok())
        .collect();

    if d_vals.len() == e_vals.len() && !d_vals.is_empty() {
        let max_diff: f64 = d_vals
            .iter()
            .zip(e_vals.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        if max_diff > 0.001 {
            findings.push(Finding {
                id: format!("DERIVATION-{dimensionality}-FAIL-{mod_name}"),
                severity: Severity::High,
                category: "Derivation Contract",
                message: format!(
                    "{mod_name}: {dimensionality} FES re-derivation differs by {max_diff:.4} kJ/mol max"
                ),
                fix: "Regenerate outputs/ from data/ with matching parameters".to_string(),
            });
        }
    } else if d_vals.len() != e_vals.len() {
        findings.push(Finding {
            id: format!("DERIVATION-SIZE-{mod_name}"),
            severity: Severity::Medium,
            category: "Derivation Contract",
            message: format!(
                "{mod_name}: re-derived {dimensionality} FES has {} points, shipped has {}",
                d_vals.len(),
                e_vals.len()
            ),
            fix: "Check GRID settings — derivation may need explicit --min/--max/--bin".to_string(),
        });
    }
}

/// Locate plumed binary — checks PATH then conda/system locations via liveness probe.
fn find_plumed() -> Option<String> {
    let alive = |bin: &str| -> bool {
        std::process::Command::new(bin)
            .args(["info", "--root"])
            .output()
            .is_ok_and(|o| o.status.success())
    };
    if alive("plumed") {
        return Some("plumed".to_string());
    }
    let home = std::env::var(litho_core::env_vars::HOME).unwrap_or_default();
    let suffixes = [
        "miniconda3/envs/gromacs-fel",
        "miniconda3",
        "anaconda3/envs/gromacs-fel",
    ];
    for sfx in &suffixes {
        let p = format!("{home}/{sfx}/bin/plumed");
        if Path::new(&p).exists() && alive(&p) {
            return Some(p);
        }
    }
    ["/usr/local/bin/plumed", "/usr/bin/plumed"]
        .iter()
        .find(|p| Path::new(*p).exists() && alive(p))
        .map(|p| (*p).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_fes_identical_files() {
        let dir = std::env::temp_dir();
        let a = dir.join("litho_test_fes_a.dat");
        let b = dir.join("litho_test_fes_b.dat");
        let content = "# comment\n0.0 1.0\n1.0 2.0\n2.0 0.5\n";
        std::fs::write(&a, content).unwrap();
        std::fs::write(&b, content).unwrap();
        let mut findings = Vec::new();
        compare_fes_files(&a, &b, "test_mod", "1D", &mut findings);
        assert!(
            findings.is_empty(),
            "identical files should produce no findings"
        );
        std::fs::remove_file(&a).ok();
        std::fs::remove_file(&b).ok();
    }

    #[test]
    fn compare_fes_different_files() {
        let dir = std::env::temp_dir();
        let a = dir.join("litho_test_fes_diff_a.dat");
        let b = dir.join("litho_test_fes_diff_b.dat");
        std::fs::write(&a, "# comment\n0.0 1.0\n1.0 2.0\n").unwrap();
        std::fs::write(&b, "# comment\n0.0 1.0\n1.0 5.0\n").unwrap();
        let mut findings = Vec::new();
        compare_fes_files(&a, &b, "test_mod", "1D", &mut findings);
        assert_eq!(
            findings.len(),
            1,
            "divergent files should produce a finding"
        );
        assert!(findings[0].id.contains("DERIVATION"));
        std::fs::remove_file(&a).ok();
        std::fs::remove_file(&b).ok();
    }

    #[test]
    fn derivation_empty_dirs_noop() {
        let dir = tempfile::tempdir().unwrap();
        let mut findings = Vec::new();
        check_derivation_contract(dir.path(), &mut findings);
        assert!(findings.is_empty());
    }

    #[test]
    fn derivation_no_data_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("outputs/m1")).unwrap();
        let mut findings = Vec::new();
        check_derivation_contract(dir.path(), &mut findings);
        assert!(findings.is_empty());
    }
}
