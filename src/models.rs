// Models and Message types

use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub rate_in: f32,  // Bytes/sec
    pub rate_out: f32, // Bytes/sec
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self { 
            bytes_in: 0, 
            bytes_out: 0, 
            rate_in: 0.0, 
            rate_out: 0.0 
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick(#[allow(dead_code)] Instant),
    BrowseConfig,
    ConfigPathSelected(Option<PathBuf>),
    ToggleVpn,
    
    // Async Results
    VpnStarted(Result<(String, String), String>), // (Output, SessionPath)
    VpnStopped(Result<String, String>),
    StatsUpdated(Option<(u64, u64)>), // (Total In, Total Out)
    SessionStatusChecked(Option<String>), // Session status output for monitoring
    TunnelIpFound(Option<String>),
    PublicIpFound(Option<String>),
    SaveLogs,
    SaveSessionReport,
    
    // UI Interaction
    ToggleGraph(bool),
    ToggleAutoReconnect(bool),
    InputCodeChanged(String),
    SubmitCode,
    AuthCodeResult(Result<String, String>),
    ShowAbout,
    CloseAbout,
    
    // Recent Files
    SelectRecentConfig(String),
    ClearRecentConfigs,
    
    // Session Management
    ShowSessions,
    SessionsListed(String),
    CloseSessions,

    // Latency update
    LatencyChecked(Option<u32>),
    SetTheme(Option<bool>), // None = system, Some(true) = dark, Some(false) = light
}
