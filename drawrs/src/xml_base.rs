use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct XMLBase {
    pub id: String,
    pub xml_class: String,
    pub xml_parent: Option<String>,
    pub tag: Option<String>,
    pub tooltip: Option<String>,
    pub visible: Option<String>,
    pub value: Option<String>,
    pub group_geometry: Option<BoundingBox>,
}

impl XMLBase {
    pub fn new(id: Option<String>) -> Self {
        Self {
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            xml_class: "xml_tag".to_string(),
            xml_parent: None,
            tag: None,
            tooltip: None,
            visible: None,
            value: None,
            group_geometry: None,
        }
    }

    pub fn with_class(xml_class: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            xml_class,
            xml_parent: None,
            tag: None,
            tooltip: None,
            visible: None,
            value: None,
            group_geometry: None,
        }
    }

    pub fn attributes(&self) -> HashMap<String, Option<String>> {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), Some(self.id.clone()));
        attrs.insert("parent".to_string(), self.xml_parent.clone());
        attrs.insert("visible".to_string(), self.visible.clone());
        attrs.insert("value".to_string(), self.value.clone());
        attrs
    }

    pub fn xml_open_tag(&self) -> String {
        let mut tag = format!("<{}", self.xml_class);
        for (key, value) in self.attributes() {
            if let Some(val) = value {
                let xml_val = Self::xml_ify(&val);
                tag.push_str(&format!(r#" {}="{}""#, key, xml_val));
            }
        }
        tag.push('>');
        tag
    }

    pub fn xml_close_tag(&self) -> String {
        format!("</{}>", self.xml_class)
    }

    pub fn value_mut(&mut self) -> Option<&mut String> {
        self.value.as_mut()
    }

    pub fn xml(&self) -> XMLBaseXml<'_> {
        XMLBaseXml(self)
    }
}

pub struct XMLBaseXml<'a>(&'a XMLBase);

impl<'a> fmt::Display for XMLBaseXml<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Special handling for group mxCell
        if self.0.xml_class == "mxCell" && self.0.group_geometry.is_some() {
            let bbox = self.0.group_geometry.unwrap();
            let parent_id = self
                .0
                .xml_parent
                .as_ref()
                .map(|p| p.as_str())
                .unwrap_or("1");
            if let Some(v) = &self.0.value {
                write!(
                    f,
                    r#"<mxCell id="{}" connectable="0" parent="{}" style="group" value="{}" vertex="1">
          <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
        </mxCell>"#,
                    XMLBase::xml_ify(&self.0.id),
                    parent_id,
                    XMLBase::xml_ify(v),
                    bbox.min_x,
                    bbox.min_y,
                    bbox.width,
                    bbox.height
                )
            } else {
                write!(
                    f,
                    r#"<mxCell id="{}" connectable="0" parent="{}" style="group" vertex="1">
          <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
        </mxCell>"#,
                    XMLBase::xml_ify(&self.0.id),
                    parent_id,
                    bbox.min_x,
                    bbox.min_y,
                    bbox.width,
                    bbox.height
                )
            }
        } else {
            // Always output as self-closing tag, value is included in attributes
            write!(f, "<{}", self.0.xml_class)?;
            write!(f, r#" id="{}""#, XMLBase::xml_ify(&self.0.id))?;
            if let Some(ref parent) = self.0.xml_parent {
                write!(f, r#" parent="{}""#, XMLBase::xml_ify(parent))?;
            }
            if let Some(ref visible) = self.0.visible {
                write!(f, r#" visible="{}""#, XMLBase::xml_ify(visible))?;
            }
            if let Some(ref value) = self.0.value {
                write!(f, r#" value="{}""#, XMLBase::xml_ify(value))?;
            }
            write!(f, " />")
        }
    }
}

impl XMLBase {
    pub fn xml_ify(parameter_string: &str) -> String {
        // First, decode any existing XML entities to avoid double-escaping
        let decoded = Self::decode_xml_entities(parameter_string);
        Self::translate_txt(&decoded, &XML_ESCAPE_MAP)
    }

    pub fn decode_xml_entities(s: &str) -> String {
        // Decode common XML entities to avoid double-escaping
        // This ensures that if a string contains &quot; as a literal, it gets decoded to "
        // before being escaped again
        s.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
    }

    pub fn translate_txt(string: &str, replacement_dict: &HashMap<char, &str>) -> String {
        let mut new_str = String::new();
        for char in string.chars() {
            if let Some(replacement) = replacement_dict.get(&char) {
                new_str.push_str(replacement);
            } else {
                new_str.push(char);
            }
        }
        new_str
    }
}

impl Default for XMLBase {
    fn default() -> Self {
        Self::new(None)
    }
}

use once_cell::sync::Lazy;

use crate::BoundingBox;

static XML_ESCAPE_MAP: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert('>', "&gt;");
    m.insert('<', "&lt;");
    m.insert('&', "&amp;");
    m.insert('"', "&quot;");
    m.insert('\'', "&apos;");
    m.insert('\n', "&#xa;");
    m.insert('\t', "&#x9;");
    m.insert('\r', "&#xd;");
    m
});
