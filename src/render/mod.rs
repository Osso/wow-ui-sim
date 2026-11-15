//! Rendering module for WoW UI frames.
//!
//! Provides both canvas-based (CPU) and shader-based (GPU) rendering.

pub(crate) mod editbox_layout;

pub mod font {
    pub use crate::font::*;
}
#[cfg(feature = "gui")]
pub mod glyph;
#[cfg(feature = "gui")]
pub mod headless;
#[cfg(feature = "gui")]
pub mod shader;
#[cfg(feature = "gui")]
pub mod text;
#[cfg(feature = "gui")]
pub use sim_render::texture;

pub use crate::BlendMode;
pub use crate::font::WowFontSystem;

pub use sim_render::inline_markup;
pub use sim_render::strip_wow_markup;

#[cfg(feature = "gui")]
pub use glyph::{GlyphAtlas, emit_text_quads};
#[cfg(feature = "gui")]
pub(crate) use shader::ThreeSlicePathParams;
#[cfg(feature = "gui")]
pub use shader::{
    FrameQuadSnapshot, GpuBcTextureData, GpuTextureAtlas, GpuTextureData, NineSliceTextures,
    QuadBatch, QuadVertex, TextureEntry, TextureRequest, WowUiPipeline, WowUiPrimitive,
    WowUiProgram, load_texture_or_crop,
};
#[cfg(feature = "gui")]
pub use text::TextRenderer;
#[cfg(feature = "gui")]
pub use texture::{
    draw_horizontal_slice_texture, draw_nine_slice_texture, draw_scaled_texture,
    draw_texture_with_texcoords, draw_tiled_texture,
};

#[cfg(test)]
mod tests {
    use super::strip_wow_markup;

    #[test]
    fn strips_spell_link_before_wow_newline_escape() {
        assert_eq!(
            strip_wow_markup("|cFF2959D3|Hspell:1225135|h[Suppression Zones]|h|r|nNext"),
            "[Suppression Zones]\nNext"
        );
    }
}
