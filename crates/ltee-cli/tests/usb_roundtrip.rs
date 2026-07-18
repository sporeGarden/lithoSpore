// SPDX-License-Identifier: AGPL-3.0-or-later

//! USB round-trip integration tests for lithoSpore.
//!
//! Tests the full assemble → verify → validate → symlink → spore pipeline
//! against a staged source repo, simulating USB-key deployment scenarios.

use std::process::Command;

fn litho_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_litho"))
}

/// Build a realistic "source repo" that `litho assemble` can stage from.
/// Populates scope.toml, data.toml, tolerances.toml, expected values,
/// data dirs, papers, figures, and docs.
fn build_source_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    let root = dir.path();

    let dirs = [
        "artifact/data/test_fitness",
        "artifact/usb-root",
        "validation/expected",
        "papers",
        "figures",
    ];
    for d in &dirs {
        std::fs::create_dir_all(root.join(d)).expect("create dir");
    }

    let scope = r#"
[guidestone]
name = "roundtrip-test"
version = "0.1.0"
target = "USB round-trip integration test"

[[spring]]
name = "testSpring"
modules = ["ltee-fitness"]
"#;
    std::fs::write(root.join("artifact/scope.toml"), scope).expect("write scope.toml");

    let data = r#"
[meta]
artifact = "roundtrip-test"

[[dataset]]
id = "test_fitness"
source_uri = ""
local_path = "artifact/data/test_fitness/"
module = "ltee-fitness"
blake3 = ""
"#;
    std::fs::write(root.join("artifact/data.toml"), data).expect("write data.toml");
    std::fs::write(
        root.join("artifact/tolerances.toml"),
        "[meta]\nartifact = \"roundtrip-test\"\n",
    )
    .expect("write tolerances.toml");

    let expected = serde_json::json!({
        "generations": [0.0, 500.0, 1000.0],
        "mean_fitness": [1.0, 1.1, 1.2],
        "model_fits": {
            "power_law": {"params": [0.004, 0.65], "r_squared": 0.999, "aic": -50.0, "bic": -49.0, "k": 2, "rss": 0.001},
            "hyperbolic": {"params": [0.0002, 0.00002], "r_squared": 0.998, "aic": -48.0, "bic": -47.0, "k": 2, "rss": 0.002},
            "logarithmic": {"params": [0.98, -6.7], "r_squared": 0.90, "aic": -10.0, "bic": -9.0, "k": 2, "rss": 2.5}
        }
    });
    std::fs::write(
        root.join("validation/expected/module1_fitness.json"),
        serde_json::to_string_pretty(&expected).expect("serialize expected"),
    )
    .expect("write expected");

    std::fs::write(
        root.join("artifact/data/test_fitness/fitness_data.csv"),
        "generation,mean_fitness\n0,1.0\n500,1.1\n1000,1.2\n",
    )
    .expect("write csv");

    std::fs::write(root.join("papers/registry.toml"), "[meta]\npapers = []\n")
        .expect("write registry");
    std::fs::write(root.join("papers/READING_ORDER.md"), "# Reading Order\n")
        .expect("write reading order");
    std::fs::write(root.join("GETTING_STARTED.md"), "# Getting Started\n")
        .expect("write getting started");
    std::fs::write(root.join("SCIENCE.md"), "# Science\n").expect("write science");

    for i in 1..=8 {
        std::fs::write(
            root.join(format!("figures/fig{i}.svg")),
            format!("<svg><text>Figure {i}</text></svg>"),
        )
        .expect("write svg");
    }

    dir
}

fn assemble_source(source: &tempfile::TempDir, target: &tempfile::TempDir) {
    let output = litho_bin()
        .args([
            "assemble",
            "--artifact-root",
            source.path().to_str().unwrap(),
            "--target",
            target.path().to_str().unwrap(),
            "--skip-build",
            "--skip-fetch",
            "--skip-python",
        ])
        .output()
        .expect("run litho assemble");

    assert!(
        output.status.success(),
        "assemble should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn assemble_then_verify() {
    let source = build_source_repo();
    let assembled = tempfile::tempdir().expect("create assembly target");

    assemble_source(&source, &assembled);

    let manifest = assembled.path().join("data_manifest.toml");
    assert!(
        manifest.exists(),
        "assemble should generate data_manifest.toml"
    );
    let manifest_content = std::fs::read_to_string(&manifest).expect("read manifest");
    assert!(
        manifest_content.contains("[[file]]"),
        "manifest should contain file entries"
    );

    let verify_output = litho_bin()
        .args([
            "verify",
            "--artifact-root",
            assembled.path().to_str().unwrap(),
        ])
        .output()
        .expect("run litho verify on assembled artifact");

    assert!(
        verify_output.status.success(),
        "verify should pass on freshly assembled artifact: {}",
        String::from_utf8_lossy(&verify_output.stderr)
    );
}

#[test]
fn assemble_then_validate() {
    let source = build_source_repo();
    let assembled = tempfile::tempdir().expect("create assembly target");

    assemble_source(&source, &assembled);

    let output = litho_bin()
        .args([
            "validate",
            "--artifact-root",
            assembled.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("run litho validate --json on assembled artifact");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value =
        serde_json::from_str(&stdout).expect("validate should produce valid JSON");

    assert_eq!(
        report["artifact"], "roundtrip-test",
        "scope identity should propagate through assembly"
    );
}

#[test]
fn symlinks_correct() {
    let source = build_source_repo();
    let assembled = tempfile::tempdir().expect("create assembly target");

    assemble_source(&source, &assembled);

    let expected = std::path::Path::new("bin/litho");
    for shim in ["validate", "verify", "refresh", "spore", "grow"] {
        let link = assembled.path().join(shim);
        let target =
            std::fs::read_link(&link).unwrap_or_else(|_| panic!("{shim} symlink should exist"));
        assert_eq!(
            target,
            expected,
            "{shim} should point to bin/litho, got {}",
            target.display()
        );
    }
}

#[test]
fn biomeos_spore_generated() {
    let source = build_source_repo();
    let assembled = tempfile::tempdir().expect("create assembly target");

    assemble_source(&source, &assembled);

    let spore_path = assembled.path().join(".biomeos-spore");
    assert!(spore_path.exists(), ".biomeos-spore should be generated");

    let content = std::fs::read_to_string(&spore_path).expect("read .biomeos-spore");
    let spore: serde_json::Value =
        serde_json::from_str(&content).expect(".biomeos-spore should be valid JSON");

    assert_eq!(spore["name"], "roundtrip-test");
    assert_eq!(spore["class"], "hypogeal-cotyledon");
    assert_eq!(spore["chassis"], "lithoSpore");
}

#[test]
fn verify_detects_post_assembly_drift() {
    let source = build_source_repo();
    let assembled = tempfile::tempdir().expect("create assembly target");

    assemble_source(&source, &assembled);

    let csv = assembled
        .path()
        .join("artifact/data/test_fitness/fitness_data.csv");
    if csv.exists() {
        std::fs::write(&csv, "TAMPERED DATA\n").expect("corrupt file");
    }

    let output = litho_bin()
        .args([
            "verify",
            "--artifact-root",
            assembled.path().to_str().unwrap(),
        ])
        .output()
        .expect("run litho verify after tampering");

    assert!(
        !output.status.success(),
        "verify should FAIL after post-assembly tampering"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("DRIFT"),
        "should report DRIFT on tampered file"
    );
}
