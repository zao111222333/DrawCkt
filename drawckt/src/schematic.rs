use core::fmt;
use std::borrow::Cow;

use drawrs::{Orient, diagram::text_format::Justify};
use indexmap::IndexSet;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    Instance,
    Annotate,
    Pin,
    Device,
    Wire,
    Text,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Layer::Instance => "instance",
            Layer::Annotate => "annotate",
            Layer::Pin => "pin",
            Layer::Device => "device",
            Layer::Wire => "wire",
            Layer::Text => "text",
        })
    }
}

impl Layer {
    pub fn id_label(&self) -> String {
        format!("layer-{self}-label")
    }
    pub fn id_shape(&self, is_intersection: bool) -> String {
        if is_intersection {
            format!("layer-{self}-intersection")
        } else {
            format!("layer-{self}-shape")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStyle {
    pub stroke_color: Cow<'static, str>,
    pub stroke_width: f64,
    pub text_color: Cow<'static, str>,
    pub font_zoom: f64,
    pub font_family: Cow<'static, str>,
    pub label_sch_visible: bool,
    pub shape_sch_visible: bool,
}

impl LayerStyle {
    pub const fn new(
        stroke_color: &'static str,
        stroke_width: f64,
        text_color: &'static str,
        font_zoom: f64,
        font_family: &'static str,
        label_sch_visible: bool,
        shape_sch_visible: bool,
    ) -> Self {
        Self {
            stroke_color: Cow::Borrowed(stroke_color),
            stroke_width,
            text_color: Cow::Borrowed(text_color),
            font_zoom,
            font_family: Cow::Borrowed(font_family),
            label_sch_visible,
            shape_sch_visible,
        }
    }
}

impl Default for LayerStyle {
    fn default() -> Self {
        Self {
            stroke_color: "#000000".into(),
            stroke_width: 1.0,
            text_color: "#000000".into(),
            font_zoom: 1.0,
            font_family: "Times New Roman".into(),
            label_sch_visible: true,
            shape_sch_visible: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStyles {
    pub layer_order: [Layer; 6],
    pub device: LayerStyle,
    pub instance: LayerStyle,
    pub wire: LayerStyle,
    pub wire_show_intersection: bool,
    pub wire_intersection_scale: f64,
    pub annotate: LayerStyle,
    pub pin: LayerStyle,
    pub text: LayerStyle,
}

impl LayerStyles {
    pub(crate) fn layer_style<'a>(&'a self, layer: &Layer) -> &'a LayerStyle {
        match layer {
            Layer::Instance => &self.instance,
            Layer::Annotate => &self.annotate,
            Layer::Pin => &self.pin,
            Layer::Device => &self.device,
            Layer::Wire => &self.wire,
            Layer::Text => &self.text,
        }
    }
}

impl Default for LayerStyles {
    fn default() -> Self {
        Self {
            layer_order: [
                Layer::Text,
                Layer::Pin,
                Layer::Wire,
                Layer::Annotate,
                Layer::Instance,
                Layer::Device,
            ],
            device: LayerStyle::default(),
            instance: LayerStyle::default(),
            wire: LayerStyle::default(),
            wire_show_intersection: true,
            wire_intersection_scale: 1.0,
            annotate: LayerStyle::default(),
            pin: LayerStyle::default(),
            text: LayerStyle::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schematic {
    pub design: Design,
    pub instances: Vec<Instance>,
    pub wires: Vec<Wire>,
    pub pins: Vec<Pin>,
    pub symbols: Vec<Symbol>,
    pub labels: Vec<Shape>,
    pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Design {
    pub lib: String,
    pub cell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub lib: String,
    pub cell: String,
    pub x: f64,
    pub y: f64,
    pub orient: Orient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    pub net: String,
    pub points: Vec<[OrderedFloat<f64>; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    pub name: String,
    pub direction: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub lib: String,
    pub cell: String,
    pub shapes: IndexSet<Shape>,
    pub pins: Vec<TemplatePin>,
}

impl Symbol {
    // Generate ID for symbol layer/object/edge: {lib}/{cell}-{layer}-{idx}
    pub fn gen_obj_id(&self, layer: &Layer, idx: usize) -> String {
        format!("{}/{}-{}-{}", self.lib, self.cell, layer, idx)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Shape {
    #[serde(rename = "polygon")]
    Polygon {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        #[serde(rename = "fillStyle", default = "default_fill_style")]
        fill_style: u8,
        points: Vec<[OrderedFloat<f64>; 2]>,
    },
    #[serde(rename = "rect")]
    Rect {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        #[serde(rename = "fillStyle", default = "default_fill_style")]
        fill_style: u8,
        #[serde(rename = "bBox")]
        b_box: [[OrderedFloat<f64>; 2]; 2],
    },
    #[serde(rename = "label")]
    Label {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        text: String,
        xy: [OrderedFloat<f64>; 2],
        orient: String,
        height: OrderedFloat<f64>,
        justify: Justify,
    },
    #[serde(rename = "line")]
    Line {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        points: Vec<[OrderedFloat<f64>; 2]>,
    },
    #[serde(rename = "ellipse")]
    Ellipse {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        #[serde(rename = "fillStyle", default = "default_fill_style")]
        fill_style: u8,
        #[serde(rename = "bBox")]
        b_box: [[OrderedFloat<f64>; 2]; 2],
    },
}

impl Shape {
    // Helper function to extract layer from Shape
    pub fn layer(&self) -> &Layer {
        match self {
            Self::Rect { layer, .. }
            | Self::Line { layer, .. }
            | Self::Label { layer, .. }
            | Self::Polygon { layer, .. }
            | Self::Ellipse { layer, .. } => layer,
        }
    }
}
fn default_fill_style() -> u8 {
    1 // Default: Not filled, only outlined
}

fn deserialize_layer<'de, D>(deserializer: D) -> Result<Layer, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "instance" => Ok(Layer::Instance),
        "annotate" => Ok(Layer::Annotate),
        "pin" => Ok(Layer::Pin),
        "device" => Ok(Layer::Device),
        "wire" => Ok(Layer::Wire),
        "text" => Ok(Layer::Text),
        _ => Err(serde::de::Error::custom(format!("Unknown layer: {}", s))),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePin {
    pub name: String,
    pub direction: String,
    pub x: f64,
    pub y: f64,
}
