use crate::diagram::Object;
use crate::error::{DrawrsError, DrawrsResult};
use std::collections::HashMap;

pub struct BarChart {
    data: HashMap<String, f64>,
    position: [f64; 2],
    bar_width: f64,
    bar_spacing: f64,
    max_bar_height: f64,
    bar_colors: Vec<String>,
    pub objects: Vec<Object>,
}

impl BarChart {
    pub const DEFAULT_BAR_WIDTH: f64 = 40.0;
    pub const DEFAULT_BAR_SPACING: f64 = 20.0;
    pub const DEFAULT_MAX_BAR_HEIGHT: f64 = 200.0;

    pub fn new(data: HashMap<String, f64>) -> DrawrsResult<Self> {
        if data.is_empty() {
            return Err(DrawrsError::EmptyData);
        }

        // Validate all values are numeric (non-negative for bar charts)
        for (key, value) in &data {
            if value.is_nan() || value.is_infinite() || *value < 0.0 {
                return Err(DrawrsError::InvalidValue(key.clone(), value.to_string()));
            }
        }

        let bar_colors = vec!["#66ccff".to_string()];
        let mut chart = Self {
            data: data.clone(),
            position: [0.0, 0.0],
            bar_width: Self::DEFAULT_BAR_WIDTH,
            bar_spacing: Self::DEFAULT_BAR_SPACING,
            max_bar_height: Self::DEFAULT_MAX_BAR_HEIGHT,
            bar_colors,
            objects: Vec::new(),
        };

        chart.build_chart();
        Ok(chart)
    }

    pub fn data(&self) -> &HashMap<String, f64> {
        &self.data
    }

    pub fn position(&self) -> [f64; 2] {
        self.position
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn calculate_scale(&self) -> f64 {
        let max_value: f64 = self.data.values().fold(0.0f64, |acc: f64, &v| acc.max(v));
        if max_value == 0.0 {
            return 1.0;
        }
        self.max_bar_height / max_value
    }

    fn build_chart(&mut self) {
        self.objects.clear();
        let scale = self.calculate_scale();
        let mut x_offset = self.position[0];

        for (i, (label, value)) in self.data.iter().enumerate() {
            let bar_height = value * scale;
            let color = self
                .bar_colors
                .get(i % self.bar_colors.len())
                .cloned()
                .unwrap_or_else(|| "#66ccff".to_string());

            // Create bar
            let mut bar = Object::new(None);
            bar.set_position([
                x_offset,
                self.position[1] + self.max_bar_height - bar_height,
            ]);
            bar.set_width(self.bar_width);
            bar.set_height(bar_height);
            bar.set_fill_color(Some(color));
            bar.set_stroke_color(Some("#000000".to_string()));

            // Create label
            let mut label_obj = Object::new(None);
            label_obj.set_value(label.clone());
            label_obj.set_position([x_offset, self.position[1] + self.max_bar_height + 5.0]);
            label_obj.set_width(self.bar_width);
            label_obj.set_height(20.0);

            self.objects.push(bar);
            self.objects.push(label_obj);

            x_offset += self.bar_width + self.bar_spacing;
        }
    }

    pub fn update_data(&mut self, data: HashMap<String, f64>) -> DrawrsResult<()> {
        if data.is_empty() {
            return Err(DrawrsError::EmptyData);
        }
        self.data = data;
        self.build_chart();
        Ok(())
    }

    pub fn move_to(&mut self, position: [f64; 2]) {
        let delta_x = position[0] - self.position[0];
        let delta_y = position[1] - self.position[1];
        self.position = position;

        for obj in &mut self.objects {
            let pos = obj.position();
            obj.set_position([pos[0] + delta_x, pos[1] + delta_y]);
        }
    }
}
