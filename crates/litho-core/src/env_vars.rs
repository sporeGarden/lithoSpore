// SPDX-License-Identifier: AGPL-3.0-or-later

//! Centralized environment variable constants for lithoSpore.
//!
//! Follows the ecosystem pattern established by sweetGrass, rhizoCrypt, et al.
//! All `std::env::var` calls across lithoSpore should reference these constants
//! rather than inline string literals.

// ── lithoSpore-specific ─────────────────────────────────────────────

/// Root directory for spring data trees (used by `litho fetch`).
pub const LITHO_SPRINGS_ROOT: &str = "LITHO_SPRINGS_ROOT";

/// Override libvirt images directory (used by `litho grow deploy`).
pub const LITHO_LIBVIRT_IMAGES: &str = "LITHO_LIBVIRT_IMAGES";

/// Override Rust target triple (used by `litho grow` cross-compile).
pub const LITHO_RUST_TARGET: &str = "LITHO_RUST_TARGET";

/// Comma-separated host:port pairs for connectivity checks.
pub const LITHO_CONNECTIVITY_HOSTS: &str = "LITHO_CONNECTIVITY_HOSTS";

// ── Ecosystem / biomeOS ─────────────────────────────────────────────

/// Indicates a biomeOS orchestrator is managing this environment.
pub const BIOMEOS_ORCHESTRATOR: &str = "BIOMEOS_ORCHESTRATOR";

/// Root directory for NUCLEUS data (mode detection in `litho ops`).
pub const NUCLEUS_ROOT: &str = "NUCLEUS_ROOT";

/// Port for capability-based JSON-RPC discovery.
pub const CAPABILITY_PORT: &str = "CAPABILITY_PORT";

// ── Discovery / mesh ────────────────────────────────────────────────

/// TURN relay server address (Songbird mesh fallback).
pub const RELAY_SERVER: &str = "RELAY_SERVER";

/// Songbird-specific TURN server alias.
pub const SONGBIRD_TURN_SERVER: &str = "SONGBIRD_TURN_SERVER";

/// TURN relay discovery port.
pub const RELAY_DISCOVERY_PORT: &str = "RELAY_DISCOVERY_PORT";

/// Songbird-specific TURN discovery port alias.
pub const SONGBIRD_TURN_DISCOVERY_PORT: &str = "SONGBIRD_TURN_DISCOVERY_PORT";

/// Override the host address for primal connections.
pub const PRIMAL_HOST: &str = "PRIMAL_HOST";

// ── Visualization ───────────────────────────────────────────────────

/// UDS path for a visualization socket (capability-generic).
pub const VISUALIZATION_SOCKET: &str = "VISUALIZATION_SOCKET";

/// UDS path for petalTongue visualization socket (legacy compat).
pub const PETALTONGUE_SOCKET: &str = "PETALTONGUE_SOCKET";

// ── Platform / system ───────────────────────────────────────────────

/// XDG runtime directory (Linux).
pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";

/// User home directory.
pub const HOME: &str = "HOME";

/// Machine hostname (Unix).
pub const HOSTNAME: &str = "HOSTNAME";

/// Machine hostname fallback (some Linux distros).
pub const HOST: &str = "HOST";

/// Machine hostname (Windows).
pub const COMPUTERNAME: &str = "COMPUTERNAME";

/// Temporary directory (Windows fallback for visualization).
pub const TEMP: &str = "TEMP";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_uppercase_ascii() {
        let all = [
            LITHO_SPRINGS_ROOT,
            LITHO_LIBVIRT_IMAGES,
            LITHO_RUST_TARGET,
            LITHO_CONNECTIVITY_HOSTS,
            BIOMEOS_ORCHESTRATOR,
            NUCLEUS_ROOT,
            CAPABILITY_PORT,
            RELAY_SERVER,
            SONGBIRD_TURN_SERVER,
            RELAY_DISCOVERY_PORT,
            SONGBIRD_TURN_DISCOVERY_PORT,
            PRIMAL_HOST,
            VISUALIZATION_SOCKET,
            PETALTONGUE_SOCKET,
            XDG_RUNTIME_DIR,
            HOME,
            HOSTNAME,
            HOST,
            COMPUTERNAME,
            TEMP,
        ];
        for name in &all {
            assert!(
                name.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
                "env var constant {name} must be UPPER_SNAKE_CASE"
            );
        }
    }
}
