//! State structs for the iced application.

use iced::Point;

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
    MouseDown(Point),
    MouseUp(Point),
    MiddleClick(Point),
}
