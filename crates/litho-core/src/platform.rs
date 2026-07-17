// SPDX-License-Identifier: AGPL-3.0-or-later

//! Platform abstraction layer — silicon atheism compliance.
//!
//! All OS-specific behavior is concentrated in this module behind the
//! [`Platform`] trait. The rest of the codebase uses trait methods rather
//! than `#[cfg]` gates, following the `petal-tongue-platform` reference
//! pattern from `STANDARDS_AND_EXPECTATIONS.md`.
//!
//! Two implementations exist:
//! - `UnixPlatform`: full functionality on unix-family systems
//! - `FallbackPlatform`: graceful degradation on all other platforms

use std::io;
use std::path::Path;

/// Trait abstracting all platform-specific operations.
///
/// Implementations live in this module behind a single `#[cfg]` boundary.
/// All other crates call [`current()`] to get the active platform.
pub trait Platform: Send + Sync {
    /// Discover the system hostname without assuming a specific OS layout.
    fn hostname(&self) -> String;

    /// Make a file executable (chmod +x equivalent). No-op on platforms
    /// without POSIX permissions.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the permissions cannot be set.
    fn set_executable(&self, path: &Path) -> io::Result<()>;

    /// Create a symbolic link. Falls back to file copy on platforms
    /// without native symlink support.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the link/copy cannot be created.
    fn create_symlink(&self, original: &Path, link: &Path) -> io::Result<()>;

    /// Strip debug symbols from a binary to reduce size. Returns `true`
    /// if stripping succeeded, `false` if unavailable.
    fn strip_binary(&self, path: &Path) -> bool;

    /// Resolve the runtime directory (`XDG_RUNTIME_DIR` or equivalent).
    fn runtime_dir(&self) -> String;

    /// Get the current user ID as a string. Returns `None` on platforms
    /// where the concept doesn't apply.
    fn uid(&self) -> Option<String>;

    /// Connect to a Unix domain socket and perform a JSON-RPC exchange.
    /// Returns `None` on platforms without UDS support.
    fn uds_rpc(&self, socket_path: &str, request: &str) -> Option<String>;

    /// Send a payload to a UDS socket and read the full response.
    ///
    /// # Errors
    ///
    /// Returns an error string on platforms without UDS support or on I/O failure.
    fn uds_send(&self, socket_path: &str, payload: &[u8]) -> Result<String, String>;
}

/// Returns the platform implementation for the current target.
#[must_use]
pub fn current() -> &'static dyn Platform {
    #[cfg(unix)]
    {
        &UnixPlatform
    }
    #[cfg(not(unix))]
    {
        &FallbackPlatform
    }
}

// ── Unix implementation ─────────────────────────────────────────────

#[cfg(unix)]
struct UnixPlatform;

#[cfg(unix)]
impl Platform for UnixPlatform {
    fn hostname(&self) -> String {
        if let Ok(val) = std::env::var(crate::env_vars::HOSTNAME) {
            let trimmed = val.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }

        for path in ["/etc/hostname", "/proc/sys/kernel/hostname"] {
            if let Ok(val) = std::fs::read_to_string(path) {
                let trimmed = val.trim().to_string();
                if !trimmed.is_empty() {
                    return trimmed;
                }
            }
        }

        "unknown".to_string()
    }

    fn set_executable(&self, path: &Path) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> io::Result<()> {
        std::os::unix::fs::symlink(original, link)
    }

    fn strip_binary(&self, path: &Path) -> bool {
        std::process::Command::new("strip")
            .arg(path)
            .output()
            .is_ok_and(|o| o.status.success())
    }

    fn runtime_dir(&self) -> String {
        std::env::var(crate::env_vars::XDG_RUNTIME_DIR).unwrap_or_else(|_| {
            let uid = self.uid().unwrap_or_else(|| "1000".to_string());
            format!("/run/user/{uid}")
        })
    }

    fn uid(&self) -> Option<String> {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("Uid:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .map(String::from)
            })
    }

    fn uds_rpc(&self, socket_path: &str, request: &str) -> Option<String> {
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixStream;
        use std::time::Duration;

        let mut stream = UnixStream::connect(socket_path).ok()?;
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .ok()?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .ok()?;

        stream.write_all(request.as_bytes()).ok()?;
        stream.write_all(b"\n").ok()?;
        stream.flush().ok()?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).ok()?;

        Some(response)
    }

    fn uds_send(&self, socket_path: &str, payload: &[u8]) -> Result<String, String> {
        use std::io::{Read, Write};
        use std::os::unix::net::UnixStream;
        use std::time::Duration;

        let mut stream = UnixStream::connect(socket_path).map_err(|e| format!("connect: {e}"))?;
        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
        stream.set_read_timeout(Some(Duration::from_secs(10))).ok();

        stream
            .write_all(payload)
            .map_err(|e| format!("write: {e}"))?;
        stream.flush().map_err(|e| format!("flush: {e}"))?;
        stream.shutdown(std::net::Shutdown::Write).ok();

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| format!("read: {e}"))?;
        Ok(response)
    }
}

// ── Fallback implementation (non-unix) ──────────────────────────────

#[cfg(not(unix))]
struct FallbackPlatform;

#[cfg(not(unix))]
impl Platform for FallbackPlatform {
    fn hostname(&self) -> String {
        for var in [crate::env_vars::HOSTNAME, crate::env_vars::COMPUTERNAME] {
            if let Ok(val) = std::env::var(var) {
                let trimmed = val.trim().to_string();
                if !trimmed.is_empty() {
                    return trimmed;
                }
            }
        }
        "unknown".to_string()
    }

    fn set_executable(&self, _path: &Path) -> io::Result<()> {
        Ok(())
    }

    fn create_symlink(&self, original: &Path, link: &Path) -> io::Result<()> {
        std::fs::copy(original, link)?;
        Ok(())
    }

    fn strip_binary(&self, _path: &Path) -> bool {
        false
    }

    fn runtime_dir(&self) -> String {
        std::env::var(crate::env_vars::TEMP)
            .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string())
    }

    fn uid(&self) -> Option<String> {
        None
    }

    fn uds_rpc(&self, _socket_path: &str, _request: &str) -> Option<String> {
        None
    }

    fn uds_send(&self, _socket_path: &str, _payload: &[u8]) -> Result<String, String> {
        Err("UDS transport not available on this platform".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_returns_platform() {
        let p = current();
        let hostname = p.hostname();
        assert!(!hostname.is_empty());
    }

    #[test]
    fn runtime_dir_returns_nonempty() {
        let p = current();
        let dir = p.runtime_dir();
        assert!(!dir.is_empty());
    }

    #[test]
    fn set_executable_on_tempfile() {
        let dir = std::env::temp_dir();
        let path = dir.join("litho_platform_test");
        std::fs::write(&path, "#!/bin/sh\n").ok();
        let result = current().set_executable(&path);
        std::fs::remove_file(&path).ok();
        assert!(result.is_ok());
    }

    #[test]
    fn uds_rpc_nonexistent_returns_none() {
        let result = current().uds_rpc("/nonexistent/test.sock", "{}");
        assert!(result.is_none());
    }
}
