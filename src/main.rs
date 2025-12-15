// Main entry point for OpenVPN GUI

use iced::window;
use iced::Size;

mod app;
mod models;
mod utils;
mod vpn;
mod ui;
mod icon;
mod tray;

use app::OpenVpnGui;
use icon::create_window_icon;

pub fn main() -> iced::Result {
    iced::application(OpenVpnGui::new, OpenVpnGui::update, OpenVpnGui::view)
        .subscription(OpenVpnGui::subscription)
        .theme(OpenVpnGui::theme)
        .window(window::Settings {
            size: Size::new(800.0, 600.0),
            min_size: Some(Size::new(600.0, 400.0)),
            icon: create_window_icon(),
            ..Default::default()
        })
        .run()
}

