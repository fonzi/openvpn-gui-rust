// Network Graph with Plotters

use circular_queue::CircularQueue;
use plotters::prelude::*;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingBackend};

use crate::models::Message;

pub const GRAPH_WINDOW: usize = 60; // 60 seconds history

pub struct NetworkGraph<'a> {
    pub data_in: &'a CircularQueue<f32>,
    pub data_out: &'a CircularQueue<f32>,
}

impl<'a> Chart<Message> for NetworkGraph<'a> {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {

        // Find max for scaling
        let max_in = self.data_in.iter().fold(0.0f32, |a, &b| a.max(b));
        let max_out = self.data_out.iter().fold(0.0f32, |a, &b| a.max(b));
        let global_max = max_in.max(max_out).max(1024.0); // Minimum scale 1KB

        // Build chart
        let mut chart = builder
            .x_label_area_size(0)
            .y_label_area_size(0)
            .build_cartesian_2d(0.0..GRAPH_WINDOW as f32, 0.0..global_max)
            .unwrap();

        // Draw download line (blue)
        let data_in_points: Vec<(f32, f32)> = self.data_in
            .iter()
            .enumerate()
            .map(|(i, &val)| (i as f32, val))
            .collect();
        
        chart.draw_series(LineSeries::new(
            data_in_points,
            &RGBColor(66, 165, 245),
        )).ok();

        // Draw upload line (red)
        let data_out_points: Vec<(f32, f32)> = self.data_out
            .iter()
            .enumerate()
            .map(|(i, &val)| (i as f32, val))
            .collect();
        
        chart.draw_series(LineSeries::new(
            data_out_points,
            &RGBColor(239, 83, 80),
        )).ok();
    }
}
