use crate::diagram::Object;
use crate::error::{DrawrsError, DrawrsResult};
use std::collections::HashMap;

pub struct PieChart {
    data: HashMap<String, f64>,
    position: [f64; 2],
    size: f64,
    slice_colors: Vec<String>,
    pub objects: Vec<Object>,
}

impl PieChart {
    pub const DEFAULT_SIZE: f64 = 200.0;
    pub const TITLE_BOTTOM_MARGIN: f64 = 20.0;
    pub const LABEL_OFFSET: f64 = 5.0;
    pub const BACKGROUND_PADDING: f64 = 20.0;

    pub fn new(data: HashMap<String, f64>) -> DrawrsResult<Self> {
        if data.is_empty() {
            return Err(DrawrsError::EmptyData);
        }

        // Validate all keys are strings and values are numeric
        for (key, value) in &data {
            if value.is_nan() || value.is_infinite() {
                return Err(DrawrsError::InvalidValue(key.clone(), value.to_string()));
            }
        }

        // Default colors for pie slices
        let default_colors = vec![
            "#66ccff".to_string(), // Blue
            "#ff6b6b".to_string(), // Red
            "#4ecdc4".to_string(), // Teal
            "#ffe66d".to_string(), // Yellow
            "#a8e6cf".to_string(), // Green
            "#ff8b94".to_string(), // Pink
            "#95e1d3".to_string(), // Mint
            "#f38181".to_string(), // Coral
        ];

        let slice_colors = default_colors;
        let mut chart = Self {
            data: data.clone(),
            position: [0.0, 0.0],
            size: Self::DEFAULT_SIZE,
            slice_colors,
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

    fn build_chart(&mut self) {
        self.objects.clear();
        let total: f64 = self.data.values().sum();

        if total == 0.0 {
            return;
        }

        let mut start_angle = 0.0;
        for (i, (label, value)) in self.data.iter().enumerate() {
            let fraction = value / total;
            let end_angle = start_angle + fraction;

            let color = self
                .slice_colors
                .get(i % self.slice_colors.len())
                .cloned()
                .unwrap_or_else(|| "#66ccff".to_string());

            // Create pie slice using mxgraph.basic.pie shape
            let mut slice = Object::new(None);
            slice.set_position(self.position);
            slice.set_width(self.size);
            slice.set_height(self.size);
            slice.set_fill_color(Some(color.clone()));
            slice.set_stroke_color(Some("#000000".to_string()));

            // Set shape to pie chart slice
            slice.set_shape("mxgraph.basic.pie".to_string());

            // Set start and end angles (normalized 0-1)
            slice.set_start_angle(start_angle);
            slice.set_end_angle(end_angle);

            // Set aspect to fixed
            slice.set_aspect("fixed".to_string());

            // Calculate label position (center of slice)
            // Python version: x and y are the top-left position of the pie
            // label_x = x + math.cos(theta) * offset
            // label_y = y + math.sin(theta) * offset
            // where x and y are the center of the pie, not top-left
            let mid_angle = start_angle + (fraction / 2.0);
            let theta = (mid_angle * 2.0 * std::f64::consts::PI) - (std::f64::consts::PI / 2.0);
            let offset = (self.size / 4.0) + Self::LABEL_OFFSET;
            // Calculate from center of pie
            let pie_center_x = self.position[0] + (self.size / 2.0);
            let pie_center_y = self.position[1] + (self.size / 2.0);
            let label_x = pie_center_x + theta.cos() * offset;
            let label_y = pie_center_y + theta.sin() * offset;

            // Create group object to contain slice and label
            let mut group = Object::new(None);
            group.set_position(self.position);
            group.set_width(self.size);
            group.set_height(self.size);
            group.set_fill_color(Some("none".to_string()));
            group.set_stroke_color(Some("none".to_string()));
            // Set group style attributes for grouping
            group.set_child_layout("stackLayout".to_string());
            group.set_resize_parent(0);
            group.set_resize_last(0);
            // Group needs to be a container
            group.set_container(1);

            let group_id = group.id().to_string();

            // Set slice parent to group (before adding to objects)
            slice.set_xml_parent(Some(group_id.clone()));

            // Create label
            let mut label_obj = Object::new(None);
            label_obj.set_value(label.clone());
            label_obj.set_position([label_x, label_y]);
            label_obj.set_width(60.0);
            label_obj.set_height(20.0);
            label_obj.set_fill_color(Some("none".to_string()));
            label_obj.set_stroke_color(Some("none".to_string()));
            // Set label parent to group
            label_obj.set_xml_parent(Some(group_id.clone()));

            // Add group first, then slice and label
            self.objects.push(group);
            self.objects.push(slice);
            self.objects.push(label_obj);

            start_angle = end_angle;
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
