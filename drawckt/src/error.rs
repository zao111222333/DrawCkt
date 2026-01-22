use drawrs::DrawrsError;
use thiserror::Error;

use crate::schematic::Layer;

/// Main error type for drawckt crate
#[derive(Error, Debug)]
pub enum DrawcktError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Drawrs error: {0}")]
    Drawrs(#[from] DrawrsError),

    #[error("XML parsing error: {0}")]
    XmlParsing(#[from] quick_xml::Error),

    #[error("Unknown layer: {0}")]
    UnknownLayer(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Symbol page '{0}' not found in symbols.drawio")]
    SymbolNotFound(String),

    #[error("Repeat layer: {0}")]
    RepeatLayer(Layer),
}

/// Convenience type alias for Result
pub type DrawcktResult<T> = Result<T, DrawcktError>;
