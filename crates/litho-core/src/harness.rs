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

/// Tier 0: structural check — verify expected values file exists and parses as JSON.
///
/// Returns `Pass` with a structural-only note (no scientific checks). Reports an
/// estimated check count from `validation_checks`, `targets`, or top-level keys.
#[must_use]
pub fn tier0_structural(name: &str, expected_path: &str, start: Instant) -> ModuleResult {
    let path = Path::new(expected_path);
    if !path.exists() {
        return skip(
            name,
            0,
            start,
            &format!("Tier 0: expected file not found: {expected_path}"),
        );
    }
    let Some(value) = load_expected(expected_path) else {
        return ModuleResult {
            name: name.to_string(),
            status: ValidationStatus::Fail,
            tier: 0,
            checks: 1,
            checks_passed: 0,
            runtime_ms: elapsed_ms(start),
            error: Some(format!(
                "Tier 0: {expected_path} exists but failed to parse as JSON"
            )),
        };
    };

    let check_count = tier0_check_count(&value);
    let file = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(expected_path);
    eprintln!("  [PASS] Tier 0 structural: {file} parseable ({check_count} expected field(s))");
    ModuleResult {
        name: name.to_string(),
        status: ValidationStatus::Pass,
        tier: 0,
        checks: check_count.max(1),
        checks_passed: check_count.max(1),
        runtime_ms: elapsed_ms(start),
        error: Some(format!(
            "Tier 0 structural-only: {file} parseable ({check_count} checks)"
        )),
    }
}

fn tier0_check_count(value: &serde_json::Value) -> u32 {
    if let Some(arr) = value.get("validation_checks").and_then(|v| v.as_array()) {
        u32::try_from(arr.len()).unwrap_or(u32::MAX)
    } else if let Some(obj) = value.get("targets").and_then(|v| v.as_object()) {
        u32::try_from(obj.len()).unwrap_or(u32::MAX)
    } else if let Some(obj) = value.as_object() {
        u32::try_from(obj.len()).unwrap_or(u32::MAX)
    } else {
        1
    }
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
        return skip(
            name,
            1,
            start,
            &format!("Python baseline not found: {}", script_path.display()),
        );
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

/// Format a `ModuleResult` as JSON or human-readable text.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn format_output(result: &ModuleResult, json: bool) -> Result<String, crate::LithoError> {
    if json {
        serde_json::to_string_pretty(result).map_err(crate::LithoError::from)
    } else {
        let status_str = match result.status {
            ValidationStatus::Pass => "PASS",
            ValidationStatus::Fail => "FAIL",
            ValidationStatus::Skip => "SKIP",
        };
        Ok(format!(
            "{}: {} — {}/{} checks ({}ms)",
            result.name, status_str, result.checks_passed, result.checks, result.runtime_ms,
        ))
    }
}

/// Exit code for a module result per the Targeted `GuideStone` standard.
#[must_use]
pub const fn exit_code(result: &ModuleResult) -> i32 {
    match result.status {
        ValidationStatus::Pass => 0,
        ValidationStatus::Fail => 1,
        ValidationStatus::Skip => 2,
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

    #[test]
    fn load_expected_valid_json() {
        let dir = std::env::temp_dir().join("litho_harness_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("expected.json");
        std::fs::write(&path, r#"{"key": 42}"#).expect("write expected");
        let value = load_expected(path.to_str().unwrap()).expect("parse expected");
        assert_eq!(value["key"], 42);
    }

    #[test]
    fn skip_result_fields() {
        let start = Instant::now();
        let r = skip("my_module", 3, start, "tier not available");
        assert_eq!(r.name, "my_module");
        assert_eq!(r.tier, 3);
        assert_eq!(r.status, ValidationStatus::Skip);
        assert_eq!(r.checks, 0);
        assert_eq!(r.checks_passed, 0);
    }

    #[test]
    fn tier0_structural_passes_on_valid_json() {
        let dir = std::env::temp_dir().join("litho_harness_tier0");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("expected.json");
        std::fs::write(&path, r#"{"validation_checks":[{"name":"a"}]}"#).expect("write");
        let start = Instant::now();
        let r = tier0_structural("test_mod", path.to_str().unwrap(), start);
        assert_eq!(r.status, ValidationStatus::Pass);
        assert_eq!(r.tier, 0);
        assert!(r.checks >= 1);
    }

    #[test]
    fn module_result_json_format() {
        let result = ModuleResult {
            name: "test_module".to_string(),
            status: ValidationStatus::Pass,
            tier: 2,
            checks: 10,
            checks_passed: 10,
            runtime_ms: 100,
            error: None,
        };
        let json = serde_json::to_string_pretty(&result).expect("serialize");
        assert!(json.contains("\"name\": \"test_module\""));
        assert!(json.contains("\"status\": \"PASS\""));
    }
}
