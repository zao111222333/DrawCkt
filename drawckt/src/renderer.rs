use crate::error::{DrawcktError, DrawcktResult};
use crate::schematic::*;
use drawrs::FillStyle;
use drawrs::diagram::text_format::{Justify, JustifyX, JustifyY};
use drawrs::xml_base::XMLBase;
use drawrs::{
    BoundingBox, DiagramObject, DrawFile, Edge, GroupTransform, Object, Page, parse_xml_to_object,
};
use indexmap::IndexMap;
use log::info;
use ordered_float::OrderedFloat;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::borrow::Cow;
use std::collections::HashMap;
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

impl LayerStyle {
    fn update_label(
        obj: &mut DiagramObject,
        old_style: &Self,
        new_style: &Self,
    ) -> DrawcktResult<()> {
        if let Some(object) = obj.as_object_mut() {
            // Update font color
            if old_style.text_color != new_style.text_color {
                object.set_font_color(Some(new_style.text_color.clone().into_owned()));
            }

            // Update font size based on font_zoom ratio
            if let Some(current_font_size) = object.font_size() {
                if old_style.font_zoom != new_style.font_zoom && old_style.font_zoom > 0.0 {
                    let new_font_size =
                        current_font_size * (new_style.font_zoom / old_style.font_zoom);
                    object.set_font_size(Some(new_font_size));

                    // Update width proportionally if it was calculated from text length
                    if let Some(text) = object.value() {
                        let font_height = new_font_size;
                        let font_width = font_height * text.len() as f64 / 2.0;
                        object.set_width(font_width);
                        object.set_height(font_height);
                    }
                }
            }

            // Update font family
            if old_style.font_family != new_style.font_family {
                object.set_font_family(Some(new_style.font_family.clone().into_owned()));
            }
        }
        Ok(())
    }
    fn update_shape(
        obj: &mut DiagramObject,
        old_style: &Self,
        new_style: &Self,
    ) -> DrawcktResult<()> {
        match obj {
            DiagramObject::Edge(edge) => {
                if old_style.stroke_color != new_style.stroke_color {
                    edge.set_stroke_color(Some(new_style.stroke_color.clone().into_owned()));
                }
                if old_style.stroke_width != new_style.stroke_width {
                    edge.set_stroke_width(Some(new_style.stroke_width));
                }
            }
            DiagramObject::Object(object) => {
                if let Some(color) = object.stroke_color()
                    && color != "none"
                {
                    object.set_stroke_width(Some(new_style.stroke_width));
                    object.set_stroke_color(Some(new_style.stroke_color.clone().into_owned()));
                }
                if let Some(color) = object.fill_color()
                    && color != "none"
                {
                    object.set_fill_color(Some(new_style.stroke_color.clone().into_owned()));
                }
            }
            DiagramObject::XmlBase(_) => {
                // XmlBase objects don't need style updates
            }
        }
        Ok(())
    }
}

impl SymbolPageData {
    pub fn update_style(
        self,
        old_style: &LayerStyles,
        new_style: &LayerStyles,
    ) -> impl Iterator<Item = DrawcktResult<Option<DiagramObject>>> {
        self.objects.into_iter().map(|mut obj| {
            match obj.xml_parent() {
                Some("layer-instance-label") => {
                    LayerStyle::update_label(&mut obj, &old_style.instance, &new_style.instance)?
                }
                Some("layer-instance-shape") => {
                    LayerStyle::update_shape(&mut obj, &old_style.instance, &new_style.instance)?
                }
                Some("layer-annotate-label") => {
                    LayerStyle::update_label(&mut obj, &old_style.annotate, &new_style.annotate)?
                }
                Some("layer-annotate-shape") => {
                    LayerStyle::update_shape(&mut obj, &old_style.annotate, &new_style.annotate)?
                }
                Some("layer-pin-label") => {
                    LayerStyle::update_label(&mut obj, &old_style.pin, &new_style.pin)?
                }
                Some("layer-pin-shape") => {
                    LayerStyle::update_shape(&mut obj, &old_style.pin, &new_style.pin)?
                }
                Some("layer-device-label") => {
                    LayerStyle::update_label(&mut obj, &old_style.device, &new_style.device)?
                }
                Some("layer-device-shape") => {
                    LayerStyle::update_shape(&mut obj, &old_style.device, &new_style.device)?
                }
                Some("layer-wire-label") => {
                    LayerStyle::update_label(&mut obj, &old_style.wire, &new_style.wire)?
                }
                Some("layer-wire-shape") => {
                    LayerStyle::update_shape(&mut obj, &old_style.wire, &new_style.wire)?
                }
                Some("layer-wire-intersection") => {
                    // update bounding box based on wire_intersection_scale change
                    if let Some((bbox, _)) = obj.mut_box() {
                        let old_scale = old_style.wire_intersection_scale;
                        let new_scale = new_style.wire_intersection_scale;

                        if (old_scale - new_scale).abs() > f64::EPSILON && old_scale > 0.0 {
                            // Calculate center point
                            let center_x = bbox.min_x + bbox.width / 2.0;
                            let center_y = bbox.min_y + bbox.height / 2.0;

                            // Calculate relative scale factor
                            let scale_ratio = new_scale / old_scale;

                            // Scale width and height
                            let new_width = bbox.width * scale_ratio;
                            let new_height = bbox.height * scale_ratio;

                            // Update bounding box while keeping center point unchanged
                            bbox.min_x = center_x - new_width / 2.0;
                            bbox.min_y = center_y - new_height / 2.0;
                            bbox.width = new_width;
                            bbox.height = new_height;
                        }
                    }
                    LayerStyle::update_shape(&mut obj, &old_style.wire, &new_style.wire)?
                }
                Some("layer-text-label") => {
                    LayerStyle::update_label(&mut obj, &old_style.text, &new_style.text)?
                }
                Some("layer-text-shape") => {
                    LayerStyle::update_shape(&mut obj, &old_style.text, &new_style.text)?
                }
                _ => {}
            }
            Ok(Some(obj))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolId<'a> {
    pub lib: Cow<'a, str>,
    pub cell: Cow<'a, str>,
}

pub struct SymbolContexts<'a>(pub IndexMap<SymbolId<'a>, Cow<'a, str>>);

impl<'a> SymbolContexts<'a> {
    /// Write all symbols to directory structure: {dir}/{lib}/{cell}.drawio
    pub fn write_to_dir(&self, dir: impl AsRef<Path>) -> DrawcktResult<()> {
        let output_path = dir.as_ref();
        fs::create_dir_all(output_path)?;

        for (symbol_id, content) in &self.0 {
            let lib_dir = output_path.join(symbol_id.lib.as_ref());
            fs::create_dir_all(&lib_dir)?;
            let cell_file = lib_dir.join(format!("{}.drawio", symbol_id.cell));
            fs::write(&cell_file, content.as_ref())?;
            info!("Symbol rendered to: {:?}", cell_file);
        }

        Ok(())
    }

    /// Load symbols from directory structure: {dir}/{lib}/{cell}.drawio
    pub fn load_from_dir(dir: impl AsRef<Path>) -> DrawcktResult<Self> {
        let symbols_path = dir.as_ref();
        let mut symbol_contexts = IndexMap::new();

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
                                lib: lib_name.to_string().into(),
                                cell: cell_name.to_string().into(),
                            };
                            symbol_contexts.insert(symbol_id, content.to_string().into());
                        }
                    }
                }
            }
            Ok(Self(symbol_contexts))
        } else {
            Err(DrawcktError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Symbols directory not found: {:?}", symbols_path),
            )))
        }
    }
}

pub struct Renderer<'a> {
    schematic: &'a Schematic,
    layer_styles: &'a LayerStyles,
}

impl<'a> Renderer<'a> {
    pub fn new(schematic: &'a Schematic, layer_styles: &'a LayerStyles) -> Self {
        Self {
            schematic,
            layer_styles,
        }
    }

    fn init_layers(style: &LayerStyles, page: &mut Page) -> DrawcktResult<()> {
        let mut instance = false;
        let mut annotate = false;
        let mut pin = false;
        let mut device = false;
        let mut wire = false;
        let mut text = false;
        for layer in style.layer_order.iter().rev() {
            match layer {
                Layer::Instance => {
                    if instance {
                        return Err(DrawcktError::RepeatLayer(*layer));
                    }
                    instance = true;
                }
                Layer::Annotate => {
                    if annotate {
                        return Err(DrawcktError::RepeatLayer(*layer));
                    }
                    annotate = true;
                }
                Layer::Pin => {
                    if pin {
                        return Err(DrawcktError::RepeatLayer(*layer));
                    }
                    pin = true;
                }
                Layer::Device => {
                    if device {
                        return Err(DrawcktError::RepeatLayer(*layer));
                    }
                    device = true;
                }
                Layer::Wire => {
                    if wire {
                        return Err(DrawcktError::RepeatLayer(*layer));
                    }
                    wire = true;
                }
                Layer::Text => {
                    if text {
                        return Err(DrawcktError::RepeatLayer(*layer));
                    }
                    text = true;
                }
            }
            if *layer == Layer::Wire {
                let mut intersection_layer_cell =
                    drawrs::xml_base::XMLBase::new(Some(layer.id_shape(true)));
                intersection_layer_cell.xml_class = "mxCell".to_string();
                intersection_layer_cell.xml_parent = Some("0".to_string());
                intersection_layer_cell.value = Some(format!("{layer}-intersection"));
                intersection_layer_cell.visible = Some(if style.wire_show_intersection {
                    "1".to_string()
                } else {
                    "0".to_string()
                });
                page.add_object(drawrs::DiagramObject::XmlBase(intersection_layer_cell));
            }
            let mut shape_layer_cell = drawrs::xml_base::XMLBase::new(Some(layer.id_shape(false)));
            shape_layer_cell.xml_class = "mxCell".to_string();
            shape_layer_cell.xml_parent = Some("0".to_string());
            shape_layer_cell.value = Some(format!("{layer}-shape"));
            shape_layer_cell.visible = Some(if style.layer_style(layer).shape_sch_visible {
                "1".to_string()
            } else {
                "0".to_string()
            });
            page.add_object(drawrs::DiagramObject::XmlBase(shape_layer_cell));
            let mut label_layer_cell = drawrs::xml_base::XMLBase::new(Some(layer.id_label()));
            label_layer_cell.xml_class = "mxCell".to_string();
            label_layer_cell.xml_parent = Some("0".to_string());
            label_layer_cell.value = Some(format!("{layer}-label"));
            label_layer_cell.visible = Some(if style.layer_style(layer).label_sch_visible {
                "1".to_string()
            } else {
                "0".to_string()
            });
            page.add_object(drawrs::DiagramObject::XmlBase(label_layer_cell));
        }
        Ok(())
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

    // Convert wires to HashMap grouped by net, with each wire as a Shape::Line
    fn wires_to_shapes_by_net(&self) -> HashMap<String, Vec<&Vec<[OrderedFloat<f64>; 2]>>> {
        let mut shapes_by_net = HashMap::new();
        for wire in &self.schematic.wires {
            if wire.points.len() >= 2 {
                _ = shapes_by_net
                    .entry(wire.net.clone())
                    .or_insert_with(Vec::new)
                    .push(&wire.points);
            }
        }
        shapes_by_net
    }

    // Merge lines that share endpoints (same net, same endpoint, only two lines at that point)
    pub(crate) fn merge_lines(
        lines: Vec<&Vec<[OrderedFloat<f64>; 2]>>,
    ) -> Vec<Vec<[OrderedFloat<f64>; 2]>> {
        if lines.is_empty() {
            return Vec::new();
        }

        let mut merged: Vec<Vec<[OrderedFloat<f64>; 2]>> = Vec::new();
        let mut processed = vec![false; lines.len()];

        for i in 0..lines.len() {
            if processed[i] {
                continue;
            }

            let mut current_line = lines[i].clone();

            if current_line.is_empty() {
                processed[i] = true;
                continue;
            }

            // Keep trying to merge until no more merges are possible
            loop {
                let mut merged_this_round = false;

                // Try to merge with all other unprocessed lines
                for j in 0..lines.len() {
                    if i == j || processed[j] {
                        continue;
                    }

                    let other_line = lines[j];

                    if other_line.is_empty() {
                        continue;
                    }

                    let current_start = &current_line[0];
                    let current_end = &current_line[current_line.len() - 1];
                    let other_start = &other_line[0];
                    let other_end = &other_line[other_line.len() - 1];

                    // Check if lines share an endpoint
                    let share_start_start = current_start == other_start;
                    let share_start_end = current_start == other_end;
                    let share_end_start = current_end == other_start;
                    let share_end_end = current_end == other_end;

                    if !share_start_start && !share_start_end && !share_end_start && !share_end_end
                    {
                        continue;
                    }

                    // Determine the shared point
                    let shared_point = if share_start_start {
                        Some(current_start)
                    } else if share_start_end {
                        Some(current_start)
                    } else if share_end_start {
                        Some(current_end)
                    } else {
                        Some(current_end)
                    };

                    if let Some(shared) = shared_point {
                        // Count how many lines connect at this point
                        // We need to check:
                        // 1. Unprocessed lines (besides i and j)
                        // 2. Already merged lines that are kept separate (in merged array)
                        let mut connection_count = 0;

                        // Check unprocessed lines
                        for k in 0..lines.len() {
                            if k == i || k == j || processed[k] {
                                continue;
                            }
                            let points = &lines[k];
                            if !points.is_empty() {
                                let line_start = &points[0];
                                let line_end = &points[points.len() - 1];
                                if line_start == shared || line_end == shared {
                                    connection_count += 1;
                                }
                            }
                        }

                        // Check already merged lines that are kept separate
                        for merged_line in &merged {
                            if !merged_line.is_empty() {
                                let line_start = &merged_line[0];
                                let line_end = &merged_line[merged_line.len() - 1];
                                if line_start == shared || line_end == shared {
                                    connection_count += 1;
                                }
                            }
                        }

                        // Only merge if no other lines connect at this point
                        // This ensures that we only merge when exactly two lines meet at a point
                        if connection_count == 0 {
                            // Merge the lines
                            let mut new_points = current_line.clone();

                            if share_start_start {
                                // Reverse current line and append other line
                                new_points.reverse();
                                new_points.pop(); // Remove duplicate point
                                new_points.extend_from_slice(other_line);
                            } else if share_start_end {
                                // Prepend other line (reversed) to current line
                                // current_start == other_end, so we reverse other_line and remove its first point
                                let mut other_reversed = other_line.clone();
                                _ = other_reversed.pop(); // Remove duplicate point (first point after reverse)
                                other_reversed.reverse();
                                other_reversed.extend_from_slice(&new_points);
                                new_points = other_reversed;
                            } else if share_end_start {
                                // Append other line to current
                                new_points.pop(); // Remove duplicate point
                                new_points.extend_from_slice(other_line);
                            } else if share_end_end {
                                // Append reversed other line to current
                                let mut other_reversed = other_line.clone();
                                other_reversed.reverse();
                                new_points.pop(); // Remove duplicate point
                                new_points.extend_from_slice(&other_reversed);
                            }

                            current_line = new_points;
                            processed[j] = true;
                            merged_this_round = true;
                            // Continue to try merging with remaining lines
                        }
                    }
                }

                // If no merge happened this round, we're done with this line
                if !merged_this_round {
                    break;
                }
            }

            merged.push(current_line);
            processed[i] = true;
        }

        merged
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
                obj.set_stroke_color(Some(layer_style.stroke_color.clone().into_owned()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some("none".to_string()));
            }
            2 => {
                // Filled with color
                obj.set_stroke_color(Some("none".to_string()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some(layer_style.stroke_color.clone().into_owned()));
            }
            3 => {
                // Filled with an X pattern
                obj.set_stroke_color(Some("none".to_string()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some(layer_style.stroke_color.clone().into_owned()));
                obj.set_fill_style(Some(FillStyle::CrossHatch));
            }
            4 => {
                // Filled with a pattern
                obj.set_stroke_color(Some("none".to_string()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some(layer_style.stroke_color.clone().into_owned()));
                obj.set_fill_style(Some(FillStyle::Hatch));
            }
            5 => {
                // Filled with pattern and outlined
                obj.set_stroke_color(Some(layer_style.stroke_color.clone().into_owned()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some(layer_style.stroke_color.clone().into_owned()));
                obj.set_fill_style(Some(FillStyle::Hatch));
            }
            _ => {
                // Fallback to not filled
                obj.set_stroke_color(Some(layer_style.stroke_color.clone().into_owned()));
                obj.set_stroke_width(Some(layer_style.stroke_width));
                obj.set_fill_color(Some("none".to_string()));
            }
        }
    }

    pub fn render_symbols_file<'b>(&'b self) -> DrawcktResult<SymbolContexts<'b>> {
        let contexts = self
            .schematic
            .symbols
            .iter()
            .map(|template| {
                let name = format!("{}/{}", template.lib, template.cell);
                let mut symbol_page = Page::new(Some(name.clone()), false);
                symbol_page.set_name(name);
                self.render_symbol(&mut symbol_page, template)?;
                let mut symbol_file = DrawFile::new();
                symbol_file.add_page(symbol_page);
                let symbol_id = SymbolId {
                    lib: template.lib.as_str().into(),
                    cell: template.cell.as_str().into(),
                };
                Ok((symbol_id, symbol_file.xml().to_string().into()))
            })
            .collect::<Result<_, DrawcktError>>()?;
        Ok(SymbolContexts(contexts))
    }

    // Unified function to render a single Shape
    fn render_shape(
        &self,
        shape: &Shape,
        page: &mut Page,
        obj_id: String,
        is_intersection: bool,
    ) -> DrawcktResult<()> {
        match shape {
            Shape::Rect {
                layer,
                fill_style,
                b_box,
            } => {
                if b_box.len() >= 2 {
                    let x = b_box[0][0] * SCALE;
                    let y = -b_box[1][1] * SCALE;
                    let width = (b_box[1][0] - b_box[0][0]) * SCALE;
                    let height = (b_box[1][1] - b_box[0][1]) * SCALE;

                    let layer_style = self.layer_styles.layer_style(layer);

                    let mut obj = Object::new(Some(obj_id));
                    obj.set_position([*x, *y]);
                    obj.set_width(width.abs());
                    obj.set_height(height.abs());
                    self.apply_fill_style(&mut obj, *fill_style, layer_style);
                    obj.set_xml_parent(Some(layer.id_shape(is_intersection)));
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
                    let source_y = -source[1] * SCALE;
                    let target_x = target[0] * SCALE;
                    let target_y = -target[1] * SCALE;

                    let layer_style = self.layer_styles.layer_style(layer);

                    let mut edge = Edge::new(Some(obj_id));
                    edge.set_stroke_width(Some(layer_style.stroke_width));
                    edge.set_stroke_color(Some(layer_style.stroke_color.clone().into_owned()));
                    edge.set_xml_parent(Some(layer.id_shape(is_intersection)));
                    edge.geometry().set_width(width);
                    edge.geometry().set_height(height);
                    edge.geometry().set_relative(Some(true));
                    edge.geometry()
                        .set_source_point(Some([*source_x, *source_y]));
                    edge.geometry()
                        .set_target_point(Some([*target_x, *target_y]));

                    for point in &intermediate {
                        let point_x = point[0] * SCALE;
                        let point_y = -point[1] * SCALE;
                        edge.geometry().add_intermediate_point([*point_x, *point_y]);
                    }

                    page.add_object(DiagramObject::Edge(edge));
                }
            }
            Shape::Label {
                layer,
                text,
                xy,
                orient: _,
                height,
                justify,
            } => {
                let layer_style = self.layer_styles.layer_style(layer);
                let mut x = xy[0] * SCALE;
                let mut y = -xy[1] * SCALE;
                let font_height = 1.2 * height.as_ref() * SCALE * layer_style.font_zoom;
                let font_width = font_height * text.len() as f64 / 2.0;
                let mut obj = Object::new(Some(obj_id));
                {
                    // Adjust x based on JustifyX
                    match justify.x {
                        JustifyX::Left => {
                            // x is already at the left edge, no adjustment needed
                        }
                        JustifyX::Center => {
                            x -= font_width / 2.0;
                        }
                        JustifyX::Right => {
                            x -= font_width;
                        }
                    }
                    // Adjust y based on JustifyY
                    match justify.y {
                        JustifyY::Top => {
                            // y is already at the top edge, no adjustment needed
                        }
                        JustifyY::Middle => {
                            y -= font_height / 2.0;
                        }
                        JustifyY::Bottom => {
                            y -= font_height;
                            obj.apply_style_property("spacingBottom", "-2");
                        }
                    }
                }

                obj.set_value(text.clone());
                obj.set_position([*x, *y]);
                obj.set_width(font_width);
                obj.set_height(font_height);
                obj.set_fill_color(Some("none".to_string()));
                obj.set_stroke_color(Some("none".to_string()));
                obj.set_font_color(Some(layer_style.text_color.clone().into_owned()));
                obj.set_font_size(Some(font_height));
                obj.set_font_family(Some(layer_style.font_family.clone().into_owned()));
                obj.set_xml_parent(Some(layer.id_label()));
                obj.set_justify(*justify);
                obj.apply_style_property("spacing", "0");
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
                    let y = -max_y_local * SCALE;
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
                            let norm_x = if *bbox_width > 0.0 {
                                (p[0].as_ref() - min_x.as_ref()) / bbox_width.as_ref()
                            } else {
                                0.0
                            };
                            // Flip Y coordinate: norm_y_flipped = 1 - norm_y
                            let norm_y = if *bbox_height > 0.0 {
                                (p[1].as_ref() - min_y_local.as_ref()) / bbox_height.as_ref()
                            } else {
                                0.0
                            };
                            let norm_y_flipped = 1.0 - norm_y;
                            [norm_x, norm_y_flipped]
                        })
                        .collect();

                    let layer_style = self.layer_styles.layer_style(layer);

                    let mut obj = Object::new(Some(obj_id));
                    obj.set_position([*x, *y]);
                    obj.set_width(width.abs());
                    obj.set_height(height.abs());
                    self.apply_fill_style(&mut obj, *fill_style, layer_style);
                    obj.set_xml_parent(Some(layer.id_shape(is_intersection)));
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
                    let y = -b_box[1][1] * SCALE;
                    let width = (b_box[1][0] - b_box[0][0]) * SCALE;
                    let height = (b_box[1][1] - b_box[0][1]) * SCALE;

                    let layer_style = self.layer_styles.layer_style(layer);

                    let mut obj = Object::new(Some(obj_id));
                    obj.set_position([*x, *y]);
                    obj.set_width(width.abs());
                    obj.set_height(height.abs());
                    self.apply_fill_style(&mut obj, *fill_style, layer_style);
                    obj.set_xml_parent(Some(layer.id_shape(is_intersection)));
                    obj.set_shape("ellipse".to_string());
                    page.add_object(obj.into());
                }
            }
        }
        Ok(())
    }

    fn render_symbol(&self, page: &mut Page, template: &Symbol) -> DrawcktResult<()> {
        Self::init_layers(&self.layer_styles, page)?;

        let mut lines_wire = Vec::new();
        let mut lines_instance = Vec::new();
        let mut lines_annotate = Vec::new();
        let mut lines_pin = Vec::new();
        let mut lines_device = Vec::new();
        let mut lines_text = Vec::new();
        let mut idx = 0;
        for shape in &template.shapes {
            if let Shape::Line { layer, points } = shape {
                match layer {
                    Layer::Wire => lines_wire.push(points),
                    Layer::Instance => lines_instance.push(points),
                    Layer::Annotate => lines_annotate.push(points),
                    Layer::Pin => lines_pin.push(points),
                    Layer::Device => lines_device.push(points),
                    Layer::Text => lines_text.push(points),
                }
            } else {
                self.render_shape(shape, page, template.gen_obj_id(shape.layer(), idx), false)?;
                idx += 1;
            }
        }
        for (layer, lines) in [
            (Layer::Wire, lines_wire),
            (Layer::Instance, lines_instance),
            (Layer::Annotate, lines_annotate),
            (Layer::Pin, lines_pin),
            (Layer::Device, lines_device),
            (Layer::Text, lines_text),
        ] {
            for points in Self::merge_lines(lines) {
                self.render_shape(
                    &Shape::Line { layer, points },
                    page,
                    template.gen_obj_id(&layer, idx),
                    false,
                )?;
                idx += 1;
            }
        }
        Ok(())
    }

    pub fn render_schematic_file(&self, symbols_content: &SymbolContexts) -> DrawcktResult<String> {
        // Parse symbol contexts to extract pages
        let mut symbol_pages = IndexMap::new();
        for (symbol_id, content) in &symbols_content.0 {
            let mut pages = Self::parse_drawio_file(content)?;
            // Each symbol file should have only one page
            if let Some((_, page_data)) = pages.pop() {
                symbol_pages.insert((symbol_id.lib.as_ref(), symbol_id.cell.as_ref()), page_data);
            } else {
                return Err(DrawcktError::SymbolNotFound(format!(
                    "{}/{}",
                    symbol_id.lib, symbol_id.cell
                )));
            }
        }

        // Create schematic.drawio canvas
        let mut schematic_file = DrawFile::new();

        // Set page name to "{lib}/{cell}"
        let page_name = format!(
            "{}/{}",
            self.schematic.design.lib, self.schematic.design.cell
        );
        let mut schematic_page = Page::new(Some(page_name.clone()), false);
        schematic_page.set_name(page_name);
        Self::init_layers(&self.layer_styles, &mut schematic_page)?;

        // Process each instance
        for instance in &self.schematic.instances {
            if let Some(symbol_page_data) =
                symbol_pages.get(&(instance.lib.as_ref(), instance.cell.as_ref()))
            {
                // Create GroupTransform using origin_bounding_box from SymbolPageData
                let group_transform = GroupTransform::new(
                    symbol_page_data.origin_bounding_box,
                    instance.x * SCALE,
                    -instance.y * SCALE,
                    instance.orient,
                    &instance.name,
                    &instance.cell,
                );
                for obj in &symbol_page_data.objects {
                    // Get the new group bounding box
                    schematic_page.add_object(group_transform.new_obj(obj)?);
                }
            } else {
                return Err(DrawcktError::SymbolNotFound(format!(
                    "{}/{}",
                    instance.lib, instance.cell
                )));
            }
        }

        // Render wires in wire layer
        // Convert wires to HashMap grouped by net, then merge lines and render using Shape::Line
        let wires_by_net = self.wires_to_shapes_by_net();
        let mut wire_counter = 0;

        for (net_name, lines) in wires_by_net {
            // Merge lines that share endpoints
            let merged_lines = Self::merge_lines(lines);

            // Render each merged line using render_shape
            for line in merged_lines {
                wire_counter += 1;
                self.render_shape(
                    &Shape::Line {
                        points: line,
                        layer: Layer::Wire,
                    },
                    &mut schematic_page,
                    Self::gen_wire_id(&net_name, wire_counter),
                    false,
                )?;
            }
        }

        // Render pins in pin layer
        for (i, pin) in self.schematic.pins.iter().enumerate() {
            self.render_shape(
                &Shape::Label {
                    layer: Layer::Pin,
                    text: pin.name.clone(),
                    xy: [
                        ordered_float::OrderedFloat(pin.x - 0.175),
                        ordered_float::OrderedFloat(pin.y),
                    ],
                    orient: "".to_string(),
                    height: ordered_float::OrderedFloat(0.1),
                    justify: Justify {
                        x: JustifyX::Right,
                        y: JustifyY::Middle,
                    },
                },
                &mut schematic_page,
                format!("pin-{i}"),
                false,
            )?;
        }

        // Render labels
        for (i, label) in self.schematic.labels.iter().enumerate() {
            self.render_shape(label, &mut schematic_page, format!("label-{i}"), false)?;
        }

        // Render shapes (with wire_show_intersection check)
        for (i, shape) in self.schematic.shapes.iter().enumerate() {
            // Skip wire layer shapes if wire_show_intersection is false
            if shape.layer().eq(&Layer::Wire) {
                if let Shape::Ellipse {
                    layer,
                    fill_style,
                    b_box,
                } = shape
                {
                    // Scale the bounding box while keeping the center point unchanged
                    let scale = self.layer_styles.wire_intersection_scale;
                    let center_x = (b_box[0][0] + b_box[1][0]) / 2.0;
                    let center_y = (b_box[0][1] + b_box[1][1]) / 2.0;
                    let width = b_box[1][0] - b_box[0][0];
                    let height = b_box[1][1] - b_box[0][1];
                    let new_width = width * scale;
                    let new_height = height * scale;
                    let scaled_b_box = [
                        [center_x - new_width / 2.0, center_y - new_height / 2.0],
                        [center_x + new_width / 2.0, center_y + new_height / 2.0],
                    ];
                    self.render_shape(
                        &Shape::Ellipse {
                            layer: *layer,
                            fill_style: *fill_style,
                            b_box: scaled_b_box,
                        },
                        &mut schematic_page,
                        format!("shape-{i}"),
                        true,
                    )?;
                } else {
                    self.render_shape(shape, &mut schematic_page, format!("shape-{i}"), false)?;
                }
            } else {
                self.render_shape(shape, &mut schematic_page, format!("shape-{i}"), false)?;
            }
        }

        schematic_file.add_page(schematic_page);
        Ok(schematic_file.xml().to_string())
    }

    // Parse symbols.drawio file to extract pages
    pub fn parse_drawio_file(content: &str) -> DrawcktResult<IndexMap<String, SymbolPageData>> {
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

    pub fn update_style(
        content: &str,
        old_style: &LayerStyles,
        new_style: &LayerStyles,
    ) -> DrawcktResult<String> {
        // Each symbol file should have only one page
        if let Some((page_name, page_data)) = Self::parse_drawio_file(content)?.pop() {
            let mut page = Page::new(Some(page_name.clone()), false);
            page.set_name(page_name);
            Self::init_layers(new_style, &mut page)?;
            for obj_res in page_data.update_style(old_style, new_style) {
                // Get the new group bounding box
                if let Some(obj) = obj_res? {
                    page.add_object(obj);
                }
            }
            let mut file = DrawFile::new();
            file.add_page(page);
            Ok(file.xml().to_string())
        } else {
            Err(DrawcktError::NoPage)
        }
    }
}
