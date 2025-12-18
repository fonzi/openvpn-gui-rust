// UI Components and View Logic

use cosmic::iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input};
use cosmic::iced::{Color, Length};
use cosmic::widget::Space;
use cosmic::Element;

use crate::app::OpenVpnGui;
use crate::models::{ConnectionState, Message};
use crate::utils::format_bytes;
use crate::ui::NetworkGraph;

/// Main view function
pub fn view_main(app: &OpenVpnGui) -> Element<'_, Message> {
    let main_view = container(build_main_content(app));

    // About overlay
    if app.show_about {
        cosmic::iced::widget::stack![main_view, build_about_modal(app)].into()
    } else if let Some(session_modal) = session_list_modal(&app.session_list, Message::CloseSessions) {
        cosmic::iced::widget::stack![main_view, session_modal].into()
    } else {
        main_view.into()
    }
}

/// Build the main application content
fn build_main_content(app: &OpenVpnGui) -> Element<'_, Message> {
    let mut content = column![
        build_status_header(app),
        Space::with_height(Length::Fixed(10.0)),
        build_config_selector(app),
        Space::with_height(Length::Fixed(10.0)),
        build_controls(app),
        Space::with_height(Length::Fixed(10.0)),
        build_options(app),
        Space::with_height(Length::Fixed(10.0)),
        build_stats_display(app),
        Space::with_height(Length::Fixed(10.0)),
    ]
    .padding(20);

    // 2FA Input (Conditional)
    if app.is_asking_2fa {
        content = content
            .push(build_auth_notice(app))
            .push(Space::with_height(Length::Fixed(10.0)));
    }

    // Network Graph
    if app.show_graph {
        content = content
            .push(
                plotters_iced::ChartWidget::new(NetworkGraph {
                    data_in: &app.graph_data_in,
                    data_out: &app.graph_data_out,
                })
                .width(Length::Fill)
                .height(Length::Fixed(90.0))
            )
            .push(Space::with_height(Length::Fixed(5.0)));
    }

    // Logs Area - using text widget
    content = content.push(
        text(app.logs.join("\n")).size(12).font(cosmic::iced::Font::MONOSPACE)
    );

    content.into()
}

/// Status header with connection state and IPs
fn build_status_header(app: &OpenVpnGui) -> Element<'_, Message> {
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
        text("●").size(24),
        text(format!("{:?}", app.state)).size(18),
        text(duration_text)
            .size(16),
        Space::with_width(Length::Fill),
        column![
            text(format!("Tunnel IP: {}", app.tunnel_ip)).size(12),
            text(format!("Public IP: {}", app.public_ip)).size(12),
        ]
        .spacing(2)
    ]
    .spacing(10)
    .align_y(cosmic::iced::Alignment::Center);

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

        // Fix recent configs container - COSMIC handles theming automatically
        let recent_border = Color::from_rgb8(80, 80, 80);
        let recent_column: Element<'_, Message> = column(recent_list)
            .spacing(2)
            .into();
        let recent_container = container(
            column![
                row![
                    text("Recent:").size(12),
                    Space::with_width(Length::Fill),
                    button(text("Clear").size(10))
                        .on_press(Message::ClearRecentConfigs)
                        .padding(2)
                ]
                .align_y(cosmic::iced::Alignment::Center),
                scrollable(recent_column)
                    .height(Length::Fixed(100.0))
            ]
            .spacing(5)
        )
                .style(move |_theme| container::Style {
            background: Some(cosmic::iced::Background::Color(Color::from_rgb8(40, 40, 40))),
            border: cosmic::iced::Border {
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

    row![
        button(btn_label)
            .on_press(Message::ToggleVpn),
        Space::with_width(Length::Fill),
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
        checkbox("Show Graph", app.show_graph)
            .on_toggle(Message::ToggleGraph),
        checkbox("Auto-Reconnect", app.auto_reconnect)
            .on_toggle(Message::ToggleAutoReconnect),
    ]
    .spacing(20)
    .into()
}

/// Network statistics display
fn build_stats_display(app: &OpenVpnGui) -> Element<'_, Message> {
    row![
        text(format!("↓ {}/s", format_bytes(app.stats.rate_in))),
        text(format!("↑ {}/s", format_bytes(app.stats.rate_out))),
        text(format!(
            "Total: ↓ {} ↑ {}",
            format_bytes(app.stats.bytes_in as f32),
            format_bytes(app.stats.bytes_out as f32)
        )),
        Space::with_width(Length::Fill),
        if let Some(lat) = app.latency_ms {
            text(format!("Latency: {} ms", lat))
                .size(14)
        } else {
            text("Latency: -- ms")
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
                .size(16),
            Space::with_height(Length::Fixed(5.0)),
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
    .style(|_theme| container::Style {
        background: Some(cosmic::iced::Background::Color(Color::from_rgb8(60, 50, 40))),
        border: cosmic::iced::Border {
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
fn build_about_modal<'a>(_app: &OpenVpnGui) -> Element<'a, Message> {
    // COSMIC handles themes automatically through system settings
    let bg = Color::from_rgb8(40, 40, 40);
    let border = Color::from_rgb8(100, 100, 100);
    
    let about_content = column![
        text("OpenVPN3 GUI").size(28),
        Space::with_height(Length::Fixed(10.0)),
        text("Version: 0.1.0").size(14),
        Space::with_height(Length::Fixed(5.0)),
        text("A modern GUI for OpenVPN3").size(13),
        Space::with_height(Length::Fixed(12.0)),
        text("Created by: fonzi").size(13),
        text("Website: https://fonzi.xyz").size(13),
        Space::with_height(Length::Fixed(15.0)),
        text("Features:").size(16),
        text("• Real-time network statistics").size(13),
        text("• Live traffic graph").size(13),
        text("• Recent configs memory").size(13),
        text("• Auto-reconnect support").size(13),
        text("• 2FA/Challenge-response authentication").size(13),
        Space::with_height(Length::Fixed(15.0)),
        text("Powered by COSMIC DE").size(13),
        Space::with_height(Length::Fixed(15.0)),
        button("Close").on_press(Message::CloseAbout).padding(12)
    ]
    .padding(40)
    .spacing(5)
    .align_x(cosmic::iced::Alignment::Start);

    container(about_content)
        .style(move |_theme| container::Style {
            background: Some(cosmic::iced::Background::Color(bg)),
            border: cosmic::iced::Border {
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
                    scrollable(text(list).size(14).font(cosmic::iced::Font::MONOSPACE)).height(Length::Fixed(300.0)),
                    button("Close").on_press(on_close)
                ]
                .spacing(10)
                .padding(10)
            )
            .style(|_theme| container::Style {
                background: Some(cosmic::iced::Background::Color(Color::from_rgb8(40, 40, 40))),
                border: cosmic::iced::Border {
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
