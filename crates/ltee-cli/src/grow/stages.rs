// SPDX-License-Identifier: AGPL-3.0-or-later

use super::util::{
    detect_sra_toolkit, find_litho_binary, pick_git_url, run_cmd, run_cmd_in, step, which,
};
use std::path::Path;
use std::process::Command;

type ValidateRunner = Box<dyn FnMut(&[&str]) -> Option<std::process::ExitStatus>>;

/// Default Rust cross-compile target (`LITHO_RUST_TARGET` overrides).
fn default_rust_target() -> String {
    std::env::var(litho_core::env_vars::LITHO_RUST_TARGET).unwrap_or_else(|_| {
        match (std::env::consts::ARCH, std::env::consts::OS) {
            ("x86_64", "linux") => "x86_64-unknown-linux-musl".into(),
            ("aarch64", "linux") => "aarch64-unknown-linux-gnu".into(),
            ("x86_64", "macos") => "x86_64-apple-darwin".into(),
            ("aarch64", "macos") => "aarch64-apple-darwin".into(),
            (arch, os) => format!("{arch}-unknown-{os}"),
        }
    })
}

pub(super) struct SourceConfig {
    pub repo: String,
    pub repo_https: String,
    pub branch: String,
    pub ecosystem_repo: String,
    pub ecosystem_repo_https: String,
    pub rust_toolchain: String,
    pub rust_target: String,
}

pub(super) fn load_source_metadata(root: &Path) -> SourceConfig {
    let scope_path = root.join("artifact/scope.toml");
    if let Ok(scope) = litho_core::ScopeManifest::load(&scope_path)
        && let Some(src) = scope.source
    {
        return SourceConfig {
            repo: src.repo,
            repo_https: src.repo_https,
            branch: if src.branch.is_empty() {
                "main".into()
            } else {
                src.branch
            },
            ecosystem_repo: src.ecosystem_repo,
            ecosystem_repo_https: src.ecosystem_repo_https,
            rust_toolchain: if src.rust_toolchain.is_empty() {
                "stable".into()
            } else {
                src.rust_toolchain
            },
            rust_target: if src.rust_target.is_empty() {
                default_rust_target()
            } else {
                src.rust_target
            },
        };
    }

    eprintln!("  WARNING: No [source] metadata in scope.toml — using defaults");
    SourceConfig {
        repo: "https://github.com/sporeGarden/lithoSpore.git".into(),
        repo_https: "https://github.com/sporeGarden/lithoSpore.git".into(),
        branch: "main".into(),
        ecosystem_repo: "https://github.com/sporeGarden/ecoPrimals.git".into(),
        ecosystem_repo_https: "https://github.com/sporeGarden/ecoPrimals.git".into(),
        rust_toolchain: "stable".into(),
        rust_target: default_rust_target(),
    }
}

pub(super) fn stage_clone(_root: &Path, target: &Path, scope: &SourceConfig, ecosystem: bool) {
    step("1. Cloning source repository");

    if ecosystem {
        let eco_target = target.parent().unwrap_or(target);
        if eco_target.join(".git").exists() {
            println!(
                "  Ecosystem repo already exists at {}",
                eco_target.display()
            );
        } else {
            let repo_url = pick_git_url(&scope.ecosystem_repo, &scope.ecosystem_repo_https);
            println!("  Cloning ecosystem: {repo_url}");
            run_cmd(
                "git",
                &[
                    "clone",
                    "--depth",
                    "1",
                    "-b",
                    &scope.branch,
                    &repo_url,
                    &eco_target.to_string_lossy(),
                ],
            );
        }
        let garden_path = eco_target.join("gardens/lithoSpore");
        if garden_path.exists() && !target.exists() {
            println!("  lithoSpore found at {}", garden_path.display());
            println!("  NOTE: Use the ecosystem path for development.");
        }
    }

    if target.join(".git").exists() {
        println!("  Source repo already exists at {}", target.display());
        println!("  Pulling latest...");
        run_cmd_in("git", &["pull", "--ff-only"], target);
        return;
    }

    if target.exists()
        && std::fs::read_dir(target)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    {
        println!("  Target directory is not empty and not a git repo.");
        println!("  Skipping clone — will attempt to use existing content.");
        return;
    }

    let repo_url = pick_git_url(&scope.repo, &scope.repo_https);
    println!("  Cloning: {repo_url} → {}", target.display());
    run_cmd(
        "git",
        &[
            "clone",
            "--depth",
            "1",
            "-b",
            &scope.branch,
            &repo_url,
            &target.to_string_lossy(),
        ],
    );
    println!("  Source cloned successfully");
}

pub(super) fn stage_toolchain(scope: &SourceConfig) {
    step("2. Checking Rust toolchain");

    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  Found: {}", version.trim());
    } else {
        println!("  Rust not found — installing via rustup...");
        let status = Command::new("sh")
            .args(["-c", &format!("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain {}", scope.rust_toolchain)])
            .status();
        match status {
            Ok(s) if s.success() => println!("  Rust installed successfully"),
            Ok(s) => {
                eprintln!("  ERROR: rustup exited with {s}");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("  ERROR: Could not run rustup installer: {e}");
                eprintln!(
                    "  Install manually: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                );
                std::process::exit(1);
            }
        }
    }

    if let Ok(output) = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        let installed = String::from_utf8_lossy(&output.stdout);
        if installed.contains(&scope.rust_target) {
            println!("  Target {} already installed", scope.rust_target);
        } else {
            println!("  Adding target: {}", scope.rust_target);
            run_cmd("rustup", &["target", "add", &scope.rust_target]);
        }
    }

    if which("musl-gcc") {
        println!("  musl-tools: available");
    } else {
        println!("  WARNING: musl-gcc not found — `apt install musl-tools` or equivalent");
        println!("           Build will proceed without musl (non-static binary)");
    }
}

pub(super) fn stage_build(target: &Path, scope: &SourceConfig) {
    step("3. Building from source");

    if !target.join("Cargo.toml").exists() {
        eprintln!("  ERROR: No Cargo.toml found at {}", target.display());
        eprintln!("  Cannot build — clone may have failed.");
        return;
    }

    let mut args = vec!["build", "--release"];
    if which("musl-gcc") {
        args.extend(["--target", &scope.rust_target]);
        println!("  Building: cargo {} (musl-static)", args.join(" "));
    } else {
        println!("  Building: cargo {} (native)", args.join(" "));
    }

    run_cmd_in("cargo", &args, target);

    let binary = if which("musl-gcc") {
        target.join(format!("target/{}/release/litho", scope.rust_target))
    } else {
        target.join("target/release/litho")
    };
    if binary.exists() {
        let size = std::fs::metadata(&binary).map(|m| m.len()).unwrap_or(0);
        println!(
            "  Built: {} ({:.1} MB)",
            binary.display(),
            size as f64 / 1_048_576.0
        );
    } else {
        eprintln!(
            "  WARNING: Expected binary not found at {}",
            binary.display()
        );
    }
}

pub(super) fn stage_seed_data(usb_root: &Path, target: &Path) {
    step("4. Seeding data from USB artifact");

    let usb_data = usb_root.join("artifact/data");
    let target_data = target.join("artifact/data");

    if !usb_data.exists() {
        println!("  No USB data to seed");
        return;
    }

    std::fs::create_dir_all(&target_data).ok();
    let mut seeded = 0u32;

    for entry in walkdir::WalkDir::new(&usb_data)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let rel = match entry.path().strip_prefix(&usb_data) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let dest = target_data.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest).ok();
        } else if !dest.exists() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            if std::fs::copy(entry.path(), &dest).is_ok() {
                seeded += 1;
            }
        }
    }

    for toml_name in ["scope.toml", "data.toml", "tolerances.toml"] {
        let src = usb_root.join(format!("artifact/{toml_name}"));
        let dst = target.join(format!("artifact/{toml_name}"));
        if src.exists() && !dst.exists() {
            std::fs::copy(&src, &dst).ok();
        }
    }

    let usb_expected = usb_root.join("validation/expected");
    let target_expected = target.join("validation/expected");
    if usb_expected.exists() {
        std::fs::create_dir_all(&target_expected).ok();
        for entry in walkdir::WalkDir::new(&usb_expected)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(&usb_expected)
                    .unwrap_or(entry.path());
                let dest = target_expected.join(rel);
                if !dest.exists() {
                    std::fs::copy(entry.path(), &dest).ok();
                    seeded += 1;
                }
            }
        }
    }

    println!("  Seeded {seeded} files from USB into cloned tree");
}

pub(super) fn stage_fetch(target: &Path) {
    step("5. Fetching upstream datasets");

    let litho = find_litho_binary(target);
    if let Some(bin) = &litho {
        println!("  Using: {}", bin.display());
        let status = Command::new(bin)
            .args(["fetch", "--all", "--artifact-root", "."])
            .current_dir(target)
            .status();
        match status {
            Ok(s) if s.success() => println!("  Fetch complete"),
            Ok(s) => eprintln!("  WARNING: fetch exited with {s}"),
            Err(e) => eprintln!("  WARNING: Could not run litho fetch: {e}"),
        }
    } else {
        println!("  No litho binary found — running cargo to fetch");
        let status = Command::new("cargo")
            .args(["run", "--release", "--bin", "litho", "--", "fetch", "--all"])
            .current_dir(target)
            .status();
        match status {
            Ok(s) if s.success() => println!("  Fetch complete"),
            _ => eprintln!("  WARNING: cargo run fetch failed"),
        }
    }

    detect_sra_toolkit();
}

pub(super) fn stage_validate(target: &Path) {
    step("6. Validating grown tree");

    let litho = find_litho_binary(target);
    let runner: ValidateRunner = if let Some(ref bin) = litho {
        let bin = bin.clone();
        let target = target.to_path_buf();
        Box::new(move |args: &[&str]| {
            Command::new(&bin)
                .args(args)
                .arg("--artifact-root")
                .arg(".")
                .current_dir(&target)
                .status()
                .ok()
        })
    } else {
        let target = target.to_path_buf();
        Box::new(move |args: &[&str]| {
            let mut cmd_args = vec!["run", "--release", "--bin", "litho", "--"];
            cmd_args.extend_from_slice(args);
            Command::new("cargo")
                .args(&cmd_args)
                .current_dir(&target)
                .status()
                .ok()
        })
    };

    let mut runner = runner;

    println!("  Running Tier 2 (Rust) validation...");
    if let Some(s) = runner(&["validate", "--max-tier", "2"]) {
        if s.success() {
            println!("  Tier 2: PASS");
        } else {
            eprintln!("  Tier 2: FAIL (exit {})", s.code().unwrap_or(-1));
        }
    }

    println!("  Running Tier 1 (Python) validation...");
    if let Some(s) = runner(&["validate", "--max-tier", "1"]) {
        if s.success() {
            println!("  Tier 1: PASS");
        } else {
            eprintln!("  Tier 1: FAIL (exit {})", s.code().unwrap_or(-1));
        }
    }
}
