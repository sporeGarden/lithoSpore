// SPDX-License-Identifier: AGPL-3.0-or-later

//! Integration tests for the pseudoSpore lifecycle pipeline.
//!
//! Exercises the full promotion flow: emit → init-validation → populate →
//! ingest → promote → spore-status.

use std::process::Command;

fn litho_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_litho"))
}

/// Build a minimal spring-style scope.toml in a temp directory.
fn build_spring_scope(dir: &std::path::Path) {
    let scope = r#"[artifact]
name = "testSpring-Integration"
version = "1.0.0"
type = "pseudoSpore"
date = "2026-07-21"
origin = "ecoPrimals/springs/testSpring"
license = "AGPL-3.0-or-later"

[evolution]
tier_0 = "Control"
tier_1 = "Python"
tier_2 = "Rust"
tier_3 = "NUCLEUS"

[[modules]]
name = "module_alpha"
entity_group = "alpha"
computation = ["compute_a"]

[[modules]]
name = "module_beta"
entity_group = "beta"
computation = ["compute_b", "compute_c"]

[[modules]]
name = "module_gamma"
entity_group = "gamma"
computation = ["compute_d"]
"#;
    std::fs::write(dir.join("scope.toml"), scope).expect("write scope");
}

#[test]
fn init_validation_generates_stub_from_scope() {
    let dir = tempfile::tempdir().expect("create temp dir");
    build_spring_scope(dir.path());

    let output = litho_bin()
        .args(["init-validation", dir.path().to_str().expect("path")])
        .output()
        .expect("run init-validation");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "init-validation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("module_alpha"));
    assert!(stdout.contains("module_beta"));
    assert!(stdout.contains("module_gamma"));
    assert!(stdout.contains("3 (all PENDING)"));

    let validation =
        std::fs::read_to_string(dir.path().join("validation.json")).expect("read validation.json");
    let doc: serde_json::Value = serde_json::from_str(&validation).expect("parse json");
    assert_eq!(doc["modules"].as_array().expect("modules").len(), 3);
    assert_eq!(doc["summary"]["modules_total"], 3);
    assert_eq!(doc["summary"]["modules_pass"], 0);
    assert_eq!(doc["summary"]["modules_in_flight"], 3);
}

#[test]
fn populate_validation_updates_module_status() {
    let dir = tempfile::tempdir().expect("create temp dir");
    build_spring_scope(dir.path());

    let init_out = litho_bin()
        .args(["init-validation", dir.path().to_str().expect("path")])
        .output()
        .expect("init");
    assert!(init_out.status.success());

    let pop_out = litho_bin()
        .args([
            "populate-validation",
            dir.path().to_str().expect("path"),
            "--module",
            "module_alpha=PASS",
            "--module",
            "module_beta=PASS",
        ])
        .output()
        .expect("populate");

    let stdout = String::from_utf8_lossy(&pop_out.stdout);
    assert!(
        pop_out.status.success(),
        "populate-validation failed: {}",
        String::from_utf8_lossy(&pop_out.stderr)
    );
    assert!(stdout.contains("2 pass"));

    let validation =
        std::fs::read_to_string(dir.path().join("validation.json")).expect("read validation.json");
    let doc: serde_json::Value = serde_json::from_str(&validation).expect("parse json");
    let modules = doc["modules"].as_array().expect("modules");

    let alpha = modules
        .iter()
        .find(|m| m["name"] == "module_alpha")
        .expect("find alpha");
    assert_eq!(alpha["status"], "PASS");

    let gamma = modules
        .iter()
        .find(|m| m["name"] == "module_gamma")
        .expect("find gamma");
    assert_eq!(gamma["status"], "PENDING");
}

#[test]
fn full_lifecycle_emit_init_populate_promote() {
    let root = tempfile::tempdir().expect("create root");
    let spring_dir = tempfile::tempdir().expect("create spring");

    build_spring_scope(spring_dir.path());
    std::fs::create_dir_all(spring_dir.path().join("data")).ok();

    std::fs::create_dir_all(root.path().join("pseudospores")).expect("mkdir pseudospores");

    // Step 1: emit-pseudospore
    let emit = litho_bin()
        .args([
            "emit-pseudospore",
            "--name",
            "testSpring-Integration",
            "--version",
            "1.0.0",
            "--origin",
            "ecoPrimals/springs/testSpring",
            "--output",
            root.path().to_str().expect("root"),
        ])
        .output()
        .expect("emit");

    let emit_stdout = String::from_utf8_lossy(&emit.stdout);
    assert!(
        emit.status.success(),
        "emit failed: {}\n{}",
        String::from_utf8_lossy(&emit.stderr),
        emit_stdout
    );

    let spore_dir = root
        .path()
        .join("pseudoSpore_testSpring-Integration_v1.0.0");
    assert!(spore_dir.exists(), "spore directory should exist");
    assert!(
        spore_dir.join("validation.json").exists(),
        "validation.json should exist"
    );

    // Step 1b: Copy spring scope.toml into emitted spore
    build_spring_scope(&spore_dir);

    // Step 2: init-validation (reads scope.toml, regenerates validation.json)
    let init = litho_bin()
        .args([
            "init-validation",
            spore_dir.to_str().expect("spore"),
            "--force",
        ])
        .output()
        .expect("init");

    let init_stdout = String::from_utf8_lossy(&init.stdout);
    assert!(
        init.status.success(),
        "init-validation failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(
        init_stdout.contains("3 (all PENDING)"),
        "should find 3 modules, got: {init_stdout}"
    );
    assert!(
        spore_dir.join("validation.json.bak").exists(),
        "backup should exist"
    );

    // Step 3: populate all modules as PASS
    let populate = litho_bin()
        .args([
            "populate-validation",
            spore_dir.to_str().expect("spore"),
            "--module",
            "module_alpha=PASS",
            "--module",
            "module_beta=PASS",
            "--module",
            "module_gamma=PASS",
        ])
        .output()
        .expect("populate");

    assert!(
        populate.status.success(),
        "populate failed: {}",
        String::from_utf8_lossy(&populate.stderr)
    );
    let pop_stdout = String::from_utf8_lossy(&populate.stdout);
    assert!(pop_stdout.contains("COMPLETE"));

    // Step 4: ingest into registry
    let ingest = litho_bin()
        .args([
            "ingest-pseudospore",
            spore_dir.to_str().expect("spore"),
            "--artifact-root",
            root.path().to_str().expect("root"),
        ])
        .output()
        .expect("ingest");

    assert!(
        ingest.status.success(),
        "ingest failed: {}\n{}",
        String::from_utf8_lossy(&ingest.stderr),
        String::from_utf8_lossy(&ingest.stdout)
    );

    // Step 5: promote-spore
    let promote = litho_bin()
        .args([
            "promote-spore",
            spore_dir.to_str().expect("spore"),
            "--artifact-root",
            root.path().to_str().expect("root"),
        ])
        .output()
        .expect("promote");

    let promote_stdout = String::from_utf8_lossy(&promote.stdout);
    assert!(
        promote.status.success(),
        "promote failed: {}\n{}",
        String::from_utf8_lossy(&promote.stderr),
        promote_stdout
    );
    assert!(promote_stdout.contains("COMPLETE"));

    // Verify final registry state
    let registry = std::fs::read_to_string(root.path().join("pseudospores/registry.toml"))
        .expect("read registry");
    assert!(
        registry.contains("COMPLETE"),
        "registry should show COMPLETE"
    );

    // Step 6: spore-status should show the promoted spore
    let status = litho_bin()
        .args([
            "spore-status",
            "--artifact-root",
            root.path().to_str().expect("root"),
            "--json",
        ])
        .output()
        .expect("status");

    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert!(status.status.success());
    let report: serde_json::Value =
        serde_json::from_str(&status_stdout).expect("parse status json");
    assert_eq!(report["complete"], 1);
    assert_eq!(report["pending"], 0);
}
