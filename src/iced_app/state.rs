//! State structs for the iced application.

use iced::{Point, mouse};

/// Inspector panel state for editing frame properties.
#[derive(Default, Clone)]
pub struct InspectorState {
    pub width: String,
    pub height: String,
    pub alpha: String,
    pub frame_level: String,
    pub visible: bool,
    pub mouse_enabled: bool,
}

/// Canvas-specific messages.
#[derive(Debug, Clone)]
pub enum CanvasMessage {
    MouseMove(Point),
    MouseLeave,
    MouseDown(Point),
    MouseUp(Point),
    MouseUpOutside,
    RightMouseDown(Point),
    RightMouseUp(Point),
    RightMouseUpOutside,
    MiddleClick(Point),
}

impl CanvasMessage {
    pub(super) fn mouse_up_outside(button: mouse::Button) -> Option<Self> {
        match button {
            mouse::Button::Left => Some(Self::MouseUpOutside),
            mouse::Button::Right => Some(Self::RightMouseUpOutside),
            _ => None,
        }
    }
}
