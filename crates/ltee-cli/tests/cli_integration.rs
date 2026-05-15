// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the litho CLI binary.
//!
//! These tests invoke the compiled binary with various subcommands and
//! verify exit codes and output structure against a temporary artifact root.

use std::process::Command;

fn litho_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_litho"))
}

fn temp_artifact_root() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::create_dir_all(dir.path().join("artifact/data")).ok();
    std::fs::create_dir_all(dir.path().join("validation/expected")).ok();
    dir
}

#[test]
fn help_exits_zero() {
    let output = litho_bin().arg("--help").output().expect("run litho --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lithoSpore"));
}

#[test]
fn version_exits_zero() {
    let output = litho_bin().arg("--version").output().expect("run litho --version");
    assert!(output.status.success());
}

#[test]
fn status_exits_zero() {
    let root = temp_artifact_root();
    let output = litho_bin()
        .args(["status", "--artifact-root", root.path().to_str().unwrap()])
        .output()
        .expect("run litho status");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Modules: 7"));
}

#[test]
fn validate_json_exits_with_report() {
    let root = temp_artifact_root();
    let output = litho_bin()
        .args(["validate", "--artifact-root", root.path().to_str().unwrap(), "--json"])
        .output()
        .expect("run litho validate --json");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should produce valid JSON even with no data
    let report: serde_json::Value = serde_json::from_str(&stdout)
        .expect("validate output is valid JSON");
    assert!(report.get("modules").is_some());
    assert!(report.get("version").is_some());
}

#[test]
fn verify_json_exits_cleanly() {
    let root = temp_artifact_root();
    let output = litho_bin()
        .args(["verify", "--artifact-root", root.path().to_str().unwrap(), "--json"])
        .output()
        .expect("run litho verify --json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .expect("verify output is valid JSON");
    assert!(result.get("online").is_some());
    assert!(result.get("summary").is_some());
}

#[test]
fn visualize_json_exits_cleanly() {
    let root = temp_artifact_root();
    let output = litho_bin()
        .args(["visualize", "--artifact-root", root.path().to_str().unwrap(), "--format", "json"])
        .output()
        .expect("run litho visualize --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let dashboard: serde_json::Value = serde_json::from_str(&stdout)
        .expect("visualize output is valid JSON");
    assert_eq!(dashboard["session_id"], "lithoSpore-dashboard");
}

#[test]
fn spore_handles_missing_livespore() {
    let root = temp_artifact_root();
    let output = litho_bin()
        .args(["spore", "--artifact-root", root.path().to_str().unwrap()])
        .output()
        .expect("run litho spore");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No liveSpore.json") || stdout.contains("validation runs"));
}

#[test]
fn unknown_subcommand_fails() {
    let output = litho_bin()
        .arg("nonexistent")
        .output()
        .expect("run litho nonexistent");
    assert!(!output.status.success());
}
