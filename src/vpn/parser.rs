// VPN parsing helpers

use regex::Regex;

/// Extract session path from openvpn3 sessions-list output
pub fn extract_session_path(output: &str) -> Option<String> {
    // Try multiple patterns to extract session path
    
    // Pattern 1: Direct session path format
    let re1 = Regex::new(r"/net/openvpn/v3/sessions/[a-zA-Z0-9_]+").ok()?;
    if let Some(m) = re1.find(output) {
        return Some(m.as_str().to_string());
    }
    
    // Pattern 2: Session path in "Path:" line
    let re2 = Regex::new(r"(?m)^\s*Path:\s*(/net/openvpn/v3/sessions/[a-zA-Z0-9_]+)").ok()?;
    if let Some(caps) = re2.captures(output) {
        return caps.get(1).map(|m| m.as_str().to_string());
    }
    
    // Pattern 3: Look for session identifier in different formats
    let re3 = Regex::new(r"[a-f0-9]{8}s[a-f0-9]{4}s[a-f0-9]{4}s[a-f0-9]{4}s[a-f0-9]{12}").ok()?;
    if let Some(m) = re3.find(output) {
        return Some(format!("/net/openvpn/v3/sessions/{}", m.as_str()));
    }
    
    None
}

/// Parse BYTES_IN and BYTES_OUT from openvpn3 session-stats output
pub fn parse_stats(output: &str) -> Option<(u64, u64)> {
    // Try multiple patterns for BYTES_IN and BYTES_OUT
    // OpenVPN3 uses dots as separators: "BYTES_IN.................1772584"
    let patterns_in = [
        r"BYTES_IN[.\s]+(\d+)",      // Dots and whitespace (OpenVPN3 format)
        r"BYTES_IN\s*:\s*(\d+)",      // Colon separator
        r"bytes_in\s*:\s*(\d+)",      // Lowercase with colon
        r"RX bytes\s*:\s*(\d+)",      // Alternative format
    ];
    
    let patterns_out = [
        r"BYTES_OUT[.\s]+(\d+)",     // Dots and whitespace (OpenVPN3 format)
        r"BYTES_OUT\s*:\s*(\d+)",    // Colon separator
        r"bytes_out\s*:\s*(\d+)",    // Lowercase with colon
        r"TX bytes\s*:\s*(\d+)",     // Alternative format
    ];
    
    let mut bytes_in = None;
    let mut bytes_out = None;
    
    // Try to find BYTES_IN
    for pattern in &patterns_in {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(output) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<u64>() {
                        bytes_in = Some(val);
                        break;
                    }
                }
            }
        }
    }
    
    // Try to find BYTES_OUT
    for pattern in &patterns_out {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(cap) = re.captures(output) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<u64>() {
                        bytes_out = Some(val);
                        break;
                    }
                }
            }
        }
    }
    
    // Return if both found
    if let (Some(bi), Some(bo)) = (bytes_in, bytes_out) {
        Some((bi, bo))
    } else {
        None
    }
}

/// Extract IP address from `ip addr show` output
pub fn extract_ip(output: &str) -> Option<String> {
    let re = Regex::new(r"inet\s+(\d+\.\d+\.\d+\.\d+)").ok()?;
    re.captures(output)?.get(1).map(|m| m.as_str().to_string())
}
