//! WoW UI shader primitive implementation.

use super::{QuadBatch, WowUiPipeline};
use iced::widget::shader::{self, Viewport};
use iced::Rectangle;

/// Loaded texture data ready for GPU upload.
#[derive(Debug, Clone)]
pub struct GpuTextureData {
    /// Texture path (normalized).
    pub path: String,
    /// Image width.
    pub width: u32,
    /// Image height.
    pub height: u32,
    /// RGBA pixel data.
    pub rgba: Vec<u8>,
}

/// Primitive data for rendering WoW UI frames.
///
/// This is created each frame and contains all quads to render.
/// The associated `WowUiPipeline` holds persistent GPU resources.
#[derive(Debug)]
pub struct WowUiPrimitive {
    /// Batched quads to render.
    pub quads: QuadBatch,
    /// Background clear color.
    pub clear_color: [f32; 4],
    /// Texture data to upload (path -> image data).
    pub textures: Vec<GpuTextureData>,
    /// Glyph atlas RGBA data for text rendering (2048x2048).
    /// When Some, uploaded to the GPU glyph atlas texture.
    pub glyph_atlas_data: Option<Vec<u8>>,
    /// Size of the glyph atlas (width = height).
    pub glyph_atlas_size: u32,
}

impl WowUiPrimitive {
    /// Create a new primitive with the given quad batch.
    pub fn new(quads: QuadBatch) -> Self {
        Self {
            quads,
            clear_color: [0.05, 0.05, 0.08, 1.0], // Dark WoW-style background
            textures: Vec::new(),
            glyph_atlas_data: None,
            glyph_atlas_size: 0,
        }
    }

    /// Create a new primitive with quads and texture data.
    pub fn with_textures(quads: QuadBatch, textures: Vec<GpuTextureData>) -> Self {
        Self {
            quads,
            clear_color: [0.05, 0.05, 0.08, 1.0],
            textures,
            glyph_atlas_data: None,
            glyph_atlas_size: 0,
        }
    }

    /// Create an empty primitive.
    pub fn empty() -> Self {
        Self::new(QuadBatch::new())
    }
}

/// Upload pending textures and glyph atlas data to the GPU atlas.
fn upload_pending_textures(
    pipeline: &mut WowUiPipeline,
    queue: &wgpu::Queue,
    textures: &[GpuTextureData],
    glyph_atlas_data: &Option<Vec<u8>>,
    glyph_atlas_size: u32,
) {
    let atlas = pipeline.texture_atlas_mut();
    for tex_data in textures {
        if atlas.get(&tex_data.path).is_none() {
            atlas.upload(
                queue,
                &tex_data.path,
                tex_data.width,
                tex_data.height,
                &tex_data.rgba,
            );
        }
    }

    if let Some(glyph_data) = glyph_atlas_data {
        atlas.upload_glyph_atlas(queue, glyph_data, glyph_atlas_size);
    }
}

/// Resolve pending texture indices (-2) and scale vertex positions to physical pixels.
fn resolve_and_scale_quads(
    pipeline: &mut WowUiPipeline,
    quads: &QuadBatch,
    scale: f32,
) -> QuadBatch {
    let mut resolved = quads.clone();
    let atlas = pipeline.texture_atlas_mut();

    // Resolve texture indices for pending vertices
    for request in &quads.texture_requests {
        if let Some(entry) = atlas.get(&request.path) {
            let start = request.vertex_start as usize;
            let end = start + request.vertex_count as usize;
            let tex_idx = entry.tex_index();

            for vertex in resolved.vertices[start..end].iter_mut() {
                if vertex.tex_index == -2 {
                    vertex.tex_index = tex_idx;
                    vertex.tex_coords[0] = entry.uv_x + vertex.tex_coords[0] * entry.uv_width;
                    vertex.tex_coords[1] = entry.uv_y + vertex.tex_coords[1] * entry.uv_height;
                }
            }
        }
    }

    // Scale vertex positions to physical pixels
    for vertex in resolved.vertices.iter_mut() {
        vertex.position[0] *= scale;
        vertex.position[1] *= scale;
    }

    resolved
}

impl shader::Primitive for WowUiPrimitive {
    type Pipeline = WowUiPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let scale = viewport.scale_factor() as f32;
        let physical_bounds = Rectangle::new(
            iced::Point::new(bounds.x * scale, bounds.y * scale),
            iced::Size::new(bounds.width * scale, bounds.height * scale),
        );

        upload_pending_textures(pipeline, queue, &self.textures, &self.glyph_atlas_data, self.glyph_atlas_size);

        let resolved_quads = resolve_and_scale_quads(pipeline, &self.quads, scale);
        pipeline.prepare(device, queue, &physical_bounds, &resolved_quads);
    }

    fn render(
        &self,
        pipeline: &Self::Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        pipeline.render(encoder, target, clip_bounds);
    }
}
