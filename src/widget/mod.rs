//! Widget system implementing WoW's UI object hierarchy.

mod anchor;
mod frame;
mod registry;

pub use anchor::{Anchor, AnchorPoint};
pub use frame::{Frame, FrameStrata};
pub use registry::WidgetRegistry;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a unique widget ID.
pub fn next_widget_id() -> u64 {
    NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed)
}

/// Widget types supported by the simulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    Frame,
    Button,
    FontString,
    Texture,
}

impl WidgetType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Frame" => Some(Self::Frame),
            "Button" => Some(Self::Button),
            "FontString" => Some(Self::FontString),
            "Texture" => Some(Self::Texture),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Frame => "Frame",
            Self::Button => "Button",
            Self::FontString => "FontString",
            Self::Texture => "Texture",
        }
    }
}
