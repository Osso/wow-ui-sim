use super::quad::{BlendMode, FrameQuadSnapshot, QuadBatch, QuadVertex, TextureRequest};
use iced::{Point, Rectangle, Size};

struct ThreeSliceParams<'a> {
    bounds: Rectangle,
    left_cap_width: f32,
    right_cap_width: f32,
    tex_width: f32,
    color: [f32; 4],
    tex_index: i32,
    blend_mode: BlendMode,
    /// Normalized V of top edge of source strip (0.0 = texture top).
    v_top: f32,
    /// Normalized V of bottom edge of source strip (1.0 = texture bottom).
    v_bottom: f32,
    /// Texture path (when known) used for path-keyed v_bottom overrides.
    path: Option<&'a str>,
}

pub struct ThreeSlicePathParams<'a> {
    pub bounds: Rectangle,
    pub left_cap_width: f32,
    pub right_cap_width: f32,
    pub path: &'a str,
    pub tex_width: f32,
    pub color: [f32; 4],
    pub blend_mode: BlendMode,
    pub v_top: f32,
    pub v_bottom: f32,
}

struct ThreeSliceLayout {
    left_x: f32,
    middle_x: f32,
    right_x: f32,
    middle_width: f32,
    left_uv: f32,
    right_uv_start: f32,
}

impl QuadBatch {
    /// Push a textured quad by path with explicit per-vertex UV coordinates.
    /// Used for rotated UV mappings (e.g., BackdropTemplateMixin edge textures).
    /// `uvs` is [TL, TR, BR, BL] with each entry being [u, v].
    pub fn push_textured_path_uv4(
        &mut self,
        bounds: Rectangle,
        uvs: [[f32; 2]; 4],
        path: &str,
        color: [f32; 4],
        blend_mode: BlendMode,
    ) {
        let vertex_start = self.vertices.len() as u32;
        let base_index = self.vertices.len() as u32;
        let positions = quad_positions(bounds);
        let flags = blend_mode as u32;

        for i in 0..4 {
            self.vertices.push(QuadVertex {
                position: positions[i],
                tex_coords: uvs[i],
                color,
                tex_index: -2,
                flags,
                local_uv: uvs[i],
                mask_tex_index: -1,
                mask_tex_coords: [0.0, 0.0],
            });
        }

        self.push_quad_indices(base_index);
        self.push_texture_request(path, vertex_start, 4);
    }

    /// Push textured quads by path, each given as its four (position, uv)
    /// corners in the order the quad's two triangles use (0-1-2, 0-2-3).
    /// Positions are final screen pixels; uvs stay within one texture repeat.
    pub fn push_textured_path_corner_quads(
        &mut self,
        quads: &[[([f32; 2], [f32; 2]); 4]],
        path: &str,
        color: [f32; 4],
        blend_mode: BlendMode,
    ) {
        if quads.is_empty() {
            return;
        }
        let vertex_start = self.vertices.len() as u32;
        let flags = blend_mode as u32;
        for quad in quads {
            let base_index = self.vertices.len() as u32;
            for &(position, uv) in quad {
                self.vertices.push(QuadVertex {
                    position,
                    tex_coords: uv,
                    color,
                    tex_index: -2,
                    flags,
                    local_uv: uv,
                    mask_tex_index: -1,
                    mask_tex_coords: [0.0, 0.0],
                });
            }
            self.push_quad_indices(base_index);
        }
        self.push_texture_request(path, vertex_start, 4 * quads.len() as u32);
    }

    /// Push a horizontal 3-slice texture with custom blend mode.
    pub fn push_three_slice_h_path_blend(&mut self, params: ThreeSlicePathParams<'_>) {
        if params.bounds.width <= params.left_cap_width + params.right_cap_width {
            self.push_textured_path(params.bounds, params.path, params.color, params.blend_mode);
            return;
        }

        let vertex_start = self.vertices.len() as u32;
        self.push_three_slice_quads(ThreeSliceParams {
            bounds: params.bounds,
            left_cap_width: params.left_cap_width,
            right_cap_width: params.right_cap_width,
            tex_width: params.tex_width,
            color: params.color,
            tex_index: -2,
            blend_mode: params.blend_mode,
            v_top: params.v_top,
            v_bottom: params.v_bottom,
            path: Some(params.path),
        });
        self.push_texture_request(params.path, vertex_start, 12);
    }

    /// Push a tiled texture filling the bounds.
    pub fn push_tiled(
        &mut self,
        bounds: Rectangle,
        tile_width: f32,
        tile_height: f32,
        tex_index: i32,
        color: [f32; 4],
    ) {
        self.for_each_tile(
            bounds,
            tile_width,
            tile_height,
            |batch, tile_bounds, tile_uvs| {
                batch.push_quad(tile_bounds, tile_uvs, color, tex_index, BlendMode::Alpha);
            },
        );
    }

    /// Push a tiled textured quad by path (for deferred texture loading).
    pub fn push_tiled_path(
        &mut self,
        bounds: Rectangle,
        tile_width: f32,
        tile_height: f32,
        path: &str,
        color: [f32; 4],
    ) {
        let vertex_start = self.vertices.len() as u32;
        let mut quad_count = 0u32;

        self.for_each_tile(
            bounds,
            tile_width,
            tile_height,
            |batch, tile_bounds, tile_uvs| {
                batch.push_quad(tile_bounds, tile_uvs, color, -2, BlendMode::Alpha);
                quad_count += 1;
            },
        );

        self.push_texture_request(path, vertex_start, quad_count * 4);
    }

    /// Push a rectangle border (4 edge quads).
    pub fn push_border(&mut self, bounds: Rectangle, thickness: f32, color: [f32; 4]) {
        for edge in border_rects(bounds, thickness) {
            self.push_solid(edge, color);
        }
    }

    /// Append all quads from another batch, adjusting indices.
    pub fn append(&mut self, other: &QuadBatch) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend(other.indices.iter().map(|i| i + base));
        self.extend_texture_requests(&other.texture_requests, base, false);
        self.extend_texture_requests(&other.mask_texture_requests, base, true);
    }

    /// OR extra flag bits into the last `count` vertices.
    pub fn set_extra_flags(&mut self, count: usize, extra: u32) {
        let start = self.vertices.len() - count;
        for v in &mut self.vertices[start..] {
            v.flags |= extra;
        }
    }

    /// Take a snapshot of quads added since the given offsets.
    pub fn take_snapshot_since(
        &self,
        vert_start: usize,
        idx_start: usize,
        tex_start: usize,
        mask_start: usize,
    ) -> FrameQuadSnapshot {
        let base = vert_start as u32;
        FrameQuadSnapshot {
            vertices: self.vertices[vert_start..].to_vec(),
            indices: self.indices[idx_start..]
                .iter()
                .map(|&i| i - base)
                .collect(),
            texture_requests: self.snapshot_requests(&self.texture_requests[tex_start..], base),
            mask_texture_requests: self
                .snapshot_requests(&self.mask_texture_requests[mask_start..], base),
        }
    }

    /// Append a cached frame snapshot, adjusting indices and offsets.
    pub fn append_snapshot(&mut self, snap: &FrameQuadSnapshot) {
        if snap.vertices.is_empty() {
            return;
        }

        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&snap.vertices);
        self.indices.extend(snap.indices.iter().map(|&i| i + base));
        self.extend_texture_requests(&snap.texture_requests, base, false);
        self.extend_texture_requests(&snap.mask_texture_requests, base, true);
    }

    fn push_three_slice_quads(&mut self, params: ThreeSliceParams) {
        let layout = three_slice_layout(&params);

        self.push_three_slice_segment(
            &params,
            layout.left_x,
            params.left_cap_width,
            0.0,
            layout.left_uv,
        );
        self.push_three_slice_segment(
            &params,
            layout.middle_x,
            layout.middle_width,
            layout.left_uv,
            layout.right_uv_start - layout.left_uv,
        );
        self.push_three_slice_segment(
            &params,
            layout.right_x,
            params.right_cap_width,
            layout.right_uv_start,
            1.0 - layout.right_uv_start,
        );
    }

    fn push_three_slice_segment(
        &mut self,
        params: &ThreeSliceParams,
        dest_x: f32,
        dest_width: f32,
        uv_x: f32,
        uv_width: f32,
    ) {
        let v_bottom = three_slice_v_bottom(params.path, params.v_bottom);
        let v_height = v_bottom - params.v_top;
        self.push_quad(
            Rectangle::new(
                Point::new(dest_x, params.bounds.y),
                Size::new(dest_width, params.bounds.height),
            ),
            Rectangle::new(
                Point::new(uv_x, params.v_top),
                Size::new(uv_width, v_height),
            ),
            params.color,
            params.tex_index,
            params.blend_mode,
        );
    }

    fn for_each_tile<F>(&mut self, bounds: Rectangle, tile_width: f32, tile_height: f32, mut f: F)
    where
        F: FnMut(&mut Self, Rectangle, Rectangle),
    {
        let max_y = bounds.y + bounds.height;
        let max_x = bounds.x + bounds.width;
        let mut y = bounds.y;

        while y < max_y {
            let h = (max_y - y).min(tile_height);
            let v_ratio = h / tile_height;
            let mut x = bounds.x;

            while x < max_x {
                let w = (max_x - x).min(tile_width);
                let u_ratio = w / tile_width;
                let tile_bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));
                let tile_uvs = Rectangle::new(Point::ORIGIN, Size::new(u_ratio, v_ratio));
                f(self, tile_bounds, tile_uvs);
                x += tile_width;
            }

            y += tile_height;
        }
    }

    fn push_quad_indices(&mut self, base_index: u32) {
        self.indices.extend_from_slice(&[
            base_index,
            base_index + 1,
            base_index + 2,
            base_index,
            base_index + 2,
            base_index + 3,
        ]);
    }

    fn push_texture_request(&mut self, path: &str, vertex_start: u32, vertex_count: u32) {
        self.texture_requests
            .push(TextureRequest::new(path, vertex_start, vertex_count));
    }

    fn extend_texture_requests(&mut self, requests: &[TextureRequest], base: u32, is_mask: bool) {
        let target = if is_mask {
            &mut self.mask_texture_requests
        } else {
            &mut self.texture_requests
        };

        target.extend(
            requests
                .iter()
                .map(|req| req.with_vertex_start(req.vertex_start + base)),
        );
    }

    fn snapshot_requests(&self, requests: &[TextureRequest], base: u32) -> Vec<TextureRequest> {
        requests
            .iter()
            .map(|r| r.with_vertex_start(r.vertex_start - base))
            .collect()
    }
}

fn quad_positions(bounds: Rectangle) -> [[f32; 2]; 4] {
    [
        [bounds.x, bounds.y],
        [bounds.x + bounds.width, bounds.y],
        [bounds.x + bounds.width, bounds.y + bounds.height],
        [bounds.x, bounds.y + bounds.height],
    ]
}

fn three_slice_layout(params: &ThreeSliceParams) -> ThreeSliceLayout {
    ThreeSliceLayout {
        left_x: params.bounds.x,
        middle_x: params.bounds.x + params.left_cap_width,
        right_x: params.bounds.x + params.bounds.width - params.right_cap_width,
        middle_width: params.bounds.width - params.left_cap_width - params.right_cap_width,
        left_uv: params.left_cap_width / params.tex_width,
        right_uv_start: 1.0 - (params.right_cap_width / params.tex_width),
    }
}

fn three_slice_v_bottom(path: Option<&str>, fallback: f32) -> f32 {
    match path {
        Some("Interface/Buttons/UI-Panel-Button-Up")
        | Some("Interface\\Buttons\\UI-Panel-Button-Up")
        | Some("Interface/Buttons/UI-Panel-Button-Highlight")
        | Some("Interface\\Buttons\\UI-Panel-Button-Highlight") => 22.0 / 32.0,
        _ => fallback,
    }
}

fn border_rects(bounds: Rectangle, thickness: f32) -> [Rectangle; 4] {
    [
        Rectangle::new(
            Point::new(bounds.x, bounds.y),
            Size::new(bounds.width, thickness),
        ),
        Rectangle::new(
            Point::new(bounds.x, bounds.y + bounds.height - thickness),
            Size::new(bounds.width, thickness),
        ),
        Rectangle::new(
            Point::new(bounds.x, bounds.y + thickness),
            Size::new(thickness, bounds.height - thickness * 2.0),
        ),
        Rectangle::new(
            Point::new(bounds.x + bounds.width - thickness, bounds.y + thickness),
            Size::new(thickness, bounds.height - thickness * 2.0),
        ),
    ]
}
