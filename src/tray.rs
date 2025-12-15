// System tray integration using ksni (Wayland/KDE compatible)

use ksni;
use ksni::blocking::TrayMethods;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TrayState {
    pub connected: bool,
    pub tooltip: String,
}

pub struct SystemTray {
    state: Arc<Mutex<TrayState>>,
}

pub struct OpenvpnTray {
    state: Arc<Mutex<TrayState>>,
}

impl ksni::Tray for OpenvpnTray {
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        use image::ImageReader;
        use std::path::Path;
        // Load the 16x16 PNG icon from disk for tray
        let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/icons/openvpn3-gui-16.png");
        let img = ImageReader::open(Path::new(icon_path)).unwrap().decode().unwrap().to_rgba8();
        let (width, height) = img.dimensions();
        let rgba_data = img.into_raw();
        let mut argb_data = Vec::with_capacity(rgba_data.len());
        for chunk in rgba_data.chunks(4) {
            if chunk.len() == 4 {
                argb_data.push(chunk[3]); // A
                argb_data.push(chunk[0]); // R
                argb_data.push(chunk[1]); // G
                argb_data.push(chunk[2]); // B
            }
        }
        vec![ksni::Icon {
            width: width as i32,
            height: height as i32,
            data: argb_data,
        }]
    }

    fn title(&self) -> String {
        "OpenVPN3 GUI".into()
    }

    fn id(&self) -> String {
        "openvpn3-gui".into()
    }
    
    fn category(&self) -> ksni::Category {
        ksni::Category::ApplicationStatus
    }

    fn status(&self) -> ksni::Status {
        let state = self.state.lock().unwrap();
        if state.connected {
            ksni::Status::Active
        } else {
            ksni::Status::Passive
        }
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        use image::ImageReader;
        use std::path::Path;
        let icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/icons/openvpn3-gui-16.png");
        let img = ImageReader::open(Path::new(icon_path)).unwrap().decode().unwrap().to_rgba8();
        let (width, height) = img.dimensions();
        let rgba_data = img.into_raw();
        let mut argb_data = Vec::with_capacity(rgba_data.len());
        for chunk in rgba_data.chunks(4) {
            if chunk.len() == 4 {
                argb_data.push(chunk[3]); // A
                argb_data.push(chunk[0]); // R
                argb_data.push(chunk[1]); // G
                argb_data.push(chunk[2]); // B
            }
        }
        let state = self.state.lock().unwrap();
        ksni::ToolTip {
            icon_name: String::new(),
            icon_pixmap: vec![ksni::Icon {
                width: width as i32,
                height: height as i32,
                data: argb_data,
            }],
            title: state.tooltip.clone(),
            description: String::from("OpenVPN3 GUI"),
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Restore Window".into(),
                activate: Box::new(|_| {
                    // Window restoration handled by desktop environment
                }),
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }.into(),
        ]
    }
}

impl SystemTray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let state = Arc::new(Mutex::new(TrayState {
            connected: false,
            tooltip: "OpenVPN3 GUI - Disconnected".into(),
        }));

        let service = OpenvpnTray {
            state: state.clone(),
        };

        // Spawn the tray service in a separate thread with its own Tokio runtime
        std::thread::spawn(move || {
            // Use spawn_without_dbus_name to avoid issues in some environments
            if let Err(e) = service.spawn() {
                eprintln!("Failed to spawn tray service: {}", e);
            }
        });

        Ok(SystemTray { 
            state
        })
    }

    pub fn update_icon(&mut self, connected: bool) {
        let mut state = self.state.lock().unwrap();
        state.connected = connected;
    }

    pub fn update_tooltip(&mut self, text: &str) {
        let mut state = self.state.lock().unwrap();
        state.tooltip = text.to_string();
    }
}
