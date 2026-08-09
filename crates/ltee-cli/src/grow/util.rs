// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn step(msg: &str) {
    println!();
    println!("=== {msg} ===");
}

pub(super) fn pick_git_url(ssh: &str, https: &str) -> String {
    if !ssh.is_empty() && which("ssh") && ssh_auth_works(ssh) {
        return ssh.to_string();
    }
    if !https.is_empty() {
        return https.to_string();
    }
    ssh.to_string()
}

fn ssh_auth_works(url: &str) -> bool {
    let host = url
        .strip_prefix("git@")
        .and_then(|s| s.split(':').next())
        .unwrap_or("git.primals.eco");

    Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            "-T",
            &format!("git@{host}"),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.code() == Some(1))
}

pub(super) fn which(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub(super) fn run_cmd(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("could not run `{cmd}`: {e}"))?;
    if !status.success() {
        return Err(format!("`{cmd} {}` exited with {status}", args.join(" ")));
    }
    Ok(())
}

pub(super) fn run_cmd_in(cmd: &str, args: &[&str], dir: &Path) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| format!("could not run `{cmd}` in {}: {e}", dir.display()))?;
    if !status.success() {
        return Err(format!(
            "`{cmd} {}` in {} exited with {status}",
            args.join(" "),
            dir.display()
        ));
    }
    Ok(())
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
