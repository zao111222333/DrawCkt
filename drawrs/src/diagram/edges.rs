use crate::XMLBase;
use crate::diagram::base_diagram::DiagramBase;
use crate::diagram::geometry::Geometry;

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
        let mut base = DiagramBase::new(id);
        base.add_style_attribute("rounded".to_string());
        base.add_style_attribute("strokeColor".to_string());
        base.add_style_attribute("strokeWidth".to_string());
        base.add_style_attribute("fillColor".to_string());
        base.add_style_attribute("endArrow".to_string());
        base.add_style_attribute("startArrow".to_string());
        base.add_style_attribute("endFill".to_string());
        base.add_style_attribute("startFill".to_string());
        base.add_style_attribute("endSize".to_string());
        base.add_style_attribute("startSize".to_string());
        base.add_style_attribute("opacity".to_string());

        // Set default style values
        base.set_style_property("endArrow".to_string(), "none".to_string());
        base.set_style_property("rounded".to_string(), "0".to_string());

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
        self.update_style();
    }

    pub fn stroke_width(&self) -> Option<f64> {
        self.stroke_width
    }

    pub fn set_stroke_width(&mut self, width: Option<f64>) {
        self.stroke_width = width;
        self.update_style();
    }

    pub fn fill_color(&self) -> Option<&String> {
        self.fill_color.as_ref()
    }

    pub fn set_fill_color(&mut self, color: Option<String>) {
        self.fill_color = color;
        self.update_style();
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
        self.update_style();
    }

    pub fn line_end_source(&self) -> Option<&String> {
        self.line_end_source.as_ref()
    }

    pub fn set_line_end_source(&mut self, end: Option<String>) {
        self.line_end_source = end;
        self.update_style();
    }

    pub fn end_fill_target(&self) -> bool {
        self.end_fill_target
    }

    pub fn set_end_fill_target(&mut self, fill: bool) {
        self.end_fill_target = fill;
        self.update_style();
    }

    pub fn end_fill_source(&self) -> bool {
        self.end_fill_source
    }

    pub fn set_end_fill_source(&mut self, fill: bool) {
        self.end_fill_source = fill;
        self.update_style();
    }

    pub fn end_size(&self) -> Option<i32> {
        self.end_size
    }

    pub fn set_end_size(&mut self, size: Option<i32>) {
        self.end_size = size;
        self.update_style();
    }

    pub fn start_size(&self) -> Option<i32> {
        self.start_size
    }

    pub fn set_start_size(&mut self, size: Option<i32>) {
        self.start_size = size;
        self.update_style();
    }

    pub fn opacity(&self) -> Option<i32> {
        self.opacity
    }

    pub fn set_opacity(&mut self, opacity: Option<i32>) {
        self.opacity = opacity;
        self.update_style();
    }

    fn update_style(&mut self) {
        if let Some(ref sc) = self.stroke_color {
            self.base
                .set_style_property("strokeColor".to_string(), sc.clone());
        }
        if let Some(sw) = self.stroke_width {
            self.base
                .set_style_property("strokeWidth".to_string(), sw.to_string());
        }
        if let Some(ref fc) = self.fill_color {
            self.base
                .set_style_property("fillColor".to_string(), fc.clone());
        }
        if let Some(ref end) = self.line_end_target {
            self.base
                .set_style_property("endArrow".to_string(), end.clone());
        }
        if let Some(ref start) = self.line_end_source {
            self.base
                .set_style_property("startArrow".to_string(), start.clone());
        }
        if self.end_fill_target {
            self.base
                .set_style_property("endFill".to_string(), "1".to_string());
        }
        if self.end_fill_source {
            self.base
                .set_style_property("startFill".to_string(), "1".to_string());
        }
        if let Some(es) = self.end_size {
            self.base
                .set_style_property("endSize".to_string(), es.to_string());
        }
        if let Some(ss) = self.start_size {
            self.base
                .set_style_property("startSize".to_string(), ss.to_string());
        }
        if let Some(op) = self.opacity {
            self.base
                .set_style_property("opacity".to_string(), op.to_string());
        }
        // Always set rounded based on the rounded field
        self.base
            .set_style_property("rounded".to_string(), self.rounded.to_string());
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
        self.base.style()
    }

    pub fn tag(&self) -> Option<&String> {
        self.base().tag.as_ref()
    }

    pub fn apply_style_string(&mut self, style_str: &str) {
        self.parse_and_set_style(style_str);
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
                        "strokeColor" => self.set_stroke_color(Some(value.to_string())),
                        "strokeWidth" => {
                            if let Ok(sw) = value.parse::<f64>() {
                                self.set_stroke_width(Some(sw));
                            }
                        }
                        "fillColor" => self.set_fill_color(Some(value.to_string())),
                        "endArrow" => self.set_line_end_target(Some(value.to_string())),
                        "startArrow" => self.set_line_end_source(Some(value.to_string())),
                        "endFill" => {
                            if let Ok(ef) = value.parse::<i32>() {
                                self.set_end_fill_target(ef != 0);
                            }
                        }
                        "startFill" => {
                            if let Ok(sf) = value.parse::<i32>() {
                                self.set_end_fill_source(sf != 0);
                            }
                        }
                        "endSize" => {
                            if let Ok(es) = value.parse::<i32>() {
                                self.set_end_size(Some(es));
                            }
                        }
                        "startSize" => {
                            if let Ok(ss) = value.parse::<i32>() {
                                self.set_start_size(Some(ss));
                            }
                        }
                        "opacity" => {
                            if let Ok(op) = value.parse::<i32>() {
                                self.set_opacity(Some(op));
                            }
                        }
                        "rounded" => {
                            if let Ok(r) = value.parse::<i32>() {
                                self.rounded = if r != 0 { 1 } else { 0 };
                                self.update_style();
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
    }

    pub fn geometry(&mut self) -> &mut Geometry {
        &mut self.geometry
    }

    pub fn geometry_ref(&self) -> &Geometry {
        &self.geometry
    }

    pub fn xml(&self) -> String {
        let style = self.base.style();
        let parent_id = self.base.xml_parent_id();
        let value = self
            .label()
            .map(|l| crate::xml_base::XMLBase::xml_ify(l))
            .unwrap_or_else(|| "".to_string());

        // Only include source and target if they are set and not "1"
        let source_id = self.source.as_ref().map(|s| s.as_str());
        let target_id = self.target.as_ref().map(|s| s.as_str());

        // Check if we should include source/target attributes
        let has_source_target = source_id.is_some()
            && target_id.is_some()
            && source_id.unwrap() != "1"
            && target_id.unwrap() != "1";

        if let Some(tag) = self.tag() {
            // When tag is present, wrap in UserObject and mxCell should not have id attribute
            // Value (label) goes to UserObject label, not mxCell value
            let mx_cell_xml = if has_source_target {
                format!(
                    r#"<mxCell style="{}" edge="{}" parent="{}" source="{}" target="{}">
          {}
        </mxCell>"#,
                    style,
                    self.edge,
                    parent_id,
                    source_id.unwrap(),
                    target_id.unwrap(),
                    self.geometry.xml()
                )
            } else {
                format!(
                    r#"<mxCell style="{}" edge="{}" parent="{}">
          {}
        </mxCell>"#,
                    style,
                    self.edge,
                    parent_id,
                    self.geometry.xml()
                )
            };
            format!(
                r#"<UserObject label="{}" tags="{}" id="{}">
        {}
        </UserObject>"#,
                value,
                crate::xml_base::XMLBase::xml_ify(tag),
                self.base.id(),
                mx_cell_xml
            )
        } else {
            // Normal case: mxCell with id
            if has_source_target {
                format!(
                    r#"<mxCell id="{}" value="{}" style="{}" edge="{}" parent="{}" source="{}" target="{}">
          {}
        </mxCell>"#,
                    self.base.id(),
                    value,
                    style,
                    self.edge,
                    parent_id,
                    source_id.unwrap(),
                    target_id.unwrap(),
                    self.geometry.xml()
                )
            } else {
                format!(
                    r#"<mxCell id="{}" value="{}" style="{}" edge="{}" parent="{}">
          {}
        </mxCell>"#,
                    self.base.id(),
                    value,
                    style,
                    self.edge,
                    parent_id,
                    self.geometry.xml()
                )
            }
        }
    }
}

impl Default for Edge {
    fn default() -> Self {
        Self::new(None)
    }
}
