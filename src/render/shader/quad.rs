//! Quad vertex format and batching for GPU rendering.

use iced::Rectangle;

/// Flag bit: clip to a circle using UV coordinates (for minimap).
pub const FLAG_CIRCLE_CLIP: u32 = 0x100;

/// Blend mode for quad rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum BlendMode {
    /// Standard alpha blending: src * alpha + dst * (1 - alpha)
    #[default]
    Alpha = 0,
    /// Additive blending: src + dst (for highlight textures)
    Additive = 1,
}

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
        const F: wgpu::VertexFormat = wgpu::VertexFormat::Float32x2;
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: F },                     // position
                wgpu::VertexAttribute { offset: 8, shader_location: 1, format: F },                     // tex_coords
                wgpu::VertexAttribute { offset: 16, shader_location: 2, format: wgpu::VertexFormat::Float32x4 }, // color
                wgpu::VertexAttribute { offset: 32, shader_location: 3, format: wgpu::VertexFormat::Sint32 },    // tex_index
                wgpu::VertexAttribute { offset: 36, shader_location: 4, format: wgpu::VertexFormat::Uint32 },    // flags
                wgpu::VertexAttribute { offset: 40, shader_location: 5, format: F },                    // local_uv
                wgpu::VertexAttribute { offset: 48, shader_location: 6, format: wgpu::VertexFormat::Sint32 },    // mask_tex_index
                wgpu::VertexAttribute { offset: 52, shader_location: 7, format: F },                    // mask_tex_coords
            ],
        }
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

        // Four corners: top-left, top-right, bottom-right, bottom-left
        let positions = [
            [bounds.x, bounds.y],                                  // TL
            [bounds.x + bounds.width, bounds.y],                   // TR
            [bounds.x + bounds.width, bounds.y + bounds.height],   // BR
            [bounds.x, bounds.y + bounds.height],                  // BL
        ];

        let tex_coords = [
            [uvs.x, uvs.y],                          // TL
            [uvs.x + uvs.width, uvs.y],              // TR
            [uvs.x + uvs.width, uvs.y + uvs.height], // BR
            [uvs.x, uvs.y + uvs.height],             // BL
        ];

        let flags = blend_mode as u32;

        for i in 0..4 {
            self.vertices.push(QuadVertex {
                position: positions[i],
                tex_coords: tex_coords[i],
                color,
                tex_index,
                flags,
                local_uv: tex_coords[i],
                mask_tex_index: -1,
                mask_tex_coords: [0.0, 0.0],
            });
        }

        // Two triangles: TL-TR-BR and TL-BR-BL
        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
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
        self.texture_requests.push(TextureRequest {
            path: path.to_string(),
            vertex_start,
            vertex_count: 4,
        });
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
        self.texture_requests.push(TextureRequest {
            path: path.to_string(),
            vertex_start,
            vertex_count: 4,
        });
    }

    /// Push a horizontal 3-slice texture by path (left cap, stretched middle, right cap).
    ///
    /// Used for WoW button textures with fixed left/right caps and stretchable middle.
    /// Texture is loaded on-demand and resolved during prepare().
    ///
    /// # Arguments
    /// * `bounds` - Target screen rectangle
    /// * `left_cap_width` - Width of left cap in screen pixels
    /// * `right_cap_width` - Width of right cap in screen pixels
    /// * `path` - Texture path (will be resolved during prepare)
    /// * `tex_width` - Source texture width in pixels (for UV calculation)
    /// * `color` - Vertex color/tint with alpha
    pub fn push_three_slice_h_path(
        &mut self,
        bounds: Rectangle,
        left_cap_width: f32,
        right_cap_width: f32,
        path: &str,
        tex_width: f32,
        color: [f32; 4],
    ) {
        self.push_three_slice_h_path_blend(
            bounds, left_cap_width, right_cap_width,
            path, tex_width, color, BlendMode::Alpha,
        );
    }

    /// Push a horizontal 3-slice texture with custom blend mode.
    #[allow(clippy::too_many_arguments)]
    pub fn push_three_slice_h_path_blend(
        &mut self,
        bounds: Rectangle,
        left_cap_width: f32,
        right_cap_width: f32,
        path: &str,
        tex_width: f32,
        color: [f32; 4],
        blend_mode: BlendMode,
    ) {
        if bounds.width <= left_cap_width + right_cap_width {
            self.push_textured_path(bounds, path, color, blend_mode);
            return;
        }

        let vertex_start = self.vertices.len() as u32;
        self.push_three_slice_quads(bounds, left_cap_width, right_cap_width, tex_width, color, -2, blend_mode);

        // Single texture request covers all 12 vertices (3 quads * 4 vertices)
        self.texture_requests.push(TextureRequest {
            path: path.to_string(),
            vertex_start,
            vertex_count: 12,
        });
    }

    /// Push a horizontal 3-slice texture (left cap, stretched middle, right cap).
    ///
    /// Used for WoW button textures that have fixed left/right caps with a
    /// stretchable middle section.
    ///
    /// # Arguments
    /// * `bounds` - Target screen rectangle
    /// * `left_cap_width` - Width of left cap in pixels
    /// * `right_cap_width` - Width of right cap in pixels
    /// * `tex_index` - Texture array index
    /// * `tex_width` - Source texture width (for UV calculation)
    /// * `color` - Vertex color/tint with alpha
    pub fn push_three_slice_h(
        &mut self,
        bounds: Rectangle,
        left_cap_width: f32,
        right_cap_width: f32,
        tex_index: i32,
        tex_width: f32,
        color: [f32; 4],
    ) {
        if bounds.width <= left_cap_width + right_cap_width {
            self.push_textured(bounds, tex_index, color, BlendMode::Alpha);
            return;
        }
        self.push_three_slice_quads(bounds, left_cap_width, right_cap_width, tex_width, color, tex_index, BlendMode::Alpha);
    }

    /// Emit the 3 quads (left cap, stretched middle, right cap) for horizontal 3-slice rendering.
    #[allow(clippy::too_many_arguments)]
    fn push_three_slice_quads(
        &mut self,
        bounds: Rectangle,
        left_cap_width: f32,
        right_cap_width: f32,
        tex_width: f32,
        color: [f32; 4],
        tex_index: i32,
        blend_mode: BlendMode,
    ) {
        let middle_width = bounds.width - left_cap_width - right_cap_width;
        let left_uv = left_cap_width / tex_width;
        let right_uv_start = 1.0 - (right_cap_width / tex_width);

        // Left cap
        self.push_quad(
            Rectangle::new(
                iced::Point::new(bounds.x, bounds.y),
                iced::Size::new(left_cap_width, bounds.height),
            ),
            Rectangle::new(iced::Point::ORIGIN, iced::Size::new(left_uv, 1.0)),
            color, tex_index, blend_mode,
        );

        // Middle (stretched)
        self.push_quad(
            Rectangle::new(
                iced::Point::new(bounds.x + left_cap_width, bounds.y),
                iced::Size::new(middle_width, bounds.height),
            ),
            Rectangle::new(
                iced::Point::new(left_uv, 0.0),
                iced::Size::new(right_uv_start - left_uv, 1.0),
            ),
            color, tex_index, blend_mode,
        );

        // Right cap
        self.push_quad(
            Rectangle::new(
                iced::Point::new(bounds.x + bounds.width - right_cap_width, bounds.y),
                iced::Size::new(right_cap_width, bounds.height),
            ),
            Rectangle::new(
                iced::Point::new(right_uv_start, 0.0),
                iced::Size::new(1.0 - right_uv_start, 1.0),
            ),
            color, tex_index, blend_mode,
        );
    }

    /// Push a 9-slice texture (corners fixed, edges stretched, center stretched).
    ///
    /// Used for WoW panel borders and frames.
    ///
    /// # Arguments
    /// * `bounds` - Target screen rectangle
    /// * `corner_size` - Size of corners in pixels (assumed square)
    /// * `edge_size` - Thickness of edges in pixels
    /// * `textures` - Texture indices for each slice: [tl, t, tr, l, c, r, bl, b, br]
    /// * `color` - Vertex color/tint with alpha
    pub fn push_nine_slice(
        &mut self,
        bounds: Rectangle,
        corner_size: f32,
        edge_size: f32,
        textures: &NineSliceTextures,
        color: [f32; 4],
    ) {
        // If bounds too small for corners, just draw center
        if bounds.width < corner_size * 2.0 || bounds.height < corner_size * 2.0 {
            if let Some(center) = textures.center {
                self.push_textured(bounds, center, color, BlendMode::Alpha);
            }
            return;
        }

        let inner_width = bounds.width - corner_size * 2.0;
        let inner_height = bounds.height - corner_size * 2.0;
        let full_uv = Rectangle::new(iced::Point::ORIGIN, iced::Size::new(1.0, 1.0));

        self.push_nine_slice_center(bounds, edge_size, textures, color, full_uv);
        self.push_nine_slice_corners(bounds, corner_size, textures, color, full_uv);
        self.push_nine_slice_edges(bounds, corner_size, edge_size, inner_width, inner_height, textures, color, full_uv);
    }

    /// Push the center quad of a 9-slice texture.
    fn push_nine_slice_center(
        &mut self,
        bounds: Rectangle,
        edge_size: f32,
        textures: &NineSliceTextures,
        color: [f32; 4],
        full_uv: Rectangle,
    ) {
        if let Some(tex) = textures.center {
            let center_bounds = Rectangle::new(
                iced::Point::new(bounds.x + edge_size, bounds.y + edge_size),
                iced::Size::new(bounds.width - edge_size * 2.0, bounds.height - edge_size * 2.0),
            );
            self.push_quad(center_bounds, full_uv, color, tex, BlendMode::Alpha);
        }
    }

    /// Push the four corner quads of a 9-slice texture.
    fn push_nine_slice_corners(
        &mut self,
        bounds: Rectangle,
        corner_size: f32,
        textures: &NineSliceTextures,
        color: [f32; 4],
        full_uv: Rectangle,
    ) {
        if let Some(tex) = textures.top_left {
            let corner = Rectangle::new(
                iced::Point::new(bounds.x, bounds.y),
                iced::Size::new(corner_size, corner_size),
            );
            self.push_quad(corner, full_uv, color, tex, BlendMode::Alpha);
        }

        if let Some(tex) = textures.top_right {
            let corner = Rectangle::new(
                iced::Point::new(bounds.x + bounds.width - corner_size, bounds.y),
                iced::Size::new(corner_size, corner_size),
            );
            self.push_quad(corner, full_uv, color, tex, BlendMode::Alpha);
        }

        if let Some(tex) = textures.bottom_left {
            let corner = Rectangle::new(
                iced::Point::new(bounds.x, bounds.y + bounds.height - corner_size),
                iced::Size::new(corner_size, corner_size),
            );
            self.push_quad(corner, full_uv, color, tex, BlendMode::Alpha);
        }

        if let Some(tex) = textures.bottom_right {
            let corner = Rectangle::new(
                iced::Point::new(
                    bounds.x + bounds.width - corner_size,
                    bounds.y + bounds.height - corner_size,
                ),
                iced::Size::new(corner_size, corner_size),
            );
            self.push_quad(corner, full_uv, color, tex, BlendMode::Alpha);
        }
    }

    /// Push the four edge quads of a 9-slice texture.
    #[allow(clippy::too_many_arguments)]
    fn push_nine_slice_edges(
        &mut self,
        bounds: Rectangle,
        corner_size: f32,
        edge_size: f32,
        inner_width: f32,
        inner_height: f32,
        textures: &NineSliceTextures,
        color: [f32; 4],
        full_uv: Rectangle,
    ) {
        if let Some(tex) = textures.top {
            let edge = Rectangle::new(
                iced::Point::new(bounds.x + corner_size, bounds.y),
                iced::Size::new(inner_width, edge_size),
            );
            self.push_quad(edge, full_uv, color, tex, BlendMode::Alpha);
        }

        if let Some(tex) = textures.bottom {
            let edge = Rectangle::new(
                iced::Point::new(bounds.x + corner_size, bounds.y + bounds.height - edge_size),
                iced::Size::new(inner_width, edge_size),
            );
            self.push_quad(edge, full_uv, color, tex, BlendMode::Alpha);
        }

        if let Some(tex) = textures.left {
            let edge = Rectangle::new(
                iced::Point::new(bounds.x, bounds.y + corner_size),
                iced::Size::new(edge_size, inner_height),
            );
            self.push_quad(edge, full_uv, color, tex, BlendMode::Alpha);
        }

        if let Some(tex) = textures.right {
            let edge = Rectangle::new(
                iced::Point::new(bounds.x + bounds.width - edge_size, bounds.y + corner_size),
                iced::Size::new(edge_size, inner_height),
            );
            self.push_quad(edge, full_uv, color, tex, BlendMode::Alpha);
        }
    }

    /// Push a tiled texture filling the bounds.
    ///
    /// # Arguments
    /// * `bounds` - Target screen rectangle
    /// * `tile_width` - Width of each tile
    /// * `tile_height` - Height of each tile
    /// * `tex_index` - Texture array index
    /// * `color` - Vertex color/tint with alpha
    pub fn push_tiled(
        &mut self,
        bounds: Rectangle,
        tile_width: f32,
        tile_height: f32,
        tex_index: i32,
        color: [f32; 4],
    ) {
        let mut y = bounds.y;
        while y < bounds.y + bounds.height {
            let h = (bounds.y + bounds.height - y).min(tile_height);
            let v_ratio = h / tile_height;

            let mut x = bounds.x;
            while x < bounds.x + bounds.width {
                let w = (bounds.x + bounds.width - x).min(tile_width);
                let u_ratio = w / tile_width;

                self.push_quad(
                    Rectangle::new(iced::Point::new(x, y), iced::Size::new(w, h)),
                    Rectangle::new(iced::Point::ORIGIN, iced::Size::new(u_ratio, v_ratio)),
                    color,
                    tex_index,
                    BlendMode::Alpha,
                );

                x += tile_width;
            }
            y += tile_height;
        }
    }

    /// Push a tiled textured quad by path (for deferred texture loading).
    ///
    /// Like `push_tiled()` but uses a texture path resolved during prepare().
    pub fn push_tiled_path(
        &mut self,
        bounds: Rectangle,
        tile_width: f32,
        tile_height: f32,
        path: &str,
        color: [f32; 4],
    ) {
        let vertex_start = self.vertices.len() as u32;
        let mut vertex_count = 0u32;

        let mut y = bounds.y;
        while y < bounds.y + bounds.height {
            let h = (bounds.y + bounds.height - y).min(tile_height);
            let v_ratio = h / tile_height;

            let mut x = bounds.x;
            while x < bounds.x + bounds.width {
                let w = (bounds.x + bounds.width - x).min(tile_width);
                let u_ratio = w / tile_width;

                self.push_quad(
                    Rectangle::new(iced::Point::new(x, y), iced::Size::new(w, h)),
                    Rectangle::new(
                        iced::Point::ORIGIN,
                        iced::Size::new(u_ratio, v_ratio),
                    ),
                    color,
                    -2,
                    BlendMode::Alpha,
                );
                vertex_count += 4;

                x += tile_width;
            }
            y += tile_height;
        }

        self.texture_requests.push(TextureRequest {
            path: path.to_string(),
            vertex_start,
            vertex_count,
        });
    }

    /// Push a rectangle border (4 edge quads).
    ///
    /// # Arguments
    /// * `bounds` - Outer bounds of the border
    /// * `thickness` - Border thickness in pixels
    /// * `color` - Border color
    pub fn push_border(&mut self, bounds: Rectangle, thickness: f32, color: [f32; 4]) {
        // Top edge
        self.push_solid(
            Rectangle::new(
                iced::Point::new(bounds.x, bounds.y),
                iced::Size::new(bounds.width, thickness),
            ),
            color,
        );

        // Bottom edge
        self.push_solid(
            Rectangle::new(
                iced::Point::new(bounds.x, bounds.y + bounds.height - thickness),
                iced::Size::new(bounds.width, thickness),
            ),
            color,
        );

        // Left edge (excluding corners already covered by top/bottom)
        self.push_solid(
            Rectangle::new(
                iced::Point::new(bounds.x, bounds.y + thickness),
                iced::Size::new(thickness, bounds.height - thickness * 2.0),
            ),
            color,
        );

        // Right edge
        self.push_solid(
            Rectangle::new(
                iced::Point::new(bounds.x + bounds.width - thickness, bounds.y + thickness),
                iced::Size::new(thickness, bounds.height - thickness * 2.0),
            ),
            color,
        );
    }

    /// Append all quads from another batch, adjusting indices.
    pub fn append(&mut self, other: &QuadBatch) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend(other.indices.iter().map(|i| i + base));
        for req in &other.texture_requests {
            self.texture_requests.push(TextureRequest {
                path: req.path.clone(),
                vertex_start: req.vertex_start + base,
                vertex_count: req.vertex_count,
            });
        }
        for req in &other.mask_texture_requests {
            self.mask_texture_requests.push(TextureRequest {
                path: req.path.clone(),
                vertex_start: req.vertex_start + base,
                vertex_count: req.vertex_count,
            });
        }
    }

    /// OR extra flag bits into the last `count` vertices.
    pub fn set_extra_flags(&mut self, count: usize, extra: u32) {
        let start = self.vertices.len() - count;
        for v in &mut self.vertices[start..] {
            v.flags |= extra;
        }
    }
}

/// Texture indices for 9-slice rendering.
#[derive(Debug, Clone, Copy, Default)]
pub struct NineSliceTextures {
    pub top_left: Option<i32>,
    pub top: Option<i32>,
    pub top_right: Option<i32>,
    pub left: Option<i32>,
    pub center: Option<i32>,
    pub right: Option<i32>,
    pub bottom_left: Option<i32>,
    pub bottom: Option<i32>,
    pub bottom_right: Option<i32>,
}
