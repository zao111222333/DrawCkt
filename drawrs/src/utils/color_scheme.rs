use crate::utils::StandardColor;

#[derive(Debug, Clone)]
pub enum ColorInput {
    Hex(String),
    Standard(StandardColor),
    None,
}

impl From<&str> for ColorInput {
    fn from(s: &str) -> Self {
        if s == "none" {
            ColorInput::None
        } else if s.starts_with('#') {
            ColorInput::Hex(s.to_string())
        } else {
            panic!("Invalid color string: {}", s);
        }
    }
}

impl From<StandardColor> for ColorInput {
    fn from(c: StandardColor) -> Self {
        ColorInput::Standard(c)
    }
}
