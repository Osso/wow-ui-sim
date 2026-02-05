//! State structs for the iced application.

use iced::Point;

use crate::iced_app::App;

/// Text overlay wrapper for shader mode.
///
/// This renders only text (FontStrings) on a transparent background,
/// layered on top of the shader which renders textures/backgrounds.
pub struct TextOverlay<'a> {
    pub(crate) app: &'a App,
}

impl<'a> TextOverlay<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

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
