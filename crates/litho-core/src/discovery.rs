// SPDX-License-Identifier: AGPL-3.0-or-later

//! Capability-based primal discovery for Tier 3 integration.
//!
//! lithoSpore has self-knowledge only. It discovers primals at runtime through
//! a priority chain: environment variable → Unix domain socket → well-known
//! default. No primal names are hardcoded in application logic — all resolution
//! goes through capability strings.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

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

/// Resolve a primal by capability name using the ecosystem discovery chain:
///
/// 1. Environment: `{CAPABILITY_UPPER}_PORT` (e.g. `NESTGATE_PORT=9500`)
/// 2. Discovery socket: `$XDG_RUNTIME_DIR/ecoPrimals/discovery.sock`
/// 3. None — caller decides how to degrade gracefully
#[must_use]
pub fn discover(capability: &str) -> Option<PrimalEndpoint> {
    discover_from_env(capability)
        .or_else(|| discover_from_socket(capability))
}

fn discover_from_env(capability: &str) -> Option<PrimalEndpoint> {
    let env_key = format!("{}_PORT", capability.to_uppercase().replace('.', "_"));
    let port_str = std::env::var(&env_key).ok()?;
    let port: u16 = port_str.parse().ok()?;
    let host = std::env::var("PRIMAL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    Some(PrimalEndpoint {
        capability: capability.to_string(),
        host,
        port,
        transport: Transport::Tcp,
    })
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
    let port = result.get("port")?.as_u64()? as u16;
    let host = result
        .get("host")
        .and_then(|h| h.as_str())
        .unwrap_or("127.0.0.1")
        .to_string();

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
pub fn rpc_call(endpoint: &PrimalEndpoint, request: &str) -> Option<serde_json::Value> {
    match endpoint.transport {
        Transport::Tcp => rpc_tcp(endpoint, request),
        Transport::Uds => None, // UDS RPC not yet wired — degrade to skip
    }
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
    fn env_discovery_not_set() {
        assert!(discover_from_env("nonexistent_test_primal_xyz").is_none());
    }
}
