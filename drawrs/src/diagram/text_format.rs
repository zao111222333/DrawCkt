use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyX {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyY {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Justify {
    pub x: JustifyX,
    pub y: JustifyY,
}
// Custom serialization to maintain backward compatibility
impl Serialize for Justify {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match (self.x, self.y) {
            (JustifyX::Left, JustifyY::Top) => "upperLeft",
            (JustifyX::Center, JustifyY::Top) => "upperCenter",
            (JustifyX::Right, JustifyY::Top) => "upperRight",
            (JustifyX::Left, JustifyY::Middle) => "centerLeft",
            (JustifyX::Center, JustifyY::Middle) => "centerCenter",
            (JustifyX::Right, JustifyY::Middle) => "centerRight",
            (JustifyX::Left, JustifyY::Bottom) => "lowerLeft",
            (JustifyX::Center, JustifyY::Bottom) => "lowerCenter",
            (JustifyX::Right, JustifyY::Bottom) => "lowerRight",
        };
        serializer.serialize_str(s)
    }
}

// Custom deserialization to maintain backward compatibility
impl<'de> Deserialize<'de> for Justify {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (x, y) = match s.as_str() {
            "upperLeft" => (JustifyX::Left, JustifyY::Top),
            "upperCenter" => (JustifyX::Center, JustifyY::Top),
            "upperRight" => (JustifyX::Right, JustifyY::Top),
            "centerLeft" => (JustifyX::Left, JustifyY::Middle),
            "centerCenter" => (JustifyX::Center, JustifyY::Middle),
            "centerRight" => (JustifyX::Right, JustifyY::Middle),
            "lowerLeft" => (JustifyX::Left, JustifyY::Bottom),
            "lowerCenter" => (JustifyX::Center, JustifyY::Bottom),
            "lowerRight" => (JustifyX::Right, JustifyY::Bottom),
            _ => return Err(serde::de::Error::custom(format!("Unknown justify: {}", s))),
        };
        Ok(Justify { x, y })
    }
}

impl Justify {
    pub fn new() -> Self {
        Self {
            x: JustifyX::Center,
            y: JustifyY::Middle,
        }
    }

    pub fn with_x(mut self, x: JustifyX) -> Self {
        self.x = x;
        self
    }

    pub fn with_y(mut self, y: JustifyY) -> Self {
        self.y = y;
        self
    }

    pub fn parse(style_str: &str) -> Self {
        let mut justify = Justify::new();
        for part in style_str.split(';') {
            if part.is_empty() {
                continue;
            } else if part.contains('=') {
                let parts: Vec<&str> = part.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    match key {
                        "align" => {
                            justify.x = match value {
                                "left" => JustifyX::Left,
                                "right" => JustifyX::Right,
                                _ => JustifyX::Center,
                            };
                        }
                        "verticalAlign" => {
                            justify.y = match value {
                                "top" => JustifyY::Top,
                                "bottom" => JustifyY::Bottom,
                                _ => JustifyY::Middle,
                            };
                        }
                        _ => {}
                    }
                }
            }
        }
        justify
    }

    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        let align = match self.x {
            JustifyX::Left => "left",
            JustifyX::Center => "center",
            JustifyX::Right => "right",
        };
        parts.push(format!("align={}", align));
        let vertical_align = match self.y {
            JustifyY::Top => "top",
            JustifyY::Middle => "middle",
            JustifyY::Bottom => "bottom",
        };
        parts.push(format!("verticalAlign={}", vertical_align));
        if parts.is_empty() {
            String::new()
        } else {
            parts.join(";") + ";"
        }
    }
}

impl Default for Justify {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct TextFormat {
    font_size: Option<f64>,
    font_color: Option<String>,
    justify: Justify,
    // bold: bool,
    // italic: bool,
    // underline: bool,
}

impl TextFormat {
    pub fn new() -> Self {
        Self {
            font_size: None,
            font_color: None,
            justify: Justify::new(),
            // bold: false,
            // italic: false,
            // underline: false,
        }
    }

    pub fn font_color(&self) -> Option<&String> {
        self.font_color.as_ref()
    }

    pub fn set_font_color(&mut self, color: Option<String>) {
        self.font_color = color;
    }

    pub fn font_size(&self) -> Option<f64> {
        self.font_size
    }

    pub fn set_font_size(&mut self, size: Option<f64>) {
        self.font_size = size;
    }

    pub fn justify(&self) -> &Justify {
        &self.justify
    }

    pub fn justify_mut(&mut self) -> &mut Justify {
        &mut self.justify
    }

    pub fn set_justify(&mut self, justify: Justify) {
        self.justify = justify;
    }
}

impl Default for TextFormat {
    fn default() -> Self {
        Self::new()
    }
}
