// Utility functions

/// Format bytes into human-readable format (B, KB, MB)
pub fn format_bytes(num: f32) -> String {
    if num < 1024.0 { 
        format!("{:.0} B", num) 
    } else if num < 1048576.0 { 
        format!("{:.1} KB", num / 1024.0) 
    } else { 
        format!("{:.2} MB", num / 1048576.0) 
    }
}
