// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn step(msg: &str) {
    println!();
    println!("=== {msg} ===");
}

pub(super) fn pick_git_url(ssh: &str, https: &str) -> String {
    if !ssh.is_empty()
        && which("ssh")
        && Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=5",
                "-T",
                "git@github.com",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.code() == Some(1))
            .unwrap_or(false)
    {
        return ssh.to_string();
    }
    if !https.is_empty() {
        return https.to_string();
    }
    ssh.to_string()
}

pub(super) fn which(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub(super) fn run_cmd(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd).args(args).status();
    match status {
        Ok(s) if !s.success() => {
            eprintln!("  ERROR: `{cmd} {}` exited with {s}", args.join(" "));
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  ERROR: Could not run `{cmd}`: {e}");
            std::process::exit(1);
        }
        _ => {}
    }
}

pub(super) fn run_cmd_in(cmd: &str, args: &[&str], dir: &Path) {
    let status = Command::new(cmd).args(args).current_dir(dir).status();
    match status {
        Ok(s) if !s.success() => {
            eprintln!(
                "  ERROR: `{cmd} {}` in {} exited with {s}",
                args.join(" "),
                dir.display()
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("  ERROR: Could not run `{cmd}` in {}: {e}", dir.display());
            std::process::exit(1);
        }
        _ => {}
    }
}

pub(super) fn find_litho_binary(target: &Path) -> Option<PathBuf> {
    [
        target.join("target/x86_64-unknown-linux-musl/release/litho"),
        target.join("target/release/litho"),
        target.join("bin/litho"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

pub(super) fn detect_sra_toolkit() {
    if which("prefetch") && which("fastq-dump") {
        println!("  SRA toolkit detected — genomic data fetch available");
        println!("  Use `prefetch PRJNA*` + `fastq-dump` for full NCBI datasets");
    }
}
