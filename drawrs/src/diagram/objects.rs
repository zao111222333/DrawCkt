use crate::XMLBase;
use crate::diagram::base_diagram::DiagramBase;
use crate::diagram::geometry::Geometry;
use crate::diagram::text_format::{Justify, TextFormat};
use std::borrow::Cow;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FillStyle {
    Hatch,
    Solid,
    Dots,
    CrossHatch,
    Dashed,
    ZigzagLine,
}

impl FillStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "hatch" => Some(FillStyle::Hatch),
            "solid" => Some(FillStyle::Solid),
            "dots" => Some(FillStyle::Dots),
            "cross-hatch" => Some(FillStyle::CrossHatch),
            "dashed" => Some(FillStyle::Dashed),
            "zigzag-line" => Some(FillStyle::ZigzagLine),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            FillStyle::Hatch => "hatch",
            FillStyle::Solid => "solid",
            FillStyle::Dots => "dots",
            FillStyle::CrossHatch => "cross-hatch",
            FillStyle::Dashed => "dashed",
            FillStyle::ZigzagLine => "zigzag-line",
        }
    }
}

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
    fill_style: Option<FillStyle>,
    // glass: Option<bool>,
    // shadow: Option<bool>,
    // line_pattern: Option<String>,
    text_format: TextFormat,
    vertex: i32,
    poly_coords: Vec<[f64; 2]>, // Polygon coordinates as normalized (0-1) points relative to bounding box
}

impl Object {
    pub fn new(id: Option<String>) -> Self {
        let base = DiagramBase::new(id);

        let mut geom = Geometry::new();
        geom.set_width(120.0);
        geom.set_height(80.0);
        Self {
            base,
            geometry: geom,
            white_space: None,
            rounded: None,
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            opacity: None,
            fill_style: None,
            // glass: None,
            // shadow: None,
            // line_pattern: Some("solid".to_string()),
            text_format: TextFormat::new(),
            vertex: 1,
            poly_coords: Vec::new(),
        }
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
    }

    pub fn stroke_color(&self) -> Option<&String> {
        self.stroke_color.as_ref()
    }

    pub fn set_stroke_color(&mut self, color: Option<String>) {
        self.stroke_color = color;
    }

    pub fn stroke_width(&self) -> Option<f64> {
        self.stroke_width
    }

    pub fn set_stroke_width(&mut self, width: Option<f64>) {
        self.stroke_width = width;
    }

    pub fn poly_coords(&self) -> &Vec<[f64; 2]> {
        &self.poly_coords
    }

    pub fn poly_coords_mut(&mut self) -> &mut Vec<[f64; 2]> {
        &mut self.poly_coords
    }

    pub fn set_poly_coords(&mut self, coords: Vec<[f64; 2]>) {
        self.poly_coords = coords;
    }

    pub fn font_color(&self) -> Option<&String> {
        self.text_format.font_color()
    }

    pub fn set_font_color(&mut self, color: Option<String>) {
        self.text_format.set_font_color(color);
    }

    pub fn font_size(&self) -> Option<f64> {
        self.text_format.font_size()
    }

    pub fn set_font_size(&mut self, size: Option<f64>) {
        self.text_format.set_font_size(size);
    }

    pub fn font_family(&self) -> Option<&String> {
        self.text_format.font_family()
    }

    pub fn set_font_family(&mut self, family: Option<String>) {
        self.text_format.set_font_family(family);
    }

    pub fn set_justify(&mut self, justify: Justify) {
        self.text_format.set_justify(justify);
    }
    pub fn justify_mut(&mut self) -> &mut Justify {
        self.text_format.justify_mut()
    }

    pub fn rounded(&self) -> Option<bool> {
        self.rounded
    }

    pub fn set_rounded(&mut self, rounded: Option<bool>) {
        self.rounded = rounded;
    }

    pub fn opacity(&self) -> Option<i32> {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: Option<i32>) {
        self.opacity = opacity;
    }

    pub fn fill_style(&self) -> Option<&FillStyle> {
        self.fill_style.as_ref()
    }

    pub fn set_fill_style(&mut self, fill_style: Option<FillStyle>) {
        self.fill_style = fill_style;
    }

    /// Internal helper to apply a single style property
    pub fn apply_style_property(&mut self, key: &str, value: &str) {
        match key {
            "whiteSpace" => self.white_space = Some(value.to_string()),
            "fillColor" => self.fill_color = Some(value.to_string()),
            "strokeColor" => self.stroke_color = Some(value.to_string()),
            "strokeWidth" => {
                if let Ok(sw) = value.parse::<f64>() {
                    self.stroke_width = Some(sw);
                }
            }
            "opacity" => {
                if let Ok(op) = value.parse::<i32>() {
                    self.opacity = Some(op);
                }
            }
            "rounded" => {
                if let Ok(r) = value.parse::<i32>() {
                    self.rounded = Some(r != 0);
                }
            }
            "fillStyle" => {
                if let Some(fill_style) = FillStyle::from_str(value) {
                    self.fill_style = Some(fill_style);
                }
            }
            "fontColor" => self.text_format.set_font_color(Some(value.to_string())),
            "fontSize" => {
                if let Ok(fs) = value.parse::<f64>() {
                    self.text_format.set_font_size(Some(fs));
                }
            }
            "fontFamily" => self.text_format.set_font_family(Some(value.to_string())),
            "align" | "verticalAlign" => {
                // Handle justify - need to parse both align and verticalAlign together
                // This is handled in parse_and_set_style
            }
            "polyCoords" => {
                if let Ok(coords) = Self::parse_poly_coords(value) {
                    self.poly_coords = coords;
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
                // For unsupported style properties, store in base
                self.base.apply_style_property(
                    Cow::Owned(key.to_string()),
                    Cow::Owned(value.to_string()),
                );
            }
        }
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

    // Parse style string and set all relevant properties
    pub fn parse_and_set_style(&mut self, style_str: &str) {
        // Parse justify from the entire style string first, but only if it contains align or verticalAlign
        // Otherwise, preserve the existing justify value
        if style_str.contains("align=") || style_str.contains("verticalAlign=") {
            self.text_format
                .set_justify(crate::diagram::text_format::Justify::parse(style_str));
        }

        // Parse style string into key-value pairs
        let key_value_list = DiagramBase::parse_style_string(style_str);

        // Apply each key-value pair
        for (key, value) in key_value_list {
            // Skip align and verticalAlign as they're already handled by justify parsing
            if key == "align" || key == "verticalAlign" {
                continue;
            }
            self.apply_style_property(key, value);
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

    pub fn style(&self) -> impl fmt::Display + '_ {
        ObjectStyleFormatter(self)
    }

    pub fn tag(&self) -> Option<&String> {
        self.base().tag.as_ref()
    }

    pub fn set_shape(&mut self, shape: String) {
        self.base
            .apply_style_property(Cow::Borrowed("shape"), Cow::Owned(shape));
    }

    pub fn set_start_angle(&mut self, angle: f64) {
        self.base
            .apply_style_property(Cow::Borrowed("startAngle"), Cow::Owned(angle.to_string()));
    }

    pub fn set_end_angle(&mut self, angle: f64) {
        self.base
            .apply_style_property(Cow::Borrowed("endAngle"), Cow::Owned(angle.to_string()));
    }

    pub fn set_aspect(&mut self, aspect: String) {
        self.base
            .apply_style_property(Cow::Borrowed("aspect"), Cow::Owned(aspect));
    }

    pub fn set_child_layout(&mut self, layout: String) {
        self.base
            .apply_style_property(Cow::Borrowed("childLayout"), Cow::Owned(layout));
    }

    pub fn set_resize_parent(&mut self, resize: i32) {
        self.base.apply_style_property(
            Cow::Borrowed("resizeParent"),
            Cow::Owned(resize.to_string()),
        );
    }

    pub fn set_resize_last(&mut self, resize: i32) {
        self.base
            .apply_style_property(Cow::Borrowed("resizeLast"), Cow::Owned(resize.to_string()));
    }

    pub fn set_container(&mut self, container: i32) {
        self.base.apply_style_property(
            Cow::Borrowed("container"),
            Cow::Owned(container.to_string()),
        );
    }

    pub fn xml(&self) -> ObjectXml<'_> {
        ObjectXml(self)
    }
}

pub struct ObjectXml<'a>(&'a Object);

impl<'a> fmt::Display for ObjectXml<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = ObjectStyleFormatter(self.0);
        let parent_id = self.0.base.xml_parent_id();
        let value =
            crate::xml_base::XMLBase::xml_ify(self.0.value().map(|s| s.as_str()).unwrap_or(""));

        if let Some(tag) = self.0.tag() {
            // When tag is present, wrap in UserObject and mxCell should not have id attribute
            // Value goes to UserObject label, not mxCell value
            write!(
                f,
                r#"<UserObject label="{}" tags="{}" id="{}">
        <mxCell style="{}" vertex="{}" parent="{}">
          {}
        </mxCell>
        </UserObject>"#,
                value,
                crate::xml_base::XMLBase::xml_ify(tag),
                self.0.base.id(),
                style,
                self.0.vertex,
                parent_id,
                self.0.geometry.xml()
            )
        } else {
            // Normal case: mxCell with id
            write!(
                f,
                r#"<mxCell id="{}" value="{}" style="{}" vertex="{}" parent="{}">
          {}
        </mxCell>"#,
                self.0.base.id(),
                value,
                style,
                self.0.vertex,
                parent_id,
                self.0.geometry.xml()
            )
        }
    }
}

struct ObjectStyleFormatter<'a>(&'a Object);

impl<'a> fmt::Display for ObjectStyleFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Add all supported style properties
        if let Some(ref ws) = self.0.white_space {
            write!(f, "whiteSpace={};", ws)?;
        }
        if let Some(ref fc) = self.0.fill_color {
            write!(f, "fillColor={};", fc)?;
        }
        if let Some(ref sc) = self.0.stroke_color {
            write!(f, "strokeColor={};", sc)?;
        }
        if let Some(sw) = self.0.stroke_width {
            write!(f, "strokeWidth={};", sw)?;
        }
        if let Some(op) = self.0.opacity {
            write!(f, "opacity={};", op)?;
        }
        if let Some(rd) = self.0.rounded {
            write!(f, "rounded={};", if rd { "1" } else { "0" })?;
        }
        if let Some(ref fs) = self.0.fill_style {
            write!(f, "fillStyle={};", fs.to_str())?;
        }
        if let Some(fc) = self.0.text_format.font_color() {
            write!(f, "fontColor={};", fc)?;
        }
        if let Some(fs) = self.0.text_format.font_size() {
            write!(f, "fontSize={};", fs)?;
        }
        if let Some(ff) = self.0.text_format.font_family() {
            write!(f, "fontFamily={};", ff)?;
        }

        // Add justify properties (align and verticalAlign)
        let justify_str = self.0.text_format.justify().format();
        if !justify_str.is_empty() {
            for part in justify_str.split(';') {
                if part.is_empty() {
                    continue;
                } else if part.contains('=') {
                    write!(f, "{};", part)?;
                }
            }
        }

        // Add polyCoords if present
        if !self.0.poly_coords.is_empty() {
            write!(f, "polyCoords=")?;
            write!(f, "[")?;
            for (i, p) in self.0.poly_coords.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "[{},{}]", p[0], p[1])?;
            }
            write!(f, "];")?;
        }

        // Add unsupported properties
        for (key, value) in self.0.base.unsupported_style_properties() {
            write!(f, "{}={};", key, value)?;
        }

        Ok(())
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new(None)
    }
}
