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
pub struct ValidationReport {
    pub artifact: String,
    pub version: String,
    pub tier_reached: u8,
    pub modules: Vec<ModuleResult>,
}

impl ValidationReport {
    #[must_use]
    pub fn new(artifact: &str, version: &str) -> Self {
        Self {
            artifact: artifact.to_string(),
            version: version.to_string(),
            tier_reached: 0,
            modules: Vec::new(),
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
