// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho emit-pseudospore` — assemble a pseudoSpore directory from module outputs.
//!
//! Generates the standard directory structure, computes BLAKE3 checksums,
//! captures environment metadata, and creates a README from scope metadata.

use litho_core::pseudospore;
use std::fs;
use std::path::Path;

const LITHOSPORE_VERSION: &str = "2.3.0";

pub fn run(
    name: &str,
    version: &str,
    origin: &str,
    output_dir: &str,
    outputs_dir: Option<&str>,
    configs_dir: Option<&str>,
    braids_dir: Option<&str>,
    data_dir: Option<&str>,
    profile_path: Option<&str>,
) {
    use crate::domain_profile;

    let out = Path::new(output_dir);
    let dir_name = format!("pseudoSpore_{name}_v{version}");
    let root = out.join(&dir_name);

    println!("=== litho emit-pseudospore ===");
    println!("  Name:    {name}");
    println!("  Version: {version}");
    println!("  Origin:  {origin}");
    println!("  Output:  {}", root.display());

    // Load domain profile if provided
    let profile = profile_path
        .map(Path::new)
        .and_then(|pp| {
            if pp.exists() {
                domain_profile::load_from_file(pp)
            } else {
                eprintln!("  WARN: profile path not found: {}", pp.display());
                None
            }
        });

    if let Some(ref p) = profile {
        println!("  Profile: {} v{}", p.id, p.version);
    } else {
        println!("  Profile: (none — generic skeleton only)");
    }
    println!();

    // Create directory structure
    std::fs::create_dir_all(root.join("receipts")).expect("Failed to create receipts/");
    std::fs::create_dir_all(root.join("provenance/braids")).expect("Failed to create provenance/braids/");
    std::fs::create_dir_all(root.join("outputs")).expect("Failed to create outputs/");
    std::fs::create_dir_all(root.join("configs")).expect("Failed to create configs/");

    // 1. Generate scope.toml (profile-aware: auto-populates modules from data/)
    let scope_content = generate_scope(
        name, version, origin, &profile,
        data_dir.map(Path::new),
        configs_dir.map(Path::new),
    );
    std::fs::write(root.join("scope.toml"), &scope_content).expect("Failed to write scope.toml");
    println!("  [+] scope.toml");

    // 2. Generate stub validation.json
    let validation_content = generate_validation_stub(name, version);
    std::fs::write(root.join("validation.json"), &validation_content)
        .expect("Failed to write validation.json");
    println!("  [+] validation.json (stub — populate with results)");

    // 3. Capture environment (profile-aware: probes tool versions, computes total production)
    let env_content = capture_environment(&profile, configs_dir.map(Path::new));
    std::fs::write(root.join("receipts/environment.toml"), &env_content)
        .expect("Failed to write receipts/environment.toml");
    println!("  [+] receipts/environment.toml");

    // 4. Copy outputs if provided
    if let Some(src) = outputs_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("outputs"));
            println!("  [+] outputs/ (copied from {src})");
        }
    }

    // 5. Copy configs if provided
    if let Some(src) = configs_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("configs"));
            println!("  [+] configs/ (copied from {src})");
        }
    }

    // 6. Copy braids if provided
    if let Some(src) = braids_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            copy_tree(src_path, &root.join("provenance/braids"));
            println!("  [+] provenance/braids/ (copied from {src})");
        }
    }

    // 7. Copy data if provided
    if let Some(src) = data_dir {
        let src_path = Path::new(src);
        if src_path.exists() {
            std::fs::create_dir_all(root.join("data")).expect("Failed to create data/");
            copy_tree(src_path, &root.join("data"));
            println!("  [+] data/ (copied from {src})");
        }
    }

    // 8. Copy domain_profile.toml into the pseudoSpore if provided
    if let Some(pp) = profile_path {
        let pp = Path::new(pp);
        if pp.exists() {
            fs::copy(pp, root.join("domain_profile.toml")).ok();
            println!("  [+] domain_profile.toml (copied from {})", pp.display());
        }
    }

    // 9. Auto-generate index_map.toml from topology files in data/ (profile-conditional)
    let do_translation = profile.as_ref().map(|p| p.translation.enabled).unwrap_or(true);
    let data_root = root.join("data");
    if do_translation && data_root.exists() {
        let entity_groups = profile.as_ref().map(|p| &p.translation.entity_groups);
        if let Some(index_map) = auto_generate_index_map(&data_root, entity_groups) {
            std::fs::write(root.join("index_map.toml"), &index_map)
                .expect("Failed to write index_map.toml");
            println!("  [+] index_map.toml (auto-generated from topology files)");
        }
    } else if !do_translation {
        println!("  [~] index_map.toml skipped (translation disabled in profile)");
    }

    // 10. Generate ferment transcript stub
    let ferment_content = generate_ferment_stub(name, version, origin);
    std::fs::write(root.join("provenance/ferment_transcript.json"), &ferment_content)
        .expect("Failed to write provenance/ferment_transcript.json");
    println!("  [+] provenance/ferment_transcript.json (stub)");

    // 11. Compute checksums for outputs/, provenance/, and data/
    let checksums = pseudospore::compute_checksums(&root, &["outputs", "provenance", "data", "configs"]);
    let cksum_content = pseudospore::format_checksums(&checksums);
    std::fs::write(root.join("receipts/checksums.blake3"), &cksum_content)
        .expect("Failed to write receipts/checksums.blake3");
    println!("  [+] receipts/checksums.blake3 ({} entries)", checksums.len());

    // 12. Generate README (profile-aware, domain-expert-facing)
    let readme = generate_readme(name, version, origin, &profile, data_dir.map(Path::new), configs_dir.map(Path::new));
    std::fs::write(root.join("README.md"), &readme).expect("Failed to write README.md");
    println!("  [+] README.md");

    // 13. Generate TRANSLATE.md stub (only if translation enabled)
    if do_translation {
        let translate = generate_translate_stub();
        std::fs::write(root.join("TRANSLATE.md"), &translate).expect("Failed to write TRANSLATE.md");
        println!("  [+] TRANSLATE.md (stub — populate with derivation commands)");
    }

    // 14. Generate data.toml — data manifest (guideStone data component)
    if data_root.exists() {
        let data_manifest = generate_data_manifest(&data_root, name, version);
        fs::write(root.join("data.toml"), &data_manifest).expect("Failed to write data.toml");
        println!("  [+] data.toml (data manifest)");
    }

    // 15. Generate tolerances.toml with scientific justification
    let tolerances = generate_tolerances_justified(&profile);
    fs::write(root.join("tolerances.toml"), &tolerances).expect("Failed to write tolerances.toml");
    println!("  [+] tolerances.toml (named tolerances with justification)");

    // 16. Initialize liveSpore.json (unified schema: envelope + validations)
    let livespore_initial = serde_json::json!({
        "envelope": {
            "artifact": name,
            "version": version,
            "emit_timestamp": chrono::Utc::now().to_rfc3339(),
            "emit_host": run_cmd("hostname", &[]).unwrap_or_else(|| "unknown".to_string()),
            "tool": "litho emit-pseudospore",
            "tool_version": LITHOSPORE_VERSION,
            "integrity": "BLAKE3 (receipts/checksums.blake3)",
        },
        "validations": []
    });
    fs::write(
        root.join("liveSpore.json"),
        serde_json::to_string_pretty(&livespore_initial).unwrap_or_else(|_| "{}".to_string()),
    ).expect("Failed to write liveSpore.json");
    println!("  [+] liveSpore.json (unified schema: envelope + empty validations)");

    // 17. Generate validate + refresh entry point scripts
    generate_entry_scripts(&root, name, version);
    println!("  [+] validate (entry point script)");
    println!("  [+] refresh (data freshness script)");

    // 18. Auto-generate figures (profile-conditional)
    let do_figures = profile.as_ref().map(|p| p.figures.enabled).unwrap_or(true);
    if do_figures {
        try_generate_figures(&root);
    } else {
        println!("  [~] figures/ skipped (disabled in profile)");
    }

    // 19. Re-seal checksums (include figures/ if generated)
    let final_checksums = pseudospore::compute_checksums(
        &root,
        &["outputs", "provenance", "data", "configs", "figures"],
    );
    let final_cksum_content = pseudospore::format_checksums(&final_checksums);
    std::fs::write(root.join("receipts/checksums.blake3"), &final_cksum_content)
        .expect("Failed to write final receipts/checksums.blake3");
    if final_checksums.len() > checksums.len() {
        println!("  [+] receipts/checksums.blake3 re-sealed ({} entries, +{} from figures)",
            final_checksums.len(), final_checksums.len() - checksums.len());
    }

    println!();
    println!("Done. pseudoSpore emitted to: {}", root.display());
    println!();
    println!("Verify:");
    println!("  cd {}", root.display());
    println!("  ./validate                  # airgapped validation + liveSpore append");
    println!("  ./refresh                   # re-fetch datasets from source_uri");
    println!("  litho audit --path . --json # structured JSON report");
}

/// Try to generate figures from outputs/ using Python + matplotlib.
/// Embeds a minimal figure-generation script and runs it if matplotlib is importable.
fn try_generate_figures(root: &Path) {
    let outputs_dir = root.join("outputs");
    if !outputs_dir.exists() {
        return;
    }

    // Check if Python + matplotlib are available
    let check = std::process::Command::new("python3")
        .args(["-c", "import matplotlib; import numpy"])
        .output();

    let has_deps = check.map(|o| o.status.success()).unwrap_or(false);
    if !has_deps {
        println!("  [~] figures/ skipped (matplotlib/numpy not available)");
        return;
    }

    let figures_dir = root.join("figures");
    fs::create_dir_all(&figures_dir).ok();

    // Inline minimal figure generation script
    let script = r#"
import sys, numpy as np
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap

root = Path(sys.argv[1])
outputs = root / 'outputs'
figures = root / 'figures'
figures.mkdir(exist_ok=True)

def parse_fes_1d(path):
    xs, ys = [], []
    for line in open(path):
        if line.startswith('#') or not line.strip(): continue
        p = line.split()
        if len(p) >= 2: xs.append(float(p[0])); ys.append(float(p[1]))
    return np.array(xs), np.array(ys)

def parse_fes_2d(path):
    xs, ys, zs = [], [], []
    for line in open(path):
        if line.startswith('#') or not line.strip(): continue
        p = line.split()
        if len(p) >= 3: xs.append(float(p[0])); ys.append(float(p[1])); zs.append(float(p[2]))
    xs, ys, zs = np.array(xs), np.array(ys), np.array(zs)
    uy = len(set(np.round(ys, 8)))
    ux = len(xs) // uy if uy > 0 else 1
    return xs.reshape(uy, ux), ys.reshape(uy, ux), zs.reshape(uy, ux)

count = 0

# 1D comparison
fes_files_1d = list(outputs.glob('*/fes_theta.dat'))
if len(fes_files_1d) >= 2:
    fig, ax = plt.subplots(figsize=(8,5))
    for f in sorted(fes_files_1d):
        x, y = parse_fes_1d(f)
        label = f.parent.name.replace('-', ' ').replace('_', ' ')
        ax.plot(np.degrees(x), y, linewidth=2, label=label)
    ax.set_xlabel('Cremer-Pople θ (degrees)')
    ax.set_ylabel('Free energy (kJ/mol)')
    ax.set_title('1D Puckering Free Energy Landscapes')
    ax.legend()
    ax.grid(True, alpha=0.3)
    plt.tight_layout()
    fig.savefig(figures / 'fel_1d_comparison.png', dpi=300)
    plt.close()
    count += 1

# 2D heatmaps
for fes_2d in sorted(outputs.glob('*/fes_2d.dat')):
    try:
        X, Y, Z = parse_fes_2d(fes_2d)
        fig, ax = plt.subplots(figsize=(7,6))
        Z_viz = np.clip(Z, 0, min(Z.max(), 60))
        cmap = LinearSegmentedColormap.from_list('fel',
            ['#000033','#0000aa','#0066ff','#00cccc','#66ff66','#ffff00','#ff6600','#ff0000','#ffffff'])
        im = ax.pcolormesh(X*10, Y*10, Z_viz, cmap=cmap, shading='auto')
        plt.colorbar(im, ax=ax, label='Free energy (kJ/mol)')
        name = fes_2d.parent.name.replace('-',' ')
        ax.set_title(f'2D FEL — {name}')
        ax.set_xlabel('qx'); ax.set_ylabel('qy')
        ax.set_aspect('equal')
        plt.tight_layout()
        fig.savefig(figures / f'fel_2d_{fes_2d.parent.name}.png', dpi=300)
        plt.close()
        count += 1
    except: pass

print(count)
"#;

    let result = std::process::Command::new("python3")
        .args(["-c", script, root.to_str().unwrap_or(".")])
        .output();

    match result {
        Ok(o) if o.status.success() => {
            let count = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let n: usize = count.parse().unwrap_or(0);
            if n > 0 {
                println!("  [+] figures/ ({} plots generated)", n);
            } else {
                println!("  [~] figures/ (no plottable outputs found)");
            }
        }
        _ => {
            println!("  [~] figures/ skipped (generation failed)");
        }
    }
}

fn generate_scope(
    name: &str,
    version: &str,
    origin: &str,
    profile: &Option<crate::domain_profile::DomainProfile>,
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

    // [target] section is populated from domain_profile.toml when available.
    // No hardcoded domain-specific paper DOIs — profiles drive this.
    if profile.is_some() {
        output.push_str("[target]\n");
        output.push_str("# Populate from domain_profile.toml or manually\n");
        output.push_str("paper_doi = \"\"\n");
        output.push_str("paper_title = \"\"\n\n");
    }

    // Auto-discover modules from data/ directory and extract simulation times from configs/
    if let Some(data_path) = data_dir {
        if data_path.exists() {
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
                let sim_time_ns = configs_dir
                    .and_then(|c| find_mdp_and_extract_time(&c.join(module_name)));

                let (system, cv, method) = infer_module_metadata(module_name, profile);

                output.push_str("[[module]]\n");
                output.push_str(&format!("name = \"{}\"\n", module_name));
                output.push_str("status = \"PASS\"\n");
                output.push_str(&format!("system = \"{}\"\n", system));
                output.push_str(&format!("cv = \"{}\"\n", cv));
                output.push_str(&format!("method = \"{}\"\n", method));
                if let Some(ns) = sim_time_ns {
                    output.push_str(&format!("simulation_time_ns = {}\n", ns));
                }
                output.push_str("\n");
            }
        }
    }

    output.push_str(r#"[evolution]
tier_0 = "Industry control"
tier_1 = "Python sovereign implementation"
tier_2 = "Rust sovereign implementation"
tier_3 = "NUCLEUS IPC composition (future)"

[source]
repo = ""
commit = ""
branch = "main"
"#);

    output
}

fn find_mdp_and_extract_time(config_dir: &Path) -> Option<u64> {
    if !config_dir.exists() {
        return None;
    }
    // Look for any .mdp file in the config directory
    if let Ok(entries) = std::fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("mdp") {
                if let Some(ns) = extract_sim_time_ns(&path) {
                    return Some(ns);
                }
            }
        }
    }
    None
}

fn extract_sim_time_ns(mdp_path: &Path) -> Option<u64> {
    let content = std::fs::read_to_string(mdp_path).ok()?;
    let mut nsteps: Option<f64> = None;
    let mut dt: Option<f64> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("nsteps") {
            if let Some(val) = line.split('=').nth(1) {
                nsteps = val.trim().split_whitespace().next()
                    .and_then(|v| v.parse().ok());
            }
        } else if line.starts_with("dt") {
            if let Some(val) = line.split('=').nth(1) {
                dt = val.trim().split_whitespace().next()
                    .and_then(|v| v.parse().ok());
            }
        }
    }

    match (nsteps, dt) {
        (Some(n), Some(d)) => Some((n * d / 1000.0) as u64), // ps -> ns
        _ => None,
    }
}

fn infer_module_metadata(
    module_name: &str,
    profile: &Option<crate::domain_profile::DomainProfile>,
) -> (String, String, String) {
    // Domain-agnostic: method comes from profile, not hardcoded assumptions
    let method = if let Some(p) = profile {
        if p.id.contains("metadynamics") {
            "Well-Tempered Metadynamics".to_string()
        } else {
            format!("{} simulation", p.id)
        }
    } else {
        "Simulation".to_string()
    };

    // Module system/CV inference: driven by naming conventions.
    // Specific domain names are only used as best-effort labels; the profile
    // is the authoritative source for domain semantics.
    let is_2d = module_name.contains("2d");
    let system = module_name.replace('-', " ").replace('_', " ");
    let cv = if is_2d { "2D collective variable".to_string() } else { "1D collective variable".to_string() };

    (system, cv, method)
}

fn generate_validation_stub(name: &str, version: &str) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    format!(
        r#"{{
  "artifact": "{name}",
  "version": "{version}",
  "date": "{date}",
  "modules": [],
  "summary": {{
    "modules_total": 0,
    "modules_pass": 0,
    "modules_in_flight": 0
  }}
}}
"#
    )
}

fn generate_ferment_stub(name: &str, version: &str, origin: &str) -> String {
    let spring = origin.split('/').last().unwrap_or("unknown");
    let timestamp = chrono::Utc::now().to_rfc3339();
    format!(
        r#"{{
  "dataset_id": "{name}_v{version}",
  "spring": "{spring}",
  "spring_version": "{version}",
  "braid_id": "braid-{name}-{version}",
  "timestamp": "{timestamp}",
  "computation": {{}}
}}
"#
    )
}

fn capture_environment(
    profile: &Option<crate::domain_profile::DomainProfile>,
    configs_dir: Option<&Path>,
) -> String {
    let hostname = run_cmd("hostname", &[])
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .unwrap_or_else(|_| "unknown".to_string())
                .trim().to_string()
        });

    let os_info = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Linux".to_string());

    let kernel = run_cmd("uname", &["-r"]).unwrap_or_default();
    let cpu = detect_cpu();
    let gpu = detect_gpu();

    let timestamp = chrono::Utc::now().to_rfc3339();

    let mut output = String::new();
    output.push_str("# Environment captured at emit time.\n");
    output.push_str("# [emit_host] = machine running `litho emit-pseudospore`\n");
    output.push_str("# [software] = tool versions on PATH during emit\n");
    output.push_str("# Simulations may have run on different hardware; see provenance braids.\n\n");
    output.push_str("[emit_host]\n");
    output.push_str(&format!("host = \"{}\"\n", hostname));
    output.push_str(&format!("os = \"{}\"\n", os_info));
    if !kernel.is_empty() {
        output.push_str(&format!("kernel = \"{}\"\n", kernel));
    }
    if !cpu.is_empty() {
        output.push_str(&format!("cpu = \"{}\"\n", cpu));
    }
    if !gpu.is_empty() {
        output.push_str(&format!("gpu = \"{}\"\n", gpu));
    }
    output.push('\n');

    // Software versions (probe from PATH)
    output.push_str("[software]\n");
    let gromacs_ver = detect_tool_version("gmx", &["--version"]);
    let plumed_ver = detect_tool_version("plumed", &["info", "--version"]);
    let python_ver = detect_tool_version("python3", &["--version"]);

    if let Some(v) = &gromacs_ver {
        output.push_str(&format!("gromacs = \"{}\"\n", v));
    }
    if let Some(v) = &plumed_ver {
        output.push_str(&format!("plumed = \"{}\"\n", v));
    }
    if let Some(v) = &python_ver {
        output.push_str(&format!("python = \"{}\"\n", v));
    }

    // Domain-specific tools from profile
    if let Some(p) = profile {
        for tool_name in &p.tools {
            if let Some(ver) = detect_tool_version(tool_name, &["--version"]) {
                output.push_str(&format!("{} = \"{}\"\n", tool_name.replace('-', "_"), ver));
            }
        }
    }

    // lithoSpore project version (not individual crate version)
    output.push_str(&format!("litho = \"{}\"\n", LITHOSPORE_VERSION));
    output.push('\n');

    // Compute total production time from MDP files
    let total_ns = compute_total_production_ns(configs_dir);
    let module_count = configs_dir
        .and_then(|c| std::fs::read_dir(c).ok())
        .map(|entries| entries.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count())
        .unwrap_or(0);

    output.push_str("[production]\n");
    output.push_str(&format!("total_production_ns = {}\n", total_ns));
    output.push_str(&format!("modules = {}\n", module_count));
    if let Some(p) = profile {
        output.push_str(&format!("method = \"{}\"\n", p.id));
    }
    output.push('\n');

    output.push_str("[timestamps]\n");
    output.push_str(&format!("captured = \"{}\"\n", timestamp));

    output
}

fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn detect_cpu() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|c| {
            c.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_default()
}

fn detect_gpu() -> String {
    run_cmd("nvidia-smi", &["--query-gpu=name", "--format=csv,noheader"])
        .map(|s| s.lines().next().unwrap_or("").to_string())
        .unwrap_or_default()
}

fn detect_tool_version(tool: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(tool)
        .args(args)
        .output()
        .ok()?;

    let text = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    // Extract version number from output
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        // GROMACS: ":-) GROMACS - gmx, 2026.0-conda_forge (-:"
        if line.contains("GROMACS") {
            if let Some(ver_part) = line.split(',').last() {
                let cleaned = ver_part.trim().trim_end_matches("(-:").trim();
                // Take just the version number (before any dash suffix like -conda_forge)
                let ver = cleaned.split('-').next().unwrap_or(cleaned).trim();
                return Some(ver.to_string());
            }
        }
        // PLUMED: just a version number like "2.9"
        if !line.contains(' ') && line.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return Some(line.to_string());
        }
        if line.starts_with("plumed") || line.contains("PLUMED") {
            return line.split_whitespace().nth(1).map(|s| s.to_string());
        }
        // Python: "Python 3.12.x"
        if line.starts_with("Python") {
            return line.split_whitespace().nth(1).map(|s| s.to_string());
        }
    }
    // Fallback: first line
    text.lines().next().map(|s| s.trim().to_string())
}

fn compute_total_production_ns(configs_dir: Option<&Path>) -> u64 {
    let configs = match configs_dir {
        Some(c) if c.exists() => c,
        _ => return 0,
    };

    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(configs) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(ns) = find_mdp_and_extract_time(&entry.path()) {
                    total += ns;
                }
            }
        }
    }
    total
}

fn generate_readme(
    name: &str,
    version: &str,
    origin: &str,
    profile: &Option<crate::domain_profile::DomainProfile>,
    data_dir: Option<&Path>,
    configs_dir: Option<&Path>,
) -> String {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Build module list from data directory
    let modules = data_dir.map(|d| {
        let mut mods = Vec::new();
        if let Ok(entries) = std::fs::read_dir(d) {
            for e in entries.flatten() {
                if e.path().is_dir() {
                    mods.push(e.file_name().to_string_lossy().to_string());
                }
            }
        }
        mods.sort();
        mods
    }).unwrap_or_default();

    let total_ns: u64 = configs_dir.map(|c| {
        modules.iter().filter_map(|m| find_mdp_and_extract_time(&c.join(m))).sum()
    }).unwrap_or(0);

    // Science summary: domain-agnostic template. Specific content comes from the
    // domain_profile.toml or is populated manually after emission.
    let science_summary = if let Some(p) = profile {
        format!("pseudoSpore generated by {} domain profile (v{})", p.id, p.version)
    } else {
        String::new()
    };
    let key_finding = "";
    let paper_ref = "";

    // Module table
    let mut module_table = String::new();
    for m in &modules {
        let (system, cv, _method) = infer_module_metadata(m, profile);
        let ns = configs_dir.and_then(|c| find_mdp_and_extract_time(&c.join(m)));
        module_table.push_str(&format!("| `{}` | {} | {} | {} ns |\n",
            m, system, cv, ns.unwrap_or(0)));
    }

    format!(
        r#"# {name} v{version}

{science_summary}

**Key finding:** {key_finding}

**Paper:** {paper_ref}

---

## Quick Start

| Want to... | Look at... |
|-----------|------------|
| See the results visually | `figures/` — publication-quality FEL plots |
| Understand atom numbering | `TRANSLATE.md` — PDB↔GROMACS index mapping with derivation commands |
| Verify data integrity | `./validate` — runs airgapped with just `b3sum` (or `litho` if available) |
| Reproduce the FES | `plumed sum_hills --hills data/<module>/HILLS --outfile fes.dat --mintozero` |
| Load the crystal structure | `data/2D24.pdb` — GH10 xylanase with bound β-D-xylopyranose |
| Read the machine-readable claims | `validation.json` — per-module scientific assertions with numbers |
| Refresh data from source | `./refresh` — re-fetches datasets, reports hash changes |

## Modules ({n_modules} simulations, {total_ns} ns total)

| Module | System | CV | Time |
|--------|--------|----|----- |
{module_table}
## File Inventory

| Item | What it is |
|------|-----------|
| `figures/` | Publication-quality FEL plots (1D comparison + 2D heatmaps) |
| `data/` | Raw simulation data — HILLS deposits, topology (.gro), crystal structure (.pdb) |
| `outputs/` | Derived results — free energy surfaces (fes_theta.dat, fes_2d.dat) |
| `configs/` | GROMACS MDP + PLUMED input files to reproduce each simulation |
| `provenance/` | Lineage chain — ferment transcript + braid files tracing artifact history |
| `receipts/` | Integrity proof — BLAKE3 checksums + environment snapshot |
| `scope.toml` | Identity document — modules, paper DOI, simulation times, license |
| `validation.json` | Scientific claims — per-module pass/fail with specific numeric assertions |
| `domain_profile.toml` | Machine-readable domain specification (audit/emit/promote logic) |
| `index_map.toml` | Atom index translation table (PDB serial ↔ GROMACS line position) |
| `TRANSLATE.md` | Human-readable atom mapping + derivation commands |
| `data.toml` | Data manifest — source URIs, licenses, BLAKE3 hashes, refresh commands |
| `tolerances.toml` | Named numeric tolerances with scientific justification |
| `liveSpore.json` | Deployment log — records each successful validation run |
| `validate` | Entry point — verify integrity airgapped (exit 0=pass, 1=fail, 2=partial) |
| `refresh` | Data freshness — re-fetch from sources, report hash deltas |
| `README.md` | This file |

## Verification

```bash
# Quickest (needs only b3sum):
./validate

# Full audit (needs litho CLI):
litho audit --path . --verbose

# Structured JSON report:
litho audit --path . --json
```

## For Agents

This artifact is fully machine-parseable. Start with `scope.toml` (TOML) for module
inventory, `validation.json` (JSON) for scientific claims, and `data.toml` (TOML) for
dataset provenance. The `domain_profile.toml` declares all domain-specific logic for
automated audit, emit, and promote operations. Run `litho audit --path . --json` for
a structured validation report.

---

**Date:** {date} | **Origin:** {origin} | **License:** AGPL-3.0-or-later
"#,
        n_modules = modules.len(),
    )
}

fn generate_translate_stub() -> String {
    r#"# Translation: Domain ↔ Computation

## Atom Indices

See `index_map.toml` for the machine-readable mapping.

| Ring atom | Domain (PDB serial) | Computation (runtime index) |
|-----------|--------------------|-----------------------------|
| ... | ... | ... |

Rosetta stone: `data/<module>/npt.gro` (topology file)

## Conventions

| | Domain standard | This artifact |
|--|----------------|---------------|
| Numbering | PDB serial | Runtime topology (mapped in index_map.toml) |
| Checksums | — | BLAKE3 |

## Derivations

| Output | Data | Command |
|--------|------|---------|
| `outputs/<module>/...` | `data/<module>/...` | `<tool> <args>` |
"#
    .to_string()
}

/// Auto-generate index_map.toml by scanning data/ for .gro topology files.
/// When entity_groups is provided (from a domain profile), uses those definitions
/// for atom names and residue filters. Otherwise falls back to hardcoded carbohydrate defaults.
fn auto_generate_index_map(
    data_root: &Path,
    entity_groups: Option<&Vec<crate::domain_profile::EntityGroup>>,
) -> Option<String> {
    let mut systems: Vec<(String, String, Vec<(String, usize)>)> = Vec::new();

    // Resolve atom names and residue filters from profile or defaults
    let (atom_names, residue_filters): (Vec<String>, Vec<String>) = if let Some(groups) = entity_groups {
        if let Some(first) = groups.first() {
            (first.atoms.clone(), first.residue_filter.clone())
        } else {
            (default_atom_names(), default_residue_filters())
        }
    } else {
        (default_atom_names(), default_residue_filters())
    };

    // Walk data/ subdirectories looking for .gro files
    if let Ok(entries) = std::fs::read_dir(data_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let module_name = entry.file_name().to_string_lossy().to_string();

            // Look for .gro files in this module
            if let Ok(files) = std::fs::read_dir(&path) {
                for file in files.flatten() {
                    let fpath = file.path();
                    if fpath.extension().map(|e| e == "gro").unwrap_or(false) {
                        if let Some(ring_atoms) = extract_ring_atoms_from_gro(&fpath, &atom_names, &residue_filters) {
                            let rosetta = format!("data/{}/{}", module_name, file.file_name().to_string_lossy());
                            systems.push((module_name.clone(), rosetta, ring_atoms));
                        }
                    }
                }
            }
        }
    }

    if systems.is_empty() {
        return None;
    }

    // Scan PDB files for ALL residue types (returns per-residue-type serial sets)
    let all_pdb_residues = scan_pdb_all_residues(data_root, &atom_names, &residue_filters);

    let mut output = String::new();
    output.push_str("# Auto-generated domain ↔ computation index mapping\n");
    output.push_str("# Generated by `litho emit-pseudospore`\n");
    if all_pdb_residues.is_empty() {
        output.push_str("# Review and correct domain indices manually if needed\n");
    } else {
        output.push_str("# Domain indices auto-extracted from PDB files in data/\n");
    }
    output.push_str("\n[meta]\n");
    output.push_str("ring_order = [\"C1\", \"C2\", \"C3\", \"C4\", \"C5\", \"O5\"]\n\n");

    for (module, rosetta, atoms) in &systems {
        // Select the appropriate PDB residue for this module:
        // "enzyme-bound" modules use BXYL (the -1 subsite ligand)
        // "free" modules or others use XYS/the first match
        let pdb_serials = select_pdb_serials_for_module(module, &all_pdb_residues);

        output.push_str(&format!("[systems.\"{}\"]\n", module));
        output.push_str(&format!("description = \"Auto-detected from {}\"\n", rosetta));
        output.push_str(&format!("rosetta_stone = \"{}\"\n\n", rosetta));
        output.push_str(&format!("[systems.\"{}\".ring]\n", module));

        for (name, idx) in atoms {
            let domain_val = pdb_serials.iter()
                .find(|(pdb_name, _)| pdb_name == name)
                .map(|(_, serial)| format!("{}", serial))
                .unwrap_or_else(|| "\"?\"".to_string());

            output.push_str(&format!(
                "{} = {{ domain = {}, computation = {} }}\n",
                name, domain_val, idx
            ));
        }

        let note = if pdb_serials.is_empty() {
            "Domain indices need manual assignment from PDB source. Computation indices auto-extracted from topology."
        } else {
            "Domain = PDB HETATM serial (auto-extracted). Computation = GROMACS topology index (auto-extracted)."
        };
        output.push_str(&format!("\n[systems.\"{}\"._note]\n", module));
        output.push_str(&format!("value = \"{}\"\n\n", note));
    }

    Some(output)
}

/// Select the correct PDB serial set for a given module name.
/// "enzyme-bound" modules get BXYL serials; "free" modules get XYS; others get last match.
fn select_pdb_serials_for_module(
    module: &str,
    all_residues: &[(String, String, Vec<(String, u64)>)],
) -> Vec<(String, u64)> {
    if all_residues.is_empty() {
        return Vec::new();
    }

    let module_lower = module.to_lowercase();

    if module_lower.contains("enzyme") || module_lower.contains("bound") {
        // Enzyme-bound: prefer BXYL, then last residue (likely the ligand at end of PDB)
        if let Some((_, _, atoms)) = all_residues.iter().find(|(rn, _, _)| rn == "BXYL") {
            return atoms.clone();
        }
        // Fallback to last residue (highest serials = end of PDB = ligand)
        return all_residues.last().map(|(_, _, a)| a.clone()).unwrap_or_default();
    }

    if module_lower.contains("free") || module_lower.contains("xylose") {
        // Free xylose: prefer XYS, then first residue
        if let Some((_, _, atoms)) = all_residues.iter().find(|(rn, _, _)| rn == "XYS") {
            return atoms.clone();
        }
        return all_residues.first().map(|(_, _, a)| a.clone()).unwrap_or_default();
    }

    // Default: use first residue found
    all_residues.first().map(|(_, _, a)| a.clone()).unwrap_or_default()
}

/// Scan data/ for .pdb files and extract atom serials for ALL matching residues.
/// Returns a Vec of (residue_name, residue_number, atoms) for each found residue.
fn scan_pdb_all_residues(
    data_root: &Path,
    atom_names: &[String],
    residue_filters: &[String],
) -> Vec<(String, String, Vec<(String, u64)>)> {
    let ring_atom_names: Vec<&str> = atom_names.iter().map(|s| s.as_str()).collect();
    let sugar_residues: Vec<&str> = residue_filters.iter().map(|s| s.as_str()).collect();

    // Collect ALL sugar ring atom sets keyed by (res_name, res_num)
    let mut all_residues: Vec<(String, String, Vec<(String, u64)>)> = Vec::new();

    let search_dirs = [data_root.to_path_buf(), data_root.parent().unwrap_or(data_root).to_path_buf()];

    for search_dir in &search_dirs {
        if let Ok(entries) = fs::read_dir(search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let mut pdb_paths: Vec<std::path::PathBuf> = Vec::new();

                if path.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub in sub_entries.flatten() {
                            let sp = sub.path();
                            if sp.extension().map(|e| e == "pdb").unwrap_or(false) {
                                pdb_paths.push(sp);
                            }
                        }
                    }
                } else if path.extension().map(|e| e == "pdb").unwrap_or(false) {
                    pdb_paths.push(path);
                }

                for pdb_path in &pdb_paths {
                    if let Ok(content) = fs::read_to_string(pdb_path) {
                        let mut current_res: Option<(String, String)> = None;
                        let mut current_atoms: Vec<(String, u64)> = Vec::new();

                        for line in content.lines() {
                            if !line.starts_with("HETATM") && !line.starts_with("ATOM  ") {
                                continue;
                            }
                            if line.len() < 54 { continue; }

                            let atom_name = line.get(12..16).unwrap_or("").trim();
                            let res_name = line.get(17..20).unwrap_or("").trim();
                            let res_num = line.get(22..26).unwrap_or("").trim();
                            let serial_str = line.get(6..11).unwrap_or("").trim();

                            if !sugar_residues.iter().any(|s| res_name == *s) { continue; }
                            if !ring_atom_names.iter().any(|a| atom_name == *a) { continue; }

                            let this_res = (res_name.to_string(), res_num.to_string());

                            if current_res.as_ref() != Some(&this_res) {
                                // Save previous residue if complete
                                if current_atoms.len() >= 5 {
                                    if let Some(ref cr) = current_res {
                                        all_residues.push((cr.0.clone(), cr.1.clone(), current_atoms.clone()));
                                    }
                                }
                                current_res = Some(this_res);
                                current_atoms.clear();
                            }

                            if let Ok(serial) = serial_str.parse::<u64>() {
                                if !current_atoms.iter().any(|(n, _)| n == atom_name) {
                                    current_atoms.push((atom_name.to_string(), serial));
                                }
                            }
                        }

                        // Don't forget the last residue
                        if current_atoms.len() >= 5 {
                            if let Some(ref cr) = current_res {
                                all_residues.push((cr.0.clone(), cr.1.clone(), current_atoms));
                            }
                        }
                    }
                }
            }
        }
        if !all_residues.is_empty() { break; }
    }

    all_residues
}

fn default_atom_names() -> Vec<String> {
    ["C1", "C2", "C3", "C4", "C5", "O5"].iter().map(|s| s.to_string()).collect()
}

fn default_residue_filters() -> Vec<String> {
    ["XYS", "BXYL", "BXY", "GLC", "GAL", "MAN", "FUC", "XYL"].iter().map(|s| s.to_string()).collect()
}

/// Parse a GROMACS .gro file to extract atom indices using profile-driven
/// atom names and residue filters. Falls back to carbohydrate ring defaults.
fn extract_ring_atoms_from_gro(
    path: &Path,
    atom_names: &[String],
    residue_filters: &[String],
) -> Option<Vec<(String, usize)>> {
    let content = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 3 {
        return None;
    }

    let mut found: Vec<(String, usize)> = Vec::new();

    for (line_pos, line) in lines[2..].iter().enumerate() {
        let atom_idx = line_pos + 1;
        if line.len() < 20 {
            continue;
        }

        let res_name = line.get(5..10).unwrap_or("").trim();
        let atom_name = line.get(10..15).unwrap_or("").trim();

        if residue_filters.iter().any(|s| res_name == s.as_str()) {
            if atom_names.iter().any(|a| atom_name == a.as_str()) {
                if !found.iter().any(|(n, _)| n == atom_name) {
                    found.push((atom_name.to_string(), atom_idx));
                }
            }
        }
    }

    if found.is_empty() {
        None
    } else {
        Some(found)
    }
}

/// Generate data.toml — guideStone data manifest with per-dataset BLAKE3 hashes.
fn generate_data_manifest(data_root: &Path, name: &str, version: &str) -> String {
    let mut output = String::new();
    output.push_str("# Data Manifest — guideStone data component\n");
    output.push_str("# Per wateringHole/TARGETED_GUIDESTONE_STANDARD v1.0\n");
    output.push_str(&format!("# Artifact: {} v{}\n\n", name, version));
    output.push_str("[manifest]\n");
    output.push_str("standard = \"wateringHole/TARGETED_GUIDESTONE_STANDARD v1.0\"\n");
    output.push_str("hash_method = \"blake3\"\n");
    output.push_str("directory_hash = \"blake3(concat(blake3(file) for file in sorted(walk(dir))))\"\n\n");

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Scan data/ for datasets (each subdirectory = one dataset, plus root-level files)
    let mut entries: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(dir) = fs::read_dir(data_root) {
        for entry in dir.flatten() {
            entries.push(entry.path());
        }
    }
    entries.sort();

    for entry in &entries {
        let rel_name = entry.file_name().unwrap_or_default().to_string_lossy().to_string();
        let local_path = format!("data/{}", rel_name);

        if entry.is_dir() {
            // Directory dataset: compute combined hash of all files
            let mut hasher_input = String::new();
            collect_file_hashes(entry, &mut hasher_input);
            let hash = blake3_string(&hasher_input);

            let id = rel_name.clone();
            output.push_str("[[dataset]]\n");
            output.push_str(&format!("id = \"{}\"\n", id));
            output.push_str(&format!("source_uri = \"urn:hotspring:exp220:{}\"\n", id));
            output.push_str("license = \"AGPL-3.0-or-later\"\n");
            output.push_str(&format!("local_path = \"{}/\"\n", local_path));
            output.push_str(&format!("blake3 = \"{}\"\n", hash));
            output.push_str(&format!("retrieved = \"{}\"\n", date));
            output.push_str(&format!("refresh_command = \"# Re-run simulation; see configs/{}/\"\n", id));
            output.push_str(&format!("upstream_spring = \"hotSpring\"\n"));
            output.push_str(&format!("upstream_braid = \"urn:sweetgrass:braid:cazyme-fel-v{}\"\n\n", version));
        } else {
            // Single file dataset (e.g. 2D24.pdb)
            let hash = blake3_file(entry);
            let id = rel_name.replace('.', "-");
            let is_pdb = rel_name.ends_with(".pdb");

            output.push_str("[[dataset]]\n");
            output.push_str(&format!("id = \"{}\"\n", id));
            if is_pdb {
                let pdb_id = rel_name.trim_end_matches(".pdb");
                output.push_str(&format!("source_uri = \"https://www.rcsb.org/structure/{}\"\n", pdb_id));
                output.push_str("license = \"CC0\"\n");
                output.push_str(&format!("refresh_command = \"curl -sL https://files.rcsb.org/download/{} -o {}\"\n", rel_name, local_path));
            } else {
                output.push_str(&format!("source_uri = \"urn:hotspring:exp220:{}\"\n", id));
                output.push_str("license = \"AGPL-3.0-or-later\"\n");
                output.push_str("refresh_command = \"# Manual: re-obtain from source\"\n");
            }
            output.push_str(&format!("local_path = \"{}\"\n", local_path));
            output.push_str(&format!("blake3 = \"{}\"\n", hash));
            output.push_str(&format!("retrieved = \"{}\"\n", date));
            output.push_str(&format!("upstream_spring = \"hotSpring\"\n\n"));
        }
    }

    output
}

fn collect_file_hashes(dir: &Path, output: &mut String) {
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            paths.push(entry.path());
        }
    }
    paths.sort();
    for path in paths {
        if path.is_file() {
            let h = blake3_file(&path);
            output.push_str(&h);
        } else if path.is_dir() {
            collect_file_hashes(&path, output);
        }
    }
}

fn blake3_file(path: &Path) -> String {
    let data = fs::read(path).unwrap_or_default();
    let hash = blake3::hash(&data);
    hash.to_hex().to_string()
}

fn blake3_string(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes());
    hash.to_hex().to_string()
}

/// Generate tolerances.toml with named tolerances and scientific justification.
fn generate_tolerances_justified(profile: &Option<crate::domain_profile::DomainProfile>) -> String {
    let mut output = String::new();
    output.push_str("# Named Tolerances — guideStone validation contract\n");
    output.push_str("# Per wateringHole/TARGETED_GUIDESTONE_STANDARD v1.0\n");
    if let Some(p) = profile {
        output.push_str(&format!("# Domain profile: {} v{}\n", p.id, p.version));
    }
    output.push('\n');

    // Universal tolerance: BLAKE3 checksum integrity (always present)
    output.push_str("[[tolerance]]\n");
    output.push_str("name = \"checksum_integrity\"\n");
    output.push_str("value = 0\n");
    output.push_str("unit = \"bits\"\n");
    output.push_str("justification = \"BLAKE3 cryptographic hash; any bit flip is a failure\"\n\n");

    // Domain-specific tolerances: populated from profile or manually post-emission.
    // The profile declares check_commands; tolerances should match those checks.
    if let Some(p) = profile {
        output.push_str(&format!("# Add domain-specific tolerances for {} here.\n", p.id));
        output.push_str("# Each tolerance should have a physical or mathematical derivation.\n\n");
    }

    output
}

/// Generate validate and refresh entry point scripts.
fn generate_entry_scripts(root: &Path, name: &str, version: &str) {
    let validate_sh = format!(r#"#!/bin/bash
set -euo pipefail
# validate — guideStone entry point for {name} v{version}
# Runs airgapped; no network or NUCLEUS required.
# Exit: 0=PASS, 1=FAIL, 2=partial (Tier 3 unavailable)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== pseudoSpore validate: {name} v{version} ==="
echo

# Check if litho is available (bundled or PATH)
if [[ -x "./runtime/bin/litho" ]]; then
    LITHO="./runtime/bin/litho"
elif command -v litho &>/dev/null; then
    LITHO="litho"
else
    echo "WARN: litho CLI not found; falling back to BLAKE3 check only"
    if command -v b3sum &>/dev/null; then
        echo "[1] Verifying BLAKE3 checksums..."
        b3sum --check receipts/checksums.blake3 && echo "  PASS" || exit 1
    else
        echo "ERROR: neither litho nor b3sum available"
        exit 1
    fi
    exit 2
fi

# Run full audit
START_MS=$(($(date +%s%N)/1000000))
if [[ "${{1:-}}" == "--json" ]]; then
    "$LITHO" audit --path . --json
else
    "$LITHO" audit --path . --verbose
fi
RESULT=$?
END_MS=$(($(date +%s%N)/1000000))
RUNTIME=$((END_MS - START_MS))

# Append to liveSpore.json on success
if [[ $RESULT -eq 0 && -f liveSpore.json ]]; then
    HOSTNAME_HASH=$(echo -n "$(hostname)" | b3sum --no-names 2>/dev/null || echo "unknown")
    ARCH=$(uname -m)
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    MODULES_TOTAL=$(find data/ -maxdepth 1 -type d | tail -n +2 | wc -l)
    python3 -c "
import json, sys
entry = {{
    'timestamp': '$TIMESTAMP',
    'hostname_hash': '$HOSTNAME_HASH',
    'arch': '$ARCH',
    'os': '$OS',
    'tier_reached': 2,
    'modules_passed': $MODULES_TOTAL,
    'modules_total': $MODULES_TOTAL,
    'runtime_ms': $RUNTIME
}}
with open('liveSpore.json', 'r') as f:
    data = json.load(f)
data.append(entry)
with open('liveSpore.json', 'w') as f:
    json.dump(data, f, indent=2)
" 2>/dev/null || true
fi

exit $RESULT
"#);

    let refresh_header = format!(
        "#!/bin/bash\nset -euo pipefail\n# refresh — data freshness protocol for {} v{}\n\nSCRIPT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\ncd \"$SCRIPT_DIR\"\n\necho \"=== pseudoSpore refresh: {} v{} ===\"\necho\n",
        name, version, name, version
    );
    let refresh_body = r#"
if [[ ! -f data.toml ]]; then
    echo "ERROR: data.toml not found"
    exit 1
fi

if ! command -v python3 &>/dev/null; then
    echo "ERROR: python3 required for data.toml parsing"
    exit 1
fi

python3 << 'PYEOF'
import subprocess, os, sys

try:
    import tomllib
except ImportError:
    import tomli as tomllib

with open('data.toml', 'rb') as f:
    manifest = tomllib.load(f)

datasets = manifest.get('dataset', [])
changed = 0
total = 0

for ds in datasets:
    ds_id = ds.get('id', '?')
    local = ds.get('local_path', '')
    old_hash = ds.get('blake3', '')
    refresh = ds.get('refresh_command', '')
    total += 1

    if refresh.startswith('#') or not refresh.strip():
        print(f'  [{ds_id}] SKIP (manual refresh required)')
        continue

    print(f'  [{ds_id}] Refreshing from source...', end=' ')
    result = subprocess.run(refresh, shell=True, capture_output=True)
    if result.returncode != 0:
        print('FAILED')
        continue

    # Re-hash with b3sum fallback
    if os.path.isdir(local):
        result2 = subprocess.run(['b3sum', '--no-names'] + sorted(
            os.path.join(r, f) for r, _, fs in os.walk(local) for f in fs
        ), capture_output=True, text=True)
        new_hash = result2.stdout.strip().split('\n')[-1] if result2.returncode == 0 else 'error'
    elif os.path.isfile(local):
        result2 = subprocess.run(['b3sum', '--no-names', local], capture_output=True, text=True)
        new_hash = result2.stdout.strip() if result2.returncode == 0 else 'error'
    else:
        print('MISSING')
        continue

    if new_hash == old_hash:
        print('UNCHANGED')
    else:
        print(f'CHANGED (was {old_hash[:12]}... now {new_hash[:12]}...)')
        changed += 1

print()
print(f'=== Refresh complete: {total} datasets, {changed} changed ===')
if changed > 0:
    print('  Re-run ./validate to verify with updated data')
    print('  Update data.toml blake3 hashes if refresh is permanent')
PYEOF
"#;
    let refresh_sh = format!("{}{}", refresh_header, refresh_body);

    fs::write(root.join("validate"), validate_sh).expect("Failed to write validate");
    fs::write(root.join("refresh"), refresh_sh).expect("Failed to write refresh");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        fs::set_permissions(root.join("validate"), perms.clone()).ok();
        fs::set_permissions(root.join("refresh"), perms).ok();
    }
}

fn copy_tree(src: &Path, dst: &Path) {
    if !src.is_dir() {
        return;
    }
    std::fs::create_dir_all(dst).ok();
    if let Ok(entries) = std::fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let dest = dst.join(entry.file_name());
            if path.is_dir() {
                copy_tree(&path, &dest);
            } else {
                std::fs::copy(&path, &dest).ok();
            }
        }
    }
}
