//! Widget system implementing WoW's UI object hierarchy.

mod anchor;
mod frame;
mod registry;

pub use anchor::{Anchor, AnchorPoint};
pub use frame::{AttributeValue, Backdrop, Color, DrawLayer, Frame, FrameStrata, TextJustify, TextOutline};
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
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Frame" => Some(Self::Frame),
            "Button" | "ItemButton" => Some(Self::Button),
            "FontString" => Some(Self::FontString),
            "Texture" => Some(Self::Texture),
            "EditBox" => Some(Self::EditBox),
            "ScrollFrame" => Some(Self::ScrollFrame),
            "Slider" => Some(Self::Slider),
            "CheckButton" => Some(Self::CheckButton),
            "StatusBar" => Some(Self::StatusBar),
            "Cooldown" => Some(Self::Cooldown),
            "Model" | "DressUpModel" => Some(Self::Model),
            "ModelScene" | "MODELSCENE" => Some(Self::ModelScene),
            "PlayerModel" | "CinematicModel" | "TabardModel" => Some(Self::PlayerModel),
            "ColorSelect" => Some(Self::ColorSelect),
            "MessageFrame" | "ScrollingMessageFrame" => Some(Self::MessageFrame),
            "SimpleHTML" => Some(Self::SimpleHTML),
            "GameTooltip" => Some(Self::GameTooltip),
            "Minimap" => Some(Self::Minimap),
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
