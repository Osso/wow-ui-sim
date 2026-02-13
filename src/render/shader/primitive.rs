//! WoW UI shader primitive implementation.

use super::{QuadBatch, WowUiPipeline};
use iced::widget::shader::{self, Viewport};
use iced::Rectangle;
use std::sync::Arc;

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

/// Load a texture by path, handling `@crop:` paths by extracting a sub-region.
///
/// Atlas sub-region paths have format `"base_path@crop:left,right,top,bottom"` where
/// left/right/top/bottom are UV coordinates in the source texture. The sub-region is
/// extracted at native resolution so small atlas entries aren't degraded by downscaling
/// the full oversized texture into a 512px GPU atlas cell.
pub fn load_texture_or_crop(
    tex_mgr: &mut crate::texture::TextureManager,
    path: &str,
) -> Option<GpuTextureData> {
    if let Some(crop_start) = path.find("@crop:") {
        let base_path = &path[..crop_start];
        let crop_str = &path[crop_start + 6..];
        let coords: Vec<f32> = crop_str.split(',').filter_map(|s| s.parse().ok()).collect();
        if coords.len() != 4 {
            return None;
        }
        let (cl, cr, ct, cb) = (coords[0], coords[1], coords[2], coords[3]);
        let tex_data = tex_mgr.load(base_path)?;
        let (w, h) = (tex_data.width, tex_data.height);
        let (crop_w, crop_h, cropped) = crop_sub_region(&tex_data.pixels, w, h, cl, cr, ct, cb);
        Some(GpuTextureData { path: path.to_string(), width: crop_w, height: crop_h, rgba: cropped })
    } else {
        let tex_data = tex_mgr.load(path)?;
        Some(GpuTextureData {
            path: path.to_string(),
            width: tex_data.width,
            height: tex_data.height,
            rgba: tex_data.pixels.clone(),
        })
    }
}

/// Extract a rectangular sub-region from RGBA pixel data using UV coordinates.
fn crop_sub_region(
    pixels: &[u8],
    width: u32,
    height: u32,
    uv_left: f32,
    uv_right: f32,
    uv_top: f32,
    uv_bottom: f32,
) -> (u32, u32, Vec<u8>) {
    let x0 = (uv_left * width as f32).round() as u32;
    let x1 = (uv_right * width as f32).round() as u32;
    let y0 = (uv_top * height as f32).round() as u32;
    let y1 = (uv_bottom * height as f32).round() as u32;
    let crop_w = x1.saturating_sub(x0).max(1).min(width);
    let crop_h = y1.saturating_sub(y0).max(1).min(height);

    let mut cropped = vec![0u8; (crop_w * crop_h * 4) as usize];
    for row in 0..crop_h {
        let src_y = (y0 + row).min(height - 1);
        let src_off = (src_y * width + x0) as usize * 4;
        let dst_off = (row * crop_w) as usize * 4;
        let row_bytes = (crop_w * 4) as usize;
        let src_end = (src_off + row_bytes).min(pixels.len());
        let copy_len = src_end.saturating_sub(src_off);
        if copy_len > 0 {
            cropped[dst_off..dst_off + copy_len]
                .copy_from_slice(&pixels[src_off..src_off + copy_len]);
        }
    }
    (crop_w, crop_h, cropped)
}

/// Primitive data for rendering WoW UI frames.
///
/// This is created each frame and contains all quads to render.
/// The associated `WowUiPipeline` holds persistent GPU resources.
#[derive(Debug)]
pub struct WowUiPrimitive {
    /// Batched quads to render (Arc-shared with the cache to avoid deep clones).
    pub quads: Arc<QuadBatch>,
    /// Small overlay batch (cursor, hover highlight) appended after world quads.
    /// Kept separate so the world batch can be cached without cloning.
    pub overlay: QuadBatch,
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
    pub fn new(quads: Arc<QuadBatch>) -> Self {
        Self {
            quads,
            overlay: QuadBatch::new(),
            clear_color: [0.10, 0.11, 0.14, 1.0], // Dark blue-grey background
            textures: Vec::new(),
            glyph_atlas_data: None,
            glyph_atlas_size: 0,
        }
    }

    /// Create a new primitive with quads and texture data.
    pub fn with_textures(quads: Arc<QuadBatch>, textures: Vec<GpuTextureData>) -> Self {
        Self {
            quads,
            overlay: QuadBatch::new(),
            clear_color: [0.10, 0.11, 0.14, 1.0],
            textures,
            glyph_atlas_data: None,
            glyph_atlas_size: 0,
        }
    }

    /// Create an empty primitive.
    pub fn empty() -> Self {
        Self::new(Arc::new(QuadBatch::new()))
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

    log_gpu_memory_once(atlas);
}

/// Log GPU atlas memory usage once after the first batch of textures.
fn log_gpu_memory_once(atlas: &crate::render::shader::atlas::GpuTextureAtlas) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static LOGGED: AtomicBool = AtomicBool::new(false);
    if LOGGED.swap(true, Ordering::Relaxed) {
        return;
    }
    let stats = atlas.memory_stats();
    eprintln!(
        "[GPU] Atlas memory: {:.0} MB allocated, {:.1} MB used | slots: 64px={} 128px={} 256px={} 512px={} 2048px={}",
        stats.allocated_bytes as f64 / (1024.0 * 1024.0),
        stats.used_bytes as f64 / (1024.0 * 1024.0),
        stats.used_slots[0], stats.used_slots[1], stats.used_slots[2], stats.used_slots[3], stats.used_slots[4],
    );
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

    // Resolve mask texture indices for pending mask vertices
    for request in &quads.mask_texture_requests {
        if let Some(entry) = atlas.get(&request.path) {
            let start = request.vertex_start as usize;
            let end = start + request.vertex_count as usize;
            let tex_idx = entry.tex_index();
            for vertex in resolved.vertices[start..end].iter_mut() {
                if vertex.mask_tex_index == -2 {
                    vertex.mask_tex_index = tex_idx;
                    vertex.mask_tex_coords[0] = entry.uv_x + vertex.mask_tex_coords[0] * entry.uv_width;
                    vertex.mask_tex_coords[1] = entry.uv_y + vertex.mask_tex_coords[1] * entry.uv_height;
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
        let scale = viewport.scale_factor();
        let physical_bounds = Rectangle::new(
            iced::Point::new(bounds.x * scale, bounds.y * scale),
            iced::Size::new(bounds.width * scale, bounds.height * scale),
        );

        upload_pending_textures(pipeline, queue, &self.textures, &self.glyph_atlas_data, self.glyph_atlas_size);

        let mut resolved_quads = resolve_and_scale_quads(pipeline, &self.quads, scale);
        if !self.overlay.vertices.is_empty() {
            let resolved_overlay = resolve_and_scale_quads(pipeline, &self.overlay, scale);
            resolved_quads.append(&resolved_overlay);
        }
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
