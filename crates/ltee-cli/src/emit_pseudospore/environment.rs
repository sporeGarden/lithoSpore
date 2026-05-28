// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Write as _;
use std::path::Path;

use super::{LITHOSPORE_VERSION, run_cmd, scope};

pub(super) fn capture_environment(
    profile: Option<&pseudospore_core::DomainProfile>,
    configs_dir: Option<&Path>,
) -> String {
    let hostname = run_cmd("hostname", &[])
        .or_else(|| std::env::var(litho_core::env_vars::HOSTNAME).ok())
        .unwrap_or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .unwrap_or_else(|_| "unknown".to_string())
                .trim()
                .to_string()
        });

    let os_info = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|c| {
            c.lines().find(|l| l.starts_with("PRETTY_NAME=")).map(|l| {
                l.trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string()
            })
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
    writeln!(output, "[emit_host]").unwrap();
    writeln!(output, "host = \"{hostname}\"").unwrap();
    writeln!(output, "os = \"{os_info}\"").unwrap();
    if !kernel.is_empty() {
        writeln!(output, "kernel = \"{kernel}\"").unwrap();
    }
    if !cpu.is_empty() {
        writeln!(output, "cpu = \"{cpu}\"").unwrap();
    }
    if !gpu.is_empty() {
        writeln!(output, "gpu = \"{gpu}\"").unwrap();
    }
    output.push('\n');

    output.push_str("[software]\n");
    let gromacs_ver = detect_tool_version("gmx", &["--version"]);
    let plumed_ver = detect_tool_version("plumed", &["info", "--version"]);
    let python_ver = detect_tool_version("python3", &["--version"]);

    if let Some(v) = &gromacs_ver {
        writeln!(output, "gromacs = \"{v}\"").unwrap();
    }
    if let Some(v) = &plumed_ver {
        writeln!(output, "plumed = \"{v}\"").unwrap();
    }
    if let Some(v) = &python_ver {
        writeln!(output, "python = \"{v}\"").unwrap();
    }

    if let Some(p) = profile {
        for tool_name in &p.tools {
            if let Some(ver) = detect_tool_version(tool_name, &["--version"]) {
                let key = tool_name.replace('-', "_");
                writeln!(output, "{key} = \"{ver}\"").unwrap();
            }
        }
    }

    writeln!(output, "litho = \"{LITHOSPORE_VERSION}\"").unwrap();
    output.push('\n');

    let total_ns = compute_total_production_ns(configs_dir);
    let module_count = configs_dir
        .and_then(|c| std::fs::read_dir(c).ok())
        .map_or(0, |entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().is_dir())
                .count()
        });

    writeln!(output, "[production]").unwrap();
    writeln!(output, "total_production_ns = {total_ns}").unwrap();
    writeln!(output, "modules = {module_count}").unwrap();
    if let Some(p) = profile {
        writeln!(output, "method = \"{}\"", p.id).unwrap();
    }
    output.push('\n');

    writeln!(output, "[timestamps]").unwrap();
    writeln!(output, "captured = \"{timestamp}\"").unwrap();

    output
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
    let output = std::process::Command::new(tool).args(args).output().ok()?;

    let text = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.contains("GROMACS")
            && let Some(ver_part) = line.split(',').next_back()
        {
            let cleaned = ver_part.trim().trim_end_matches("(-:").trim();
            let ver = cleaned.split('-').next().unwrap_or(cleaned).trim();
            return Some(ver.to_string());
        }
        if !line.contains(' ') && line.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Some(line.to_string());
        }
        if line.starts_with("plumed") || line.contains("PLUMED") {
            return line
                .split_whitespace()
                .nth(1)
                .map(std::string::ToString::to_string);
        }
        if line.starts_with("Python") {
            return line
                .split_whitespace()
                .nth(1)
                .map(std::string::ToString::to_string);
        }
    }
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
            if entry.path().is_dir()
                && let Some(ns) = scope::find_mdp_and_extract_time(&entry.path())
            {
                total += ns;
            }
        }
    }
    total
}
