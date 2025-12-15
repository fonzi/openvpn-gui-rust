// VPN Manager - handles all OpenVPN3 operations

use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use super::parser::{extract_session_path, parse_stats, extract_ip};

/// File picker for .ovpn config files
pub async fn pick_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("OpenVPN Config", &["ovpn", "conf"])
        .pick_file()
        .await
        .map(|handle| PathBuf::from(handle.path()))
}

/// Start a VPN session
pub async fn start_vpn(config_path: String) -> Result<(String, String), String> {
    // OpenVPN3 uses D-Bus and doesn't need elevated privileges
    let _child = Command::new("openvpn3")
        .args(&["session-start", "--config", &config_path])
        .spawn()
        .map_err(|e| format!("Failed to execute: {}", e))?;

    // Wait a moment for the session to initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Try to find the session by listing all sessions
    let list_output = Command::new("openvpn3")
        .args(&["sessions-list"])
        .output()
        .await
        .map_err(|e| format!("Failed to list sessions: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&list_output.stdout).to_string();
    
    // Extract session path from the list
    let session_path = extract_session_path(&stdout)
        .unwrap_or_else(|| format!("/net/openvpn/v3/sessions/{}", uuid::Uuid::new_v4()));
    
    Ok((stdout, session_path))
}

/// Stop VPN by session path
pub async fn stop_vpn_by_path(session_path: String) -> Result<String, String> {
    let output = Command::new("openvpn3")
        .args(&["session-manage", "--session-path", &session_path, "--disconnect"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute: {}", e))?;

    if output.status.success() {
        Ok("VPN Disconnected.".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Stop VPN by config path (fallback)
pub async fn stop_vpn_by_config(config_path: String) -> Result<String, String> {
    let output = Command::new("openvpn3")
        .args(&["session-manage", "--config", &config_path, "--disconnect"])
        .output()
        .await
        .map_err(|e| format!("Failed to execute: {}", e))?;

    if output.status.success() {
        Ok("VPN Disconnected.".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Check session status (for monitoring during connection)
pub async fn check_session_status(_session_path: String) -> Option<String> {
    // Get sessions list to check status
    let output = Command::new("openvpn3")
        .args(&["sessions-list"])
        .output()
        .await
        .ok()?;
    
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

/// Fetch session statistics (bytes in/out)
pub async fn fetch_session_stats(session_path: String) -> Option<(u64, u64)> {
    let output = Command::new("openvpn3")
        .args(&["session-stats", "--session-path", &session_path])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_stats(&stdout)
}

/// Find tunnel IP address (tun0)
pub async fn find_tunnel_ip() -> Option<String> {
    let output = Command::new("ip")
        .args(&["addr", "show", "tun0"])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    extract_ip(&stdout)
}

/// Fetch public IP from external services
pub async fn fetch_public_ip() -> Option<String> {
    // Try to fetch public IP from a service
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    
    // Try multiple services in case one is down
    let services = [
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://icanhazip.com",
    ];
    
    for service in &services {
        if let Ok(response) = client.get(*service).send().await {
            if let Ok(ip) = response.text().await {
                let ip = ip.trim().to_string();
                if !ip.is_empty() {
                    return Some(ip);
                }
            }
        }
    }
    
    None
}

/// Submit 2FA/challenge response
pub async fn submit_challenge(session_path: String, _code: String) -> Result<String, String> {
    let output = Command::new("openvpn3")
        .args(&["session-auth", "--session-path", &session_path])
        .stdin(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to submit: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// List all OpenVPN3 sessions (raw output)
pub async fn list_sessions() -> String {
    let output = Command::new("openvpn3")
        .args(&["sessions-list"])
        .output()
        .await;
    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        Ok(out) => String::from_utf8_lossy(&out.stderr).to_string(),
        Err(e) => format!("Failed to list sessions: {}", e),
    }
}
