// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho visualize` — generate scientific visualizations for all modules.

use crate::validate::LIVE_MODULES;

const NOTEBOOKS: &[(&str, &str)] = &[
    ("module1_fitness", "notebooks/module1_fitness/power_law_fitness.py"),
    ("module2_mutations", "notebooks/module2_mutations/mutation_accumulation.py"),
    ("module3_alleles", "notebooks/module3_alleles/allele_trajectories.py"),
    ("module4_citrate", "notebooks/module4_citrate/citrate_innovation.py"),
    ("module5_biobricks", "notebooks/module5_biobricks/biobrick_burden.py"),
    ("module6_breseq", "notebooks/module6_breseq/breseq_comparison.py"),
    ("module7_anderson", "notebooks/module7_anderson/anderson_predictions.py"),
];

const BASELINE_TOOLS: &[&str] = &[
    "breseq", "plannotate", "ostir", "cryptkeeper",
    "efm", "marker_divergence", "rna_mi",
];

pub fn run(root: &str, format: &str, output_dir: &str) {
    let root_path = std::path::Path::new(root);

    let modules: Vec<(&str, &str)> = LIVE_MODULES
        .iter()
        .map(|(name, _, _, expected)| (*name, *expected))
        .collect();

    match format {
        "json" => run_json(root_path, &modules),
        "svg" => run_svg(root_path, output_dir),
        "dashboard" => run_dashboard(root_path, &modules),
        "baselines" => run_baselines(root_path),
        _ => {
            eprintln!("ERROR: unknown format '{format}' (use: svg, json, dashboard, baselines)");
            std::process::exit(1);
        }
    }
}

fn run_json(root_path: &std::path::Path, modules: &[(&str, &str)]) {
    let all = load_module_data(root_path, modules);
    let refs: Vec<(&str, &serde_json::Value)> = all.iter().map(|(n, v)| (*n, v)).collect();
    let dashboard = litho_core::viz::build_dashboard(&refs);
    println!("{}", serde_json::to_string_pretty(&dashboard).unwrap_or_default());
}

fn run_svg(root_path: &std::path::Path, output_dir: &str) {
    eprintln!("litho visualize --format svg: generating static figures via Python baselines");

    let out_path = root_path.join(output_dir);
    std::fs::create_dir_all(&out_path).ok();

    let mut generated = 0u32;
    for (mod_name, notebook) in NOTEBOOKS {
        let nb_path = root_path.join(notebook);
        if !nb_path.exists() {
            eprintln!("  SKIP {mod_name}: notebook not found");
            continue;
        }

        eprintln!("  {mod_name}...");
        let status = std::process::Command::new("python3")
            .arg(&nb_path)
            .current_dir(root_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match status {
            Ok(s) if s.success() => generated += 1,
            Ok(s) => eprintln!("    WARNING: exit {}", s.code().unwrap_or(-1)),
            Err(e) => eprintln!("    WARNING: {e}"),
        }
    }

    eprintln!("  {generated} modules processed, figures in {}", out_path.display());

    let svgs: Vec<String> = std::fs::read_dir(&out_path)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "svg").unwrap_or(false))
        .map(|e| e.path().display().to_string())
        .collect();

    for svg in &svgs {
        println!("  {svg}");
    }
    eprintln!("  {} SVG figures generated", svgs.len());
}

fn run_dashboard(root_path: &std::path::Path, modules: &[(&str, &str)]) {
    let socket_path = discover_petaltongue_socket();
    if socket_path.is_empty() {
        eprintln!("ERROR: petalTongue socket not found");
        eprintln!("  Set PETALTONGUE_SOCKET or start petalTongue first");
        eprintln!("  Falling back to JSON output:");
        run_json(root_path, modules);
        return;
    }

    eprintln!("litho visualize --format dashboard");
    eprintln!("  petalTongue socket: {socket_path}");

    let all = load_module_data(root_path, modules);
    let refs: Vec<(&str, &serde_json::Value)> = all.iter().map(|(n, v)| (*n, v)).collect();
    let dashboard = litho_core::viz::build_dashboard(&refs);

    push_to_petaltongue(&socket_path, &dashboard, "Dashboard");
}

fn run_baselines(root_path: &std::path::Path) {
    let baselines_dir = root_path.join("baselines");
    if !baselines_dir.exists() {
        eprintln!("ERROR: baselines/ directory not found at {}", baselines_dir.display());
        std::process::exit(1);
    }

    let mut all_tools: Vec<(&str, serde_json::Value)> = Vec::new();
    for tool in BASELINE_TOOLS {
        let ref_path = baselines_dir.join(tool).join("reference_data.json");
        if !ref_path.exists() {
            eprintln!("  SKIP {tool}: reference_data.json not found");
            continue;
        }
        let content = match std::fs::read_to_string(&ref_path) {
            Ok(c) => c,
            Err(e) => { eprintln!("  SKIP {tool}: {e}"); continue; }
        };
        let data: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => { eprintln!("  SKIP {tool}: parse error: {e}"); continue; }
        };
        eprintln!("  Loaded baseline: {tool}");
        all_tools.push((tool, data));
    }

    let refs: Vec<(&str, &serde_json::Value)> = all_tools.iter().map(|(n, v)| (*n, v)).collect();
    let dashboard = litho_core::viz::build_baseline_dashboard(&refs);
    let bindings_count = dashboard["bindings"].as_array().map(|a| a.len()).unwrap_or(0);
    eprintln!("  {bindings_count} DataBindings from {} tools", all_tools.len());

    let socket_path = discover_petaltongue_socket();
    if socket_path.is_empty() {
        eprintln!("  petalTongue not found — outputting JSON");
        println!("{}", serde_json::to_string_pretty(&dashboard).unwrap_or_default());
        return;
    }

    push_to_petaltongue(&socket_path, &dashboard, "Baselines");
}

fn load_module_data<'a>(root_path: &std::path::Path, modules: &[(&'a str, &str)]) -> Vec<(&'a str, serde_json::Value)> {
    let mut all = Vec::new();
    for (name, expected_path) in modules {
        let path = root_path.join(expected_path);
        if !path.exists() {
            eprintln!("  SKIP {name}: expected values not found");
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => { eprintln!("  SKIP {name}: {e}"); continue; }
        };
        let expected: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => { eprintln!("  SKIP {name}: parse error: {e}"); continue; }
        };
        all.push((*name, expected));
    }
    all
}

fn push_to_petaltongue(socket_path: &str, dashboard: &serde_json::Value, label: &str) {
    eprintln!("  Pushing to petalTongue: {socket_path}");

    let rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "visualization.render",
        "params": dashboard,
        "id": 1,
    });

    let payload = serde_json::to_vec(&rpc_request).unwrap_or_default();
    match send_uds(socket_path, &payload) {
        Ok(response) => {
            eprintln!("  {label} pushed to petalTongue");
            if !response.is_empty() {
                println!("{response}");
            }
        }
        Err(e) => {
            eprintln!("  WARNING: petalTongue push failed: {e}");
            println!("{}", serde_json::to_string_pretty(dashboard).unwrap_or_default());
        }
    }
}

/// Discover the petalTongue socket using the capability-based chain:
///   1. `$PETALTONGUE_SOCKET` explicit path
///   2. `$XDG_RUNTIME_DIR/biomeos/petaltongue*.sock` (XDG standard)
///   3. `/tmp/biomeos/petaltongue.sock` (fallback)
///
/// No primal-specific names are encoded — only the "visualization"
/// capability socket convention.
pub(crate) fn discover_petaltongue_socket() -> String {
    if let Ok(path) = std::env::var("PETALTONGUE_SOCKET") {
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }

    let xdg_runtime = resolve_xdg_runtime();

    let candidates = [
        format!("{xdg_runtime}/biomeos/petaltongue.sock"),
        format!("{xdg_runtime}/biomeos/petaltongue-nat0.sock"),
        "/tmp/biomeos/petaltongue.sock".into(),
    ];

    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return candidate.clone();
        }
    }

    String::new()
}

fn resolve_xdg_runtime() -> String {
    std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        #[cfg(unix)]
        {
            let uid = std::fs::read_to_string("/proc/self/status")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("Uid:"))
                        .and_then(|l| l.split_whitespace().nth(1))
                        .map(String::from)
                })
                .unwrap_or_else(|| "1000".to_string());
            format!("/run/user/{uid}")
        }
        #[cfg(not(unix))]
        {
            std::env::var("TEMP").unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_petaltongue_socket_returns_string() {
        // In test environments the socket likely doesn't exist, so we just
        // verify the function returns without panicking.
        let result = discover_petaltongue_socket();
        // Empty is expected unless a live petalTongue is running
        let _ = result;
    }

    #[test]
    fn load_module_data_skips_missing() {
        let root = std::path::Path::new("/nonexistent");
        let modules = [("test_module", "nonexistent/path.json")];
        let result = load_module_data(root, &modules);
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_xdg_runtime_returns_path() {
        let path = resolve_xdg_runtime();
        assert!(!path.is_empty());
    }
}

#[cfg(unix)]
fn send_uds(socket_path: &str, payload: &[u8]) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| format!("connect: {e}"))?;

    stream.set_write_timeout(Some(std::time::Duration::from_secs(5))).ok();
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10))).ok();

    stream.write_all(payload).map_err(|e| format!("write: {e}"))?;
    stream.flush().map_err(|e| format!("flush: {e}"))?;

    stream.shutdown(std::net::Shutdown::Write).ok();

    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|e| format!("read: {e}"))?;

    Ok(response)
}

#[cfg(not(unix))]
fn send_uds(_socket_path: &str, _payload: &[u8]) -> Result<String, String> {
    Err("UDS transport not available on this platform".to_string())
}
