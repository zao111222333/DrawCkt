use crate::xml_base::XMLBase;
use indexmap::IndexMap;

#[derive(Clone, Debug)]
pub struct DiagramBase {
    base: XMLBase,
    style_attributes: Vec<String>,
    page: Option<String>, // Reference to page by ID
    base_style: Option<String>,
    style_properties: IndexMap<String, String>,
}

impl DiagramBase {
    pub fn new(id: Option<String>) -> Self {
        let mut base = XMLBase::new(id);
        base.xml_class = "xml_tag".to_string();
        Self {
            base,
            style_attributes: vec!["html".to_string()],
            page: None,
            base_style: None,
            style_properties: IndexMap::new(),
        }
    }

    pub fn with_page(id: Option<String>, page: Option<String>) -> Self {
        let mut base = XMLBase::new(id);
        base.xml_class = "xml_tag".to_string();
        Self {
            base,
            style_attributes: vec!["html".to_string()],
            page,
            base_style: None,
            style_properties: IndexMap::new(),
        }
    }
    pub fn base(&self) -> &XMLBase {
        &self.base
    }
    pub fn base_mut(&mut self) -> &mut XMLBase {
        &mut self.base
    }
    pub fn id(&self) -> &str {
        &self.base.id
    }

    pub fn set_id(&mut self, id: String) {
        self.base.id = id;
    }

    pub fn style_attributes(&self) -> &[String] {
        &self.style_attributes
    }

    pub fn add_style_attribute(&mut self, attr: String) {
        if !self.style_attributes.contains(&attr) {
            self.style_attributes.push(attr);
        }
    }

    pub fn page(&self) -> Option<&String> {
        self.page.as_ref()
    }

    pub fn set_page(&mut self, page: Option<String>) {
        self.page = page;
        // Update base XML parent if page is set
        if let Some(ref page_id) = self.page {
            self.base.xml_parent = Some(page_id.clone());
        }
    }

    pub fn xml_parent(&self) -> Option<&String> {
        self.base.xml_parent.as_ref()
    }

    pub fn set_xml_parent(&mut self, parent: Option<String>) {
        self.base.xml_parent = parent;
    }

    pub fn xml_parent_id(&self) -> String {
        self.base
            .xml_parent
            .clone()
            .unwrap_or_else(|| "1".to_string())
    }

    pub fn page_id(&self) -> String {
        self.page.clone().unwrap_or_else(|| "1".to_string())
    }

    pub fn set_style_property(&mut self, key: String, value: String) {
        let key_clone = key.clone();
        self.style_properties.insert(key, value);
        // Add to style_attributes if not already present
        if !self.style_attributes.contains(&key_clone) {
            self.add_style_attribute(key_clone);
        }
    }

    pub fn remove_style_property(&mut self, key: &str) {
        self.style_properties.shift_remove(key);
    }

    pub fn style(&self) -> String {
        let mut style_str = String::new();

        if let Some(ref base) = self.base_style {
            style_str.push_str(base);
            style_str.push(';');
        }

        for attr in &self.style_attributes {
            if let Some(value) = self.style_properties.get(attr) {
                style_str.push_str(attr);
                style_str.push('=');
                style_str.push_str(value);
                style_str.push(';');
            }
        }

        style_str
    }

    pub fn apply_style_string(&mut self, style_str: &str) {
        for part in style_str.split(';') {
            if part.is_empty() {
                continue;
            } else if part.contains('=') {
                let parts: Vec<&str> = part.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    self.set_style_property(key, value);
                }
            } else {
                // Base style without '='
                self.base_style = Some(part.to_string());
            }
        }
    }
}

impl Default for DiagramBase {
    fn default() -> Self {
        Self::new(None)
    }
}

pub fn color_input_check(color_str: Option<&str>) -> Option<String> {
    match color_str {
        None => None,
        Some("none") => Some("none".to_string()),
        Some("default") => Some("default".to_string()),
        Some(s) if s.starts_with('#') && s.len() == 7 => Some(s.to_string()),
        _ => None,
    }
}

pub fn width_input_check(width: Option<i32>) -> Option<i32> {
    match width {
        None => None,
        Some(w) if w < 1 => Some(1),
        Some(w) if w > 999 => Some(999),
        Some(w) => Some(w),
    }
}
