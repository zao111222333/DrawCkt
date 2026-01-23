#[derive(Clone, Debug)]
pub struct TextFormat {
    font_size: Option<f64>,
    font_color: Option<String>,
    // bold: bool,
    // italic: bool,
    // underline: bool,
}

impl TextFormat {
    pub fn new() -> Self {
        Self {
            font_size: None,
            font_color: None,
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
}

impl Default for TextFormat {
    fn default() -> Self {
        Self::new()
    }
}
