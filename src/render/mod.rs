//! Rendering backend using iced.

mod nine_slice;
mod ui;

pub use nine_slice::{
    button_texture_path, draw_button, draw_nine_slice, preload_nine_slice_textures, ButtonState,
    NineSliceFrame,
};
pub use ui::run_ui;

/// Computed layout position for a frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
