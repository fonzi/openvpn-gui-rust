// Main entry point for OpenVPN GUI with COSMIC DE integration

mod app;
mod models;
mod utils;
mod vpn;
mod ui;
mod icon;
mod tray;

use app::OpenVpnGui;

pub fn main() -> cosmic::iced::Result {
    cosmic::app::run::<OpenVpnGui>(Default::default(), ())
}

