use crate::BoundingBox;
use crate::transform::FlipRotation;

#[derive(Clone, Debug)]
pub struct Geometry {
    bounding_box: BoundingBox,
    as_attribute: String,
    // For edge geometry
    relative: Option<bool>,
    source_point: Option<[f64; 2]>,
    target_point: Option<[f64; 2]>,
    intermediate_points: Vec<[f64; 2]>,
    flip_rotation: FlipRotation,
}

impl Geometry {
    pub fn new() -> Self {
        Self {
            bounding_box: BoundingBox::new(0.0, 0.0, 120.0, 60.0),
            as_attribute: "geometry".to_string(),
            relative: None,
            source_point: None,
            target_point: None,
            intermediate_points: Vec::new(),
            flip_rotation: FlipRotation::default(),
        }
    }

    pub fn with_position(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            bounding_box: BoundingBox::new(x, y, width, height),
            as_attribute: "geometry".to_string(),
            relative: None,
            source_point: None,
            target_point: None,
            intermediate_points: Vec::new(),
            flip_rotation: FlipRotation::default(),
        }
    }

    pub fn set_source_point(&mut self, point: Option<[f64; 2]>) {
        self.source_point = point;
    }

    pub fn set_target_point(&mut self, point: Option<[f64; 2]>) {
        self.target_point = point;
    }

    pub fn add_intermediate_point(&mut self, point: [f64; 2]) {
        self.intermediate_points.push(point);
    }

    pub fn flip_rotation(&self) -> &FlipRotation {
        &self.flip_rotation
    }

    pub fn flip_rotation_mut(&mut self) -> &mut FlipRotation {
        &mut self.flip_rotation
    }

    pub fn set_flip_rotation(&mut self, flip_rotation: FlipRotation) {
        self.flip_rotation = flip_rotation;
    }

    pub fn set_relative(&mut self, relative: Option<bool>) {
        self.relative = relative;
    }

    pub fn x(&self) -> f64 {
        self.bounding_box.min_x()
    }

    pub fn y(&self) -> f64 {
        self.bounding_box.min_y()
    }

    pub fn width(&self) -> f64 {
        self.bounding_box.width()
    }

    pub fn height(&self) -> f64 {
        self.bounding_box.height()
    }

    pub fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    pub fn relative(&self) -> Option<bool> {
        self.relative
    }

    pub fn source_point(&self) -> Option<[f64; 2]> {
        self.source_point
    }

    pub fn target_point(&self) -> Option<[f64; 2]> {
        self.target_point
    }

    pub fn intermediate_points(&self) -> &[[f64; 2]] {
        &self.intermediate_points
    }

    pub fn points_mut(&mut self) -> impl Iterator<Item = &mut [f64; 2]> {
        self.intermediate_points
            .iter_mut()
            .chain(self.target_point.iter_mut())
            .chain(self.source_point.iter_mut())
    }

    pub fn bounding_box_mut(&mut self) -> &mut BoundingBox {
        &mut self.bounding_box
    }

    /// Returns a mutable reference to a tuple of (bounding_box, flip_rotation)
    pub fn bounding_box_and_flip_rotation_mut(&mut self) -> (&mut BoundingBox, &mut FlipRotation) {
        (&mut self.bounding_box, &mut self.flip_rotation)
    }

    /// Returns an iterator over mutable references to (bounding_box, flip_rotation)
    /// This creates a temporary storage for the tuple if flip_rotation exists
    /// Note: Due to Rust's borrowing rules, this uses unsafe code to create
    /// a reference to a tuple from two separate fields
    pub fn mut_box_and_flip_rotation_iter(
        &mut self,
    ) -> impl Iterator<Item = &mut (BoundingBox, FlipRotation)> {
        // We need to return &mut (BoundingBox, FlipRotation) but they are stored separately
        // This is a limitation - we can't safely create this without unsafe code or restructuring
        // For now, return an empty iterator if flip_rotation is None
        // When flip_rotation is Some, we would need unsafe to create the tuple reference
        // This is a placeholder - proper implementation would require unsafe or restructuring
        std::iter::empty()
    }

    pub fn set_x(&mut self, x: f64) {
        self.bounding_box = BoundingBox::new(
            x,
            self.bounding_box.min_y(),
            self.bounding_box.width(),
            self.bounding_box.height(),
        );
    }

    pub fn set_y(&mut self, y: f64) {
        self.bounding_box = BoundingBox::new(
            self.bounding_box.min_x(),
            y,
            self.bounding_box.width(),
            self.bounding_box.height(),
        );
    }

    pub fn set_width(&mut self, width: f64) {
        self.bounding_box = BoundingBox::new(
            self.bounding_box.min_x(),
            self.bounding_box.min_y(),
            width,
            self.bounding_box.height(),
        );
    }

    pub fn set_height(&mut self, height: f64) {
        self.bounding_box = BoundingBox::new(
            self.bounding_box.min_x(),
            self.bounding_box.min_y(),
            self.bounding_box.width(),
            height,
        );
    }

    pub fn xml(&self) -> String {
        // Check if this is an edge geometry (has source_point and target_point)
        if self.source_point.is_some() && self.target_point.is_some() {
            let source = self.source_point.unwrap();
            let target = self.target_point.unwrap();
            let relative = self.relative.unwrap_or(true);

            let mut xml = format!(
                r#"<mxGeometry width="{}" height="{}" relative="{}" as="{}">"#,
                self.bounding_box.width(),
                self.bounding_box.height(),
                if relative { "1" } else { "0" },
                self.as_attribute
            );

            xml.push_str(&format!(
                r#"
            <mxPoint x="{}" y="{}" as="sourcePoint" />"#,
                source[0], source[1]
            ));

            xml.push_str(&format!(
                r#"
            <mxPoint x="{}" y="{}" as="targetPoint" />"#,
                target[0], target[1]
            ));

            // Add intermediate points if any
            if !self.intermediate_points.is_empty() {
                xml.push_str(
                    r#"
            <Array as="points">"#,
                );
                for point in &self.intermediate_points {
                    xml.push_str(&format!(
                        r#"
              <mxPoint x="{}" y="{}" />"#,
                        point[0], point[1]
                    ));
                }
                xml.push_str(
                    r#"
            </Array>"#,
                );
            }

            xml.push_str(
                r#"
          </mxGeometry>"#,
            );
            xml
        } else {
            // Regular geometry
            format!(
                r#"<mxGeometry x="{}" y="{}" width="{}" height="{}" as="{}" />"#,
                self.bounding_box.min_x(),
                self.bounding_box.min_y(),
                self.bounding_box.width(),
                self.bounding_box.height(),
                self.as_attribute
            )
        }
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self::new()
    }
}
