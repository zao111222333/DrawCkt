use thiserror::Error;

use crate::Orient;

/// Main error type for drawrs crate
#[derive(Error, Debug)]
pub enum DrawrsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parsing error: {0}")]
    XmlParsing(#[from] quick_xml::Error),

    #[error("Failed to read file: {0}")]
    FileRead(String),

    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Invalid value for key '{0}': {1}")]
    InvalidValue(String, String),

    #[error("Binary node error: {0}")]
    BinaryNode(String),

    #[error("Unknown layer: {0}")]
    UnknownLayer(String),

    #[error("Data cannot be empty")]
    EmptyData,

    #[error("Mapping cannot be empty")]
    EmptyMapping,

    #[error("Root dict must contain exactly one key")]
    InvalidRootDict,

    #[error("BinaryNodeObject cannot have more than two children")]
    TooManyChildren,

    #[error("UnsupportedOrient: {0:?}")]
    UnsupportedOrient(Orient),
}

/// Convenience type alias for Result
pub type DrawrsResult<T> = Result<T, DrawrsError>;

impl From<String> for DrawrsError {
    fn from(msg: String) -> Self {
        DrawrsError::InvalidData(msg)
    }
}
