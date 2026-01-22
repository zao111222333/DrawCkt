use crate::page::Page;
use crate::xml_base::XMLBase;
use chrono::Utc;

pub struct File {
    pub base: XMLBase,
    pub pages: Vec<Page>,
    pub host: String,
    pub file_type: String,
    pub version: String,
}

impl File {
    pub fn new() -> Self {
        let mut base = XMLBase::new(None);
        base.xml_class = "mxfile".to_string();
        Self {
            base,
            pages: Vec::new(),
            host: "Electron".to_string(),
            file_type: "device".to_string(),
            version: "21.6.5".to_string(),
        }
    }

    pub fn with_name(_file_name: String) -> Self {
        Self::new()
    }

    pub fn with_path(_output_path: std::path::PathBuf) -> Self {
        Self::new()
    }

    pub fn add_page(&mut self, mut page: crate::page::Page) {
        page.set_page_num(self.pages.len() + 1);
        self.pages.push(page);
    }

    pub fn remove_page(&mut self, page_id: &str) {
        self.pages.retain(|p| p.id() != page_id);
    }

    pub fn stats(&self) -> String {
        let object_count: usize = self.pages.iter().map(|p| p.objects().len()).sum();
        format!("Pages: {} | Objects: {}", self.pages.len(), object_count)
    }

    pub fn modified(&self) -> String {
        Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()
    }

    pub fn agent(&self) -> String {
        format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }

    pub fn xml(&self) -> String {
        let mut xml = self.xml_open_tag();
        for page in &self.pages {
            xml.push_str("\n  ");
            xml.push_str(&page.xml());
        }
        xml.push('\n');
        xml.push_str(&self.xml_close_tag());
        xml
    }

    pub fn write(&self) -> String {
        self.xml()
    }

    fn xml_open_tag(&self) -> String {
        format!(
            r#"<mxfile host="{}" modified="{}" agent="{}" version="{}" pages="{}">"#,
            self.host,
            self.modified(),
            self.agent(),
            self.version,
            self.pages.len()
        )
    }

    fn xml_close_tag(&self) -> String {
        "</mxfile>".to_string()
    }
}

impl Default for File {
    fn default() -> Self {
        Self::new()
    }
}
