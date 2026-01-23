use crate::XMLBase;
use crate::diagram::base_diagram::DiagramBase;
use crate::diagram::geometry::Geometry;
use std::borrow::Cow;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Edge {
    base: DiagramBase,
    source: Option<String>,
    target: Option<String>,
    waypoints: String,
    connection: String,
    pattern: String,
    edge: i32,
    stroke_color: Option<String>,
    stroke_width: Option<f64>,
    fill_color: Option<String>,
    line_end_target: Option<String>,
    line_end_source: Option<String>,
    end_fill_target: bool,
    end_fill_source: bool,
    end_size: Option<i32>,
    start_size: Option<i32>,
    rounded: i32,
    opacity: Option<i32>,
    geometry: Geometry,
}

impl Edge {
    pub fn new(id: Option<String>) -> Self {
        let base = DiagramBase::new(id);

        Self {
            base,
            source: None,
            target: None,
            waypoints: "orthogonal".to_string(),
            connection: "line".to_string(),
            pattern: "solid".to_string(),
            edge: 1,
            stroke_color: None,
            stroke_width: None,
            fill_color: None,
            line_end_target: Some("none".to_string()),
            line_end_source: None,
            end_fill_target: false,
            end_fill_source: false,
            end_size: None,
            start_size: None,
            rounded: 0,
            opacity: None,
            geometry: Geometry::new(),
        }
    }
    pub fn base(&self) -> &XMLBase {
        self.base.base()
    }
    pub fn base_mut(&mut self) -> &mut XMLBase {
        self.base.base_mut()
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

    pub fn source(&self) -> Option<&String> {
        self.source.as_ref()
    }

    pub fn set_source(&mut self, source: Option<String>) {
        self.source = source;
    }

    pub fn target(&self) -> Option<&String> {
        self.target.as_ref()
    }

    pub fn set_target(&mut self, target: Option<String>) {
        self.target = target;
    }

    pub fn label(&self) -> Option<&String> {
        self.base().value.as_ref()
    }

    pub fn set_label(&mut self, label: Option<String>) {
        self.base_mut().value = label;
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

    pub fn fill_color(&self) -> Option<&String> {
        self.fill_color.as_ref()
    }

    pub fn set_fill_color(&mut self, color: Option<String>) {
        self.fill_color = color;
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn set_pattern(&mut self, pattern: String) {
        self.pattern = pattern;
    }

    pub fn waypoints(&self) -> &str {
        &self.waypoints
    }

    pub fn set_waypoints(&mut self, waypoints: String) {
        self.waypoints = waypoints;
    }

    pub fn connection(&self) -> &str {
        &self.connection
    }

    pub fn set_connection(&mut self, connection: String) {
        self.connection = connection;
    }

    pub fn edge(&self) -> i32 {
        self.edge
    }

    pub fn line_end_target(&self) -> Option<&String> {
        self.line_end_target.as_ref()
    }

    pub fn set_line_end_target(&mut self, end: Option<String>) {
        self.line_end_target = end;
    }

    pub fn line_end_source(&self) -> Option<&String> {
        self.line_end_source.as_ref()
    }

    pub fn set_line_end_source(&mut self, end: Option<String>) {
        self.line_end_source = end;
    }

    pub fn end_fill_target(&self) -> bool {
        self.end_fill_target
    }

    pub fn set_end_fill_target(&mut self, fill: bool) {
        self.end_fill_target = fill;
    }

    pub fn end_fill_source(&self) -> bool {
        self.end_fill_source
    }

    pub fn set_end_fill_source(&mut self, fill: bool) {
        self.end_fill_source = fill;
    }

    pub fn end_size(&self) -> Option<i32> {
        self.end_size
    }

    pub fn set_end_size(&mut self, size: Option<i32>) {
        self.end_size = size;
    }

    pub fn start_size(&self) -> Option<i32> {
        self.start_size
    }

    pub fn set_start_size(&mut self, size: Option<i32>) {
        self.start_size = size;
    }

    pub fn opacity(&self) -> Option<i32> {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: Option<i32>) {
        self.opacity = opacity;
    }

    /// Internal helper to apply a single style property
    pub fn apply_style_property(&mut self, key: &str, value: &str) {
        match key {
            "strokeColor" => self.stroke_color = Some(value.to_string()),
            "strokeWidth" => {
                if let Ok(sw) = value.parse::<f64>() {
                    self.stroke_width = Some(sw);
                }
            }
            "fillColor" => self.fill_color = Some(value.to_string()),
            "endArrow" => self.line_end_target = Some(value.to_string()),
            "startArrow" => self.line_end_source = Some(value.to_string()),
            "endFill" => {
                if let Ok(ef) = value.parse::<i32>() {
                    self.end_fill_target = ef != 0;
                }
            }
            "startFill" => {
                if let Ok(sf) = value.parse::<i32>() {
                    self.end_fill_source = sf != 0;
                }
            }
            "endSize" => {
                if let Ok(es) = value.parse::<i32>() {
                    self.end_size = Some(es);
                }
            }
            "startSize" => {
                if let Ok(ss) = value.parse::<i32>() {
                    self.start_size = Some(ss);
                }
            }
            "opacity" => {
                if let Ok(op) = value.parse::<i32>() {
                    self.opacity = Some(op);
                }
            }
            "rounded" => {
                if let Ok(r) = value.parse::<i32>() {
                    self.rounded = if r != 0 { 1 } else { 0 };
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

    pub fn set_page(&mut self, page: Option<String>) {
        self.base.set_page(page);
    }

    pub fn set_xml_parent(&mut self, parent: Option<String>) {
        self.base.set_xml_parent(parent);
    }

    pub fn set_tag(&mut self, tag: Option<String>) {
        self.base_mut().tag = tag;
    }

    pub fn tag(&self) -> Option<&String> {
        self.base().tag.as_ref()
    }

    // Parse style string and set all relevant properties
    pub fn parse_and_set_style(&mut self, style_str: &str) {
        // Parse style string into key-value pairs
        let key_value_list = DiagramBase::parse_style_string(style_str);

        // Apply each key-value pair
        for (key, value) in key_value_list {
            self.apply_style_property(key, value);
        }
    }

    pub fn geometry(&mut self) -> &mut Geometry {
        &mut self.geometry
    }

    pub fn geometry_ref(&self) -> &Geometry {
        &self.geometry
    }

    pub fn xml(&self) -> EdgeXml<'_> {
        EdgeXml(self)
    }
}

pub struct EdgeXml<'a>(&'a Edge);

impl<'a> fmt::Display for EdgeXml<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = EdgeStyleFormatter(self.0);
        let parent_id = self.0.base.xml_parent_id();
        let value = self
            .0
            .label()
            .map(|l| crate::xml_base::XMLBase::xml_ify(l))
            .unwrap_or_else(|| "".to_string());

        // Only include source and target if they are set and not "1"
        let source_id = self.0.source.as_ref().map(|s| s.as_str());
        let target_id = self.0.target.as_ref().map(|s| s.as_str());

        // Check if we should include source/target attributes
        let has_source_target = source_id.is_some()
            && target_id.is_some()
            && source_id.unwrap() != "1"
            && target_id.unwrap() != "1";

        if let Some(tag) = self.0.tag() {
            // When tag is present, wrap in UserObject and mxCell should not have id attribute
            // Value (label) goes to UserObject label, not mxCell value
            if has_source_target {
                write!(
                    f,
                    r#"<UserObject label="{}" tags="{}" id="{}">
        <mxCell style="{}" edge="{}" parent="{}" source="{}" target="{}">
          {}
        </mxCell>
        </UserObject>"#,
                    value,
                    crate::xml_base::XMLBase::xml_ify(tag),
                    self.0.base.id(),
                    style,
                    self.0.edge,
                    parent_id,
                    source_id.unwrap(),
                    target_id.unwrap(),
                    self.0.geometry.xml()
                )
            } else {
                write!(
                    f,
                    r#"<UserObject label="{}" tags="{}" id="{}">
        <mxCell style="{}" edge="{}" parent="{}">
          {}
        </mxCell>
        </UserObject>"#,
                    value,
                    crate::xml_base::XMLBase::xml_ify(tag),
                    self.0.base.id(),
                    style,
                    self.0.edge,
                    parent_id,
                    self.0.geometry.xml()
                )
            }
        } else {
            // Normal case: mxCell with id
            if has_source_target {
                write!(
                    f,
                    r#"<mxCell id="{}" value="{}" style="{}" edge="{}" parent="{}" source="{}" target="{}">
          {}
        </mxCell>"#,
                    self.0.base.id(),
                    value,
                    style,
                    self.0.edge,
                    parent_id,
                    source_id.unwrap(),
                    target_id.unwrap(),
                    self.0.geometry.xml()
                )
            } else {
                write!(
                    f,
                    r#"<mxCell id="{}" value="{}" style="{}" edge="{}" parent="{}">
          {}
        </mxCell>"#,
                    self.0.base.id(),
                    value,
                    style,
                    self.0.edge,
                    parent_id,
                    self.0.geometry.xml()
                )
            }
        }
    }
}

struct EdgeStyleFormatter<'a>(&'a Edge);

impl<'a> fmt::Display for EdgeStyleFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Add all supported style properties
        if let Some(ref sc) = self.0.stroke_color {
            write!(f, "strokeColor={};", sc)?;
        }
        if let Some(sw) = self.0.stroke_width {
            write!(f, "strokeWidth={};", sw)?;
        }
        if let Some(ref fc) = self.0.fill_color {
            write!(f, "fillColor={};", fc)?;
        }
        if let Some(ref end) = self.0.line_end_target {
            write!(f, "endArrow={};", end)?;
        }
        if let Some(ref start) = self.0.line_end_source {
            write!(f, "startArrow={};", start)?;
        }
        if self.0.end_fill_target {
            write!(f, "endFill=1;")?;
        }
        if self.0.end_fill_source {
            write!(f, "startFill=1;")?;
        }
        if let Some(es) = self.0.end_size {
            write!(f, "endSize={};", es)?;
        }
        if let Some(ss) = self.0.start_size {
            write!(f, "startSize={};", ss)?;
        }
        if let Some(op) = self.0.opacity {
            write!(f, "opacity={};", op)?;
        }
        // Always include rounded
        write!(f, "rounded={};", self.0.rounded)?;

        // Add unsupported properties
        for (key, value) in self.0.base.unsupported_style_properties() {
            write!(f, "{}={};", key, value)?;
        }

        Ok(())
    }
}

impl Edge {
    pub fn style(&self) -> impl fmt::Display + '_ {
        EdgeStyleFormatter(self)
    }
}

impl Default for Edge {
    fn default() -> Self {
        Self::new(None)
    }
}
