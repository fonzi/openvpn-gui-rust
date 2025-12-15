// UI Components and View Logic

use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space};
use iced::{Color, Element, Length, Theme};

use crate::app::OpenVpnGui;
use crate::models::{ConnectionState, Message};
use crate::utils::format_bytes;
use crate::ui::NetworkGraph;

/// Main view function
pub fn view_main(app: &OpenVpnGui) -> Element<'_, Message> {
    let main_view = container(build_main_content(app));

    // About overlay
    if app.show_about {
        iced::widget::stack![main_view, build_about_modal(app)].into()
    } else if let Some(session_modal) = session_list_modal(&app.session_list, Message::CloseSessions) {
        iced::widget::stack![main_view, session_modal].into()
    } else {
        main_view.into()
    }
}

/// Build the main application content
fn build_main_content(app: &OpenVpnGui) -> Element<'_, Message> {
    let mut content = column![
        build_status_header(app),
        Space::new().height(10),
        build_config_selector(app),
        Space::new().height(10),
        build_controls(app),
        Space::new().height(10),
        build_options(app),
        Space::new().height(10),
        build_stats_display(app),
        Space::new().height(10),
    ]
    .padding(20);

    // 2FA Input (Conditional)
    if app.is_asking_2fa {
        content = content
            .push(build_auth_notice(app))
            .push(Space::new().height(10));
    }

    // Network Graph
    if app.show_graph {
        content = content
            .push(build_graph_container(app))
            .push(Space::new().height(5));
    }

    // Logs Area - make it fill the width, increase height
    content = content.push(
        build_logs_view(app)
    );

    content.into()
}

/// Status header with connection state and IPs
fn build_status_header(app: &OpenVpnGui) -> Element<'_, Message> {
    let status_color = match app.state {
        ConnectionState::Connected => Color::from_rgb8(76, 175, 80),
        ConnectionState::Connecting => Color::from_rgb8(255, 152, 0),
        _ => Color::from_rgb8(117, 117, 117),
    };

    let duration_text = if let Some(start) = app.connection_start {
        let elapsed = start.elapsed().as_secs();
        format!(
            "{:02}:{:02}:{:02}",
            elapsed / 3600,
            (elapsed % 3600) / 60,
            elapsed % 60
        )
    } else {
        String::new()
    };

    let row = row![
        text("●").size(24).color(status_color),
        text(format!("{:?}", app.state)).size(18),
        text(duration_text)
            .size(16)
            .color(Color::from_rgb8(158, 158, 158)),
        Space::new().width(Length::Fill),
        column![
            text(format!("Tunnel IP: {}", app.tunnel_ip)).size(12),
            text(format!("Public IP: {}", app.public_ip)).size(12),
        ]
        .spacing(2)
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    row.into()
}

/// Config file selector
fn build_config_selector(app: &OpenVpnGui) -> Element<'_, Message> {
    let config_row = row![
        text_input(
            "Select .ovpn config...",
            app.config_path.as_deref().unwrap_or("")
        )
        .on_input(|_| Message::BrowseConfig),
        button("Browse").on_press(Message::BrowseConfig),
    ]
    .spacing(10);

    // Add recent configs dropdown if we have any
    if !app.recent_configs.is_empty() {
        let recent_list = app
            .recent_configs
            .iter()
            .map(|path| {
                let display_name = std::path::Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path);
                
                button(text(display_name).size(12))
                    .on_press(Message::SelectRecentConfig(path.clone()))
                    .width(Length::Fill)
                    .into()
            })
            .collect::<Vec<Element<'_, Message>>>();

        // Fix recent configs container
        let (recent_bg, recent_fg, recent_border) = match app.theme() {
            Theme::Dark => (Color::from_rgb8(40, 40, 40), Color::WHITE, Color::from_rgb8(80, 80, 80)),
            Theme::Light => (Color::from_rgb8(245, 245, 245), Color::BLACK, Color::from_rgb8(200, 200, 200)),
            _ => (Color::from_rgb8(40, 40, 40), Color::WHITE, Color::from_rgb8(80, 80, 80)),
        };
        let recent_column: Element<'_, Message> = column(recent_list)
            .spacing(2)
            .into();
        let recent_container = container(
            column![
                row![
                    text("Recent:").size(12).color(recent_fg),
                    Space::new().width(Length::Fill),
                    button(text("Clear").size(10).color(recent_fg))
                        .on_press(Message::ClearRecentConfigs)
                        .padding(2)
                ]
                .align_y(iced::Alignment::Center),
                scrollable(recent_column)
                    .height(Length::Fixed(100.0))
            ]
            .spacing(5)
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(recent_bg)),
            border: iced::Border {
                color: recent_border,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .padding(10);

        column![config_row, recent_container]
            .spacing(10)
            .into()
    } else {
        config_row.into()
    }
}

/// Control buttons (Start/Stop, Kill Switch, etc.)
fn build_controls(app: &OpenVpnGui) -> Element<'_, Message> {
    let btn_label = if app.state == ConnectionState::Disconnected {
        "Start VPN"
    } else {
        "Disconnect"
    };

    let state = app.state; // Capture for closure
    row![
        button(btn_label)
            .style(move |theme: &Theme, status| {
                let color = if state == ConnectionState::Disconnected {
                    Color::from_rgb8(33, 150, 243)
                } else {
                    Color::from_rgb8(244, 67, 54)
                };
                button::Style {
                    background: Some(iced::Background::Color(color)),
                    text_color: Color::WHITE,
                    ..button::primary(theme, status)
                }
            })
            .on_press(Message::ToggleVpn),
        Space::new().width(Length::Fill),
        button("Save Logs").on_press(Message::SaveLogs),
        button("About").on_press(Message::ShowAbout),
        button("Export Session Report").on_press(Message::SaveSessionReport),
        show_sessions_button()
    ]
    .spacing(10)
    .into()
}

/// Settings checkboxes
fn build_options(app: &OpenVpnGui) -> Element<'_, Message> {
    row![
        checkbox(app.show_graph)
            .label("Show Graph")
            .on_toggle(Message::ToggleGraph),
        checkbox(app.auto_reconnect)
            .label("Auto-Reconnect")
            .on_toggle(Message::ToggleAutoReconnect),
    ]
    .spacing(20)
    .into()
}

/// Network statistics display
fn build_stats_display(app: &OpenVpnGui) -> Element<'_, Message> {
    row![
        text(format!("↓ {}/s", format_bytes(app.stats.rate_in)))
            .color(Color::from_rgb8(66, 165, 245)),
        text(format!("↑ {}/s", format_bytes(app.stats.rate_out)))
            .color(Color::from_rgb8(239, 83, 80)),
        text(format!(
            "Total: ↓ {} ↑ {}",
            format_bytes(app.stats.bytes_in as f32),
            format_bytes(app.stats.bytes_out as f32)
        )),
        Space::new().width(Length::Fill),
        if let Some(lat) = app.latency_ms {
            text(format!("Latency: {} ms", lat))
                .color(Color::from_rgb8(76, 175, 80))
                .size(14)
        } else {
            text("Latency: -- ms")
                .color(Color::from_rgb8(120, 120, 120))
                .size(14)
        },
    ]
    .spacing(20)
    .into()
}

/// Authentication notice with input
fn build_auth_notice(app: &OpenVpnGui) -> Element<'_, Message> {
    container(
        column![
            text("⚠ Authentication Required")
                .size(16)
                .color(Color::from_rgb8(255, 193, 7)),
            Space::new().height(5),
            row![
                text_input("Enter 2FA/Challenge Code", &app.input_code)
                    .on_input(Message::InputCodeChanged)
                    .padding(10),
                button("Submit").on_press(Message::SubmitCode).padding(10)
            ]
            .spacing(10),
        ]
        .spacing(5)
    )
    .style(|_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(60, 50, 40))),
        border: iced::Border {
            color: Color::from_rgb8(255, 193, 7),
            width: 2.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .padding(15)
    .into()
}

/// About modal dialog
fn build_about_modal<'a>(app: &OpenVpnGui) -> Element<'a, Message> {
    use iced::widget::pick_list;
    let theme_options = ["System", "Dark", "Light"];
    let current_theme = match app.theme_mode {
        Some(true) => "Dark",
        Some(false) => "Light",
        None => "System",
    };
    let (bg, fg, border) = match app.theme() {
        Theme::Dark => (Color::from_rgb8(40, 40, 40), Color::WHITE, Color::from_rgb8(100, 100, 100)),
        Theme::Light => (Color::from_rgb8(255, 255, 255), Color::BLACK, Color::from_rgb8(180, 180, 180)),
        _ => (Color::from_rgb8(40, 40, 40), Color::WHITE, Color::from_rgb8(100, 100, 100)),
    };
    let about_content = column![
        text("OpenVPN3 GUI").size(28).color(fg),
        Space::new().height(10),
        text("Version: 0.1.0").size(14).color(fg),
        Space::new().height(5),
        text("A modern GUI for OpenVPN3").size(13).color(fg),
        Space::new().height(12),
        text("Created by: fonzi").size(13).color(fg),
        text("Website: https://fonzi.xyz").size(13).color(Color::from_rgb8(66, 165, 245)),
        Space::new().height(15),
        text("Features:").size(16).color(fg),
        text("• Real-time network statistics").size(13).color(fg),
        text("• Live traffic graph").size(13).color(fg),
        text("• Recent configs memory").size(13).color(fg),
        text("• Auto-reconnect support").size(13).color(fg),
        text("• 2FA/Challenge-response authentication").size(13).color(fg),
        Space::new().height(15),
        row![
            text("Theme: ").size(14).color(fg),
            pick_list(theme_options, Some(current_theme), |selected| match selected {
                "Dark" => Message::SetTheme(Some(true)),
                "Light" => Message::SetTheme(Some(false)),
                _ => Message::SetTheme(None),
            })
        ]
        .spacing(10),
        Space::new().height(15),
        button("Close").on_press(Message::CloseAbout).padding(12)
    ]
    .padding(40)
    .spacing(5)
    .align_x(iced::Alignment::Start);

    container(about_content)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                color: border,
                width: 2.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .padding(20)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fixed(800.0))
        .height(Length::Fixed(600.0))
        .into()
}

/// Show sessions button
pub fn show_sessions_button<'a>() -> Element<'a, crate::models::Message> {
    button("Show Sessions")
        .on_press(crate::models::Message::ShowSessions)
        .into()
}

// Session list modal
pub fn session_list_modal<'a>(session_list: &'a Option<String>, on_close: crate::models::Message) -> Option<Element<'a, crate::models::Message>> {
    if let Some(list) = session_list {
        Some(
            container(
                column![
                    text("OpenVPN3 Sessions:").size(22),
                    scrollable(text(list).size(14).font(iced::Font::MONOSPACE)).height(Length::Fixed(300.0)),
                    button("Close").on_press(on_close)
                ]
                .spacing(10)
                .padding(10)
            )
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(40, 40, 40))),
                border: iced::Border {
                    color: Color::from_rgb8(100, 100, 100),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
        )
    } else {
        None
    }
}

/// Network graph container
fn build_graph_container(app: &OpenVpnGui) -> Element<'_, Message> {
    let (bg, border) = match app.theme() {
        Theme::Dark => (Color::from_rgb8(50, 50, 50), Color::from_rgb8(80, 80, 80)),
        Theme::Light => (Color::from_rgb8(255, 255, 255), Color::from_rgb8(200, 200, 200)),
        _ => (Color::from_rgb8(50, 50, 50), Color::from_rgb8(80, 80, 80)),
    };
    container(
        iced::widget::canvas(NetworkGraph {
            data_in: &app.graph_data_in,
            data_out: &app.graph_data_out,
        })
        .width(Length::Fill)
        .height(Length::Fixed(90.0))
    )
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border {
            color: border,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .padding(1)
    .into()
}

/// Logs view - a simple text area for logs
fn build_logs_view(app: &OpenVpnGui) -> Element<'_, Message> {
    let (bg, fg, border) = match app.theme() {
        Theme::Dark => (Color::from_rgb8(30, 30, 30), Color::WHITE, Color::from_rgb8(60, 60, 60)),
        Theme::Light => (Color::from_rgb8(255, 255, 255), Color::BLACK, Color::from_rgb8(200, 200, 200)),
        _ => (Color::from_rgb8(30, 30, 30), Color::WHITE, Color::from_rgb8(60, 60, 60)),
    };
    container(
        scrollable(
            text(app.logs.join("\n")).size(12).font(iced::Font::MONOSPACE).color(fg)
        )
        .height(Length::Fixed(320.0))
        .width(Length::Fill)
    )
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border {
            color: border,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .padding(10)
    .width(Length::Fill)
    .into()
}
