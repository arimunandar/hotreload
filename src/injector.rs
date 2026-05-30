use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::error::HotReloadError;

fn connect_tcp(host: &str, port: u16) -> anyhow::Result<TcpStream> {
    let addr_str = format!("{}:{}", host, port);
    let addrs: Vec<_> = addr_str.to_socket_addrs()
        .map(|a| a.collect())
        .unwrap_or_default();

    // Try all resolved addresses (IPv4 and IPv6)
    let mut last_err = None;
    for addr in &addrs {
        match TcpStream::connect_timeout(addr, Duration::from_secs(5)) {
            Ok(stream) => return Ok(stream),
            Err(e) => last_err = Some(e),
        }
    }

    // Fallback: try IPv6 localhost explicitly
    if host == "127.0.0.1" || host == "localhost" {
        let v6_addr = format!("[::1]:{}", port);
        if let Ok(addrs) = v6_addr.to_socket_addrs() {
            for addr in addrs {
                if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
                    return Ok(stream);
                }
            }
        }
    }

    Err(last_err
        .map(|e| anyhow::anyhow!("Connection failed: {}", e))
        .unwrap_or_else(|| anyhow::anyhow!("{}", HotReloadError::ConnectionRefused(host.to_string(), port))))
}

/// Send an injection command to the iOS app's TCP server
pub fn send_injection(
    host: &str,
    port: u16,
    dylib_name: &str,
    dylib_url: &str,
) -> anyhow::Result<String> {
    let mut stream = connect_tcp(host, port)?;

    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    let command = serde_json::json!({
        "command": "reload",
        "dylib_name": dylib_name,
        "dylib_url": dylib_url,
    });

    let json = serde_json::to_string(&command)?;
    let bytes = json.as_bytes();
    let len = bytes.len() as u32;

    // Send length-prefixed JSON: 4 bytes (big-endian length) + JSON payload
    let len_bytes = len.to_be_bytes();
    stream.write_all(&len_bytes)?;
    stream.write_all(bytes)?;

    tracing::debug!("Sent injection command: {}", json);

    // Read response: 4 bytes length prefix + JSON response
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let response_len = u32::from_be_bytes(len_buf) as usize;

    let mut response_buf = vec![0u8; response_len];
    stream.read_exact(&mut response_buf)?;

    let response = String::from_utf8_lossy(&response_buf).to_string();
    tracing::debug!("Injection response: {}", response);

    Ok(response)
}

/// Expected protocol version for CLI <-> HotReloadKit handshake.
/// Update this when the wire protocol changes.
const EXPECTED_KIT_VERSION: &str = "v1";

/// Ping the app to check connectivity and verify version compatibility.
/// The response is expected to contain a version string (e.g. "pong:v1").
/// If the version is missing or mismatched, a warning is printed but
/// connectivity is still reported as successful.
pub fn ping(host: &str, port: u16) -> anyhow::Result<String> {
    let mut stream = connect_tcp(host, port)?;

    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    let command = serde_json::json!({"command": "ping"});
    let json = serde_json::to_string(&command)?;
    let bytes = json.as_bytes();
    let len = bytes.len() as u32;

    let len_bytes = len.to_be_bytes();
    stream.write_all(&len_bytes)?;
    stream.write_all(bytes)?;

    // Read response
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let response_len = u32::from_be_bytes(len_buf) as usize;

    let mut response_buf = vec![0u8; response_len];
    stream.read_exact(&mut response_buf)?;

    let response = String::from_utf8_lossy(&response_buf).to_string();

    // Version handshake: expect response like "pong:v1"
    check_version_handshake(&response);

    Ok(response)
}

/// Check the ping response for a version string and warn on mismatch.
fn check_version_handshake(response: &str) {
    // Try to parse version from response. Expected formats:
    //   "pong:v1"  (plain text)
    //   {"status":"pong","version":"v1"}  (JSON)
    let message = serde_json::from_str::<serde_json::Value>(response).ok()
        .and_then(|j| j.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| response.to_string());

    let version = if let Some(idx) = message.find("pong:") {
        Some(message[idx + 5..].trim().to_string())
    } else {
        None
    };

    match version {
        Some(v) if v == EXPECTED_KIT_VERSION => {
            tracing::debug!("Version handshake OK: {}", v);
        }
        Some(v) => {
            eprintln!(
                "⚠️  Version mismatch: HotReloadKit reports '{}', CLI expects '{}'. \
                 Consider updating HotReloadKit to the latest version.",
                v, EXPECTED_KIT_VERSION
            );
        }
        None => {
            eprintln!(
                "⚠️  HotReloadKit did not report a version (pre-v1?). \
                 Consider updating HotReloadKit to the latest version for best compatibility."
            );
        }
    }
}
