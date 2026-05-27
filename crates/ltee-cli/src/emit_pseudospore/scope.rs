// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Write as _;
use std::path::Path;

pub(super) fn generate_scope(
    name: &str,
    version: &str,
    origin: &str,
    profile: Option<&pseudospore_core::DomainProfile>,
    data_dir: Option<&Path>,
    configs_dir: Option<&Path>,
) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut output = format!(
        r#"[artifact]
name = "{name}"
version = "{version}"
type = "pseudoSpore"
date = "{date}"
origin = "{origin}"
license = "AGPL-3.0-or-later"

"#
    );

    if profile.is_some() {
        writeln!(output, "[target]").unwrap();
        writeln!(output, "# Populate from domain_profile.toml or manually").unwrap();
        writeln!(output, "paper_doi = \"\"").unwrap();
        writeln!(output, "paper_title = \"\"\n").unwrap();
    }

    if let Some(data_path) = data_dir
        && data_path.exists()
    {
        let mut modules: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(data_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    modules.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        modules.sort();

        for module_name in &modules {
            let sim_time_ns =
                configs_dir.and_then(|c| find_mdp_and_extract_time(&c.join(module_name)));

            let (system, cv, method) = infer_module_metadata(module_name, profile);

            writeln!(output, "[[module]]").unwrap();
            writeln!(output, "name = \"{module_name}\"").unwrap();
            writeln!(output, "status = \"PASS\"").unwrap();
            writeln!(output, "system = \"{system}\"").unwrap();
            writeln!(output, "cv = \"{cv}\"").unwrap();
            writeln!(output, "method = \"{method}\"").unwrap();
            if let Some(ns) = sim_time_ns {
                writeln!(output, "simulation_time_ns = {ns}").unwrap();
            }
            output.push('\n');
        }
    }

    output.push_str(
        r#"[evolution]
tier_0 = "Industry control"
tier_1 = "Python sovereign implementation"
tier_2 = "Rust sovereign implementation"
tier_3 = "NUCLEUS IPC composition (future)"

[source]
repo = ""
commit = ""
branch = "main"
"#,
    );

    output
}

pub(crate) fn find_mdp_and_extract_time(config_dir: &Path) -> Option<u64> {
    if !config_dir.exists() {
        return None;
    }
    if let Ok(entries) = std::fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("mdp")
                && let Some(ns) = extract_sim_time_ns(&path)
            {
                return Some(ns);
            }
        }
    }
    None
}

pub(crate) fn extract_sim_time_ns(mdp_path: &Path) -> Option<u64> {
    let content = std::fs::read_to_string(mdp_path).ok()?;
    let mut nsteps: Option<f64> = None;
    let mut dt: Option<f64> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("nsteps") {
            if let Some(val) = line.split('=').nth(1) {
                nsteps = val.split_whitespace().next().and_then(|v| v.parse().ok());
            }
        } else if line.starts_with("dt")
            && let Some(val) = line.split('=').nth(1)
        {
            dt = val.split_whitespace().next().and_then(|v| v.parse().ok());
        }
    }

    match (nsteps, dt) {
        (Some(n), Some(d)) => f64_to_u64_ns(n * d / 1000.0),
        _ => None,
    }
}

fn f64_to_u64_ns(value: f64) -> Option<u64> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    format!("{value:.0}").parse().ok()
}

pub(crate) fn infer_module_metadata(
    module_name: &str,
    profile: Option<&pseudospore_core::DomainProfile>,
) -> (String, String, String) {
    let method = if let Some(p) = profile {
        if p.id.contains("metadynamics") {
            "Well-Tempered Metadynamics".to_string()
        } else {
            format!("{} simulation", p.id)
        }
    } else {
        "Simulation".to_string()
    };

    let is_2d = module_name.contains("2d");
    let system = module_name.replace(['-', '_'], " ");
    let cv = if is_2d {
        "2D collective variable".to_string()
    } else {
        "1D collective variable".to_string()
    };

    (system, cv, method)
}
