use crate::xml_base::XMLBase;
use indexmap::IndexMap;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct DiagramBase {
    base: XMLBase,
    page: Option<String>, // Reference to page by ID
    // Only stores unsupported style keys
    unsupported_style_properties: IndexMap<Cow<'static, str>, Cow<'static, str>>,
}

impl DiagramBase {
    pub fn new(id: Option<String>) -> Self {
        let mut base = XMLBase::new(id);
        base.xml_class = "xml_tag".to_string();
        Self {
            base,
            page: None,
            unsupported_style_properties: IndexMap::new(),
        }
    }

    pub fn with_page(id: Option<String>, page: Option<String>) -> Self {
        let mut base = XMLBase::new(id);
        base.xml_class = "xml_tag".to_string();
        Self {
            base,
            page,
            unsupported_style_properties: IndexMap::new(),
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

    pub fn unsupported_style_properties(&self) -> &IndexMap<Cow<'static, str>, Cow<'static, str>> {
        &self.unsupported_style_properties
    }

    pub fn unsupported_style_properties_mut(
        &mut self,
    ) -> &mut IndexMap<Cow<'static, str>, Cow<'static, str>> {
        &mut self.unsupported_style_properties
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

    /// Parse style string into key-value pairs
    pub fn parse_style_string(style_str: &str) -> Vec<(&str, &str)> {
        let mut key_value_list = Vec::new();
        for part in style_str.split(';') {
            if part.is_empty() {
                continue;
            } else if part.contains('=') {
                let parts: Vec<&str> = part.splitn(2, '=').collect();
                if parts.len() == 2 {
                    key_value_list.push((parts[0], parts[1]));
                }
            }
        }
        key_value_list
    }

    /// Apply a single style property (for unsupported keys only)
    pub fn apply_style_property(&mut self, key: Cow<'static, str>, value: Cow<'static, str>) {
        self.unsupported_style_properties.insert(key, value);
    }

    /// Remove an unsupported style property
    pub fn remove_style_property(&mut self, key: &str) {
        self.unsupported_style_properties.shift_remove(key);
    }

    /// Build style string from supported properties and unsupported properties
    /// supported_properties: Vec of (key, value) pairs for supported style keys
    pub fn build_style_string(&self, supported_properties: &[(&str, String)]) -> String {
        let mut style_str = String::new();

        // Add supported properties first
        for (key, value) in supported_properties {
            style_str.push_str(key);
            style_str.push('=');
            style_str.push_str(value);
            style_str.push(';');
        }

        // Add unsupported properties
        for (key, value) in &self.unsupported_style_properties {
            style_str.push_str(key);
            style_str.push('=');
            style_str.push_str(value);
            style_str.push(';');
        }

        style_str
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
