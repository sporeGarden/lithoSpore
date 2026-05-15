// SPDX-License-Identifier: AGPL-3.0-or-later

//! Capability-based primal discovery for Tier 2/3 integration.
//!
//! lithoSpore has self-knowledge only. It discovers primals at runtime through
//! a priority chain: environment variable → Unix domain socket → Songbird TURN
//! relay → None. No primal names are hardcoded in application logic — all
//! resolution goes through capability strings.
//!
//! The discovery path maps to lithoSpore's three operating modes:
//! - **Standalone** (no discovery): Tier 1 Python-only against bundled data
//! - **LAN** (env or UDS): Tier 2 Rust + primal IPC via local sockets
//! - **Geo-delocalized** (TURN): Tier 2 via Songbird TURN through cellMembrane

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

/// Which discovery mechanism resolved the primal.
/// Recorded in `liveSpore.json` for provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryPath {
    /// Resolved via environment variable (`$CAPABILITY_PORT`)
    Env,
    /// Resolved via UDS filesystem convention (`discovery.sock`)
    Uds,
    /// Resolved via Songbird TURN relay through cellMembrane
    Turn,
    /// No primals discovered — standalone mode
    Standalone,
}

impl std::fmt::Display for DiscoveryPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Env => write!(f, "env"),
            Self::Uds => write!(f, "uds"),
            Self::Turn => write!(f, "turn"),
            Self::Standalone => write!(f, "standalone"),
        }
    }
}

/// A discovered primal endpoint.
#[derive(Debug, Clone)]
pub struct PrimalEndpoint {
    pub capability: String,
    pub host: String,
    pub port: u16,
    pub transport: Transport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Tcp,
    Uds,
}

/// Result of the full discovery chain, including which path resolved.
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub endpoint: PrimalEndpoint,
    pub path: DiscoveryPath,
    /// Set when discovery routed through a TURN relay.
    pub turn_relay: Option<String>,
}

/// Resolve a primal by capability name using the ecosystem discovery chain:
///
/// 1. Environment: `{CAPABILITY_UPPER}_PORT` (e.g. `NESTGATE_PORT=9500`)
/// 2. Discovery socket: `$XDG_RUNTIME_DIR/ecoPrimals/discovery.sock`
/// 3. Songbird TURN: `$SONGBIRD_TURN_SERVER` (geo-delocalized mode)
/// 4. None — caller decides how to degrade gracefully
#[must_use]
pub fn discover(capability: &str) -> Option<PrimalEndpoint> {
    discover_full(capability).map(|r| r.endpoint)
}

/// Like `discover`, but also returns the path and relay metadata
/// for `liveSpore.json` provenance recording.
#[must_use]
pub fn discover_full(capability: &str) -> Option<DiscoveryResult> {
    if let Some(ep) = discover_from_env(capability) {
        return Some(DiscoveryResult { endpoint: ep, path: DiscoveryPath::Env, turn_relay: None });
    }
    if let Some(ep) = discover_from_socket(capability) {
        return Some(DiscoveryResult { endpoint: ep, path: DiscoveryPath::Uds, turn_relay: None });
    }
    if let Some(result) = discover_from_turn(capability) {
        return Some(result);
    }
    None
}

/// Probe the best available discovery path without resolving a specific
/// capability. Returns `Standalone` if no primals are reachable.
///
/// Checks are transport-level only — no hardcoded primal names.
#[must_use]
pub fn probe_operating_mode() -> (DiscoveryPath, Option<String>) {
    if has_any_capability_env() {
        return (DiscoveryPath::Env, None);
    }
    if discovery_socket_path().is_some() {
        return (DiscoveryPath::Uds, None);
    }
    if let Ok(turn) = std::env::var("SONGBIRD_TURN_SERVER") {
        return (DiscoveryPath::Turn, Some(turn));
    }
    (DiscoveryPath::Standalone, None)
}

/// Check whether any `*_PORT` capability environment variable is set.
/// This avoids hardcoding specific primal names in the discovery probe.
fn has_any_capability_env() -> bool {
    std::env::vars().any(|(key, _)| key.ends_with("_PORT") && key != "PORT")
}

fn discover_from_env(capability: &str) -> Option<PrimalEndpoint> {
    let env_key = format!("{}_PORT", capability.to_uppercase().replace('.', "_"));
    let port_str = std::env::var(&env_key).ok()?;
    let port: u16 = port_str.parse().ok()?;
    let host = resolve_primal_host();

    Some(PrimalEndpoint {
        capability: capability.to_string(),
        host,
        port,
        transport: Transport::Tcp,
    })
}

/// Resolve the primal host address from `$PRIMAL_HOST`, defaulting to
/// localhost. The environment variable is the single source of truth —
/// no primal-specific IPs are encoded anywhere in lithoSpore.
fn resolve_primal_host() -> String {
    std::env::var("PRIMAL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
}

fn discovery_socket_path() -> Option<PathBuf> {
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path = PathBuf::from(runtime).join("ecoPrimals/discovery.sock");
    if path.exists() { Some(path) } else { None }
}

fn discover_from_socket(capability: &str) -> Option<PrimalEndpoint> {
    let sock_path = discovery_socket_path()?;

    #[cfg(unix)]
    {
        use std::os::unix::net::UnixStream;
        let mut stream = UnixStream::connect(&sock_path).ok()?;
        stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
        stream.set_write_timeout(Some(Duration::from_secs(2))).ok()?;

        let request = format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"capability.resolve\",\"params\":{{\"capability\":\"{capability}\"}},\"id\":1}}\n"
        );
        stream.write_all(request.as_bytes()).ok()?;
        stream.flush().ok()?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).ok()?;

        parse_discovery_response(capability, &response)
    }

    #[cfg(not(unix))]
    {
        let _ = sock_path;
        None
    }
}

fn parse_discovery_response(capability: &str, response: &str) -> Option<PrimalEndpoint> {
    let v: serde_json::Value = serde_json::from_str(response).ok()?;
    let result = v.get("result")?;

    // UDS discovery can return a socket path instead of host:port
    if let Some(socket_path) = result.get("socket").and_then(|s| s.as_str()) {
        return Some(PrimalEndpoint {
            capability: capability.to_string(),
            host: socket_path.to_string(),
            port: 0,
            transport: Transport::Uds,
        });
    }

    let port = u16::try_from(result.get("port")?.as_u64()?).ok()?;
    let host = result
        .get("host")
        .and_then(|h| h.as_str())
        .map_or_else(resolve_primal_host, String::from);

    Some(PrimalEndpoint {
        capability: capability.to_string(),
        host,
        port,
        transport: Transport::Tcp,
    })
}

/// Send a JSON-RPC request to a discovered primal and return the response.
///
/// Returns `None` on connection/timeout/parse failure — callers degrade
/// gracefully rather than panicking.
///
/// # Transport support
///
/// - **TCP**: Standard JSON-RPC over TCP to `host:port`.
/// - **UDS**: JSON-RPC over Unix domain socket (path stored in `host` field).
/// - **TURN**: Resolves a relay endpoint but uses TCP transport (actual TURN
///   client integration requires the upstream Songbird client library).
#[must_use]
pub fn rpc_call(endpoint: &PrimalEndpoint, request: &str) -> Option<serde_json::Value> {
    match endpoint.transport {
        Transport::Tcp => rpc_tcp(endpoint, request),
        Transport::Uds => rpc_uds(endpoint, request),
    }
}

/// Attempt discovery through a Songbird TURN relay on the cellMembrane.
///
/// The relay address comes from `$SONGBIRD_TURN_SERVER`. TURN-relayed
/// discovery is structurally identical to UDS discovery but routes
/// through the cellMembrane's Channel 2 relay. Actual TURN client
/// integration requires the Songbird client library (upstream).
fn discover_from_turn(capability: &str) -> Option<DiscoveryResult> {
    let turn_server = std::env::var("SONGBIRD_TURN_SERVER").ok()?;
    let turn_port = std::env::var("SONGBIRD_TURN_DISCOVERY_PORT").ok()?;
    let port: u16 = turn_port.parse().ok()?;

    Some(DiscoveryResult {
        endpoint: PrimalEndpoint {
            capability: capability.to_string(),
            host: turn_server.split(':').next().map_or_else(resolve_primal_host, String::from),
            port,
            transport: Transport::Tcp,
        },
        path: DiscoveryPath::Turn,
        turn_relay: Some(turn_server),
    })
}

fn rpc_tcp(endpoint: &PrimalEndpoint, request: &str) -> Option<serde_json::Value> {
    let addr = format!("{}:{}", endpoint.host, endpoint.port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().ok()?,
        Duration::from_secs(5),
    ).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok()?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok()?;

    stream.write_all(request.as_bytes()).ok()?;
    stream.write_all(b"\n").ok()?;
    stream.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).ok()?;

    serde_json::from_str(&response).ok()
}

/// JSON-RPC over Unix domain socket. The socket path is stored in
/// `endpoint.host` (port is ignored for UDS transport).
#[cfg(unix)]
fn rpc_uds(endpoint: &PrimalEndpoint, request: &str) -> Option<serde_json::Value> {
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(&endpoint.host).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok()?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok()?;

    stream.write_all(request.as_bytes()).ok()?;
    stream.write_all(b"\n").ok()?;
    stream.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).ok()?;

    serde_json::from_str(&response).ok()
}

#[cfg(not(unix))]
fn rpc_uds(_endpoint: &PrimalEndpoint, _request: &str) -> Option<serde_json::Value> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_discovery_response_valid() {
        let json = r#"{"jsonrpc":"2.0","result":{"port":9500,"host":"127.0.0.1"},"id":1}"#;
        let ep = parse_discovery_response("storage", json).unwrap();
        assert_eq!(ep.port, 9500);
        assert_eq!(ep.host, "127.0.0.1");
    }

    #[test]
    fn parse_discovery_response_missing_port() {
        let json = r#"{"jsonrpc":"2.0","result":{},"id":1}"#;
        assert!(parse_discovery_response("storage", json).is_none());
    }

    #[test]
    fn parse_discovery_response_uds_socket() {
        let json = r#"{"jsonrpc":"2.0","result":{"socket":"/run/user/1000/biomeos/petaltongue.sock"},"id":1}"#;
        let ep = parse_discovery_response("visualization", json).unwrap();
        assert_eq!(ep.transport, Transport::Uds);
        assert_eq!(ep.host, "/run/user/1000/biomeos/petaltongue.sock");
        assert_eq!(ep.port, 0);
    }

    #[test]
    fn has_any_capability_env_detects_port_vars() {
        // This test relies on the _PORT suffix convention
        // In a clean environment, no *_PORT vars should be set
        let result = has_any_capability_env();
        // Just verify it doesn't panic — actual detection depends on env
        let _ = result;
    }

    #[test]
    fn env_discovery_not_set() {
        assert!(discover_from_env("nonexistent_test_primal_xyz").is_none());
    }

    #[test]
    fn discovery_path_serializes_snake_case() {
        let json = serde_json::to_string(&DiscoveryPath::Env).unwrap();
        assert_eq!(json, "\"env\"");
        let json = serde_json::to_string(&DiscoveryPath::Uds).unwrap();
        assert_eq!(json, "\"uds\"");
        let json = serde_json::to_string(&DiscoveryPath::Turn).unwrap();
        assert_eq!(json, "\"turn\"");
        let json = serde_json::to_string(&DiscoveryPath::Standalone).unwrap();
        assert_eq!(json, "\"standalone\"");
    }

    #[test]
    fn discovery_path_display() {
        assert_eq!(DiscoveryPath::Env.to_string(), "env");
        assert_eq!(DiscoveryPath::Turn.to_string(), "turn");
        assert_eq!(DiscoveryPath::Standalone.to_string(), "standalone");
    }
}
