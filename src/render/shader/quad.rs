//! Quad vertex format and batching for GPU rendering.

use iced::Rectangle;
use std::sync::{Arc, Mutex};

const FLOAT2: wgpu::VertexFormat = wgpu::VertexFormat::Float32x2;

/// Flag bit: clip to a circle using UV coordinates (for minimap).
pub const FLAG_CIRCLE_CLIP: u32 = 0x100;

/// Flag bit: cooldown swipe (radial clock wipe). Uses local_uv for position,
/// tex_coords.x for progress (0.0 = full coverage, 1.0 = empty).
pub const FLAG_COOLDOWN_SWIPE: u32 = 0x200;

/// Flag bit: desaturate (convert to greyscale).
pub const FLAG_DESATURATE: u32 = 0x400;

/// Flag bit: mask samples use alpha channel coverage instead of RGB intensity.
pub const FLAG_MASK_ALPHA_COVERAGE: u32 = 0x800;

pub use crate::BlendMode;

/// Vertex format for textured quads.
///
/// Each quad consists of 4 vertices forming a rectangle.
/// Uses interleaved vertex layout for cache efficiency.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    /// Position in screen coordinates (pixels from top-left).
    pub position: [f32; 2],
    /// Texture coordinates (0.0-1.0 UV space, remapped to atlas during prepare).
    pub tex_coords: [f32; 2],
    /// Vertex color (RGBA, premultiplied alpha).
    pub color: [f32; 4],
    /// Texture index in the texture array (-1 for solid color).
    pub tex_index: i32,
    /// Blend mode and flags.
    pub flags: u32,
    /// Quad-local UV coordinates (0-1, preserved across atlas remapping).
    /// Used by effects like circle clip that need quad-relative position.
    pub local_uv: [f32; 2],
    /// Mask texture index (-1 = no mask, -2 = pending resolution, >=0 = atlas tier).
    pub mask_tex_index: i32,
    /// Mask texture UV coordinates (remapped to atlas during prepare).
    pub mask_tex_coords: [f32; 2],
}

impl QuadVertex {
    /// Vertex buffer layout for wgpu.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        // Field offsets in bytes (all f32=4 bytes, i32=4, u32=4):
        // position(8) tex_coords(8) color(16) tex_index(4) flags(4)
        // local_uv(8) mask_tex_index(4) mask_tex_coords(8)
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &[
            vertex_attribute(0, 0, FLOAT2),                         // position
            vertex_attribute(8, 1, FLOAT2),                         // tex_coords
            vertex_attribute(16, 2, wgpu::VertexFormat::Float32x4), // color
            vertex_attribute(32, 3, wgpu::VertexFormat::Sint32),    // tex_index
            vertex_attribute(36, 4, wgpu::VertexFormat::Uint32),    // flags
            vertex_attribute(40, 5, FLOAT2),                        // local_uv
            vertex_attribute(48, 6, wgpu::VertexFormat::Sint32),    // mask_tex_index
            vertex_attribute(52, 7, FLOAT2),                        // mask_tex_coords
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

const fn vertex_attribute(
    offset: wgpu::BufferAddress,
    shader_location: u32,
    format: wgpu::VertexFormat,
) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        offset,
        shader_location,
        format,
    }
}

/// A texture request for deferred loading.
#[derive(Debug, Clone)]
pub struct TextureRequest {
    /// Texture path (WoW format like "Interface\\Buttons\\UI-Panel-Button-Up").
    pub path: String,
    /// Starting vertex index (4 vertices per quad).
    pub vertex_start: u32,
    /// Number of vertices using this texture.
    pub vertex_count: u32,
    /// Whether atlas UV remapping should inset by half a texel for bleed protection.
    pub use_uv_inset: bool,
    /// Request-local load state shared across cloned batches.
    pub handle: TextureRequestHandle,
}

impl TextureRequest {
    /// Create a new deferred texture request with fresh request-local state.
    pub fn new(path: impl Into<String>, vertex_start: u32, vertex_count: u32) -> Self {
        Self::new_with_uv_inset(path, vertex_start, vertex_count, true)
    }

    /// Create a new deferred texture request with explicit atlas UV inset behavior.
    pub fn new_with_uv_inset(
        path: impl Into<String>,
        vertex_start: u32,
        vertex_count: u32,
        use_uv_inset: bool,
    ) -> Self {
        Self {
            path: path.into(),
            vertex_start,
            vertex_count,
            use_uv_inset,
            handle: TextureRequestHandle::default(),
        }
    }

    /// Copy the request while preserving the request-local state handle.
    pub fn with_vertex_start(&self, vertex_start: u32) -> Self {
        Self {
            path: self.path.clone(),
            vertex_start,
            vertex_count: self.vertex_count,
            use_uv_inset: self.use_uv_inset,
            handle: self.handle.clone(),
        }
    }
}

/// Shared request-local load state for a deferred texture request.
#[derive(Debug, Clone, Default)]
pub struct TextureRequestHandle(Arc<Mutex<TextureRequestHandleState>>);

#[derive(Debug, Default)]
struct TextureRequestHandleState {
    staged: bool,
    ready: bool,
    force_rgba: bool,
    failed: bool,
}

impl TextureRequestHandle {
    fn state(&self) -> std::sync::MutexGuard<'_, TextureRequestHandleState> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Whether the request finished loading and GPU upload can resolve it.
    pub fn is_ready(&self) -> bool {
        self.state().ready
    }

    /// Whether the request failed and should stay quiet until reset.
    pub fn is_failed(&self) -> bool {
        self.state().failed
    }

    /// Whether the request is staged for upload but not yet ready.
    pub fn is_staged(&self) -> bool {
        self.state().staged
    }

    /// Whether the request still needs to be driven by preload or draw.
    pub fn is_pending(&self) -> bool {
        let state = self.state();
        !state.ready && !state.failed
    }

    /// Whether the request still needs to be driven by preload or draw.
    pub fn should_load(&self) -> bool {
        let state = self.state();
        !state.ready && !state.failed && (!state.staged || state.force_rgba)
    }

    /// Mark the request as staged for GPU upload.
    pub fn mark_staged(&self) {
        let mut state = self.state();
        state.staged = true;
        state.ready = false;
        state.force_rgba = false;
        state.failed = false;
    }

    /// Mark the request as ready in the atlas after GPU upload.
    pub fn mark_ready(&self) {
        let mut state = self.state();
        state.staged = true;
        state.ready = true;
        state.force_rgba = false;
        state.failed = false;
    }

    /// Mark the request so the next load pass retries through the RGBA atlas.
    pub fn mark_force_rgba(&self) {
        self.mark_retry();
        self.state().force_rgba = true;
    }

    /// Whether the next retry should bypass BC upload and force RGBA upload.
    pub fn needs_force_rgba(&self) -> bool {
        self.state().force_rgba
    }

    /// Clear staged state so the request can be retried on the next frame.
    pub fn mark_retry(&self) {
        let mut state = self.state();
        state.staged = false;
        state.ready = false;
        state.force_rgba = false;
        state.failed = false;
    }

    /// Mark the request as failed and stop retrying it.
    pub fn mark_failed(&self) {
        let mut state = self.state();
        state.staged = false;
        state.ready = false;
        state.force_rgba = false;
        state.failed = true;
    }
}

/// Cached quad data for a single frame, used for incremental strata rebuilds.
/// All indices and texture request offsets are relative (0-based).
#[derive(Debug, Clone, Default)]
pub struct FrameQuadSnapshot {
    pub vertices: Vec<QuadVertex>,
    pub indices: Vec<u32>,
    pub texture_requests: Vec<TextureRequest>,
    pub mask_texture_requests: Vec<TextureRequest>,
}

#[derive(Clone, Copy)]
struct QuadVertexSet {
    positions: [[f32; 2]; 4],
    tex_coords: [[f32; 2]; 4],
    local_uvs: [[f32; 2]; 4],
    mask_tex_coords: [[f32; 2]; 4],
}

/// Batched quad collection for efficient GPU rendering.
///
/// Collects quads during frame traversal, then uploads to GPU in one batch.
#[derive(Debug, Default, Clone)]
pub struct QuadBatch {
    /// Vertex data for all quads.
    pub vertices: Vec<QuadVertex>,
    /// Index data (6 indices per quad: 2 triangles).
    pub indices: Vec<u32>,
    /// Texture requests for deferred loading (path -> vertices to update).
    pub texture_requests: Vec<TextureRequest>,
    /// Mask texture requests — resolved into mask_tex_index/mask_tex_coords during prepare.
    pub mask_texture_requests: Vec<TextureRequest>,
}

impl QuadBatch {
    /// Create a new empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a batch with pre-allocated capacity.
    pub fn with_capacity(quad_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(quad_count * 4),
            indices: Vec::with_capacity(quad_count * 6),
            texture_requests: Vec::new(),
            mask_texture_requests: Vec::new(),
        }
    }

    /// Clear the batch for reuse.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.texture_requests.clear();
        self.mask_texture_requests.clear();
    }

    /// Number of quads in the batch.
    pub fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }

    /// Push a simple textured or colored quad.
    ///
    /// # Arguments
    /// * `bounds` - Screen-space rectangle (pixels)
    /// * `uvs` - Texture coordinates (0.0-1.0), or (0,0,1,1) for full texture
    /// * `color` - Vertex color/tint with alpha
    /// * `tex_index` - Texture array index, or -1 for solid color
    /// * `blend_mode` - How to blend with background
    pub fn push_quad(
        &mut self,
        bounds: Rectangle,
        uvs: Rectangle,
        color: [f32; 4],
        tex_index: i32,
        blend_mode: BlendMode,
    ) {
        let base_index = self.vertices.len() as u32;
        let tex_coords = rect_corners(uvs);
        let vertices = QuadVertexSet {
            positions: rect_corners(bounds),
            tex_coords,
            local_uvs: tex_coords,
            mask_tex_coords: zero_quad_uvs(),
        };
        self.push_quad_vertices(vertices, color, tex_index, blend_mode as u32);
        push_quad_indices(&mut self.indices, base_index);
    }

    /// Push a single triangle with explicit vertex positions and UVs.
    pub fn push_triangle(
        &mut self,
        positions: [[f32; 2]; 3],
        uvs: [[f32; 2]; 3],
        color: [f32; 4],
        tex_index: i32,
        blend_mode: BlendMode,
    ) {
        let base_index = self.vertices.len() as u32;
        let flags = blend_mode as u32;
        for i in 0..3 {
            self.vertices.push(QuadVertex {
                position: positions[i],
                tex_coords: uvs[i],
                color,
                tex_index,
                flags,
                local_uv: uvs[i],
                mask_tex_index: -1,
                mask_tex_coords: [0.0, 0.0],
            });
        }
        self.indices
            .extend_from_slice(&[base_index, base_index + 1, base_index + 2]);
    }

    /// Push a cooldown swipe quad (radial clock wipe overlay).
    ///
    /// `progress` is 0.0 (fully covered) to 1.0 (fully revealed/done).
    /// `sample_uv_range` remaps the optional texture sample coordinates while
    /// `local_uv` remains normalized for radial wipe math.
    fn push_cooldown_swipe_impl(
        &mut self,
        bounds: Rectangle,
        progress: f32,
        color: [f32; 4],
        tex_index: i32,
        sample_uv_range: Option<(f32, f32, f32, f32)>,
    ) {
        let base_index = self.vertices.len() as u32;
        let (low_x, low_y, high_x, high_y) = sample_uv_range.unwrap_or((0.0, 0.0, 1.0, 1.0));
        let vertices = QuadVertexSet {
            positions: rect_corners(bounds),
            tex_coords: [[progress, 0.0]; 4],
            local_uvs: unit_quad_uvs(),
            mask_tex_coords: [
                [low_x, low_y],
                [high_x, low_y],
                [high_x, high_y],
                [low_x, high_y],
            ],
        };
        let flags = BlendMode::Alpha as u32 | FLAG_COOLDOWN_SWIPE;
        self.push_quad_vertices(vertices, color, tex_index, flags);
        push_quad_indices(&mut self.indices, base_index);
    }

    fn push_quad_vertices(
        &mut self,
        vertices: QuadVertexSet,
        color: [f32; 4],
        tex_index: i32,
        flags: u32,
    ) {
        for i in 0..4 {
            self.vertices.push(QuadVertex {
                position: vertices.positions[i],
                tex_coords: vertices.tex_coords[i],
                color,
                tex_index,
                flags,
                local_uv: vertices.local_uvs[i],
                mask_tex_index: -1,
                mask_tex_coords: vertices.mask_tex_coords[i],
            });
        }
    }

    /// Push a solid-color cooldown swipe quad.
    pub fn push_cooldown_swipe(&mut self, bounds: Rectangle, progress: f32, color: [f32; 4]) {
        self.push_cooldown_swipe_impl(bounds, progress, color, -1, None);
    }

    /// Push a textured cooldown swipe quad by path with optional sample UV range.
    pub fn push_cooldown_swipe_path(
        &mut self,
        bounds: Rectangle,
        progress: f32,
        path: &str,
        color: [f32; 4],
        sample_uv_range: Option<(f32, f32, f32, f32)>,
    ) {
        let vertex_start = self.vertices.len() as u32;
        self.push_cooldown_swipe_impl(bounds, progress, color, -2, sample_uv_range);
        self.texture_requests
            .push(TextureRequest::new(path, vertex_start, 4));
    }

    /// Push a solid color quad (no texture).
    pub fn push_solid(&mut self, bounds: Rectangle, color: [f32; 4]) {
        self.push_quad(
            bounds,
            Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1.0, 1.0)),
            color,
            -1, // No texture
            BlendMode::Alpha,
        );
    }

    /// Push a solid color triangle (no texture).
    pub fn push_solid_triangle(&mut self, positions: [[f32; 2]; 3], color: [f32; 4]) {
        self.push_triangle(
            positions,
            [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
            color,
            -1,
            BlendMode::Alpha,
        );
    }

    /// Push a solid color quad with per-vertex colors (for gradients).
    /// `colors` order: [TL, TR, BR, BL].
    pub fn push_gradient(&mut self, bounds: Rectangle, colors: [[f32; 4]; 4]) {
        let base_index = self.vertices.len() as u32;
        let positions = [
            [bounds.x, bounds.y],
            [bounds.x + bounds.width, bounds.y],
            [bounds.x + bounds.width, bounds.y + bounds.height],
            [bounds.x, bounds.y + bounds.height],
        ];
        let uv = [0.0, 0.0];
        for i in 0..4 {
            self.vertices.push(QuadVertex {
                position: positions[i],
                tex_coords: uv,
                color: colors[i],
                tex_index: -1,
                flags: BlendMode::Alpha as u32,
                local_uv: uv,
                mask_tex_index: -1,
                mask_tex_coords: [0.0, 0.0],
            });
        }
        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    /// Push a textured quad with full UV coverage.
    pub fn push_textured(
        &mut self,
        bounds: Rectangle,
        tex_index: i32,
        color: [f32; 4],
        blend_mode: BlendMode,
    ) {
        self.push_quad(
            bounds,
            Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1.0, 1.0)),
            color,
            tex_index,
            blend_mode,
        );
    }

    /// Push a textured quad with custom UV coordinates.
    pub fn push_textured_uv(
        &mut self,
        bounds: Rectangle,
        uvs: Rectangle,
        tex_index: i32,
        color: [f32; 4],
        blend_mode: BlendMode,
    ) {
        self.push_quad(bounds, uvs, color, tex_index, blend_mode);
    }

    /// Push a textured quad by path (for deferred texture loading).
    ///
    /// The texture will be loaded and the vertex tex_index updated during prepare().
    /// Uses a placeholder color (white) for tinting.
    pub fn push_textured_path(
        &mut self,
        bounds: Rectangle,
        path: &str,
        color: [f32; 4],
        blend_mode: BlendMode,
    ) {
        let vertex_start = self.vertices.len() as u32;
        // Push with tex_index = -2 as marker for "pending texture"
        self.push_quad(
            bounds,
            Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1.0, 1.0)),
            color,
            -2, // Marker for pending texture
            blend_mode,
        );
        self.texture_requests
            .push(TextureRequest::new(path, vertex_start, 4));
    }

    /// Push a textured quad by path with custom UV coordinates.
    pub fn push_textured_path_uv(
        &mut self,
        bounds: Rectangle,
        uvs: Rectangle,
        path: &str,
        color: [f32; 4],
        blend_mode: BlendMode,
    ) {
        let vertex_start = self.vertices.len() as u32;
        self.push_quad(bounds, uvs, color, -2, blend_mode);
        self.texture_requests
            .push(TextureRequest::new(path, vertex_start, 4));
    }

    /// Push a textured triangle by path (for deferred texture loading).
    pub fn push_textured_triangle_path(
        &mut self,
        positions: [[f32; 2]; 3],
        uvs: [[f32; 2]; 3],
        path: &str,
        color: [f32; 4],
        blend_mode: BlendMode,
    ) {
        let vertex_start = self.vertices.len() as u32;
        self.push_triangle(positions, uvs, color, -2, blend_mode);
        self.texture_requests
            .push(TextureRequest::new(path, vertex_start, 3));
    }
}

fn rect_corners(rect: Rectangle) -> [[f32; 2]; 4] {
    [
        [rect.x, rect.y],
        [rect.x + rect.width, rect.y],
        [rect.x + rect.width, rect.y + rect.height],
        [rect.x, rect.y + rect.height],
    ]
}

fn zero_quad_uvs() -> [[f32; 2]; 4] {
    [[0.0, 0.0]; 4]
}

fn unit_quad_uvs() -> [[f32; 2]; 4] {
    [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
}

fn push_quad_indices(indices: &mut Vec<u32>, base_index: u32) {
    indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);
}
