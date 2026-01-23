use crate::XMLBase;
use crate::diagram::base_diagram::DiagramBase;
use crate::diagram::geometry::Geometry;
use crate::diagram::text_format::TextFormat;

#[derive(Clone, Debug)]
pub struct Object {
    base: DiagramBase,
    geometry: Geometry,
    white_space: Option<String>,
    rounded: Option<bool>,
    fill_color: Option<String>,
    stroke_color: Option<String>,
    stroke_width: Option<f64>,
    opacity: Option<i32>,
    // glass: Option<bool>,
    // shadow: Option<bool>,
    // line_pattern: Option<String>,
    text_format: TextFormat,
    vertex: i32,
    poly_coords: Vec<[f64; 2]>, // Polygon coordinates as normalized (0-1) points relative to bounding box
}

impl Object {
    pub fn new(id: Option<String>) -> Self {
        let mut base = DiagramBase::new(id);
        base.add_style_attribute("rounded".to_string());
        base.add_style_attribute("fillColor".to_string());
        base.add_style_attribute("strokeColor".to_string());
        base.add_style_attribute("opacity".to_string());

        let mut geom = Geometry::new();
        geom.set_width(120.0);
        geom.set_height(80.0);
        let mut obj = Self {
            base,
            geometry: geom,
            white_space: None,
            rounded: None,
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            opacity: None,
            // glass: None,
            // shadow: None,
            // line_pattern: Some("solid".to_string()),
            text_format: TextFormat::new(),
            vertex: 1,
            poly_coords: Vec::new(),
        };

        obj.update_style();
        obj
    }
    pub fn points_mut(&mut self) -> impl Iterator<Item = &mut [f64; 2]> {
        self.geometry.points_mut()
    }
    pub fn id(&self) -> &str {
        self.base.id()
    }

    pub fn set_id(&mut self, id: String) {
        self.base.set_id(id);
    }

    pub fn xml_parent(&self) -> Option<&String> {
        self.base.xml_parent()
    }

    pub fn value(&self) -> Option<&String> {
        self.base().value.as_ref()
    }
    pub fn base(&self) -> &XMLBase {
        self.base.base()
    }
    pub fn base_mut(&mut self) -> &mut XMLBase {
        self.base.base_mut()
    }

    pub fn set_value(&mut self, value: String) {
        self.base_mut().value = Some(value);
    }

    pub fn value_mut(&mut self) -> Option<&mut String> {
        self.base_mut().value.as_mut()
    }

    pub fn position(&self) -> [f64; 2] {
        [self.geometry.x(), self.geometry.y()]
    }

    pub fn set_position(&mut self, position: [f64; 2]) {
        self.geometry.set_x(position[0]);
        self.geometry.set_y(position[1]);
    }

    pub fn width(&self) -> f64 {
        self.geometry.width()
    }

    pub fn set_width(&mut self, width: f64) {
        self.geometry.set_width(width);
    }

    pub fn height(&self) -> f64 {
        self.geometry.height()
    }

    pub fn set_height(&mut self, height: f64) {
        self.geometry.set_height(height);
    }

    pub fn geometry_mut(&mut self) -> &mut Geometry {
        &mut self.geometry
    }

    pub fn fill_color(&self) -> Option<&String> {
        self.fill_color.as_ref()
    }

    pub fn set_fill_color(&mut self, color: Option<String>) {
        self.fill_color = color;
        self.update_style();
    }

    pub fn stroke_color(&self) -> Option<&String> {
        self.stroke_color.as_ref()
    }

    pub fn set_stroke_color(&mut self, color: Option<String>) {
        self.stroke_color = color;
        self.update_style();
    }

    pub fn stroke_width(&self) -> Option<f64> {
        self.stroke_width
    }

    pub fn set_stroke_width(&mut self, width: Option<f64>) {
        self.stroke_width = width;
        self.update_style();
    }

    pub fn poly_coords(&self) -> &Vec<[f64; 2]> {
        &self.poly_coords
    }

    pub fn poly_coords_mut(&mut self) -> &mut Vec<[f64; 2]> {
        &mut self.poly_coords
    }

    pub fn set_poly_coords(&mut self, coords: Vec<[f64; 2]>) {
        self.poly_coords = coords;
        self.update_style();
    }

    pub fn font_color(&self) -> Option<&String> {
        self.text_format.font_color()
    }

    pub fn set_font_color(&mut self, color: Option<String>) {
        self.text_format.set_font_color(color);
        self.update_style();
    }

    pub fn font_size(&self) -> Option<f64> {
        self.text_format.font_size()
    }

    pub fn set_font_size(&mut self, size: Option<f64>) {
        self.text_format.set_font_size(size);
        self.update_style();
    }

    pub fn rounded(&self) -> Option<bool> {
        self.rounded
    }

    pub fn set_rounded(&mut self, rounded: Option<bool>) {
        self.rounded = rounded;
        self.update_style();
    }

    pub fn opacity(&self) -> Option<i32> {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: Option<i32>) {
        self.opacity = opacity;
        self.update_style();
    }

    pub fn apply_style_string(&mut self, style_str: &str) {
        self.parse_and_set_style(style_str);
    }

    // Parse polyCoords string format: [[x1,y1],[x2,y2],...]
    fn parse_poly_coords(value: &str) -> Result<Vec<[f64; 2]>, ()> {
        let mut coords = Vec::new();
        let trimmed = value.trim();

        // Check if it starts with [ and ends with ]
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err(());
        }

        // Remove outer brackets
        let inner = &trimmed[1..trimmed.len() - 1];

        // Split by ],[ to get individual coordinate pairs
        // Handle empty case
        if inner.trim().is_empty() {
            return Ok(coords);
        }

        // Find all [x,y] patterns
        let mut start = 0;
        while start < inner.len() {
            // Find next [
            if let Some(bracket_start) = inner[start..].find('[') {
                let bracket_start = start + bracket_start;
                // Find matching ]
                if let Some(bracket_end) = inner[bracket_start..].find(']') {
                    let bracket_end = bracket_start + bracket_end;
                    let coord_str = &inner[bracket_start + 1..bracket_end];
                    // Parse x,y
                    let parts: Vec<&str> = coord_str.split(',').collect();
                    if parts.len() == 2 {
                        if let (Ok(x), Ok(y)) = (
                            parts[0].trim().parse::<f64>(),
                            parts[1].trim().parse::<f64>(),
                        ) {
                            coords.push([x, y]);
                        }
                    }
                    start = bracket_end + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if coords.is_empty() {
            Err(())
        } else {
            Ok(coords)
        }
    }

    // Parse style string and set all relevant properties using setters
    pub fn parse_and_set_style(&mut self, style_str: &str) {
        for part in style_str.split(';') {
            if part.is_empty() {
                continue;
            } else if part.contains('=') {
                let parts: Vec<&str> = part.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0];
                    let value = parts[1];
                    match key {
                        "whiteSpace" => self.white_space = Some(value.to_string()),
                        "fillColor" => self.set_fill_color(Some(value.to_string())),
                        "strokeColor" => self.set_stroke_color(Some(value.to_string())),
                        "strokeWidth" => {
                            if let Ok(sw) = value.parse::<f64>() {
                                self.set_stroke_width(Some(sw));
                            }
                        }
                        "opacity" => {
                            if let Ok(op) = value.parse::<i32>() {
                                self.set_opacity(Some(op));
                            }
                        }
                        "rounded" => {
                            if let Ok(r) = value.parse::<i32>() {
                                self.set_rounded(Some(r != 0));
                            }
                        }
                        "fontColor" => self.set_font_color(Some(value.to_string())),
                        "fontSize" => {
                            if let Ok(fs) = value.parse::<f64>() {
                                self.set_font_size(Some(fs));
                            }
                        }
                        "polyCoords" => {
                            // Parse polyCoords format: [[x1,y1],[x2,y2],...]
                            if let Ok(coords) = Self::parse_poly_coords(value) {
                                self.set_poly_coords(coords);
                                // Remove polyCoords from style_properties since we store it in poly_coords field
                                self.base.remove_style_property("polyCoords");
                            }
                        }
                        "flipH" => {
                            if let Ok(flip_h) = value.parse::<usize>() {
                                self.geometry_mut()
                                    .flip_rotation_mut()
                                    .set_flip_h(Some(flip_h));
                            }
                        }
                        "flipV" => {
                            if let Ok(flip_v) = value.parse::<usize>() {
                                self.geometry_mut()
                                    .flip_rotation_mut()
                                    .set_flip_v(Some(flip_v));
                            }
                        }
                        "rotation" => {
                            if let Ok(rotation) = value.parse::<f64>() {
                                self.geometry_mut()
                                    .flip_rotation_mut()
                                    .set_rotation(Some(rotation));
                            }
                        }
                        "legacyAnchorPoints" => {
                            if let Ok(legacy_anchor_points) = value.parse::<usize>() {
                                self.geometry_mut()
                                    .flip_rotation_mut()
                                    .set_legacy_anchor_points(Some(legacy_anchor_points));
                            }
                        }
                        _ => {
                            // For other style properties, use the base apply_style_string
                            self.base.apply_style_string(part);
                        }
                    }
                }
            } else {
                // Base style without '='
                self.base.apply_style_string(part);
            }
        }
        self.update_style();
    }

    fn update_style(&mut self) {
        if let Some(ref ws) = self.white_space {
            self.base
                .set_style_property("whiteSpace".to_string(), ws.clone());
        }
        if let Some(ref fc) = self.fill_color {
            self.base
                .set_style_property("fillColor".to_string(), fc.clone());
        }
        if let Some(ref sc) = self.stroke_color {
            self.base
                .set_style_property("strokeColor".to_string(), sc.clone());
        }
        if let Some(sw) = self.stroke_width {
            self.base.add_style_attribute("strokeWidth".to_string());
            self.base
                .set_style_property("strokeWidth".to_string(), sw.to_string());
        }
        if let Some(op) = self.opacity {
            self.base
                .set_style_property("opacity".to_string(), op.to_string());
        }
        if let Some(rd) = self.rounded {
            self.base.set_style_property(
                "rounded".to_string(),
                if rd { "1".to_string() } else { "0".to_string() },
            );
        }
        // Note: polyCoords is stored in poly_coords field, not in style_properties
        // So we don't add it to style_properties here
        if let Some(fc) = self.text_format.font_color() {
            self.base.add_style_attribute("fontColor".to_string());
            self.base
                .set_style_property("fontColor".to_string(), fc.clone());
        }
        if let Some(fs) = self.text_format.font_size() {
            self.base.add_style_attribute("fontSize".to_string());
            self.base
                .set_style_property("fontSize".to_string(), fs.to_string());
        }
    }

    pub fn set_page(&mut self, page: Option<String>) {
        self.base.set_page(page);
    }

    pub fn set_xml_parent(&mut self, parent: Option<String>) {
        self.base.set_xml_parent(parent);
    }

    pub fn set_tag(&mut self, tag: Option<String>) {
        self.base_mut().tag = tag;
    }

    pub fn style(&self) -> String {
        let mut style_str = self.base.style();

        // Add polyCoords to style if present (since it's stored in poly_coords field, not style_properties)
        if !self.poly_coords.is_empty() {
            // Convert Vec<[f64; 2]> to string format "[[x1,y1],[x2,y2],...]"
            let poly_coords_str: Vec<String> = self
                .poly_coords
                .iter()
                .map(|p| format!("[{},{}]", p[0], p[1]))
                .collect();
            let poly_coords_str = format!("[{}]", poly_coords_str.join(","));
            style_str.push_str("polyCoords=");
            style_str.push_str(&poly_coords_str);
            style_str.push(';');
        }

        style_str
    }

    pub fn tag(&self) -> Option<&String> {
        self.base().tag.as_ref()
    }

    pub fn set_shape(&mut self, shape: String) {
        self.base.add_style_attribute("shape".to_string());
        self.base.set_style_property("shape".to_string(), shape);
    }

    pub fn set_start_angle(&mut self, angle: f64) {
        self.base.add_style_attribute("startAngle".to_string());
        self.base
            .set_style_property("startAngle".to_string(), angle.to_string());
    }

    pub fn set_end_angle(&mut self, angle: f64) {
        self.base.add_style_attribute("endAngle".to_string());
        self.base
            .set_style_property("endAngle".to_string(), angle.to_string());
    }

    pub fn set_aspect(&mut self, aspect: String) {
        self.base.add_style_attribute("aspect".to_string());
        self.base.set_style_property("aspect".to_string(), aspect);
    }

    pub fn set_child_layout(&mut self, layout: String) {
        self.base.add_style_attribute("childLayout".to_string());
        self.base
            .set_style_property("childLayout".to_string(), layout);
    }

    pub fn set_resize_parent(&mut self, resize: i32) {
        self.base.add_style_attribute("resizeParent".to_string());
        self.base
            .set_style_property("resizeParent".to_string(), resize.to_string());
    }

    pub fn set_resize_last(&mut self, resize: i32) {
        self.base.add_style_attribute("resizeLast".to_string());
        self.base
            .set_style_property("resizeLast".to_string(), resize.to_string());
    }

    pub fn set_container(&mut self, container: i32) {
        self.base.add_style_attribute("container".to_string());
        self.base
            .set_style_property("container".to_string(), container.to_string());
    }

    pub fn xml(&self) -> String {
        let style = self.style(); // Use self.style() instead of self.base.style() to include polyCoords
        let parent_id = self.base.xml_parent_id();

        if let Some(tag) = self.tag() {
            // When tag is present, wrap in UserObject and mxCell should not have id attribute
            // Value goes to UserObject label, not mxCell value
            let mx_cell_xml = format!(
                r#"<mxCell style="{}" vertex="{}" parent="{}">
          {}
        </mxCell>"#,
                style,
                self.vertex,
                parent_id,
                self.geometry.xml()
            );
            format!(
                r#"<UserObject label="{}" tags="{}" id="{}">
        {}
        </UserObject>"#,
                crate::xml_base::XMLBase::xml_ify(self.value().map(|s| s.as_str()).unwrap_or("")),
                crate::xml_base::XMLBase::xml_ify(tag),
                self.base.id(),
                mx_cell_xml
            )
        } else {
            // Normal case: mxCell with id
            format!(
                r#"<mxCell id="{}" value="{}" style="{}" vertex="{}" parent="{}">
          {}
        </mxCell>"#,
                self.base.id(),
                crate::xml_base::XMLBase::xml_ify(self.value().map(|s| s.as_str()).unwrap_or("")),
                style,
                self.vertex,
                parent_id,
                self.geometry.xml()
            )
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new(None)
    }
}
