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

// ── Fault injection tests ──────────────────────────────────────────

fn artifact_with_manifest(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("artifact/data")).ok();
    std::fs::create_dir_all(root.join("validation/expected")).ok();

    let mut manifest = String::from("[meta]\nartifact = \"test\"\n\n");
    for (path, content) in files {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&full, content).expect("write test file");
        let hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        manifest.push_str(&format!("[[file]]\npath = \"{path}\"\nblake3 = \"{hash}\"\n\n"));
    }
    std::fs::write(root.join("data_manifest.toml"), manifest).ok();
    dir
}

#[test]
fn verify_detects_drifted_file() {
    let root = artifact_with_manifest(&[("artifact/data/test.json", "{\"valid\": true}")]);
    // Corrupt the file after hashing
    std::fs::write(root.path().join("artifact/data/test.json"), "{\"corrupted\": true}").ok();

    let output = litho_bin()
        .args(["verify", "--artifact-root", root.path().to_str().unwrap()])
        .output()
        .expect("run litho verify");

    assert!(!output.status.success(), "verify should fail on DRIFT");
    let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(combined.contains("DRIFT"), "output should mention DRIFT");
}

#[test]
fn verify_detects_missing_file() {
    let root = artifact_with_manifest(&[("artifact/data/ghost.json", "content")]);
    // Remove the file so it's in manifest but not on disk
    std::fs::remove_file(root.path().join("artifact/data/ghost.json")).ok();

    let output = litho_bin()
        .args(["verify", "--artifact-root", root.path().to_str().unwrap()])
        .output()
        .expect("run litho verify");

    assert!(!output.status.success(), "verify should fail on MISSING");
}

#[test]
fn verify_detects_corrupt_manifest() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::write(dir.path().join("data_manifest.toml"), "NOT VALID TOML {{{{").ok();

    let output = litho_bin()
        .args(["verify", "--artifact-root", dir.path().to_str().unwrap()])
        .output()
        .expect("run litho verify");

    assert!(!output.status.success(), "verify should fail on corrupt manifest");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Corrupt") || stderr.contains("ERROR"));
}

#[test]
fn verify_detects_empty_manifest() {
    let dir = tempfile::tempdir().expect("create temp dir");
    std::fs::write(dir.path().join("data_manifest.toml"), "[meta]\nartifact = \"test\"\n").ok();

    let output = litho_bin()
        .args(["verify", "--artifact-root", dir.path().to_str().unwrap()])
        .output()
        .expect("run litho verify");

    assert!(!output.status.success(), "verify should fail on manifest with no [[file]] entries");
}

#[test]
fn verify_passes_clean_manifest() {
    let root = artifact_with_manifest(&[
        ("artifact/data/a.json", "{\"ok\": 1}"),
        ("artifact/data/b.json", "{\"ok\": 2}"),
    ]);

    let output = litho_bin()
        .args(["verify", "--artifact-root", root.path().to_str().unwrap()])
        .output()
        .expect("run litho verify");

    assert!(output.status.success(), "verify should pass with clean data");
}

#[test]
fn livespore_survives_corruption() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("artifact/data")).ok();
    std::fs::create_dir_all(root.join("validation/expected")).ok();

    // Write corrupt liveSpore.json
    std::fs::write(root.join("artifact/liveSpore.json"), "THIS IS NOT JSON{{{{").ok();

    let output = litho_bin()
        .args(["validate", "--artifact-root", root.to_str().unwrap(), "--json"])
        .output()
        .expect("run litho validate");

    // Should still produce valid report JSON on stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _report: serde_json::Value = serde_json::from_str(&stdout)
        .expect("validate should produce valid JSON even with corrupt liveSpore");

    // Stderr should mention the backup
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("corrupt") || stderr.contains("backed up"),
        "should warn about corrupt liveSpore.json");
}

#[test]
fn self_test_detects_missing_components() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let output = litho_bin()
        .args(["self-test", "--artifact-root", dir.path().to_str().unwrap()])
        .output()
        .expect("run litho self-test");

    assert!(!output.status.success(), "self-test should fail on empty dir");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MISSING"), "should report MISSING files");
}

#[test]
fn deploy_report_produces_toml() {
    let root = temp_artifact_root();

    let output = litho_bin()
        .args(["deploy-report", "--artifact-root", root.path().to_str().unwrap(), "--pattern", "test"])
        .output()
        .expect("run litho deploy-report");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[meta]"), "deploy-report should produce TOML");
    assert!(stdout.contains("deployment_pattern = \"test\""));
}
