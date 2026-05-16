// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared validation harness utilities extracted from module binaries.
//!
//! Eliminates duplication of `skip_result`, `load_expected`, Python dispatch,
//! and JSON output formatting across all lithoSpore module crates.

use crate::{ModuleResult, ValidationStatus};
use std::path::Path;
use std::time::Instant;

/// Construct a `Skip` result with timing — replaces the duplicated
/// `skip_result` function that existed in every module binary.
#[must_use]
pub fn skip(name: &str, tier: u8, start: Instant, reason: &str) -> ModuleResult {
    ModuleResult {
        name: name.to_string(),
        status: ValidationStatus::Skip,
        tier,
        checks: 0,
        checks_passed: 0,
        runtime_ms: elapsed_ms(start),
        error: Some(reason.to_string()),
    }
}

fn elapsed_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Load and parse a JSON expected-values file.
///
/// Returns `None` on missing file or parse failure — callers produce
/// a `Skip` result rather than panicking.
#[must_use]
pub fn load_expected(path: &str) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Dispatch a Python baseline script (Tier 1) and parse pass/fail from stdout.
///
/// Protocol: the Python script prints `[PASS]` / `[FAIL]` markers.
/// Exit code 0 with no `[FAIL]` = pass, exit code 2 = skip, else fail.
#[must_use]
pub fn dispatch_python(name: &str, script_path: &Path, working_dir: &Path) -> ModuleResult {
    let start = Instant::now();

    if !script_path.exists() {
        return skip(name, 1, start, &format!(
            "Python baseline not found: {}", script_path.display()
        ));
    }

    let output = std::process::Command::new("python3")
        .arg(script_path)
        .current_dir(working_dir)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            eprintln!("{stdout}");
            if !stderr.is_empty() {
                eprintln!("{stderr}");
            }

            let passed = u32::try_from(stdout.matches("[PASS]").count()).unwrap_or(u32::MAX);
            let failed = u32::try_from(stdout.matches("[FAIL]").count()).unwrap_or(u32::MAX);

            let status = if out.status.code() == Some(0) && failed == 0 {
                ValidationStatus::Pass
            } else if out.status.code() == Some(2) {
                ValidationStatus::Skip
            } else {
                ValidationStatus::Fail
            };

            ModuleResult {
                name: name.to_string(),
                status,
                tier: 1,
                checks: passed + failed,
                checks_passed: passed,
                runtime_ms: elapsed_ms(start),
                error: if failed > 0 {
                    Some(format!("{failed} check(s) failed"))
                } else {
                    None
                },
            }
        }
        Err(e) => skip(name, 1, start, &format!("Python dispatch failed: {e}")),
    }
}

/// Print a `ModuleResult` as JSON or human-readable text, then exit
/// with the appropriate code per the Targeted `GuideStone` standard.
pub fn output_and_exit(result: &ModuleResult, json: bool) -> ! {
    if json {
        match serde_json::to_string_pretty(result) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("Error serializing result: {e}");
                std::process::exit(2);
            }
        }
    } else {
        let status_str = match result.status {
            ValidationStatus::Pass => "PASS",
            ValidationStatus::Fail => "FAIL",
            ValidationStatus::Skip => "SKIP",
        };
        println!(
            "{}: {} — {}/{} checks ({}ms)",
            result.name, status_str, result.checks_passed, result.checks, result.runtime_ms,
        );
    }

    match result.status {
        ValidationStatus::Fail => std::process::exit(1),
        ValidationStatus::Skip => std::process::exit(2),
        ValidationStatus::Pass => std::process::exit(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_captures_timing() {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let r = skip("test", 2, start, "reason");
        assert_eq!(r.status, ValidationStatus::Skip);
        assert!(r.runtime_ms >= 1);
        assert_eq!(r.error.as_deref(), Some("reason"));
    }

    #[test]
    fn load_expected_missing_file() {
        assert!(load_expected("/nonexistent/path.json").is_none());
    }
}
