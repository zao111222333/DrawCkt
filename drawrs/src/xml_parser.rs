use crate::BoundingBox;
use crate::diagram::{Edge, Object};
use crate::error::{DrawrsError, DrawrsResult};
use crate::page::DiagramObject;
use crate::xml_base::XMLBase;
use quick_xml::Reader;
use quick_xml::events::Event;
use uuid;

/// Parse XML string to Object or Edge (without transformation)
pub fn parse_xml_to_object(xml_obj: &str) -> DrawrsResult<DiagramObject> {
    // Parse XML to extract attributes
    let mut reader = Reader::from_str(xml_obj);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut obj_id: Option<String> = None;
    let mut user_object_id: Option<String> = None; // id from UserObject tag
    let mut user_object_tag: Option<String> = None; // tags from UserObject tag
    let mut user_object_label: Option<String> = None; // label from UserObject tag (takes priority over mxCell value)
    let mut parent_id: Option<String> = None;
    let mut value: Option<String> = None;
    let mut style: Option<String> = None;
    let mut edge: Option<i32> = None;

    // Geometry attributes
    let mut geom_x: Option<f64> = None;
    let mut geom_y: Option<f64> = None;
    let mut geom_width: Option<f64> = None;
    let mut geom_height: Option<f64> = None;
    let mut geom_relative: Option<bool> = None;
    let mut source_point: Option<[f64; 2]> = None;
    let mut target_point: Option<[f64; 2]> = None;
    let mut intermediate_points: Vec<[f64; 2]> = Vec::new();
    let mut in_geometry = false;
    let mut in_array = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if name == "UserObject" {
                    // Parse UserObject to extract id, tags, and label (takes priority over mxCell id and value)
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "id" => user_object_id = Some(val),
                            "tags" => user_object_tag = Some(val),
                            "label" => user_object_label = Some(val),
                            _ => {}
                        }
                    }
                } else if name == "mxCell" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "id" => obj_id = Some(val),
                            "parent" => parent_id = Some(val),
                            "value" => value = Some(val),
                            "style" => style = Some(val),
                            "edge" => edge = val.parse().ok(),
                            _ => {}
                        }
                    }
                } else if name == "mxGeometry" {
                    in_geometry = true;
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "x" => geom_x = val.parse().ok(),
                            "y" => geom_y = val.parse().ok(),
                            "width" => geom_width = val.parse().ok(),
                            "height" => geom_height = val.parse().ok(),
                            "relative" => geom_relative = Some(val == "1"),
                            _ => {}
                        }
                    }
                    // Default missing x or y to 0.0
                    if geom_x.is_none() {
                        geom_x = Some(0.0);
                    }
                    if geom_y.is_none() {
                        geom_y = Some(0.0);
                    }
                } else if name == "mxPoint" && in_geometry {
                    let mut point_x: Option<f64> = None;
                    let mut point_y: Option<f64> = None;
                    let mut point_as: Option<String> = None;

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "x" => point_x = val.parse().ok(),
                            "y" => point_y = val.parse().ok(),
                            "as" => point_as = Some(val),
                            _ => {}
                        }
                    }

                    // Default missing x or y to 0.0
                    let x = point_x.unwrap_or(0.0);
                    let y = point_y.unwrap_or(0.0);
                    match point_as.as_deref() {
                        Some("sourcePoint") => source_point = Some([x, y]),
                        Some("targetPoint") => target_point = Some([x, y]),
                        _ => {
                            if in_array {
                                intermediate_points.push([x, y]);
                            } else {
                                intermediate_points.push([x, y]);
                            }
                        }
                    }
                } else if name == "Array" && in_geometry {
                    in_array = true;
                }
            }
            Ok(Event::Empty(e)) => {
                // Handle self-closing tags like <mxGeometry ... />
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "mxGeometry" {
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "x" => geom_x = val.parse().ok(),
                            "y" => geom_y = val.parse().ok(),
                            "width" => geom_width = val.parse().ok(),
                            "height" => geom_height = val.parse().ok(),
                            "relative" => geom_relative = Some(val == "1"),
                            _ => {}
                        }
                    }
                    // Default missing x or y to 0.0
                    if geom_x.is_none() {
                        geom_x = Some(0.0);
                    }
                    if geom_y.is_none() {
                        geom_y = Some(0.0);
                    }
                } else if name == "mxPoint" && in_geometry {
                    let mut point_x: Option<f64> = None;
                    let mut point_y: Option<f64> = None;
                    let mut point_as: Option<String> = None;

                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let val = String::from_utf8_lossy(&attr.value).to_string();
                        match key.as_str() {
                            "x" => point_x = val.parse().ok(),
                            "y" => point_y = val.parse().ok(),
                            "as" => point_as = Some(val),
                            _ => {}
                        }
                    }

                    // Default missing x or y to 0.0
                    let x = point_x.unwrap_or(0.0);
                    let y = point_y.unwrap_or(0.0);
                    match point_as.as_deref() {
                        Some("sourcePoint") => source_point = Some([x, y]),
                        Some("targetPoint") => target_point = Some([x, y]),
                        _ => {
                            intermediate_points.push([x, y]);
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "mxGeometry" {
                    in_geometry = false;
                } else if name == "Array" {
                    in_array = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(DrawrsError::XmlParsing(e)),
            _ => {}
        }
        buf.clear();
    }

    // Check if this is a group mxCell (has style="group" or style contains "group")
    let is_group = style
        .as_ref()
        .map(|s| s == "group" || s.contains("group"))
        .unwrap_or(false);

    // If this is a group mxCell without UserObject, return XMLBase
    if is_group && user_object_id.is_none() {
        let final_id = obj_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let final_parent_id = parent_id.unwrap_or_else(|| "1".to_string());

        let mut xml_base = XMLBase::new(Some(final_id));
        xml_base.xml_class = "mxCell".to_string();
        xml_base.xml_parent = Some(final_parent_id);

        // Set group_geometry if geometry is available
        if let (Some(x), Some(y), Some(w), Some(h)) = (geom_x, geom_y, geom_width, geom_height) {
            xml_base.group_geometry = Some(BoundingBox::new(x, y, w, h));
        }

        return Ok(DiagramObject::XmlBase(xml_base));
    }

    // Determine if this is an edge or an object
    let is_edge = edge == Some(1) || source_point.is_some() || target_point.is_some();

    // Use UserObject id if available, then mxCell id, otherwise generate new one
    let final_id = user_object_id
        .or(obj_id)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let final_parent_id = parent_id.unwrap_or_else(|| "1".to_string());

    if is_edge {
        // Create Edge
        let mut edge_obj = Edge::new(Some(final_id));

        if let Some(s) = style {
            edge_obj.parse_and_set_style(&s);
        }

        // Use UserObject label if available, otherwise use mxCell value
        let final_value = user_object_label.or(value);
        if let Some(v) = final_value {
            edge_obj.base_mut().value = Some(v);
        }

        edge_obj.set_xml_parent(Some(final_parent_id));

        // Set tag from UserObject if available
        if let Some(tag) = user_object_tag {
            edge_obj.base_mut().tag = Some(tag);
        }

        let geom = edge_obj.geometry();

        if let Some(sp) = source_point {
            geom.set_source_point(Some(sp));
        }

        if let Some(tp) = target_point {
            geom.set_target_point(Some(tp));
        }

        for point in intermediate_points {
            geom.add_intermediate_point(point);
        }

        if let Some(w) = geom_width {
            geom.set_width(w);
        }
        if let Some(h) = geom_height {
            geom.set_height(h);
        }
        if let Some(r) = geom_relative {
            geom.set_relative(Some(r));
        }

        Ok(DiagramObject::Edge(edge_obj))
    } else {
        // Create Object
        let mut obj = Object::new(Some(final_id));

        if let Some(s) = style {
            obj.parse_and_set_style(&s);
        }

        // Use UserObject label if available, otherwise use mxCell value
        let final_value = user_object_label.or(value);
        if let Some(v) = final_value {
            obj.set_value(v);
        }

        obj.set_xml_parent(Some(final_parent_id));

        // Set tag from UserObject if available
        if let Some(tag) = user_object_tag {
            obj.base_mut().tag = Some(tag);
        }

        if let (Some(x), Some(y)) = (geom_x, geom_y) {
            obj.set_position([x, y]);
        }

        if let Some(w) = geom_width {
            obj.set_width(w);
        }
        if let Some(h) = geom_height {
            obj.set_height(h);
        }

        Ok(DiagramObject::Object(obj))
    }
}
