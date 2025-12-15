// Network Graph Canvas Widget

use circular_queue::CircularQueue;
use iced::widget::canvas::{self, Path, Stroke};
use iced::{Color, Point, Rectangle, Theme};

use crate::models::Message;

pub const GRAPH_WINDOW: usize = 60; // 60 seconds history

pub struct NetworkGraph<'a> {
    pub data_in: &'a CircularQueue<f32>,
    pub data_out: &'a CircularQueue<f32>,
}

impl<'a> canvas::Program<Message> for NetworkGraph<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Background
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), Color::from_rgb8(35, 35, 35));

        if self.data_in.len() == 0 {
            return vec![frame.into_geometry()];
        }

        // Find max for scaling
        let max_in = self.data_in.iter().fold(0.0f32, |a, &b| a.max(b));
        let max_out = self.data_out.iter().fold(0.0f32, |a, &b| a.max(b));
        let global_max = max_in.max(max_out).max(1024.0); // Minimum scale 1KB

        // --- AXIS PADDING ---
        let y_axis_pad = 48.0; // px for Y axis (increased for label margin)
        let x_axis_pad = 28.0; // px for X axis (increased for label margin)
        let plot_width = bounds.width - y_axis_pad;
        let plot_height = bounds.height - x_axis_pad;
        let step_x = plot_width / GRAPH_WINDOW as f32;

        // Lower the lines by adding a vertical offset (e.g., 20% of plot height)
        let vertical_offset = plot_height * 0.18;

        // Helper function to create smooth curves using quadratic bezier
        let create_smooth_path = |data: &CircularQueue<f32>| {
            Path::new(|p| {
                let points: Vec<_> = data
                    .iter()
                    .enumerate()
                    .map(|(i, &val)| {
                        let x = y_axis_pad + i as f32 * step_x;
                        let y = plot_height - (val / global_max * (plot_height - vertical_offset)) + vertical_offset;
                        Point::new(x, y)
                    })
                    .collect();

                if points.is_empty() {
                    return;
                }

                if points.len() == 1 {
                    p.move_to(points[0]);
                    return;
                }

                p.move_to(points[0]);

                // Use quadratic bezier curves for smooth interpolation
                // The control point is positioned at the midpoint between consecutive points
                for i in 0..points.len() - 1 {
                    let current = points[i];
                    let next = points[i + 1];
                    
                    // Control point at the midpoint creates smooth flowing curves
                    let control = Point::new(
                        (current.x + next.x) / 2.0,
                        (current.y + next.y) / 2.0,
                    );
                    
                    // Draw smooth quadratic curve from current through control to next
                    p.quadratic_curve_to(control, next);
                }
            })
        };

        // Draw Download (Blue) with smooth curves
        let line_in = create_smooth_path(self.data_in);
        let has_in_data = self.data_in.iter().any(|&v| v > 0.0);
        if has_in_data {
            frame.stroke(
                &line_in,
                Stroke::default()
                    .with_color(Color::from_rgb8(66, 165, 245))
                    .with_width(2.0),
            );
        }

        // Draw Upload (Red) with smooth curves, only if there is nonzero data
        let line_out = create_smooth_path(self.data_out);
        let has_out_data = self.data_out.iter().any(|&v| v > 0.0);
        if has_out_data {
            frame.stroke(
                &line_out,
                Stroke::default()
                    .with_color(Color::from_rgb8(239, 83, 80))
                    .with_width(2.0),
            );
        }

        vec![frame.into_geometry()]
    }
}
