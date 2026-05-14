// SPDX-License-Identifier: AGPL-3.0-or-later

//! liveSpore: deployment tracking — records every validation run.
//!
//! Each `./validate` run appends a `LiveSporeEntry` to `liveSpore.json`.
//! The journal is append-only and publishable: no PII (hostname is
//! BLAKE3-hashed), and the `discovery_path` + `turn_relay` fields record
//! the operating mode for geo-delocalized provenance.

use serde::{Deserialize, Serialize};
use super::discovery::DiscoveryPath;

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
    pub discovery_path: DiscoveryPath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_relay: Option<String>,
}

impl LiveSporeEntry {
    /// Create a new entry from the current system and a validation report.
    /// `discovery_path` and `turn_relay` are determined by probing the
    /// environment before validation begins.
    #[must_use]
    pub fn from_report(report: &super::ValidationReport) -> Self {
        let hostname = hostname_hash();
        let passed = report
            .modules
            .iter()
            .filter(|m| m.status == super::ValidationStatus::Pass)
            .count();

        let total_runtime: u64 = report.modules.iter().map(|m| m.runtime_ms).sum();
        let (path, relay) = super::discovery::probe_operating_mode();

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            hostname_hash: hostname,
            arch: std::env::consts::ARCH.to_string(),
            os: std::env::consts::OS.to_string(),
            tier_reached: report.tier_reached,
            modules_passed: u32::try_from(passed).unwrap_or(u32::MAX),
            modules_total: u32::try_from(report.modules.len()).unwrap_or(u32::MAX),
            runtime_ms: total_runtime,
            discovery_path: path,
            turn_relay: relay,
        }
    }
}

/// BLAKE3 hash of the system hostname — no PII stored.
fn hostname_hash() -> String {
    let hostname = discover_hostname();
    let hash = blake3::hash(hostname.as_bytes());
    hash.to_hex().to_string()
}

/// Capability-based hostname discovery — tries multiple platform-agnostic
/// sources in priority order, never assumes a specific OS layout.
fn discover_hostname() -> String {
    if let Ok(val) = std::env::var("HOSTNAME") {
        let trimmed = val.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    if let Ok(val) = std::fs::read_to_string("/etc/hostname") {
        let trimmed = val.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    if let Ok(output) = std::process::Command::new("hostname").output()
        && output.status.success()
    {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return name;
        }
    }

    "unknown".to_string()
}
