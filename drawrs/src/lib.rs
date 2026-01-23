pub mod diagram;
pub mod diagram_types;
pub mod error;
pub mod file;
pub mod page;
pub mod transform;
pub mod utils;
pub mod xml_base;
pub mod xml_parser;

pub use diagram::{DiagramBase, Edge, FillStyle, Geometry, Object};
pub use diagram_types::{BarChart, BinaryNodeObject, BinaryTreeDiagram, Legend, PieChart};
pub use error::{DrawrsError, DrawrsResult};
pub use file::DrawFile;
pub use page::{DiagramObject, Page};
pub use transform::{BoundingBox, GroupTransform, Orient};
pub use utils::{PageSize, StandardColor};
pub use xml_base::XMLBase;
pub use xml_parser::parse_xml_to_object;
