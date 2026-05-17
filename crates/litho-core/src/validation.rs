// SPDX-License-Identifier: AGPL-3.0-or-later

//! Validation harness types: structured JSON output for module results.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ValidationStatus {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResult {
    pub name: String,
    pub status: ValidationStatus,
    pub tier: u8,
    pub checks: u32,
    pub checks_passed: u32,
    pub runtime_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetCoverage {
    pub id: String,
    pub module: String,
    pub claim: String,
    pub status: String,
}

/// Tier 3 provenance session — recorded when NUCLEUS primals are available
/// and the provenance trio (rhizoCrypt + loamSpine + sweetGrass) successfully
/// anchors the validation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier3Session {
    pub dag_session_id: String,
    pub dag_merkle_root: String,
    pub spine_id: String,
    pub braid_id: String,
    pub primals_reached: Vec<String>,
}

/// Per-module parity result — records whether Tier 1 and Tier 2 agree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityResult {
    pub module: String,
    pub tier1_status: ValidationStatus,
    pub tier2_status: ValidationStatus,
    pub tier1_checks: u32,
    pub tier2_checks: u32,
    pub tier1_passed: u32,
    pub tier2_passed: u32,
    pub parity: ParityStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParityStatus {
    Match,
    Divergence,
    Skipped,
}

/// Cross-tier parity report — verifies math stability between Tier 1 and Tier 2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityReport {
    pub artifact: String,
    pub version: String,
    pub modules: Vec<ParityResult>,
    pub parity_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub artifact: String,
    pub version: String,
    pub tier_reached: u8,
    pub modules: Vec<ModuleResult>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub target_coverage: Vec<TargetCoverage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier3: Option<Tier3Session>,
}

impl ValidationReport {
    #[must_use]
    pub fn new(artifact: &str, version: &str) -> Self {
        Self {
            artifact: artifact.to_string(),
            version: version.to_string(),
            tier_reached: 0,
            modules: Vec::new(),
            target_coverage: Vec::new(),
            tier3: None,
        }
    }

    pub fn add_module(&mut self, result: ModuleResult) {
        if result.status == ValidationStatus::Pass && result.tier > self.tier_reached {
            self.tier_reached = result.tier;
        }
        self.modules.push(result);
    }

    /// Exit code per the Targeted `GuideStone` standard.
    /// 0 = all pass, 1 = failure, 2 = partial (Tier 3 unavailable).
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        let any_fail = self
            .modules
            .iter()
            .any(|m| m.status == ValidationStatus::Fail);
        let any_skip = self
            .modules
            .iter()
            .any(|m| m.status == ValidationStatus::Skip);

        if any_fail {
            1
        } else if any_skip {
            2
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_report_exit_zero() {
        let r = ValidationReport::new("test", "0.0.0");
        assert_eq!(r.exit_code(), 0);
        assert_eq!(r.tier_reached, 0);
    }

    #[test]
    fn pass_updates_tier() {
        let mut r = ValidationReport::new("test", "0.0.0");
        r.add_module(ModuleResult {
            name: "m1".into(),
            status: ValidationStatus::Pass,
            tier: 2,
            checks: 5,
            checks_passed: 5,
            runtime_ms: 10,
            error: None,
        });
        assert_eq!(r.tier_reached, 2);
        assert_eq!(r.exit_code(), 0);
    }

    #[test]
    fn fail_yields_exit_one() {
        let mut r = ValidationReport::new("test", "0.0.0");
        r.add_module(ModuleResult {
            name: "m1".into(),
            status: ValidationStatus::Fail,
            tier: 1,
            checks: 3,
            checks_passed: 1,
            runtime_ms: 5,
            error: Some("2 check(s) failed".into()),
        });
        assert_eq!(r.exit_code(), 1);
        assert_eq!(r.tier_reached, 0);
    }

    #[test]
    fn skip_yields_exit_two() {
        let mut r = ValidationReport::new("test", "0.0.0");
        r.add_module(ModuleResult {
            name: "m1".into(),
            status: ValidationStatus::Pass,
            tier: 2,
            checks: 5,
            checks_passed: 5,
            runtime_ms: 10,
            error: None,
        });
        r.add_module(ModuleResult {
            name: "scaffold".into(),
            status: ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some("awaiting upstream".into()),
        });
        assert_eq!(r.exit_code(), 2);
    }

    #[test]
    fn fail_takes_priority_over_skip() {
        let mut r = ValidationReport::new("test", "0.0.0");
        r.add_module(ModuleResult {
            name: "fail".into(),
            status: ValidationStatus::Fail,
            tier: 1,
            checks: 1,
            checks_passed: 0,
            runtime_ms: 1,
            error: Some("failed".into()),
        });
        r.add_module(ModuleResult {
            name: "skip".into(),
            status: ValidationStatus::Skip,
            tier: 1,
            checks: 0,
            checks_passed: 0,
            runtime_ms: 0,
            error: Some("skipped".into()),
        });
        assert_eq!(r.exit_code(), 1);
    }

    #[test]
    fn module_result_json_roundtrip() {
        let m = ModuleResult {
            name: "test_module".into(),
            status: ValidationStatus::Pass,
            tier: 2,
            checks: 8,
            checks_passed: 8,
            runtime_ms: 42,
            error: None,
        };
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: ModuleResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test_module");
        assert_eq!(deserialized.status, ValidationStatus::Pass);
        assert_eq!(deserialized.checks, 8);
    }

    #[test]
    fn status_serializes_uppercase() {
        let json = serde_json::to_string(&ValidationStatus::Pass).unwrap();
        assert_eq!(json, "\"PASS\"");
        let json = serde_json::to_string(&ValidationStatus::Fail).unwrap();
        assert_eq!(json, "\"FAIL\"");
        let json = serde_json::to_string(&ValidationStatus::Skip).unwrap();
        assert_eq!(json, "\"SKIP\"");
    }
}
