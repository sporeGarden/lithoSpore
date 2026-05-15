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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report(n_pass: usize, n_skip: usize) -> super::super::ValidationReport {
        let mut report = super::super::ValidationReport::new("test-artifact", "0.1.0");
        for i in 0..n_pass {
            report.add_module(super::super::ModuleResult {
                name: format!("pass_{i}"),
                status: super::super::ValidationStatus::Pass,
                tier: 2,
                checks: 5,
                checks_passed: 5,
                runtime_ms: 10,
                error: None,
            });
        }
        for i in 0..n_skip {
            report.add_module(super::super::ModuleResult {
                name: format!("skip_{i}"),
                status: super::super::ValidationStatus::Skip,
                tier: 1,
                checks: 0,
                checks_passed: 0,
                runtime_ms: 0,
                error: Some("scaffold".into()),
            });
        }
        report
    }

    #[test]
    fn from_report_counts_passes() {
        let report = sample_report(4, 2);
        let entry = LiveSporeEntry::from_report(&report);
        assert_eq!(entry.modules_passed, 4);
        assert_eq!(entry.modules_total, 6);
    }

    #[test]
    fn from_report_records_tier() {
        let report = sample_report(3, 1);
        let entry = LiveSporeEntry::from_report(&report);
        assert_eq!(entry.tier_reached, 2);
    }

    #[test]
    fn from_report_sums_runtime() {
        let report = sample_report(3, 0);
        let entry = LiveSporeEntry::from_report(&report);
        assert_eq!(entry.runtime_ms, 30);
    }

    #[test]
    fn hostname_hash_is_64_hex_chars() {
        let hash = hostname_hash();
        assert_eq!(hash.len(), 64, "BLAKE3 hex should be 64 chars");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hostname_hash_deterministic() {
        let h1 = hostname_hash();
        let h2 = hostname_hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn discover_hostname_returns_nonempty() {
        let name = discover_hostname();
        assert!(!name.is_empty());
    }

    #[test]
    fn entry_json_roundtrip() {
        let report = sample_report(2, 1);
        let entry = LiveSporeEntry::from_report(&report);
        let json = serde_json::to_string_pretty(&entry).unwrap();
        let back: LiveSporeEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.modules_passed, entry.modules_passed);
        assert_eq!(back.modules_total, entry.modules_total);
        assert_eq!(back.arch, entry.arch);
    }

    #[test]
    fn discovery_path_standalone_by_default() {
        let report = sample_report(1, 0);
        let entry = LiveSporeEntry::from_report(&report);
        assert!(
            matches!(entry.discovery_path, DiscoveryPath::Standalone | DiscoveryPath::Env),
            "expected standalone or env, got {:?}", entry.discovery_path
        );
    }
}
