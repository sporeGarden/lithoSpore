// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho verify` — data integrity: rehash files against manifest, probe upstream.

pub fn run(root: &str, json_output: bool) {
    let root_path = std::path::Path::new(root);
    let manifest_path = root_path.join("data_manifest.toml");
    let data_toml_path = root_path.join("artifact/data.toml");

    let mut results = VerifyResults::default();

    if manifest_path.exists() {
        verify_local_integrity(&manifest_path, root_path, &mut results, json_output);
    } else if !json_output {
        println!("  No data_manifest.toml found — cannot verify local integrity\n");
    }

    let online = check_connectivity();
    results.online = online;

    if !json_output {
        println!("=== Upstream source check ===");
        println!("  Connectivity: {}", if online { "ONLINE" } else { "OFFLINE (airgapped) — skipping upstream checks" });
    }

    if online && data_toml_path.exists() {
        verify_upstream(&data_toml_path, &mut results, json_output);
    }

    print_summary(&results, json_output);

    let local_drift = results.local_checks.iter().filter(|c| c.status == "DRIFT").count();
    if local_drift > 0 {
        std::process::exit(1);
    }
}

fn verify_local_integrity(
    manifest_path: &std::path::Path,
    root_path: &std::path::Path,
    results: &mut VerifyResults,
    json_output: bool,
) {
    let content = std::fs::read_to_string(manifest_path).unwrap_or_default();
    let manifest: toml::Value = toml::from_str(&content).unwrap_or(toml::Value::Table(Default::default()));

    if let Some(files) = manifest.get("file").and_then(|v| v.as_array()) {
        if !json_output { println!("=== Local integrity check (BLAKE3) ==="); }

        for entry in files {
            let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let expected_hash = entry.get("blake3").and_then(|v| v.as_str()).unwrap_or("");

            if path.is_empty() || expected_hash.is_empty() { continue; }

            let full_path = root_path.join(path);
            let check = if full_path.exists() {
                match hash_file(&full_path) {
                    Ok(actual) => {
                        if actual == expected_hash {
                            FileCheck { path: path.into(), status: "ok".into(), expected: expected_hash.into(), actual, detail: None }
                        } else {
                            FileCheck { path: path.into(), status: "DRIFT".into(), expected: expected_hash.into(), actual, detail: Some("local file hash does not match manifest".into()) }
                        }
                    }
                    Err(e) => FileCheck { path: path.into(), status: "ERROR".into(), expected: expected_hash.into(), actual: String::new(), detail: Some(format!("hash error: {e}")) },
                }
            } else {
                FileCheck { path: path.into(), status: "MISSING".into(), expected: expected_hash.into(), actual: String::new(), detail: Some("file not found on disk".into()) }
            };

            if !json_output && check.status != "ok" {
                println!("  [{:>7}] {}{}", check.status, check.path, check.detail.as_deref().map(|d| format!(" — {d}")).unwrap_or_default());
            }
            results.local_checks.push(check);
        }

        let ok_count = results.local_checks.iter().filter(|c| c.status == "ok").count();
        let total = results.local_checks.len();
        if !json_output { println!("  {ok_count}/{total} files verified\n"); }
    }
}

fn verify_upstream(data_toml_path: &std::path::Path, results: &mut VerifyResults, json_output: bool) {
    let content = std::fs::read_to_string(data_toml_path).unwrap_or_default();
    let data_toml: toml::Value = toml::from_str(&content).unwrap_or(toml::Value::Table(Default::default()));

    if let Some(datasets) = data_toml.get("dataset").and_then(|v| v.as_array()) {
        for ds in datasets {
            let id = ds.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let uri = ds.get("source_uri").and_then(|v| v.as_str()).unwrap_or("");

            if uri.is_empty() {
                results.upstream_checks.push(UpstreamCheck {
                    dataset_id: id.into(),
                    source_uri: String::new(),
                    status: "no_uri".into(),
                    detail: Some("no source URI configured".into()),
                });
                continue;
            }

            let mut probe = probe_upstream(uri);
            probe.dataset_id = id.into();

            if !json_output {
                match &probe.status[..] {
                    "reachable" => println!("  [{id}] {uri} — reachable"),
                    "unreachable" => println!("  [{id}] {uri} — UNREACHABLE: {}", probe.detail.as_deref().unwrap_or("?")),
                    _ => println!("  [{id}] {uri} — {}", probe.status),
                }
            }
            results.upstream_checks.push(probe);
        }
    }
}

fn print_summary(results: &VerifyResults, json_output: bool) {
    let local_ok = results.local_checks.iter().filter(|c| c.status == "ok").count();
    let local_total = results.local_checks.len();
    let local_drift = results.local_checks.iter().filter(|c| c.status == "DRIFT").count();
    let upstream_reachable = results.upstream_checks.iter().filter(|c| c.status == "reachable").count();
    let upstream_total = results.upstream_checks.iter().filter(|c| !c.source_uri.is_empty()).count();

    if json_output {
        let mut out = results.clone();
        out.summary = VerifySummary {
            local_files_ok: local_ok,
            local_files_total: local_total,
            local_drift,
            upstream_reachable,
            upstream_total,
            online: results.online,
        };
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!();
        println!("=== Verification Summary ===");
        println!("  Local:    {local_ok}/{local_total} files intact, {local_drift} drifted");
        if results.online {
            println!("  Upstream: {upstream_reachable}/{upstream_total} sources reachable");
        } else {
            println!("  Upstream: skipped (offline)");
        }
    }
}

fn hash_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Well-known upstream hosts used only for connectivity probing (not data fetch).
/// Overridable via `$LITHO_CONNECTIVITY_HOSTS` (comma-separated host:port).
const DEFAULT_CONNECTIVITY_HOSTS: &[&str] = &[
    "datadryad.org:443",
    "www.ncbi.nlm.nih.gov:443",
    "github.com:443",
];

fn check_connectivity() -> bool {
    use std::net::{TcpStream, ToSocketAddrs};

    let custom = std::env::var("LITHO_CONNECTIVITY_HOSTS").ok();
    let custom_hosts: Vec<String> = custom
        .as_deref()
        .map(|s| s.split(',').map(|h| h.trim().to_string()).collect())
        .unwrap_or_default();

    let hosts: Vec<&str> = if custom_hosts.is_empty() {
        DEFAULT_CONNECTIVITY_HOSTS.to_vec()
    } else {
        custom_hosts.iter().map(|s| s.as_str()).collect()
    };

    for addr in &hosts {
        if let Ok(mut iter) = addr.to_socket_addrs() {
            if let Some(sock) = iter.next() {
                if TcpStream::connect_timeout(&sock, std::time::Duration::from_secs(3)).is_ok() {
                    return true;
                }
            }
        }
    }
    false
}

fn probe_upstream(uri: &str) -> UpstreamCheck {
    use std::net::{TcpStream, ToSocketAddrs};

    let host = uri
        .strip_prefix("https://").or_else(|| uri.strip_prefix("http://"))
        .and_then(|s| s.split('/').next())
        .unwrap_or("");

    if host.is_empty() {
        return UpstreamCheck {
            dataset_id: String::new(),
            source_uri: uri.into(),
            status: "invalid_uri".into(),
            detail: Some("cannot parse host from URI".into()),
        };
    }

    let addr_str = format!("{host}:443");
    match addr_str.to_socket_addrs() {
        Ok(mut iter) => {
            if let Some(sock) = iter.next() {
                match TcpStream::connect_timeout(&sock, std::time::Duration::from_secs(5)) {
                    Ok(_) => UpstreamCheck {
                        dataset_id: String::new(),
                        source_uri: uri.into(),
                        status: "reachable".into(),
                        detail: None,
                    },
                    Err(e) => UpstreamCheck {
                        dataset_id: String::new(),
                        source_uri: uri.into(),
                        status: "unreachable".into(),
                        detail: Some(format!("TCP connect failed: {e}")),
                    },
                }
            } else {
                UpstreamCheck {
                    dataset_id: String::new(),
                    source_uri: uri.into(),
                    status: "unreachable".into(),
                    detail: Some("DNS resolved but no addresses".into()),
                }
            }
        }
        Err(e) => UpstreamCheck {
            dataset_id: String::new(),
            source_uri: uri.into(),
            status: "unreachable".into(),
            detail: Some(format!("DNS resolution failed: {e}")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_file_produces_consistent_result() {
        let dir = std::env::temp_dir().join("litho-test-verify");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_hash.txt");
        std::fs::write(&path, b"hello lithoSpore").unwrap();

        let h1 = hash_file(&path).unwrap();
        let h2 = hash_file(&path).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // BLAKE3 hex is 64 chars

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn hash_file_error_on_missing() {
        let result = hash_file(std::path::Path::new("/nonexistent/file.bin"));
        assert!(result.is_err());
    }

    #[test]
    fn probe_upstream_rejects_empty_host() {
        let check = probe_upstream("not-a-uri");
        assert_eq!(check.status, "invalid_uri");
    }

    #[test]
    fn probe_upstream_parses_https() {
        let check = probe_upstream("https://example.com/data/file.gz");
        // Will either be reachable or unreachable — but not invalid_uri
        assert_ne!(check.status, "invalid_uri");
    }

    #[test]
    fn verify_results_serializes() {
        let results = VerifyResults::default();
        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("\"online\":false"));
    }

    #[test]
    fn file_check_skip_none_detail() {
        let check = FileCheck {
            path: "test.txt".into(),
            status: "ok".into(),
            expected: "abc".into(),
            actual: "abc".into(),
            detail: None,
        };
        let json = serde_json::to_string(&check).unwrap();
        assert!(!json.contains("detail"));
    }
}

#[derive(Default, Clone, serde::Serialize)]
struct VerifyResults {
    online: bool,
    local_checks: Vec<FileCheck>,
    upstream_checks: Vec<UpstreamCheck>,
    summary: VerifySummary,
}

#[derive(Clone, serde::Serialize)]
struct FileCheck {
    path: String,
    status: String,
    expected: String,
    actual: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Clone, serde::Serialize)]
struct UpstreamCheck {
    dataset_id: String,
    source_uri: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Default, Clone, serde::Serialize)]
struct VerifySummary {
    local_files_ok: usize,
    local_files_total: usize,
    local_drift: usize,
    upstream_reachable: usize,
    upstream_total: usize,
    online: bool,
}
