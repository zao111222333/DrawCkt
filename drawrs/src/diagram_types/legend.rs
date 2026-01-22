use crate::diagram::Object;
use crate::error::{DrawrsError, DrawrsResult};
use std::collections::HashMap;

pub struct Legend {
    mapping: HashMap<String, String>,
    position: [f64; 2],
    objects: Vec<Object>,
}

impl Legend {
    pub fn new(mapping: HashMap<String, String>) -> DrawrsResult<Self> {
        if mapping.is_empty() {
            return Err(DrawrsError::EmptyMapping);
        }

        let mut legend = Self {
            mapping: mapping.clone(),
            position: [0.0, 0.0],
            objects: Vec::new(),
        };

        legend.build_legend();
        Ok(legend)
    }

    fn build_legend(&mut self) {
        self.objects.clear();
        let mut y_offset = self.position[1];

        for (label, color) in &self.mapping {
            // Create color box
            let mut color_box = Object::new(None);
            color_box.set_position([self.position[0], y_offset]);
            color_box.set_width(20.0);
            color_box.set_height(20.0);
            color_box.set_fill_color(Some(color.clone()));
            color_box.set_stroke_color(Some("#000000".to_string()));

            // Create label
            let mut label_obj = Object::new(None);
            label_obj.set_value(label.clone());
            label_obj.set_position([self.position[0] + 25.0, y_offset]);
            label_obj.set_width(100.0);
            label_obj.set_height(20.0);

            self.objects.push(color_box);
            self.objects.push(label_obj);

            y_offset += 25.0;
        }
    }

    pub fn items(&self) -> usize {
        self.mapping.len()
    }

    pub fn position(&self) -> [f64; 2] {
        self.position
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

    pub fn update_mapping(&mut self, mapping: HashMap<String, String>) -> DrawrsResult<()> {
        if mapping.is_empty() {
            return Err(DrawrsError::EmptyMapping);
        }
        self.mapping = mapping;
        self.build_legend();
        Ok(())
    }
}
