//! Widget system implementing WoW's UI object hierarchy.

mod anchor;
mod frame;
mod registry;

pub use anchor::{Anchor, AnchorPoint};
pub use frame::{AttributeValue, Backdrop, Color, DrawLayer, Frame, FrameStrata, TextJustify, TextOutline};
pub use crate::atlas::NineSliceAtlasInfo;
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
    EditBox,
    ScrollFrame,
    Slider,
    CheckButton,
    StatusBar,
    Cooldown,
    Model,
    ModelScene,
    PlayerModel,
    ColorSelect,
    MessageFrame,
    SimpleHTML,
    GameTooltip,
    Minimap,
}

impl WidgetType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        // WoW Lua uses both PascalCase ("Button") and ALLCAPS ("BUTTON")
        // for frame type names, so match case-insensitively.
        let lower = s.to_ascii_lowercase();
        match lower.as_str() {
            "frame" => Some(Self::Frame),
            "button" | "dropdownbutton" | "itembutton" | "containedalertframe" => Some(Self::Button),
            "fontstring" => Some(Self::FontString),
            "texture" => Some(Self::Texture),
            "editbox" => Some(Self::EditBox),
            "scrollframe" => Some(Self::ScrollFrame),
            "slider" => Some(Self::Slider),
            "checkbutton" => Some(Self::CheckButton),
            "statusbar" => Some(Self::StatusBar),
            "cooldown" => Some(Self::Cooldown),
            "model" | "dressupmodel" => Some(Self::Model),
            "modelscene" => Some(Self::ModelScene),
            "playermodel" | "cinematicmodel" | "tabardmodel" => Some(Self::PlayerModel),
            "colorselect" => Some(Self::ColorSelect),
            "messageframe" | "scrollingmessageframe" => Some(Self::MessageFrame),
            "simplehtml" => Some(Self::SimpleHTML),
            "gametooltip" => Some(Self::GameTooltip),
            "minimap" => Some(Self::Minimap),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Frame => "Frame",
            Self::Button => "Button",
            Self::FontString => "FontString",
            Self::Texture => "Texture",
            Self::EditBox => "EditBox",
            Self::ScrollFrame => "ScrollFrame",
            Self::Slider => "Slider",
            Self::CheckButton => "CheckButton",
            Self::StatusBar => "StatusBar",
            Self::Cooldown => "Cooldown",
            Self::Model => "Model",
            Self::ModelScene => "ModelScene",
            Self::PlayerModel => "PlayerModel",
            Self::ColorSelect => "ColorSelect",
            Self::MessageFrame => "MessageFrame",
            Self::SimpleHTML => "SimpleHTML",
            Self::GameTooltip => "GameTooltip",
            Self::Minimap => "Minimap",
        }
    }
}
