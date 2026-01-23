use core::fmt;

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
    pub fn id(&self) -> String {
        format!("layer-{self}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStyle {
    pub stroke_color: String,
    pub stroke_width: f64,
    pub text_color: String,
    pub font_zoom: f64,
    #[serde(default = "default_sch_visible")]
    pub sch_visible: bool,
}

fn default_sch_visible() -> bool {
    true
}

impl Default for LayerStyle {
    fn default() -> Self {
        Self {
            stroke_color: "#000000".to_string(),
            stroke_width: 1.0,
            text_color: "#000000".to_string(),
            font_zoom: 1.0,
            sch_visible: true,
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
    pub annotate: LayerStyle,
    pub pin: LayerStyle,
    pub text: LayerStyle,
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
            device: LayerStyle {
                stroke_color: "#00FF00".to_string(),
                stroke_width: 2.0,
                text_color: "#FF0000".to_string(),
                font_zoom: 1.0,
                sch_visible: true,
            },
            instance: LayerStyle {
                stroke_color: "#0000FF".to_string(),
                stroke_width: 1.0,
                text_color: "#0000FF".to_string(),
                font_zoom: 1.0,
                sch_visible: false,
            },
            wire: LayerStyle {
                stroke_color: "#00FFFF".to_string(),
                stroke_width: 2.0,
                text_color: "#00CCCC".to_string(),
                font_zoom: 1.0,
                sch_visible: true,
            },
            wire_show_intersection: true,
            annotate: LayerStyle {
                stroke_color: "#00FF00".to_string(),
                stroke_width: 1.0,
                text_color: "#FF9900".to_string(),
                font_zoom: 1.0,
                sch_visible: false,
            },
            pin: LayerStyle {
                stroke_color: "#FF0000".to_string(),
                stroke_width: 2.0,
                text_color: "#FF0000".to_string(),
                font_zoom: 1.0,
                sch_visible: true,
            },
            text: LayerStyle {
                stroke_color: "#666666".to_string(),
                stroke_width: 1.0,
                text_color: "#666666".to_string(),
                font_zoom: 1.0,
                sch_visible: true,
            },
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
    pub orient: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    pub net: String,
    #[serde(rename = "points")]
    pub points: Vec<[f64; 2]>,
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
    pub shapes: Vec<Shape>,
    pub pins: Vec<TemplatePin>,
}

impl Symbol {
    // Generate ID for symbol layer/object/edge: {lib}/{cell}-{layer}-{idx}
    pub fn gen_obj_id(&self, layer: &Layer, idx: usize) -> String {
        format!("{}/{}-{}-{}", self.lib, self.cell, layer, idx)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Shape {
    #[serde(rename = "polygon")]
    Polygon {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        #[serde(rename = "fillStyle", default = "default_fill_style")]
        fill_style: u8,
        points: Vec<[f64; 2]>,
    },
    #[serde(rename = "rect")]
    Rect {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        #[serde(rename = "fillStyle", default = "default_fill_style")]
        fill_style: u8,
        #[serde(rename = "bBox")]
        b_box: Vec<[f64; 2]>,
    },
    #[serde(rename = "label")]
    Label {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        text: String,
        xy: [f64; 2],
        orient: String,
        height: f64,
        justify: Justify,
    },
    #[serde(rename = "line")]
    Line {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        points: Vec<[f64; 2]>,
    },
    #[serde(rename = "ellipse")]
    Ellipse {
        #[serde(deserialize_with = "deserialize_layer")]
        layer: Layer,
        #[serde(rename = "fillStyle", default = "default_fill_style")]
        fill_style: u8,
        #[serde(rename = "bBox")]
        b_box: Vec<[f64; 2]>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Justify {
    #[serde(rename = "upperLeft")]
    UpperLeft,
    #[serde(rename = "upperCenter")]
    UpperCenter,
    #[serde(rename = "upperRight")]
    UpperRight,
    #[serde(rename = "centerLeft")]
    CenterLeft,
    #[serde(rename = "centerCenter")]
    CenterCenter,
    #[serde(rename = "centerRight")]
    CenterRight,
    #[serde(rename = "lowerLeft")]
    LowerLeft,
    #[serde(rename = "lowerCenter")]
    LowerCenter,
    #[serde(rename = "lowerRight")]
    LowerRight,
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
