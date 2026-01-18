//! Anchor system for widget positioning.

/// Anchor points on a widget (matches WoW's anchor system).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnchorPoint {
    #[default]
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl AnchorPoint {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "CENTER" => Some(Self::Center),
            "TOP" => Some(Self::Top),
            "BOTTOM" => Some(Self::Bottom),
            "LEFT" => Some(Self::Left),
            "RIGHT" => Some(Self::Right),
            "TOPLEFT" => Some(Self::TopLeft),
            "TOPRIGHT" => Some(Self::TopRight),
            "BOTTOMLEFT" => Some(Self::BottomLeft),
            "BOTTOMRIGHT" => Some(Self::BottomRight),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Center => "CENTER",
            Self::Top => "TOP",
            Self::Bottom => "BOTTOM",
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::TopLeft => "TOPLEFT",
            Self::TopRight => "TOPRIGHT",
            Self::BottomLeft => "BOTTOMLEFT",
            Self::BottomRight => "BOTTOMRIGHT",
        }
    }
}

/// An anchor defines how a widget is positioned relative to another widget.
#[derive(Debug, Clone)]
pub struct Anchor {
    /// The point on this widget to anchor.
    pub point: AnchorPoint,
    /// The widget name to anchor to (used for XML parsing, None = parent).
    pub relative_to: Option<String>,
    /// The widget ID to anchor to (used for Lua API, takes precedence over name).
    pub relative_to_id: Option<usize>,
    /// The point on the relative widget to anchor to.
    pub relative_point: AnchorPoint,
    /// X offset from the anchor point.
    pub x_offset: f32,
    /// Y offset from the anchor point.
    pub y_offset: f32,
}

impl Default for Anchor {
    fn default() -> Self {
        Self {
            point: AnchorPoint::Center,
            relative_to: None,
            relative_to_id: None,
            relative_point: AnchorPoint::Center,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }
}
