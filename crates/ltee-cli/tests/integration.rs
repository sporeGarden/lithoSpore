// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for pseudoSpore lifecycle CLI subcommands.

use std::fs;
use std::process::Command;

fn litho_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_litho"))
}

const SUBCOMMANDS: &[&str] = &[
    "validate",
    "parity",
    "refresh",
    "status",
    "spore",
    "verify",
    "visualize",
    "self-test",
    "tier",
    "assemble",
    "chaos-test",
    "deploy-test",
    "fetch",
    "deploy-report",
    "grow",
    "ingest-pseudospore",
    "emit-pseudospore",
    "audit",
    "promote",
    "translate-config",
];

#[test]
fn all_subcommands_help_exits_zero() {
    for sub in SUBCOMMANDS {
        let output = litho_bin()
            .args([sub, "--help"])
            .output()
            .unwrap_or_else(|e| panic!("run litho {sub} --help: {e}"));
        assert!(
            output.status.success(),
            "litho {sub} --help failed: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn emit_pseudospore_requires_name_and_version() {
    let output = litho_bin()
        .args(["emit-pseudospore", "--output", "/tmp"])
        .output()
        .expect("run emit-pseudospore without required args");
    assert!(
        !output.status.success(),
        "emit-pseudospore should fail without --name and --version"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("required") || combined.contains("error"),
        "expected clap required-arg message: {combined}"
    );
}

#[test]
fn emit_pseudospore_help_lists_profile_flag() {
    let output = litho_bin()
        .args(["emit-pseudospore", "--help"])
        .output()
        .expect("emit-pseudospore --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("profile") || stdout.contains("domain-profile"),
        "help should document profile flag"
    );
}

#[test]
fn ingest_pseudospore_requires_path_arg() {
    let output = litho_bin()
        .arg("ingest-pseudospore")
        .output()
        .expect("run ingest-pseudospore without path");
    assert!(
        !output.status.success(),
        "ingest-pseudospore should fail without path positional"
    );
}

#[test]
fn ingest_pseudospore_help_documents_verify_flag() {
    let output = litho_bin()
        .args(["ingest-pseudospore", "--help"])
        .output()
        .expect("ingest-pseudospore --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("verify"), "help should document --verify");
}

#[test]
fn fetch_pseudospore_help_documents_url_and_ingest_flags() {
    let output = litho_bin()
        .args(["fetch-pseudospore", "--help"])
        .output()
        .expect("fetch-pseudospore --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--url"), "help should document --url");
    assert!(stdout.contains("--ingest"), "help should document --ingest");
    assert!(
        stdout.contains("tarball"),
        "help should mention tarball format"
    );
}

#[test]
fn audit_mock_spore_runs_core_checks() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join("data/mod-a")).expect("data dir");
    fs::create_dir_all(root.join("outputs/mod-a")).expect("outputs dir");
    fs::create_dir_all(root.join("configs/mod-a")).expect("configs dir");
    fs::create_dir_all(root.join("receipts")).expect("receipts dir");
    fs::write(
        root.join("scope.toml"),
        b"[artifact]\nname = \"mock\"\nversion = \"0.1.0\"\n",
    )
    .expect("scope");

    let content_hash = blake3::hash(b"payload").to_hex().to_string();
    fs::write(root.join("outputs/mod-a/result.dat"), b"payload").expect("output");
    fs::write(
        root.join("receipts/checksums.blake3"),
        format!("{content_hash}  outputs/mod-a/result.dat\n"),
    )
    .expect("checksums");

    let output = litho_bin()
        .args(["audit", "--path", root.to_str().unwrap(), "--json"])
        .output()
        .expect("run audit");

    assert!(
        output.status.success(),
        "audit should exit 0 for clean mock spore"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_start = stdout
        .find('{')
        .expect("audit --json should emit JSON report");
    let report: serde_json::Value =
        serde_json::from_str(&stdout[json_start..]).expect("parse audit JSON");
    assert_eq!(report["artifact"], "mock");
    assert_eq!(
        report["status"], "PASS",
        "valid checksums should pass audit"
    );
}

#[test]
fn audit_missing_path_fails() {
    let output = litho_bin()
        .args(["audit", "--path", "/nonexistent/pseudospore/path"])
        .output()
        .expect("run audit on missing path");
    assert!(
        !output.status.success(),
        "audit should fail for missing path"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("ERROR"),
        "stderr should report missing path: {stderr}"
    );
}
