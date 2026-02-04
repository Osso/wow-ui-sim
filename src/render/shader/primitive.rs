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
}

impl WowUiPrimitive {
    /// Create a new primitive with the given quad batch.
    pub fn new(quads: QuadBatch) -> Self {
        Self {
            quads,
            clear_color: [0.05, 0.05, 0.08, 1.0], // Dark WoW-style background
            textures: Vec::new(),
        }
    }

    /// Create a new primitive with quads and texture data.
    pub fn with_textures(quads: QuadBatch, textures: Vec<GpuTextureData>) -> Self {
        Self {
            quads,
            clear_color: [0.05, 0.05, 0.08, 1.0],
            textures,
        }
    }

    /// Create an empty primitive.
    pub fn empty() -> Self {
        Self::new(QuadBatch::new())
    }
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
        // Scale bounds to physical pixels to match viewport dimensions
        let scale = viewport.scale_factor() as f32;
        let physical_bounds = Rectangle::new(
            iced::Point::new(bounds.x * scale, bounds.y * scale),
            iced::Size::new(bounds.width * scale, bounds.height * scale),
        );

        // Upload new textures to the atlas
        let atlas = pipeline.texture_atlas_mut();
        for tex_data in &self.textures {
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

        // Check if any vertices need texture resolution (tex_index == -2)
        let needs_resolution = self
            .quads
            .vertices
            .iter()
            .any(|v| v.tex_index == -2);

        if needs_resolution {
            // Clone and resolve texture indices only when needed
            let mut resolved_quads = self.quads.clone();
            for request in &self.quads.texture_requests {
                if let Some(entry) = atlas.get(&request.path) {
                    let start = request.vertex_start as usize;
                    let end = start + request.vertex_count as usize;
                    let tex_idx = entry.tex_index();

                    for vertex in resolved_quads.vertices[start..end].iter_mut() {
                        if vertex.tex_index == -2 {
                            vertex.tex_index = tex_idx;
                            // Scale UV coordinates to actual texture region within atlas
                            vertex.tex_coords[0] =
                                entry.uv_x + vertex.tex_coords[0] * entry.uv_width;
                            vertex.tex_coords[1] =
                                entry.uv_y + vertex.tex_coords[1] * entry.uv_height;
                        }
                    }
                }
            }
            // Scale vertex positions to physical
            for vertex in resolved_quads.vertices.iter_mut() {
                vertex.position[0] *= scale;
                vertex.position[1] *= scale;
            }
            pipeline.prepare(device, queue, &physical_bounds, &resolved_quads);
        } else {
            // Scale vertex positions to physical
            let mut scaled_quads = self.quads.clone();
            for vertex in scaled_quads.vertices.iter_mut() {
                vertex.position[0] *= scale;
                vertex.position[1] *= scale;
            }
            pipeline.prepare(device, queue, &physical_bounds, &scaled_quads);
        }
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
