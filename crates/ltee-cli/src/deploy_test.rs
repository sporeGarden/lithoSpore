// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho deploy-test` — local deployment simulation.
//!
//! Replaces `scripts/deploy-test-local.sh`. Assembles a staging artifact,
//! runs full validation, and reports success/failure.

use std::path::Path;

pub fn run(root: &str) {
    let tmpdir = std::env::temp_dir().join("litho-deploy-test");
    let _ = std::fs::remove_dir_all(&tmpdir);

    println!("litho deploy-test — local deployment simulation");
    println!("  Source: {root}");
    println!("  Stage:  {}", tmpdir.display());
    println!();

    // Step 1: Assemble into tmp
    println!("  [1/4] Assembling artifact...");
    crate::assemble::run(
        root,
        tmpdir.to_str().unwrap_or("/tmp/litho-deploy-test"),
        true,   // skip_python
        false,  // skip_fetch (data is needed)
        true,   // skip_build (binaries already built)
        false,  // dry_run
    );

    // Step 2: Verify manifest integrity
    println!("  [2/4] Verifying data integrity...");
    let verify_ok = crate::verify::run_check(tmpdir.to_str().unwrap_or("."));
    println!("    Integrity: {}", if verify_ok { "PASS" } else { "FAIL" });

    // Step 3: Run validation
    println!("  [3/4] Running validation...");
    let validate_ok = run_validation_quiet(&tmpdir);
    println!("    Validation: {}", if validate_ok { "PASS" } else { "FAIL" });

    // Step 4: Verify liveSpore was created/updated
    println!("  [4/4] Checking liveSpore...");
    let livespore = tmpdir.join("liveSpore.json");
    let spore_ok = livespore.exists();
    // Run validation should have created it, but if not, it's non-fatal
    println!("    liveSpore:  {}", if spore_ok { "PRESENT" } else { "ABSENT (non-fatal)" });

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmpdir);

    println!();
    let all_ok = verify_ok && validate_ok;
    if all_ok {
        println!("  Deploy test: PASS");
    } else {
        println!("  Deploy test: FAIL");
        std::process::exit(1);
    }
}

fn run_validation_quiet(dir: &Path) -> bool {
    let root = dir.to_str().unwrap_or(".");
    let root_path = Path::new(root);

    let mut any_ran = false;
    let mut all_pass = true;

    for (_name, binary, data_dir, expected) in crate::validate::LTEE_MODULES {
        let data_path = root_path.join(data_dir);
        let expected_path = root_path.join(expected);
        if data_path.exists() && expected_path.exists() {
            let dispatch: &[(&str, fn(&str, &str, u8) -> litho_core::ModuleResult)] = &[
                ("ltee-fitness", ltee_fitness::run_validation),
                ("ltee-mutations", ltee_mutations::run_validation),
                ("ltee-alleles", ltee_alleles::run_validation),
                ("ltee-citrate", ltee_citrate::run_validation),
                ("ltee-biobricks", ltee_biobricks::run_validation),
                ("ltee-breseq", ltee_breseq::run_validation),
                ("ltee-anderson", ltee_anderson::run_validation),
            ];
            if let Some((_, func)) = dispatch.iter().find(|(n, _)| n == binary) {
                let result = func(
                    data_path.to_str().unwrap_or(data_dir),
                    expected_path.to_str().unwrap_or(expected),
                    2,
                );
                any_ran = true;
                if result.status != litho_core::ValidationStatus::Pass {
                    all_pass = false;
                }
            }
        }
    }

    any_ran && all_pass
}
