use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Lua error: {0}")]
    Lua(#[from] mlua::Error),

    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("Widget not found: {0}")]
    WidgetNotFound(String),

    #[error("Invalid widget type: expected {expected}, got {actual}")]
    InvalidWidgetType { expected: String, actual: String },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
