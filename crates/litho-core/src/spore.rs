// SPDX-License-Identifier: AGPL-3.0-or-later

//! liveSpore: deployment tracking — records every validation run.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSporeEntry {
    pub timestamp: String,
    pub hostname_hash: String,
    pub arch: String,
    pub os: String,
    pub tier_reached: u8,
    pub modules_passed: u32,
    pub modules_total: u32,
    pub runtime_ms: u64,
}

impl LiveSporeEntry {
    /// Create a new entry from the current system and a validation report.
    #[must_use]
    pub fn from_report(report: &super::ValidationReport) -> Self {
        let hostname = hostname_hash();
        let passed = report
            .modules
            .iter()
            .filter(|m| m.status == super::ValidationStatus::Pass)
            .count();

        let total_runtime: u64 = report.modules.iter().map(|m| m.runtime_ms).sum();

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            hostname_hash: hostname,
            arch: std::env::consts::ARCH.to_string(),
            os: std::env::consts::OS.to_string(),
            tier_reached: report.tier_reached,
            #[allow(clippy::cast_possible_truncation)]
            modules_passed: passed as u32,
            #[allow(clippy::cast_possible_truncation)]
            modules_total: report.modules.len() as u32,
            runtime_ms: total_runtime,
        }
    }
}

/// BLAKE3 hash of the system hostname — no PII stored.
fn hostname_hash() -> String {
    let hostname = gethostname();
    let hash = blake3::hash(hostname.as_bytes());
    hash.to_hex().to_string()
}

fn gethostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}
