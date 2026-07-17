// SPDX-License-Identifier: AGPL-3.0-or-later

//! Named constants for timeouts, buffer sizes, and network defaults.
//!
//! Extracted from scattered magic numbers across platform.rs, discovery.rs,
//! and verify.rs. All timeout values are in seconds; buffer sizes in bytes.
//! These are operational defaults — callers may override via environment
//! variables where documented.

use std::time::Duration;

// ── IPC / socket timeouts ──────────────────────────────────────────

/// Read timeout for UDS and TCP IPC connections.
pub const IPC_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Write timeout for UDS and TCP IPC connections.
pub const IPC_WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// Connect timeout for TCP IPC connections (discovery, TURN relay).
pub const IPC_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

// ── Network probe timeouts ─────────────────────────────────────────

/// Timeout for upstream connectivity probes (data source reachability).
pub const UPSTREAM_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout for connectivity check probes (network availability).
pub const CONNECTIVITY_PROBE_TIMEOUT: Duration = Duration::from_secs(3);

// ── I/O buffers ────────────────────────────────────────────────────

/// Buffer size for streaming file hashing (BLAKE3).
pub const HASH_BUFFER_SIZE: usize = 65_536;

// ── VM / deploy defaults ───────────────────────────────────────────

/// Default VM disk size for `litho grow deploy` VMs.
pub const VM_DISK_SIZE: &str = "20G";

/// Default VM RAM in megabytes.
pub const VM_RAM_MB: &str = "4096";

/// Default VM vCPU count.
pub const VM_VCPU_COUNT: &str = "2";

/// Default OS variant for virt-install.
pub const VM_OS_VARIANT: &str = "ubuntu24.04";

/// Maximum attempts to poll for VM boot (IP assignment).
pub const VM_BOOT_POLL_ATTEMPTS: u32 = 60;

/// Interval between VM boot poll attempts.
pub const VM_BOOT_POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Default cloud image URL for Ubuntu 24.04.
pub const VM_CLOUD_IMAGE_URL: &str =
    "https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-amd64.img";

// ── Discovery defaults ─────────────────────────────────────────────

/// Default host for primal connections when no env var is set.
pub const DEFAULT_PRIMAL_HOST: &str = "127.0.0.1";

/// Subdirectory under `$XDG_RUNTIME_DIR` for ecosystem socket discovery.
pub const RUNTIME_SUBDIR: &str = "ecoPrimals";

/// Filename of the discovery socket within the runtime subdirectory.
pub const DISCOVERY_SOCKET_NAME: &str = "discovery.sock";

// ── Scientific defaults ────────────────────────────────────────────

/// Default RMSD acceptance threshold (kJ/mol) for promote/verify.
pub const DEFAULT_RMSD_KJ: f64 = 2.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeouts_are_positive() {
        assert!(!IPC_READ_TIMEOUT.is_zero());
        assert!(!IPC_WRITE_TIMEOUT.is_zero());
        assert!(!IPC_CONNECT_TIMEOUT.is_zero());
        assert!(!UPSTREAM_PROBE_TIMEOUT.is_zero());
        assert!(!CONNECTIVITY_PROBE_TIMEOUT.is_zero());
    }

    #[test]
    fn hash_buffer_is_power_of_two() {
        assert!(HASH_BUFFER_SIZE.is_power_of_two());
    }

    #[test]
    fn read_timeout_exceeds_write() {
        assert!(IPC_READ_TIMEOUT > IPC_WRITE_TIMEOUT);
    }
}
