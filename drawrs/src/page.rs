use crate::BoundingBox;
use crate::transform::FlipRotation;
use crate::xml_base::XMLBase;
use itertools::Either;

pub struct Page {
    objects: Vec<DiagramObject>,
    name: String,
    page_num: usize,
    dx: f64,
    dy: f64,
    grid: i32,
    grid_size: i32,
    guides: i32,
    tooltips: i32,
    connect: i32,
    arrows: i32,
    fold: i32,
    scale: f64,
    width: f64,
    height: f64,
    math: i32,
    shadow: i32,
    diagram: Diagram,
}

struct Diagram {
    base: XMLBase,
    name: String,
}

impl Page {
    pub fn new(id: Option<String>, need_background: bool) -> Self {
        let page_num = 1;
        let name = format!("Page-{}", page_num);

        let mut diagram_base = XMLBase::new(id);
        diagram_base.xml_class = "diagram".to_string();
        let diagram = Diagram {
            base: diagram_base,
            name: name.clone(),
        };

        let mut page = Self {
            objects: Vec::new(),
            name,
            page_num,
            dx: 2037.0,
            dy: 830.0,
            grid: 1,
            grid_size: 10,
            guides: 1,
            tooltips: 1,
            connect: 1,
            arrows: 1,
            fold: 1,
            scale: 1.0,
            width: 850.0,
            height: 1100.0,
            math: 0,
            shadow: 0,
            diagram,
        };

        // Add two empty mxCell objects
        let mut cell0 = XMLBase::new(Some("0".to_string()));
        cell0.xml_class = "mxCell".to_string();
        page.objects.push(DiagramObject::XmlBase(cell0));
        if need_background {
            let mut cell1 = XMLBase::new(Some("1".to_string()));
            cell1.xml_class = "mxCell".to_string();
            cell1.xml_parent = Some("0".to_string());
            page.objects.push(DiagramObject::XmlBase(cell1));
        }

        page
    }

    pub fn id(&self) -> &str {
        &self.diagram.base.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name.clone();
        self.diagram.name = name;
    }

    pub fn set_page_num(&mut self, page_num: usize) {
        self.page_num = page_num;
    }

    pub fn objects(&self) -> &[DiagramObject] {
        &self.objects
    }

    pub fn add_object(&mut self, obj: DiagramObject) {
        self.objects.push(obj);
    }

    pub fn remove_object(&mut self, obj_id: &str) {
        self.objects.retain(|o| o.id() != obj_id);
    }

    pub fn xml(&self) -> String {
        let mut xml = self.xml_open_tag();
        for obj in &self.objects {
            xml.push_str("\n        ");
            xml.push_str(&obj.xml());
        }
        xml.push('\n');
        xml.push_str(&self.xml_close_tag());
        xml
    }
    fn xml_open_tag(&self) -> String {
        format!(
            r#"<diagram name="{}" id="{}">
    <mxGraphModel dx="{}" dy="{}" grid="{}" gridSize="{}" guides="{}" toolTips="{}" connect="{}" arrows="{}" fold="{}" page="{}" pageScale="{}" pageWidth="{}" pageHeight="{}" math="{}" shadow="{}">
      <root>"#,
            self.name,
            self.diagram.base.id,
            self.dx,
            self.dy,
            self.grid,
            self.grid_size,
            self.guides,
            self.tooltips,
            self.connect,
            self.arrows,
            self.fold,
            self.page_num,
            self.scale,
            self.width,
            self.height,
            self.math,
            self.shadow
        )
    }

    fn xml_close_tag(&self) -> String {
        format!(
            r#"      </root>
    </mxGraphModel>
  </diagram>"#
        )
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new(None, true)
    }
}

use crate::diagram::{Edge, Object};

#[derive(Clone, Debug)]
pub enum DiagramObject {
    XmlBase(XMLBase),
    Object(Object),
    Edge(Edge),
}

impl DiagramObject {
    pub fn base(&self) -> &XMLBase {
        match self {
            DiagramObject::XmlBase(x) => x,
            DiagramObject::Object(o) => o.base(),
            DiagramObject::Edge(e) => e.base(),
        }
    }
    pub fn base_mut(&mut self) -> &mut XMLBase {
        match self {
            DiagramObject::XmlBase(x) => x,
            DiagramObject::Object(o) => o.base_mut(),
            DiagramObject::Edge(e) => e.base_mut(),
        }
    }
    pub fn text(&self) -> Option<&String> {
        self.base().value.as_ref()
    }

    pub fn set_text(&mut self, text: String) {
        self.base_mut().value = Some(text);
    }

    pub fn text_mut(&mut self) -> Option<&mut String> {
        self.base_mut().value.as_mut()
    }
    pub fn mut_points(&mut self) -> impl Iterator<Item = &mut [f64; 2]> {
        match self {
            DiagramObject::XmlBase(_) => {
                // XmlBase has no points
                Either::Left(Either::Left(std::iter::empty()))
            }
            DiagramObject::Object(o) => Either::Left(Either::Right(o.points_mut())),
            DiagramObject::Edge(e) => {
                let geom = e.geometry();
                Either::Right(geom.points_mut())
            }
        }
    }

    pub fn mut_box(&mut self) -> Option<(&mut BoundingBox, &mut FlipRotation)> {
        match self {
            DiagramObject::XmlBase(_) => {
                // XmlBase has no boxes
                None
            }
            DiagramObject::Object(o) => {
                let (bbox, fr) = o.geometry_mut().bounding_box_and_flip_rotation_mut();
                Some((bbox, fr))
            }
            DiagramObject::Edge(_e) => None,
        }
    }
    pub fn id(&self) -> &str {
        &self.base().id
    }
    pub fn set_id(&mut self, id: String) {
        self.base_mut().id = id;
    }
    pub fn set_tag(&mut self, tag: Option<String>) {
        self.base_mut().tag = tag;
    }

    pub fn xml(&self) -> String {
        match self {
            DiagramObject::XmlBase(x) => XMLBase::xml(x),
            DiagramObject::Object(o) => o.xml(),
            DiagramObject::Edge(e) => e.xml(),
        }
    }

    /// Get bounding box for objects (for Objects only, returns None for XmlBase and Edge)
    pub fn bounding_box(&self) -> Option<crate::transform::BoundingBox> {
        match self {
            DiagramObject::Object(o) => {
                let pos = o.position();
                Some(crate::transform::BoundingBox::new(
                    pos[0],
                    pos[1],
                    o.width(),
                    o.height(),
                ))
            }
            _ => None,
        }
    }

    /// Get the XML parent of this object
    pub fn xml_parent(&self) -> Option<&String> {
        self.base().xml_parent.as_ref()
    }

    /// Set the XML parent of this object
    pub fn set_xml_parent(&mut self, parent: Option<String>) {
        self.base_mut().xml_parent = parent;
    }
}

impl From<XMLBase> for DiagramObject {
    fn from(x: XMLBase) -> Self {
        DiagramObject::XmlBase(x)
    }
}

impl From<Object> for DiagramObject {
    fn from(o: Object) -> Self {
        DiagramObject::Object(o)
    }
}

impl From<Edge> for DiagramObject {
    fn from(e: Edge) -> Self {
        DiagramObject::Edge(e)
    }
}
