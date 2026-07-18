// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho deploy-test` — USB round-trip deployment simulation.
//!
//! Replaces `scripts/deploy-test-local.sh`. Assembles a staging artifact,
//! then runs the full verification cycle: BLAKE3 integrity, artifact
//! self-test, module validation, symlink shims, and liveSpore presence.

use std::path::Path;

pub fn run(root: &str) {
    let tmpdir = std::env::temp_dir().join("litho-deploy-test");
    let target = tmpdir.to_string_lossy();
    let _ = std::fs::remove_dir_all(&tmpdir);

    println!("litho deploy-test — USB round-trip simulation");
    println!("  Source: {root}");
    println!("  Stage:  {}", tmpdir.display());
    println!();

    // Step 1: Assemble into tmp
    println!("  [1/6] Assembling artifact...");
    crate::assemble::run(&crate::assemble::AssembleOptions {
        root,
        target: &target,
        skip: crate::assemble::AssembleSkipFlags {
            python: true,
            fetch: false,
            build: true,
        },
        dry_run: false,
    });

    // Step 2: Verify BLAKE3 manifest integrity
    println!("  [2/6] Verifying data integrity (BLAKE3)...");
    let verify_ok = crate::verify::run_check(tmpdir.to_str().unwrap_or("."));
    println!("    Integrity: {}", if verify_ok { "PASS" } else { "FAIL" });

    // Step 3: Artifact self-test (structural completeness)
    println!("  [3/6] Running self-test...");
    let selftest = crate::ops::run_self_test(&tmpdir);
    let selftest_ok = selftest.all_passed();
    println!(
        "    Self-test: {}/{} checks ({})",
        selftest.passed,
        selftest.total,
        if selftest_ok { "PASS" } else { "FAIL" }
    );

    // Step 4: Run module validation
    println!("  [4/6] Running validation...");
    let validate_ok = run_validation_quiet(&tmpdir);
    println!(
        "    Validation: {}",
        if validate_ok { "PASS" } else { "FAIL" }
    );

    // Step 5: Verify symlink shims point at bin/litho
    println!("  [5/6] Checking symlink shims...");
    let shims_ok = check_symlink_shims(&tmpdir);
    println!("    Shims: {}", if shims_ok { "PASS" } else { "FAIL" });

    // Step 6: Verify liveSpore was created/updated
    println!("  [6/6] Checking liveSpore...");
    let livespore = tmpdir.join("liveSpore.json");
    let spore_ok = livespore.exists();
    println!(
        "    liveSpore:  {}",
        if spore_ok {
            "PRESENT"
        } else {
            "ABSENT (non-fatal)"
        }
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmpdir);

    println!();
    let all_ok = verify_ok && selftest_ok && validate_ok && shims_ok;
    if all_ok {
        println!("  Deploy test: PASS (6/6 steps)");
    } else {
        println!("  Deploy test: FAIL");
        if !verify_ok {
            println!("    BLAKE3 integrity check failed");
        }
        if !selftest_ok {
            println!(
                "    Artifact self-test incomplete ({}/{})",
                selftest.passed, selftest.total
            );
        }
        if !validate_ok {
            println!("    Module validation failed");
        }
        if !shims_ok {
            println!("    Symlink shims missing or broken");
        }
        std::process::exit(1);
    }
}

fn check_symlink_shims(dir: &Path) -> bool {
    let expected_target = Path::new("bin/litho");
    let mut all_ok = true;

    for shim in ["validate", "verify", "refresh", "spore", "grow"] {
        let link = dir.join(shim);
        match std::fs::read_link(&link) {
            Ok(target) if target == expected_target => {}
            Ok(target) => {
                eprintln!(
                    "    WARN: {shim} → {} (expected bin/litho)",
                    target.display()
                );
                all_ok = false;
            }
            Err(_) => {
                eprintln!("    WARN: {shim} symlink missing");
                all_ok = false;
            }
        }
    }

    all_ok
}

fn run_validation_quiet(dir: &Path) -> bool {
    let root_path = dir;

    let modules = crate::registry::load_module_table(root_path);
    let mut any_ran = false;
    let mut all_pass = true;

    for entry in &modules {
        let data_path = root_path.join(&entry.data_dir);
        let expected_path = root_path.join(&entry.expected);
        if data_path.exists() && expected_path.exists() {
            let result = crate::registry::dispatch_module(
                &entry.binary,
                data_path.to_str().unwrap_or(&entry.data_dir),
                expected_path.to_str().unwrap_or(&entry.expected),
                2,
            );
            any_ran = true;
            if result.status != litho_core::ValidationStatus::Pass {
                all_pass = false;
            }
        }
    }

    any_ran && all_pass
}
