//! Widget system implementing WoW's UI object hierarchy.

mod anchor;
mod frame;
mod frame_enums;
mod frame_size;
mod frame_types;
mod registry;

pub use crate::atlas::NineSliceAtlasInfo;
pub use anchor::{Anchor, AnchorPoint};
pub use frame::{
    AlphaGradient, AttributeValue, Backdrop, Color, Frame, Gradient, LineAnchor,
    MinimapBlobLayerStyle, MinimapBlobRingStyle, TextJustify, TextOutline,
};
pub use frame_enums::{DrawLayer, FrameStrata};
pub use frame_types::TextSegment;
pub(crate) use registry::has_queryable_rect;
pub use registry::{AnchorCyclePath, RenderDirtyBatch, RenderDirtySource, WidgetRegistry};

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_REGION_ORDER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique widget ID.
pub fn next_widget_id() -> u64 {
    NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed)
}

/// Generate a monotonic ordering value for same-layer regions.
pub fn next_region_order() -> u64 {
    NEXT_REGION_ORDER.fetch_add(1, Ordering::Relaxed)
}

/// Widget types supported by the simulator.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    Frame,
    Button,
    FontString,
    Texture,
    Line,
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
    WorldFrame,
}

const WIDGET_TYPE_NAMES: [&str; 20] = [
    "Frame",
    "Button",
    "FontString",
    "Texture",
    "Line",
    "EditBox",
    "ScrollFrame",
    "Slider",
    "CheckButton",
    "StatusBar",
    "Cooldown",
    "Model",
    "ModelScene",
    "PlayerModel",
    "ColorSelect",
    "MessageFrame",
    "SimpleHTML",
    "GameTooltip",
    "Minimap",
    "Frame",
];

impl WidgetType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        // WoW Lua uses both PascalCase ("Button") and ALLCAPS ("BUTTON")
        // for frame type names, so match case-insensitively.
        let normalized = s.to_ascii_lowercase();
        Self::direct_widget_type(&normalized).or_else(|| Self::alias_group_widget_type(&normalized))
    }

    fn direct_widget_type(alias: &str) -> Option<Self> {
        match alias {
            "frame" => Some(Self::Frame),
            "fontstring" => Some(Self::FontString),
            "texture" => Some(Self::Texture),
            "line" => Some(Self::Line),
            "editbox" => Some(Self::EditBox),
            "scrollframe" => Some(Self::ScrollFrame),
            "slider" => Some(Self::Slider),
            "checkbutton" => Some(Self::CheckButton),
            "statusbar" => Some(Self::StatusBar),
            "cooldown" => Some(Self::Cooldown),
            "model" => Some(Self::Model),
            "modelscene" => Some(Self::ModelScene),
            "colorselect" => Some(Self::ColorSelect),
            "simplehtml" => Some(Self::SimpleHTML),
            "gametooltip" => Some(Self::GameTooltip),
            "minimap" => Some(Self::Minimap),
            _ => None,
        }
    }

    fn alias_group_widget_type(alias: &str) -> Option<Self> {
        if Self::is_button_alias(alias) {
            return Some(Self::Button);
        }
        if Self::is_player_model_alias(alias) {
            return Some(Self::PlayerModel);
        }
        if Self::is_message_frame_alias(alias) {
            return Some(Self::MessageFrame);
        }
        if Self::is_frame_alias(alias) {
            return Some(Self::Frame);
        }

        // WorldFrame is internal only — CreateFrame("WorldFrame") should error
        None
    }

    fn is_button_alias(alias: &str) -> bool {
        matches!(
            alias,
            "button" | "dropdownbutton" | "itembutton" | "containedalertframe"
        )
    }

    fn is_player_model_alias(alias: &str) -> bool {
        matches!(
            alias,
            "playermodel" | "cinematicmodel" | "tabardmodel" | "dressupmodel"
        )
    }

    fn is_message_frame_alias(alias: &str) -> bool {
        matches!(alias, "messageframe" | "scrollingmessageframe")
    }

    fn is_frame_alias(alias: &str) -> bool {
        matches!(
            alias,
            // EventFrame is a Frame subtype for event-only handling
            "eventframe"
                // Checkout is a special frame type for the in-game shop
                | "checkout"
                // Specialty frame types — no custom behavior needed, treat as plain Frame
                | "archaeologydigsiteframe"
                | "browser"
                | "fogofwarframe"
                | "movieframe"
                | "offscreenframe"
                | "questpoiframe"
                | "scenariopoiframe"
                | "unitpositionframe"
        )
    }

    pub fn as_str(&self) -> &'static str {
        WIDGET_TYPE_NAMES[*self as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::WidgetType;

    #[test]
    fn widget_type_as_str_returns_wow_object_type_names() {
        assert_eq!(WidgetType::Frame.as_str(), "Frame");
        assert_eq!(WidgetType::Button.as_str(), "Button");
        assert_eq!(WidgetType::FontString.as_str(), "FontString");
        assert_eq!(WidgetType::Texture.as_str(), "Texture");
        assert_eq!(WidgetType::Line.as_str(), "Line");
        assert_eq!(WidgetType::EditBox.as_str(), "EditBox");
        assert_eq!(WidgetType::ScrollFrame.as_str(), "ScrollFrame");
        assert_eq!(WidgetType::Slider.as_str(), "Slider");
        assert_eq!(WidgetType::CheckButton.as_str(), "CheckButton");
        assert_eq!(WidgetType::StatusBar.as_str(), "StatusBar");
        assert_eq!(WidgetType::Cooldown.as_str(), "Cooldown");
        assert_eq!(WidgetType::Model.as_str(), "Model");
        assert_eq!(WidgetType::ModelScene.as_str(), "ModelScene");
        assert_eq!(WidgetType::PlayerModel.as_str(), "PlayerModel");
        assert_eq!(WidgetType::ColorSelect.as_str(), "ColorSelect");
        assert_eq!(WidgetType::MessageFrame.as_str(), "MessageFrame");
        assert_eq!(WidgetType::SimpleHTML.as_str(), "SimpleHTML");
        assert_eq!(WidgetType::GameTooltip.as_str(), "GameTooltip");
        assert_eq!(WidgetType::Minimap.as_str(), "Minimap");
        assert_eq!(WidgetType::WorldFrame.as_str(), "Frame");
    }
}
