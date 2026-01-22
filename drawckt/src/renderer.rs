use crate::error::{DrawcktError, DrawcktResult};
use crate::schematic::*;
use drawrs::xml_base::XMLBase;
use drawrs::{
    parse_xml_to_object, BoundingBox, DiagramObject, Edge, File, GroupTransform, Object, Orient,
    Page,
};
use indexmap::IndexMap;
use log::info;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;
use std::path::Path;

// Scale factor to convert from schematic units to Draw.io pixels
const SCALE: f64 = 200.0;

// Structure to hold parsed symbol page data
#[derive(Debug, Clone)]
pub struct SymbolPageData {
    objects: Vec<drawrs::page::DiagramObject>, // Parsed drawrs objects (Object or Edge)
    origin_bounding_box: BoundingBox,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolId {
    pub lib: String,
    pub cell: String,
}

pub struct SymbolContexts(IndexMap<SymbolId, String>);

impl SymbolContexts {
    pub fn new() -> Self {
        Self(IndexMap::new())
    }

    pub fn insert(&mut self, id: SymbolId, content: String) {
        self.0.insert(id, content);
    }

    pub fn get(&self, id: &SymbolId) -> Option<&String> {
        self.0.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SymbolId, &String)> {
        self.0.iter()
    }

    /// Write all symbols to directory structure: {dir}/{lib}/{cell}.drawio
    pub fn write_to_dir(&self, dir: impl AsRef<Path>) -> DrawcktResult<()> {
        let output_path = dir.as_ref();
        fs::create_dir_all(output_path)?;

        for (symbol_id, content) in self.iter() {
            let lib_dir = output_path.join(&symbol_id.lib);
            fs::create_dir_all(&lib_dir)?;
            let cell_file = lib_dir.join(format!("{}.drawio", symbol_id.cell));
            fs::write(&cell_file, content)?;
            info!("Symbol rendered to: {:?}", cell_file);
        }

        Ok(())
    }

    /// Load symbols from directory structure: {dir}/{lib}/{cell}.drawio
    pub fn load_from_dir(dir: impl AsRef<Path>) -> DrawcktResult<Self> {
        let symbols_path = dir.as_ref();
        let mut symbol_contexts = SymbolContexts::new();

        if symbols_path.exists() && symbols_path.is_dir() {
            for lib_entry in fs::read_dir(symbols_path)? {
                let lib_entry = lib_entry?;
                let lib_path = lib_entry.path();
                if lib_path.is_dir() {
                    let lib_name =
                        lib_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .ok_or_else(|| {
                                DrawcktError::Io(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!("Invalid lib directory name: {:?}", lib_path),
                                ))
                            })?;

                    for cell_entry in fs::read_dir(&lib_path)? {
                        let cell_entry = cell_entry?;
                        let cell_path = cell_entry.path();
                        if cell_path.is_file()
                            && cell_path.extension().and_then(|s| s.to_str()) == Some("drawio")
                        {
                            let cell_name = cell_path
                                .file_stem()
                                .and_then(|n| n.to_str())
                                .ok_or_else(|| {
                                    DrawcktError::Io(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        format!("Invalid cell file name: {:?}", cell_path),
                                    ))
                                })?;

                            let content = fs::read_to_string(&cell_path)?;
                            let symbol_id = SymbolId {
                                lib: lib_name.to_string(),
                                cell: cell_name.to_string(),
                            };
                            symbol_contexts.insert(symbol_id, content);
                        }
                    }
                }
            }
        }

        Ok(symbol_contexts)
    }
}

impl Default for SymbolContexts {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Renderer {
    schematic: Schematic,
    layer_styles: LayerStyles,
}

impl Renderer {
    pub fn new(schematic: Schematic) -> Self {
        Self {
            schematic,
            layer_styles: LayerStyles::default(),
        }
    }

    pub fn with_layer_styles(mut self, styles: LayerStyles) -> Self {
        self.layer_styles = styles;
        self
    }

    fn get_layer_style(&self, layer: &Layer) -> &LayerStyle {
        match layer {
            Layer::Instance => &self.layer_styles.instance,
            Layer::Annotate => &self.layer_styles.annotate,
            Layer::Pin => &self.layer_styles.pin,
            Layer::Device => &self.layer_styles.device,
            Layer::Wire => &self.layer_styles.wire,
            Layer::Text => &self.layer_styles.text,
        }
    }

    fn init_layers<const IS_SCH: bool>(&self, page: &mut Page, layer_order: &[Layer; 6]) -> DrawcktResult<()> {
        let mut instance = false;
        let mut annotate = false;
        let mut pin = false;
        let mut device = false;
        let mut wire = false;
        let mut text = false;
        for layer in layer_order.iter().rev() {
            match layer {
                Layer::Instance => {
                    if instance {
                        return Err(DrawcktError::RepeatLayer(*layer))
                    }
                    instance = true;
                },
                Layer::Annotate => {
                    if annotate {
                        return Err(DrawcktError::RepeatLayer(*layer))
                    }
                    annotate = true;
                },
                Layer::Pin => {
                    if pin {
                        return Err(DrawcktError::RepeatLayer(*layer))
                    }
                    pin = true;
                },
                Layer::Device => {
                    if device {
                        return Err(DrawcktError::RepeatLayer(*layer))
                    }
                    device = true;
                },
                Layer::Wire => {
                    if wire {
                        return Err(DrawcktError::RepeatLayer(*layer))
                    }
                    wire = true;
                },
                Layer::Text => {
                    if text {
                        return Err(DrawcktError::RepeatLayer(*layer))
                    }
                    text = true;
                },
            }
            let mut layer_cell = drawrs::xml_base::XMLBase::new(Some(layer.id()));
            layer_cell.xml_class = "mxCell".to_string();
            layer_cell.xml_parent = Some("0".to_string());
            layer_cell.value = Some(layer.to_string());
            layer_cell.visible = Some(if IS_SCH && !self.get_layer_style(layer).sch_visible {
                "0".to_string()
            } else {
                "1".to_string()
            });
            page.add_object(drawrs::DiagramObject::XmlBase(layer_cell));
        }
        Ok(())
    }

    fn get_wire_style(&self) -> &LayerStyle {
        &self.layer_styles.wire
    }

    // Generate ID for wire: wire-{net}-{counter} or wire-{uuid}
    fn gen_wire_id(net: &str, counter: usize) -> String {
        if !net.is_empty() {
            let safe_net = net.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
            format!("wire-{}-{}", safe_net, counter)
        } else {
            format!("wire-{}", uuid::Uuid::new_v4().to_string())
        }
    }

    // Apply fill style to an Object based on fillStyle value (0-5)
    // 0: Unknown, treat as 1 (Not filled, only outlined)
    // 1: Not filled, only outlined
    // 2: Filled with color
    // 3: Filled with an X pattern (filled + special pattern)
    // 4: Filled with a pattern (filled + dashed pattern)
    // 5: Filled with a pattern and outlined (filled + dashed pattern + outline)
    fn apply_fill_style(&self, obj: &mut Object, fill_style: u8, layer_style: &LayerStyle) {
        let normalized_style = if fill_style == 0 { 1 } else { fill_style };

        match normalized_style {
            1 => {
                // Not filled, only outlined
                obj.set_stroke_color(Some(layer_style.stroke_color.clone()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some("none".to_string()));
            }
            2 => {
                // Filled with color
                obj.set_stroke_color(Some("none".to_string()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some(layer_style.stroke_color.clone()));
            }
            3 => {
                // Filled with an X pattern
                obj.set_stroke_color(Some("none".to_string()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some(layer_style.stroke_color.clone()));
                obj.apply_style_string("fillStyle=cross-hatch;");
            }
            4 => {
                // Filled with a pattern
                obj.set_stroke_color(Some("none".to_string()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some(layer_style.stroke_color.clone()));
                obj.apply_style_string("fillStyle=hatch;");
            }
            5 => {
                // Filled with pattern and outlined
                obj.set_stroke_color(Some(layer_style.stroke_color.clone()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some(layer_style.stroke_color.clone()));
                obj.apply_style_string("fillStyle=hatch;");
            }
            _ => {
                // Fallback to not filled
                obj.set_stroke_color(Some(layer_style.stroke_color.clone()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some("none".to_string()));
            }
        }
    }

    pub fn render_symbols_file(&self) -> DrawcktResult<SymbolContexts> {
        let mut contexts = SymbolContexts::new();

        // Create a separate drawio file for each symbol template
        let symbols: Vec<_> = self.schematic.symbols.iter().collect();
        for template in symbols {
            let mut symbol_file = File::new();
            let name = format!("{}/{}", template.lib, template.cell);
            let mut symbol_page = Page::new(Some(name.clone()), false);
            symbol_page.set_name(name);
            self.render_symbol(&mut symbol_page, template)?;

            symbol_file.add_page(symbol_page);
            let symbol_id = SymbolId {
                lib: template.lib.clone(),
                cell: template.cell.clone(),
            };
            contexts.insert(symbol_id, symbol_file.write());
        }

        Ok(contexts)
    }



    // Unified function to render a single Shape
    fn render_shape(
        &self,
        shape: &Shape,
        page: &mut Page,
        flip_y: &dyn Fn(f64) -> f64,
        obj_id: String,
    ) -> DrawcktResult<()> {
        match shape {
            Shape::Rect {
                    layer,
                    fill_style,
                    b_box,
                } => {
                    if b_box.len() >= 2 {
                        let x = b_box[0][0] * SCALE;
                        let y = flip_y(b_box[1][1]);
                        let width = (b_box[1][0] - b_box[0][0]) * SCALE;
                        let height = (b_box[1][1] - b_box[0][1]) * SCALE;

                        let layer_style = self.get_layer_style(layer);

                        let mut obj = Object::new(Some(obj_id));
                        obj.set_position([x, y]);
                        obj.set_width(width.abs());
                        obj.set_height(height.abs());
                        self.apply_fill_style(&mut obj, *fill_style, layer_style);
                        obj.set_xml_parent(Some(layer.id()));
                        page.add_object(DiagramObject::Object(obj));
                    }
                }
                Shape::Line { layer, points } => {
                    if points.len() >= 2 {
                        let source = &points[0];
                        let target = &points[points.len() - 1];
                        let intermediate = if points.len() > 2 {
                            points[1..points.len() - 1].to_vec()
                        } else {
                            Vec::new()
                        };

                        let width = (target[0] - source[0]).abs() * SCALE;
                        let height = (target[1] - source[1]).abs() * SCALE;

                        let source_x = source[0] * SCALE;
                        let source_y = flip_y(source[1]);
                        let target_x = target[0] * SCALE;
                        let target_y = flip_y(target[1]);

                        let layer_style = self.get_layer_style(layer);

                        let mut edge = Edge::new(Some(obj_id));
                        edge.set_stroke_width(Some(layer_style.stroke_width));
                        edge.set_stroke_color(Some(layer_style.stroke_color.clone()));
                        edge.set_xml_parent(Some(layer.id()));
                        edge.geometry().set_width(width);
                        edge.geometry().set_height(height);
                        edge.geometry().set_relative(Some(true));
                        edge.geometry().set_source_point(Some([source_x, source_y]));
                        edge.geometry().set_target_point(Some([target_x, target_y]));

                        for point in &intermediate {
                            let point_x = point[0] * SCALE;
                            let point_y = flip_y(point[1]);
                            edge.geometry().add_intermediate_point([point_x, point_y]);
                        }

                        page.add_object(DiagramObject::Edge(edge));
                    }
                }
                Shape::Label {
                    layer,
                    text,
                    xy,
                    orient: _,
                    height: _,
                    justify: _,
                } => {
                    let x = xy[0] * SCALE;
                    let y = flip_y(xy[1]);

                    let layer_style = self.get_layer_style(layer);
                    let mut obj = Object::new(Some(obj_id));
                    obj.set_value(text.clone());
                    obj.set_position([x, y]);
                    obj.set_width(100.0);
                    obj.set_height(30.0);
                    obj.set_fill_color(Some("none".to_string()));
                    obj.set_stroke_color(Some("none".to_string()));
                    obj.set_font_color(Some(layer_style.text_color.clone()));
                    obj.set_font_size(Some(layer_style.font_size as i32));
                    obj.set_xml_parent(Some(layer.id()));
                    page.add_object(obj.into());
                }
                Shape::Polygon {
                    layer,
                    fill_style,
                    points,
                } => {
                    if points.len() >= 3 {
                        // Calculate bounding box for the polygon
                        let mut min_x = points[0][0];
                        let mut min_y_local = points[0][1];
                        let mut max_x = points[0][0];
                        let mut max_y_local = points[0][1];

                        for point in points.iter() {
                            min_x = min_x.min(point[0]);
                            min_y_local = min_y_local.min(point[1]);
                            max_x = max_x.max(point[0]);
                            max_y_local = max_y_local.max(point[1]);
                        }

                        let x = min_x * SCALE;
                        let y = flip_y(max_y_local); // Flip the top y coordinate
                        let width = (max_x - min_x) * SCALE;
                        let height = (max_y_local - min_y_local) * SCALE;

                        // Convert points to normalized coordinates (0-1) within the bounding box
                        // Draw.io polygon uses polyCoords in format "[[x1,y1],[x2,y2],...]"
                        // Coordinates are normalized (0-1) relative to the bounding box
                        // Note: Y coordinates in polyCoords are also flipped (1 - norm_y)
                        let bbox_width = max_x - min_x;
                        let bbox_height = max_y_local - min_y_local;

                        let poly_coords: Vec<[f64; 2]> = points
                            .iter()
                            .map(|p| {
                                let norm_x = if bbox_width > 0.0 {
                                    (p[0] - min_x) / bbox_width
                                } else {
                                    0.0
                                };
                                // Flip Y coordinate: norm_y_flipped = 1 - norm_y
                                let norm_y = if bbox_height > 0.0 {
                                    (p[1] - min_y_local) / bbox_height
                                } else {
                                    0.0
                                };
                                let norm_y_flipped = 1.0 - norm_y;
                                [norm_x, norm_y_flipped]
                            })
                            .collect();

                        let layer_style = self.get_layer_style(layer);

                        let mut obj = Object::new(Some(obj_id));
                        obj.set_position([x, y]);
                        obj.set_width(width.abs());
                        obj.set_height(height.abs());
                        self.apply_fill_style(&mut obj, *fill_style, layer_style);
                        obj.set_xml_parent(Some(layer.id()));
                        obj.set_shape("mxgraph.basic.polygon".to_string());
                        obj.set_poly_coords(poly_coords);
                        page.add_object(DiagramObject::Object(obj));
                    }
                }
                Shape::Ellipse {
                    layer,
                    fill_style,
                    b_box,
                } => {
                    if b_box.len() >= 2 {
                        let x = b_box[0][0] * SCALE;
                        let y = flip_y(b_box[1][1]);
                        let width = (b_box[1][0] - b_box[0][0]) * SCALE;
                        let height = (b_box[1][1] - b_box[0][1]) * SCALE;

                        let layer_style = self.get_layer_style(layer);

                        let mut obj = Object::new(Some(obj_id));
                        obj.set_position([x, y]);
                        obj.set_width(width.abs());
                        obj.set_height(height.abs());
                        self.apply_fill_style(&mut obj, *fill_style, layer_style);
                        obj.set_xml_parent(Some(layer.id()));
                        obj.set_shape("ellipse".to_string());
                        page.add_object(obj.into());
                    }
                }
            }
        Ok(())
    }

    fn render_symbol(&self, page: &mut Page, template: &Symbol) -> DrawcktResult<()> {
        self.init_layers::<false>(page, &self.layer_styles.layer_order)?;

        // Helper function to flip Y coordinate: y_drawio = (symbol_max_y - y_json) * SCALE
        let flip_y = |y: f64| -> f64 { -y * SCALE };
        // Render each shape in the template using SCALE to convert coordinates and flip Y
        for (idx, shape) in template.shapes.iter().enumerate() {
            self.render_shape(shape, page, &flip_y, template.gen_obj_id(shape.layer(), idx))?;
        }

        Ok(())
    }

    pub fn render_schematic_file(&self, symbols_content: &SymbolContexts) -> DrawcktResult<String> {
        // Parse symbol contexts to extract pages
        let mut symbol_pages = IndexMap::new();
        for (symbol_id, content) in symbols_content.iter() {
            let symbol_key = format!("{}/{}", symbol_id.lib, symbol_id.cell);
            let pages = Self::parse_symbols_file(content)?;
            // Each symbol file should have only one page
            if let Some((_, page_data)) = pages.into_iter().next() {
                symbol_pages.insert(symbol_key, page_data);
            } else {
                return Err(DrawcktError::SymbolNotFound(symbol_key));
            }
        }
        // schematic.json uses normal coordinate system (Y increases upward)
        // drawio uses coordinate system where Y increases downward
        // Both instances and wires use y=0 as reference point for flipping
        // Helper function to flip Y coordinate: y_drawio = -y_json * SCALE
        let flip_y = |y: f64| -> f64 { -y * SCALE };

        // Create schematic.drawio canvas
        let mut schematic_file = File::new();

        // Set page name to "{lib}/{cell}"
        let page_name = format!(
            "{}/{}",
            self.schematic.design.lib, self.schematic.design.cell
        );
        let mut schematic_page = Page::new(Some(page_name.clone()), false);
        schematic_page.set_name(page_name.clone());
        self.init_layers::<true>(&mut schematic_page, &self.layer_styles.layer_order)?;

        // Process each instance
        for instance in &self.schematic.instances {
            let symbol_key = format!("{}/{}", instance.lib, instance.cell);

            if let Some(symbol_page_data) = symbol_pages.get(&symbol_key) {
                // Create GroupTransform using origin_bounding_box from SymbolPageData
                let group_transform = GroupTransform::new(
                    symbol_page_data.origin_bounding_box,
                    instance.x * SCALE,
                    flip_y(instance.y),
                    Orient::from_str(&instance.orient),
                    instance.name.clone(),
                );
                for obj in &symbol_page_data.objects {
                    // Get the new group bounding box
                    schematic_page.add_object(group_transform.new_obj(obj)?);
                }
            } else {
                return Err(DrawcktError::SymbolNotFound(symbol_key));
            }
        }

        // Render wires in wire layer
        let mut wire_counter = 0;
        for wire in &self.schematic.wires {
            if wire.points.len() >= 2 {
                let source = &wire.points[0];
                let target = &wire.points[wire.points.len() - 1];
                let intermediate = if wire.points.len() > 2 {
                    wire.points[1..wire.points.len() - 1].to_vec()
                } else {
                    Vec::new()
                };

                // Wire coordinates from schematic.json are in normal coordinate system
                // Need to flip Y coordinates to match drawio coordinate system
                // Wires use y=0 as reference point for flipping
                let source_x = source[0] * SCALE;
                let source_y = flip_y(source[1]);
                let target_x = target[0] * SCALE;
                let target_y = flip_y(target[1]);

                let width = (target_x - source_x).abs();
                let height = (target_y - source_y).abs();

                let mut edge = Edge::new(Some(Self::gen_wire_id(&wire.net, wire_counter)));
                wire_counter += 1;

                let wire_style = self.get_wire_style();
                edge.set_stroke_width(Some(wire_style.stroke_width));
                edge.set_stroke_color(Some(wire_style.stroke_color.clone()));
                edge.set_xml_parent(Some("layer-wire".to_string()));

                edge.geometry().set_width(width);
                edge.geometry().set_height(height);
                edge.geometry().set_relative(Some(true));
                edge.geometry().set_source_point(Some([source_x, source_y]));
                edge.geometry().set_target_point(Some([target_x, target_y]));

                for point in &intermediate {
                    let point_x = point[0] * SCALE;
                    let point_y = flip_y(point[1]);
                    edge.geometry().add_intermediate_point([point_x, point_y]);
                }

                schematic_page.add_object(DiagramObject::Edge(edge));
            }
        }

        // Render pins in pin layer
        let mut pin_counter = 0;
        for pin in &self.schematic.pins {
            let x = pin.x * SCALE;
            let y = flip_y(pin.y);

            let layer_style = self.get_layer_style(&Layer::Pin);
            let pin_id = format!("pin-{}-{}", pin.name, pin_counter);
            pin_counter += 1;

            let mut obj = Object::new(Some(pin_id));
            obj.set_value(pin.name.clone());
            obj.set_position([x, y]);
            obj.set_width(100.0);
            obj.set_height(30.0);
            obj.set_fill_color(Some("none".to_string()));
            obj.set_stroke_color(Some("none".to_string()));
            obj.set_font_color(Some(layer_style.text_color.clone()));
            obj.set_font_size(Some(layer_style.font_size as i32));
            // Set parent to pin layer id
            obj.set_xml_parent(Some(Layer::Pin.id()));
            schematic_page.add_object(obj.into());
        }

        // Render labels
        let mut label_counter = 0;
        for label in &self.schematic.labels {
            self.render_shape(label, &mut schematic_page, &flip_y, format!("label-{}", label_counter))?;
            label_counter += 1;
        }

        // Render shapes (with wire_show_intersection check)
        let mut shape_counter = 0;
        for shape in &self.schematic.shapes {
            
            // Skip wire layer shapes if wire_show_intersection is false
            if shape.layer().eq(&Layer::Wire) && !self.layer_styles.wire_show_intersection {
                continue;
            }
            
            self.render_shape(shape, &mut schematic_page, &flip_y, format!("shape-{}", shape_counter))?;
            shape_counter += 1;
        }

        schematic_file.add_page(schematic_page);
        Ok(schematic_file.write())
    }

    // Parse symbols.drawio file to extract pages
    pub fn parse_symbols_file(content: &str) -> DrawcktResult<IndexMap<String, SymbolPageData>> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);

        let mut pages = IndexMap::new();
        let mut buf = Vec::new();

        let mut current_page_name: Option<String> = None;
        let mut current_layer_names: Vec<String> = Vec::new();
        let mut current_layer_set: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut current_objects: Vec<drawrs::page::DiagramObject> = Vec::new();

        let mut in_diagram = false;
        let mut in_root = false;
        let mut in_object = false;
        let mut current_object_xml = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if name == "diagram" {
                        // Save previous page if exists
                        if let Some(prev_page_name) = current_page_name.take() {
                            let objects = std::mem::take(&mut current_objects);
                            let origin_bounding_box = BoundingBox::union(
                                objects.iter().filter_map(DiagramObject::bounding_box),
                            )
                            .unwrap_or_else(|| BoundingBox::new(0.0, 0.0, 0.0, 0.0));
                            pages.insert(
                                prev_page_name.clone(),
                                SymbolPageData {
                                    objects,
                                    origin_bounding_box,
                                },
                            );
                            current_layer_set.clear();
                        }

                        // Start new page (already cleared by take above)
                        in_diagram = false;

                        // Get name attribute
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "name" {
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                current_page_name = Some(value);
                                in_diagram = true;
                                break;
                            }
                        }
                    } else if name == "root" && in_diagram {
                        in_root = true;
                    } else if name == "UserObject" && in_root {
                        // Parse UserObject to extract tag (layer name) and start capturing XML
                        let mut tag_value: Option<String> = None;
                        let mut label_value: Option<String> = None;
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "tags" {
                                tag_value = Some(val.clone());
                                // Extract layer name from tag
                                if Self::parse_layer_name(&val).is_ok() {
                                    if !current_layer_set.contains(&val) {
                                        current_layer_names.push(val.clone());
                                        current_layer_set.insert(val.clone());
                                    }
                                }
                            } else if key == "label" {
                                label_value = Some(val);
                            }
                        }
                        // Start capturing XML for UserObject
                        in_object = true;
                        if let Some(ref tag) = tag_value {
                            if let Some(ref label) = label_value {
                                let label_escaped = XMLBase::xml_ify(label);
                                current_object_xml = format!(
                                    r#"<UserObject label="{}" tags="{}""#,
                                    label_escaped,
                                    XMLBase::xml_ify(tag)
                                );
                            } else {
                                current_object_xml = format!(
                                    r#"<UserObject label="" tags="{}""#,
                                    XMLBase::xml_ify(tag)
                                );
                            }
                        } else {
                            if let Some(ref label) = label_value {
                                let label_escaped = XMLBase::xml_ify(label);
                                current_object_xml =
                                    format!(r#"<UserObject label="{}""#, label_escaped);
                            } else {
                                current_object_xml = format!(r#"<UserObject label="""#);
                            }
                        }
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key != "tags" && key != "label" {
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                let val_escaped = XMLBase::xml_ify(&val);
                                current_object_xml
                                    .push_str(&format!(r#" {}="{}""#, key, val_escaped));
                            }
                        }
                        current_object_xml.push_str(">");
                    } else if name == "mxCell" && (in_root || in_object) {
                        // This is an object, start capturing XML
                        // IMPORTANT: quick_xml automatically decodes XML entities in attribute values
                        // So if symbols.drawio has value="cdsTerm(&quot;G&quot;)",
                        // attr.value will be "cdsTerm(\"G\")" (decoded)
                        // We need to re-escape it for the XML string we're building
                        if !in_object {
                            in_object = true;
                            current_object_xml = format!("<mxCell");
                        } else {
                            // Inside UserObject, add mxCell
                            current_object_xml.push_str("<mxCell");
                        }
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            // attr.value is already decoded by quick_xml, so we need to escape it
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            // Escape special characters for XML output
                            // Use xml_ify which properly handles &, <, >, ", '
                            let val_escaped = XMLBase::xml_ify(&val);
                            current_object_xml.push_str(&format!(r#" {}="{}""#, key, val_escaped));
                        }
                        current_object_xml.push_str(">");
                    } else if in_object
                        && (name == "mxGeometry" || name == "mxPoint" || name == "Array")
                    {
                        let tag = format!("<{}", name);
                        current_object_xml.push_str(&tag);
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            // attr.value is already decoded by quick_xml
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            // Escape for XML output
                            let val_escaped = XMLBase::xml_ify(&val);
                            current_object_xml.push_str(&format!(r#" {}="{}""#, key, val_escaped));
                        }
                        if name == "mxPoint" || name == "Array" {
                            current_object_xml.push_str(" />");
                        } else {
                            current_object_xml.push_str(">");
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if name == "diagram" {
                        if let Some(page_name) = current_page_name.take() {
                            let objects = std::mem::take(&mut current_objects);
                            let origin_bounding_box = BoundingBox::union(
                                objects.iter().filter_map(DiagramObject::bounding_box),
                            )
                            .unwrap_or_else(|| BoundingBox::new(0.0, 0.0, 0.0, 0.0));
                            pages.insert(
                                page_name.clone(),
                                SymbolPageData {
                                    objects,
                                    origin_bounding_box,
                                },
                            );
                            current_layer_set.clear();
                        }
                        in_diagram = false;
                        in_root = false;
                    } else if name == "root" {
                        in_root = false;
                    } else if name == "UserObject" && in_object {
                        // End of UserObject, parse the complete object (including inner mxCell)
                        current_object_xml.push_str("</UserObject>");
                        // Parse XML and create Object or Edge instance
                        current_objects.push(parse_xml_to_object(&current_object_xml)?);
                        in_object = false;
                        current_object_xml.clear();
                    } else if name == "mxCell" && in_object {
                        current_object_xml.push_str("</mxCell>");
                        // Parse XML and create Object, Edge, or XmlBase (for groups) instance
                        current_objects.push(parse_xml_to_object(&current_object_xml)?);
                        in_object = false;
                        current_object_xml.clear();
                    } else if in_object && name == "mxGeometry" {
                        current_object_xml.push_str("</mxGeometry>");
                    }
                }
                Ok(Event::Empty(e)) => {
                    // Handle self-closing tags
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if in_object && (name == "mxGeometry" || name == "mxPoint" || name == "Array") {
                        // Handle self-closing tags like <mxGeometry ... /> within objects
                        let tag = format!("<{}", name);
                        current_object_xml.push_str(&tag);
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            // attr.value is already decoded by quick_xml
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            // Escape for XML output
                            let val_escaped = XMLBase::xml_ify(&val);
                            current_object_xml.push_str(&format!(r#" {}="{}""#, key, val_escaped));
                        }
                        current_object_xml.push_str(" />");
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(DrawcktError::XmlParsing(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(pages)
    }

    fn parse_layer_name(s: &str) -> DrawcktResult<String> {
        match s {
            "instance" | "annotate" | "pin" | "device" => Ok(s.to_string()),
            _ => Err(DrawcktError::UnknownLayer(s.to_string())),
        }
    }
}
