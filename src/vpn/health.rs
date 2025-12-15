use std::process::Command;

/// Ping a known endpoint and return latency in ms (None if failed)
pub async fn ping_latency() -> Option<u32> {
    // Use system ping for simplicity (Linux only)
    let output = Command::new("ping")
        .args(["-c", "1", "-w", "1", "8.8.8.8"]) // 1 packet, 1s timeout
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(idx) = line.find("time=") {
            let ms = &line[idx + 5..];
            if let Some(end) = ms.find(' ') {
                if let Ok(val) = ms[..end].parse::<f32>() {
                    return Some(val.round() as u32);
                }
            }
        }
    }
    None
}
