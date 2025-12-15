// Icon generation for window and system tray

use iced::window;
use image::{ImageBuffer, Rgba, RgbaImage};

/// Create a window icon (32x32) with a VPN shield design
pub fn create_window_icon() -> Option<window::Icon> {
    let icon_data = create_icon_rgba(32);
    window::icon::from_rgba(icon_data.clone(), 32, 32).ok()
}

/// Create a tray icon (32x32) for the system tray with connection status
#[allow(dead_code)]
pub fn create_tray_icon(_connected: bool) -> Vec<u8> {
    create_icon_rgba(32)
}

/// Create a tray icon image buffer for the tray-icon crate
pub fn create_tray_icon_image(connected: bool) -> RgbaImage {
    let size = 16; // Use 16x16 for tray icon
    let mut img = ImageBuffer::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let pixel = create_pixel(x, y, size, connected);
            img.put_pixel(x, y, Rgba(pixel));
        }
    }
    img
}

/// Create the icon RGBA data
fn create_icon_rgba(size: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let pixel = create_pixel(x, y, size, false);
            rgba[idx] = pixel[0];
            rgba[idx + 1] = pixel[1];
            rgba[idx + 2] = pixel[2];
            rgba[idx + 3] = pixel[3];
        }
    }
    
    rgba
}

/// Create a pixel for the VPN key/lock icon
fn create_pixel(x: u32, y: u32, size: u32, connected: bool) -> [u8; 4] {
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0 - 1.0; // Slightly up
    let dx = x as f32 - cx;
    let dy = y as f32 - cy;

    // Key shape: circle (head) + rectangle (shaft) + small rectangle (tooth)
    let key_head_radius = size as f32 * 0.13; // Scaled for larger icon
    let key_shaft_width = size as f32 * 0.045;
    let key_shaft_length = size as f32 * 0.32;
    let key_tooth_width = size as f32 * 0.045;
    let key_tooth_length = size as f32 * 0.09;

    // Key head (circle)
    let in_key_head = (dx * dx + dy * dy).sqrt() < key_head_radius;

    // Key shaft (rectangle to the right of the head)
    let in_key_shaft = dx > 0.0 && dx < key_shaft_length && dy.abs() < key_shaft_width;

    // Key tooth (small rectangle at the end of the shaft)
    let in_key_tooth = dx > key_shaft_length - key_tooth_length && dx < key_shaft_length
        && dy > 0.0 && dy < key_tooth_width;

    if in_key_head || in_key_shaft || in_key_tooth {
        // Key color: green if connected, gray if not
        let base_color = if connected {
            [60, 220, 60] // Green when connected
        } else {
            [180, 180, 180] // Gray when disconnected
        };
        [base_color[0], base_color[1], base_color[2], 255]
    } else {
        [0, 0, 0, 0] // Transparent background
    }
}
