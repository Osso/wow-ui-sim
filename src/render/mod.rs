//! Rendering module for WoW UI frames.
//!
//! Provides both canvas-based (CPU) and shader-based (GPU) rendering.

pub mod shader;
pub mod text;
pub mod texture;

pub use shader::{
    BlendMode, GpuTextureAtlas, GpuTextureData, NineSliceTextures, QuadBatch, QuadVertex,
    TextureEntry, TextureRequest, WowUiPipeline, WowUiPrimitive, WowUiProgram,
};
pub use text::TextRenderer;
pub use texture::{
    draw_horizontal_slice_texture, draw_nine_slice_texture, draw_scaled_texture,
    draw_texture_with_texcoords, draw_tiled_texture,
};
