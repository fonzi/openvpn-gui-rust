// Application state and business logic

use cosmic::iced::Subscription;
use cosmic::app::{Task, Core};
use cosmic::{Application, Element};
use std::time::{Duration, Instant};
use circular_queue::CircularQueue;
use notify_rust::Notification;

use crate::models::{ConnectionState, Message, NetworkStats};
use crate::vpn::{
    pick_file, start_vpn, stop_vpn_by_path, stop_vpn_by_config, 
    check_session_status, fetch_session_stats,
    find_tunnel_ip, fetch_public_ip, submit_challenge
};
use crate::ui::{view_main, GRAPH_WINDOW};
use crate::tray::SystemTray;
use crate::vpn::health::ping_latency;

/// The main application state
pub struct OpenVpnGui {
    pub core: Core,
    pub state: ConnectionState,
    pub config_path: Option<String>,
    pub session_path: Option<String>,
    pub logs: Vec<String>,
    
    // Recent configs
    pub recent_configs: Vec<String>,
    
    // Stats & Graphing
    pub stats: NetworkStats,
    pub graph_data_in: CircularQueue<f32>,
    pub graph_data_out: CircularQueue<f32>,
    pub show_graph: bool,
    
    // Connection Info
    pub connection_start: Option<Instant>,
    pub tunnel_ip: String,
    pub public_ip: String,
    
    // Auto-Reconnect
    pub auto_reconnect: bool,
    
    // 2FA / Input
    pub input_code: String,
    pub is_asking_2fa: bool,
    
    // About dialog
    pub show_about: bool,
    
    // System Tray
    pub tray: Option<SystemTray>,

    // Session List
    pub session_list: Option<String>,

    // Latency (used for health and stats)
    pub latency_ms: Option<u32>,
}

impl Default for OpenVpnGui {
    fn default() -> Self {
        let mut q_in = CircularQueue::with_capacity(GRAPH_WINDOW);
        let mut q_out = CircularQueue::with_capacity(GRAPH_WINDOW);
        // Fill with zeros
        for _ in 0..GRAPH_WINDOW { 
            q_in.push(0.0); 
            q_out.push(0.0); 
        }
        
        Self {
            core: Core::default(),
            state: ConnectionState::Disconnected,
            config_path: None,
            session_path: None,
            logs: vec!["Application started.".to_string()],
            recent_configs: Self::load_recent_configs(),
            stats: NetworkStats::default(),
            graph_data_in: q_in,
            graph_data_out: q_out,
            show_graph: true,
            connection_start: None,
            tunnel_ip: "-".to_string(),
            public_ip: "Checking...".to_string(),
            auto_reconnect: false,
            input_code: String::new(),
            is_asking_2fa: false,
            show_about: false,
            tray: SystemTray::new().ok(),
            session_list: None,
            latency_ms: None,
        }
    }
}

impl Application for OpenVpnGui {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "xyz.fonzi.openvpn3gui";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut app = Self::default();
        app.core = core;
        
        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick(_) => self.handle_tick(),
            Message::BrowseConfig => self.handle_browse_config(),
            Message::ConfigPathSelected(path_opt) => self.handle_config_selected(path_opt),
            Message::SelectRecentConfig(path) => self.handle_select_recent(path),
            Message::ClearRecentConfigs => self.handle_clear_recent(),
            Message::ToggleVpn => self.handle_toggle_vpn(),
            Message::VpnStarted(result) => self.handle_vpn_started(result),
            Message::VpnStopped(result) => self.handle_vpn_stopped(result),
            Message::StatsUpdated(stats_opt) => self.handle_stats_updated(stats_opt),
            Message::ToggleGraph(val) => self.handle_toggle_graph(val),
            Message::ToggleAutoReconnect(val) => self.handle_toggle_auto_reconnect(val),
            Message::SessionStatusChecked(status_opt) => self.handle_session_status(status_opt),
            Message::TunnelIpFound(ip) => self.handle_tunnel_ip(ip),
            Message::PublicIpFound(ip) => self.handle_public_ip(ip),
            Message::SaveLogs => self.handle_save_logs(),
            Message::InputCodeChanged(s) => self.handle_input_changed(s),
            Message::SubmitCode => self.handle_submit_code(),
            Message::AuthCodeResult(res) => self.handle_auth_result(res),
            Message::ShowAbout => self.handle_show_about(),
            Message::CloseAbout => self.handle_close_about(),
            Message::ShowSessions => {
                Task::perform(crate::vpn::manager::list_sessions(), |x| cosmic::Action::App(Message::SessionsListed(x)))
            }
            Message::SessionsListed(output) => {
                self.session_list = Some(output);
                Task::none()
            }
            Message::CloseSessions => {
                self.session_list = None;
                Task::none()
            }
            Message::SaveSessionReport => self.handle_save_session_report(),
            Message::LatencyChecked(lat) => self.handle_latency_checked(lat),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Run the tick every second
        cosmic::iced::time::every(Duration::from_secs(1)).map(Message::Tick)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        view_main(self)
    }
}

impl OpenVpnGui {

    pub fn log(&mut self, msg: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        let log_line = format!("[{}] {}", timestamp, msg);
        println!("{}", log_line); // Also print to CLI
        self.logs.push(log_line);
        // Keep logs manageable
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    pub fn cleanup_connection(&mut self) {
        self.state = ConnectionState::Disconnected;
        self.session_path = None;
        self.connection_start = None;
        self.tunnel_ip = "-".to_string();
        self.stats = NetworkStats::default();
        self.is_asking_2fa = false;
    }
}

// --- Message Handlers ---

impl OpenVpnGui {
    fn handle_tick(&mut self) -> Task<Message> {
        let mut cmds = Vec::new();
        
        // Update tray icon/tooltip
        self.update_tray();
        
        // 1. Monitor session status while connecting
        if self.state == ConnectionState::Connecting {
            if let Some(path) = &self.session_path {
                cmds.push(Task::perform(
                    check_session_status(path.clone()), 
                    |x| cosmic::Action::App(Message::SessionStatusChecked(x))
                ));
            }
        }
        
        // 2. Update Stats if connected
        if self.state == ConnectionState::Connected {
            if let Some(path) = &self.session_path {
                cmds.push(Task::perform(
                    fetch_session_stats(path.clone()), 
                    |x| cosmic::Action::App(Message::StatsUpdated(x))
                ));
            }
        }

        // 3. Check Tunnel IP occasionally
        if self.state == ConnectionState::Connected && self.tunnel_ip == "-" {
            cmds.push(Task::perform(find_tunnel_ip(), |x| cosmic::Action::App(Message::TunnelIpFound(x))));
        }
        
        // 4. Check Public IP on startup or when connecting
        if self.public_ip == "Checking..." {
            cmds.push(Task::perform(fetch_public_ip(), |x| cosmic::Action::App(Message::PublicIpFound(x))));
        }

        // 5. Ping for latency every tick (update live)
        cmds.push(Task::perform(ping_latency(), |x| cosmic::Action::App(Message::LatencyChecked(x))));
        Task::batch(cmds)
    }

    fn handle_browse_config(&mut self) -> Task<Message> {
        Task::perform(pick_file(), |x| cosmic::Action::App(Message::ConfigPathSelected(x)))
    }

    fn handle_config_selected(&mut self, path_opt: Option<std::path::PathBuf>) -> Task<Message> {
        if let Some(path) = path_opt {
            let path_str = path.to_string_lossy().to_string();
            self.config_path = Some(path_str.clone());
            self.add_to_recent_configs(path_str);
            self.log(format!("Selected config: {:?}", path));
        }
        Task::none()
    }

    fn handle_toggle_vpn(&mut self) -> Task<Message> {
        match self.state {
            ConnectionState::Disconnected => {
                if let Some(cfg) = self.config_path.clone() {
                    self.state = ConnectionState::Connecting;
                    self.log(format!("Starting VPN with {}", cfg));
                    return Task::perform(start_vpn(cfg), |x| cosmic::Action::App(Message::VpnStarted(x)));
                } else {
                    self.log("No config selected.".to_string());
                }
            }
            ConnectionState::Connected | ConnectionState::Connecting => {
                if let Some(path) = self.session_path.clone() {
                    self.log("Disconnecting...".to_string());
                    return Task::perform(stop_vpn_by_path(path), |x| cosmic::Action::App(Message::VpnStopped(x)));
                } else if let Some(cfg) = self.config_path.clone() {
                    return Task::perform(stop_vpn_by_config(cfg), |x| cosmic::Action::App(Message::VpnStopped(x)));
                }
            }
        }
        Task::none()
    }

    fn handle_vpn_started(&mut self, result: Result<(String, String), String>) -> Task<Message> {
        match result {
            Ok((output, session_path)) => {
                self.log("VPN session initiated. Waiting for authentication...".to_string());
                
                // Check if authentication is required
                if output.contains("CHALLENGE") || output.contains("password") || output.contains("Authentication") {
                    self.is_asking_2fa = true;
                    self.log("Authentication required - please enter your code".to_string());
                }
                
                // Check if SSO/web authentication is required
                if output.contains("AUTH_PENDING") || output.contains("Web based authentication") 
                    || output.contains("awaiting external authentication") {
                    self.log("Waiting for SSO authentication in browser...".to_string());
                }
                
                self.session_path = Some(session_path);
                self.state = ConnectionState::Connecting;
            }
            Err(e) => {
                self.log(format!("Failed to start: {}", e));
                self.state = ConnectionState::Disconnected;
            }
        }
        Task::none()
    }

    fn handle_vpn_stopped(&mut self, result: Result<String, String>) -> Task<Message> {
        match result {
            Ok(msg) => self.log(msg),
            Err(e) => self.log(format!("Error stopping: {}", e)),
        }
        // Show notification with icon path (16x16)
        let _ = Notification::new()
            .summary("OpenVPN3 GUI")
            .body("VPN Disconnected.")
            .icon(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/openvpn3-gui-16.png"))
            .show();
        self.cleanup_connection();
        Task::none()
    }

    fn handle_stats_updated(&mut self, stats_opt: Option<(u64, u64)>) -> Task<Message> {
        if let Some((total_in, total_out)) = stats_opt {
            // Calculate rates based on diff from previous
            let diff_in = if total_in >= self.stats.bytes_in { 
                total_in - self.stats.bytes_in 
            } else { 
                0 
            };
            let diff_out = if total_out >= self.stats.bytes_out { 
                total_out - self.stats.bytes_out 
            } else { 
                0 
            };
            
            self.stats.bytes_in = total_in;
            self.stats.bytes_out = total_out;
            self.stats.rate_in = diff_in as f32;
            self.stats.rate_out = diff_out as f32;

            // Update Graph
            self.graph_data_in.push(self.stats.rate_in);
            self.graph_data_out.push(self.stats.rate_out);
        }
        Task::none()
    }

    fn handle_toggle_graph(&mut self, val: bool) -> Task<Message> {
        self.show_graph = val;
        Task::none()
    }

    fn handle_toggle_auto_reconnect(&mut self, val: bool) -> Task<Message> {
        self.auto_reconnect = val;
        Task::none()
    }

    fn handle_session_status(&mut self, status_opt: Option<String>) -> Task<Message> {
        // Only process if we're in Connecting state
        if self.state != ConnectionState::Connecting {
            return Task::none();
        }
        
        if let Some(status) = status_opt {
            let status_lower = status.to_lowercase();
            
            // Check for successful connection
            if status_lower.contains("client connected") 
                || status_lower.contains("connection, client connected") {
                self.log("VPN Connected Successfully!".to_string());
                self.state = ConnectionState::Connected;
                self.connection_start = Some(Instant::now());
                // Show notification with icon path (16x16)
                let _ = Notification::new()
                    .summary("OpenVPN3 GUI")
                    .body("VPN Connected Successfully!")
                    .icon(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/openvpn3-gui-16.png"))
                    .show();
                // Also trigger IP checks
                return Task::batch(vec![
                    Task::perform(find_tunnel_ip(), |x| cosmic::Action::App(Message::TunnelIpFound(x))),
                    Task::perform(fetch_public_ip(), |x| cosmic::Action::App(Message::PublicIpFound(x))),
                ]);
            }
            
            // Check for authentication requirements
            if (status_lower.contains("challenge") || status_lower.contains("enter") && status_lower.contains("token"))
                && !self.is_asking_2fa {
                self.is_asking_2fa = true;
                self.log("2FA/Challenge required".to_string());
            }
            
            // Check for web authentication
            if status_lower.contains("auth_pending") 
                || status_lower.contains("web based authentication")
                || status_lower.contains("awaiting external authentication") {
                if !self.logs.iter().any(|l| l.contains("SSO authentication")) {
                    self.log("Complete SSO authentication in your browser...".to_string());
                }
            }
            
            // Check for failures
            if status_lower.contains("auth_failed") || status_lower.contains("authentication failed") {
                self.log("Authentication failed".to_string());
                self.state = ConnectionState::Disconnected;
                self.session_path = None;
            }
        }
        Task::none()
    }

    fn handle_tunnel_ip(&mut self, ip: Option<String>) -> Task<Message> {
        if let Some(ip) = ip {
            self.tunnel_ip = ip;
        }
        Task::none()
    }

    fn handle_public_ip(&mut self, ip: Option<String>) -> Task<Message> {
        if let Some(ip) = ip {
            self.public_ip = ip;
        }
        Task::none()
    }

    fn handle_save_logs(&mut self) -> Task<Message> {
        let logs_content = self.logs.join("\n");
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("openvpn_logs_{}.txt", timestamp);
        
        if let Err(e) = std::fs::write(&filename, logs_content) {
            self.log(format!("Failed to save logs: {}", e));
        } else {
            self.log(format!("Logs saved to: {}", filename));
        }
        Task::none()
    }

    fn handle_input_changed(&mut self, s: String) -> Task<Message> {
        self.input_code = s;
        Task::none()
    }

    fn handle_submit_code(&mut self) -> Task<Message> {
        if let Some(path) = self.session_path.clone() {
            let code_clone = self.input_code.clone();
            self.log(format!("Submitting challenge response: {}", code_clone));
            Task::perform(submit_challenge(path, code_clone), |x| cosmic::Action::App(Message::AuthCodeResult(x)))
        } else {
            Task::none()
        }
    }

    fn handle_auth_result(&mut self, res: Result<String, String>) -> Task<Message> {
        match res {
            Ok(out) => { 
                self.log(format!("Auth Result: {}", out)); 
                self.input_code.clear(); 
                self.is_asking_2fa = false; 
            }
            Err(e) => self.log(format!("Auth Error: {}", e)),
        }
        Task::none()
    }

    fn handle_show_about(&mut self) -> Task<Message> {
        self.show_about = true;
        Task::none()
    }

    fn handle_close_about(&mut self) -> Task<Message> {
        self.show_about = false;
        Task::none()
    }

    fn handle_select_recent(&mut self, path: String) -> Task<Message> {
        self.config_path = Some(path.clone());
        self.add_to_recent_configs(path);
        self.log(format!("Selected recent config: {}", self.config_path.as_ref().unwrap()));
        Task::none()
    }

    fn handle_clear_recent(&mut self) -> Task<Message> {
        self.recent_configs.clear();
        Self::save_recent_configs(&self.recent_configs);
        self.log("Recent configs cleared".to_string());
        Task::none()
    }

    // Helper methods for recent configs
    fn load_recent_configs() -> Vec<String> {
        let config_path = Self::get_config_file_path();
        if let Ok(contents) = std::fs::read_to_string(&config_path) {
            contents
                .lines()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .take(10) // Keep only last 10
                .collect()
        } else {
            Vec::new()
        }
    }

    fn save_recent_configs(configs: &[String]) {
        let config_path = Self::get_config_file_path();
        let contents = configs.join("\n");
        let _ = std::fs::write(&config_path, contents);
    }

    fn get_config_file_path() -> std::path::PathBuf {
        if let Some(mut path) = dirs::config_dir() {
            path.push("openvpn-gui");
            std::fs::create_dir_all(&path).ok();
            path.push("recent_configs.txt");
            path
        } else {
            std::path::PathBuf::from("recent_configs.txt")
        }
    }

    fn add_to_recent_configs(&mut self, path: String) {
        // Remove if already exists (to move it to top)
        self.recent_configs.retain(|p| p != &path);
        
        // Add to front
        self.recent_configs.insert(0, path);
        
        // Keep only last 10
        self.recent_configs.truncate(10);
        
        // Save to disk
        Self::save_recent_configs(&self.recent_configs);
    }
    
    fn update_tray(&mut self) {
        if let Some(ref mut tray) = self.tray {
            // Update icon based on connection state
            let connected = self.state == ConnectionState::Connected;
            tray.update_icon(connected);
            
            // Update tooltip with status
            let tooltip = match self.state {
                ConnectionState::Connected => {
                    if let Some(start) = self.connection_start {
                        let duration = start.elapsed();
                        let mins = duration.as_secs() / 60;
                        let secs = duration.as_secs() % 60;
                        format!("OpenVPN3 GUI - Connected ({}:{:02})", mins, secs)
                    } else {
                        "OpenVPN3 GUI - Connected".to_string()
                    }
                }
                ConnectionState::Connecting => "OpenVPN3 GUI - Connecting...".to_string(),
                ConnectionState::Disconnected => "OpenVPN3 GUI - Disconnected".to_string(),
            };
            tray.update_tooltip(&tooltip);
        }
    }

    fn handle_save_session_report(&mut self) -> Task<Message> {
        // Compose session report
        let config = self.config_path.clone().unwrap_or_else(|| "-".to_string());
        let duration = if let Some(start) = self.connection_start {
            let elapsed = start.elapsed().as_secs();
            format!("{:02}:{:02}:{:02}", elapsed / 3600, (elapsed % 3600) / 60, elapsed % 60)
        } else {
            "-".to_string()
        };
        let tunnel_ip = &self.tunnel_ip;
        let public_ip = &self.public_ip;
        let stats = &self.stats;
        let log_excerpt = self.logs.iter().rev().take(20).cloned().collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
        let report = format!(
            "OpenVPN3 Session Report\n\
            Config: {}\n\
            Duration: {}\n\
            Tunnel IP: {}\n\
            Public IP: {}\n\
            Bytes In: {}\n\
            Bytes Out: {}\n\
            Log Excerpt:\n{}\n",
            config, duration, tunnel_ip, public_ip, stats.bytes_in, stats.bytes_out, log_excerpt
        );
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("openvpn_session_report_{}.txt", timestamp);
        if let Err(e) = std::fs::write(&filename, report) {
            self.log(format!("Failed to save session report: {}", e));
        } else {
            // Get absolute path
            let abs_path = std::env::current_dir()
                .map(|p| p.join(&filename))
                .unwrap_or_else(|_| std::path::PathBuf::from(&filename));
            self.log(format!("Session report saved to: {}", abs_path.display()));
        }
        Task::none()
    }

    // Add a new message handler for latency
    fn handle_latency_checked(&mut self, lat: Option<u32>) -> Task<Message> {
        self.latency_ms = lat;
        Task::none()
    }
}
