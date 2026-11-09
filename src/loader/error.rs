//! Error types for addon loading.

use std::path::PathBuf;

/// Error type for addon loading.
#[derive(Debug)]
pub enum LoadError {
    /// IO error without a known path (propagated via `?` from contexts that
    /// don't know which file failed). Prefer `IoWithPath` when the path is
    /// available — it makes diagnosis dramatically easier.
    Io(std::io::Error),
    /// IO error with the failing path attached. Use this whenever the call
    /// site has the path in scope (e.g. `std::fs::read(path)`).
    IoWithPath {
        path: PathBuf,
        source: std::io::Error,
    },
    Toc(std::io::Error),
    Xml(crate::xml::XmlLoadError),
    Lua(String),
    /// Caller asked to load an addon whose dependency chain contains
    /// a disabled addon. Renders to exactly `"DEP_DISABLED"` so it can
    /// flow through `LoadAddOn`'s reason string unchanged. Tuple holds
    /// the disabled dependency name for diagnostics (logged but not
    /// surfaced through `Display`, matching retail's flat reason code).
    DepDisabled(String),
    /// The addon itself is not valid on the active UI screen.
    WrongActiveInterface,
    /// A required dependency is not valid on the active UI screen.
    DepWrongActiveInterface(String),
}

impl LoadError {
    /// Wrap a fresh `io::Error` with the path that produced it.
    pub fn io_with_path(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        LoadError::IoWithPath {
            path: path.into(),
            source,
        }
    }
}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        LoadError::Io(e)
    }
}

impl From<crate::xml::XmlLoadError> for LoadError {
    fn from(e: crate::xml::XmlLoadError) -> Self {
        LoadError::Xml(e)
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {}", e),
            LoadError::IoWithPath { path, source } => {
                write!(f, "IO error reading {}: {}", path.display(), source)
            }
            LoadError::Toc(e) => write!(f, "TOC error: {}", e),
            LoadError::Xml(e) => write!(f, "XML error: {}", e),
            LoadError::Lua(e) => write!(f, "Lua error: {}", e),
            LoadError::DepDisabled(_) => write!(f, "DEP_DISABLED"),
            LoadError::WrongActiveInterface => write!(f, "WRONG_ACTIVE_INTERFACE"),
            LoadError::DepWrongActiveInterface(_) => write!(f, "DEP_WRONG_ACTIVE_INTERFACE"),
        }
    }
}

impl std::error::Error for LoadError {}
