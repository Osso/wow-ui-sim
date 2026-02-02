//! Rendering module for WoW UI frames using iced canvas.

pub mod text;
pub mod texture;

pub use text::TextRenderer;
pub use texture::{
    draw_horizontal_slice_texture, draw_nine_slice_texture, draw_scaled_texture,
    draw_texture_with_texcoords, draw_tiled_texture,
};
