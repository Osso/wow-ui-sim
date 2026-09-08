//! Mask texture UV computation and application.

use iced::Rectangle;

use crate::render::texture::UI_SCALE;
use crate::render::{QuadBatch, TextureRequest};

/// Apply mask texture to recently emitted quads by setting mask_tex_index/mask_tex_coords
/// for GPU alpha sampling (resolved to atlas coords during prepare).
///
/// Uses the mask frame's computed layout bounds to determine the UV mapping. The mask
/// texture is stretched to fill the mask's layout area; the icon UVs are computed based
/// on where the icon falls within that area.
pub fn apply_mask_texture(
    batch: &mut QuadBatch,
    vert_before: usize,
    icon_bounds: Rectangle,
    mask_textures: &[u64],
    registry: &crate::widget::WidgetRegistry,
) {
    let count = batch.vertices.len() - vert_before;
    if count == 0 || mask_textures.is_empty() {
        return;
    }
    let Some(mask_info) =
        resolve_mask_info(mask_textures, registry, icon_bounds, batch, vert_before)
    else {
        return;
    };
    let mask_path = mask_info.path.clone();
    apply_mask_to_quads(batch, vert_before, mask_info);
    batch.mask_texture_requests.push(TextureRequest::new(
        mask_path,
        vert_before as u32,
        count as u32,
    ));
}

/// Apply a direct mask texture path to recently emitted quads.
///
/// This is used by widget types like `Minimap`, where the mask is stored as
/// frame state rather than as a child mask texture frame.
pub fn apply_mask_path(
    batch: &mut QuadBatch,
    vert_before: usize,
    icon_bounds: Rectangle,
    mask_path: &str,
) {
    let count = batch.vertices.len() - vert_before;
    if count == 0 {
        return;
    }
    let mask_info = MaskInfo {
        path: mask_path.to_string(),
        screen_rect: icon_bounds,
        tex_coords: (0.0, 1.0, 0.0, 1.0),
    };
    apply_mask_to_quads(batch, vert_before, mask_info);
    batch.mask_texture_requests.push(TextureRequest::new(
        mask_path,
        vert_before as u32,
        count as u32,
    ));
}

struct MaskInfo {
    path: String,
    screen_rect: Rectangle,
    tex_coords: (f32, f32, f32, f32),
}

fn resolve_mask_info(
    mask_textures: &[u64],
    registry: &crate::widget::WidgetRegistry,
    icon_bounds: Rectangle,
    batch: &mut QuadBatch,
    vert_before: usize,
) -> Option<MaskInfo> {
    let mask_frame = registry.get(mask_textures[0])?;
    let path = mask_frame.texture.clone()?;
    let mask_screen = mask_to_screen_rect(mask_frame.layout_rect?);
    if !rects_overlap(icon_bounds, mask_screen) {
        truncate_masked_vertices(batch, vert_before);
        return None;
    }
    Some(MaskInfo {
        path,
        screen_rect: mask_screen,
        tex_coords: mask_frame.tex_coords.unwrap_or((0.0, 1.0, 0.0, 1.0)),
    })
}

fn truncate_masked_vertices(batch: &mut QuadBatch, vert_before: usize) {
    batch.vertices.truncate(vert_before);
    batch.indices.truncate(vert_before / 4 * 6);
    // Remove texture requests that reference the truncated vertices.
    // Without this, orphaned requests resolve later frames' vertices
    // to the wrong texture (the truncated frame's texture instead of
    // the correct one).
    let vb = vert_before as u32;
    batch.texture_requests.retain(|r| r.vertex_start < vb);
}

fn apply_mask_to_quads(batch: &mut QuadBatch, vert_before: usize, mask_info: MaskInfo) {
    let (tl, tr, tt, tb) = mask_info.tex_coords;
    for i in (vert_before..batch.vertices.len()).step_by(4) {
        let end = (i + 4).min(batch.vertices.len());
        if end - i < 4 {
            continue;
        }
        apply_mask_to_quad(
            &mut batch.vertices[i..end],
            mask_info.screen_rect,
            tl,
            tr,
            tt,
            tb,
        );
    }
}

fn apply_mask_to_quad(
    vertices: &mut [crate::render::shader::QuadVertex],
    mask_screen: Rectangle,
    tl: f32,
    tr: f32,
    tt: f32,
    tb: f32,
) {
    let quad_bounds = quad_rect(vertices);
    let Some(clipped) = rect_intersection(quad_bounds, mask_screen) else {
        hide_quad(vertices);
        return;
    };
    clip_quad_to_rect(vertices, quad_bounds, clipped);
    let mask_uvs = compute_mask_uvs_from_rects(mask_screen, clipped, tl, tr, tt, tb);
    for (index, vertex) in vertices.iter_mut().enumerate() {
        vertex.mask_tex_index = -2;
        vertex.mask_tex_coords = mask_uvs[index];
    }
}

fn mask_to_screen_rect(r: crate::LayoutRect) -> Rectangle {
    Rectangle::new(
        iced::Point::new(r.x * UI_SCALE, r.y * UI_SCALE),
        iced::Size::new(r.width * UI_SCALE, r.height * UI_SCALE),
    )
}

fn rects_overlap(a: Rectangle, b: Rectangle) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn rect_intersection(a: Rectangle, b: Rectangle) -> Option<Rectangle> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    (right > left && bottom > top).then(|| {
        Rectangle::new(
            iced::Point::new(left, top),
            iced::Size::new(right - left, bottom - top),
        )
    })
}

fn quad_rect(vertices: &[crate::render::shader::QuadVertex]) -> Rectangle {
    debug_assert!(vertices.len() >= 4);
    let left = vertices[0].position[0];
    let top = vertices[0].position[1];
    let right = vertices[2].position[0];
    let bottom = vertices[2].position[1];
    Rectangle::new(
        iced::Point::new(left, top),
        iced::Size::new(right - left, bottom - top),
    )
}

fn hide_quad(vertices: &mut [crate::render::shader::QuadVertex]) {
    let x = vertices[0].position[0];
    let y = vertices[0].position[1];
    for vertex in vertices.iter_mut() {
        vertex.position = [x, y];
        vertex.color[3] = 0.0;
        vertex.mask_tex_index = -1;
        vertex.mask_tex_coords = [0.0, 0.0];
    }
}

fn clip_quad_to_rect(
    vertices: &mut [crate::render::shader::QuadVertex],
    original: Rectangle,
    clipped: Rectangle,
) {
    let x0 = fraction_within(original.x, original.width, clipped.x);
    let x1 = fraction_within(original.x, original.width, clipped.x + clipped.width);
    let y0 = fraction_within(original.y, original.height, clipped.y);
    let y1 = fraction_within(original.y, original.height, clipped.y + clipped.height);

    let orig = [vertices[0], vertices[1], vertices[2], vertices[3]];
    let positions = [
        [clipped.x, clipped.y],
        [clipped.x + clipped.width, clipped.y],
        [clipped.x + clipped.width, clipped.y + clipped.height],
        [clipped.x, clipped.y + clipped.height],
    ];
    let x_fracs = [x0, x1, x1, x0];
    let y_fracs = [y0, y0, y1, y1];

    for (idx, vertex) in vertices.iter_mut().enumerate().take(4) {
        vertex.position = positions[idx];
        vertex.tex_coords = remap_uv(&orig, x_fracs[idx], y_fracs[idx], |v| v.tex_coords);
        vertex.local_uv = remap_uv(&orig, x_fracs[idx], y_fracs[idx], |v| v.local_uv);
    }
}

fn fraction_within(start: f32, span: f32, value: f32) -> f32 {
    if span.abs() <= f32::EPSILON {
        0.0
    } else {
        ((value - start) / span).clamp(0.0, 1.0)
    }
}

fn remap_uv(
    original: &[crate::render::shader::QuadVertex; 4],
    x_frac: f32,
    y_frac: f32,
    getter: impl Fn(&crate::render::shader::QuadVertex) -> [f32; 2],
) -> [f32; 2] {
    let tl = getter(&original[0]);
    let tr = getter(&original[1]);
    let bl = getter(&original[3]);
    [
        tl[0] + (tr[0] - tl[0]) * x_frac,
        tl[1] + (bl[1] - tl[1]) * y_frac,
    ]
}

/// Compute mask UVs from pre-computed screen-space rectangles.
///
/// Maps the icon position within the mask area to UV space, clamping to
/// the mask's tex_coord range to prevent atlas sampling artifacts.
fn compute_mask_uvs_from_rects(
    mask_screen: Rectangle,
    icon_bounds: Rectangle,
    tl: f32,
    tr: f32,
    tt: f32,
    tb: f32,
) -> [[f32; 2]; 4] {
    let (mw, mh) = (mask_screen.width, mask_screen.height);
    if mw <= 0.0 || mh <= 0.0 {
        return [[tl, tt], [tr, tt], [tr, tb], [tl, tb]];
    }
    let dx = icon_bounds.x - mask_screen.x;
    let dy = icon_bounds.y - mask_screen.y;
    let (u0, v0) = (dx / mw, dy / mh);
    let (u1, v1) = (
        (dx + icon_bounds.width) / mw,
        (dy + icon_bounds.height) / mh,
    );
    // Clamp to mask's tex_coord range to avoid sampling outside the
    // mask's atlas sub-region (GPU ClampToEdge would hit unrelated pixels).
    let ul = (tl + u0 * (tr - tl)).clamp(tl, tr);
    let ur = (tl + u1 * (tr - tl)).clamp(tl, tr);
    let ut = (tt + v0 * (tb - tt)).clamp(tt, tb);
    let ub = (tt + v1 * (tb - tt)).clamp(tt, tb);
    [[ul, ut], [ur, ut], [ur, ub], [ul, ub]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::shader::QuadVertex;
    use crate::widget::{Frame, WidgetRegistry};

    fn quad(bounds: Rectangle) -> [QuadVertex; 4] {
        let positions = [
            [bounds.x, bounds.y],
            [bounds.x + bounds.width, bounds.y],
            [bounds.x + bounds.width, bounds.y + bounds.height],
            [bounds.x, bounds.y + bounds.height],
        ];
        let tex = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        positions.map(|position| QuadVertex {
            position,
            tex_coords: tex[0],
            color: [1.0; 4],
            tex_index: -2,
            flags: 0,
            local_uv: tex[0],
            mask_tex_index: -1,
            mask_tex_coords: [0.0, 0.0],
        })
    }

    #[test]
    fn clip_quad_to_mask_intersection_remaps_positions_and_uvs() {
        let original = Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(100.0, 20.0));
        let clipped = Rectangle::new(iced::Point::new(25.0, 0.0), iced::Size::new(50.0, 20.0));
        let mut vertices = quad(original);
        vertices[0].tex_coords = [0.0, 0.0];
        vertices[1].tex_coords = [1.0, 0.0];
        vertices[2].tex_coords = [1.0, 1.0];
        vertices[3].tex_coords = [0.0, 1.0];
        vertices[0].local_uv = [0.0, 0.0];
        vertices[1].local_uv = [1.0, 0.0];
        vertices[2].local_uv = [1.0, 1.0];
        vertices[3].local_uv = [0.0, 1.0];

        clip_quad_to_rect(&mut vertices, original, clipped);

        assert_eq!(vertices[0].position, [25.0, 0.0]);
        assert_eq!(vertices[1].position, [75.0, 0.0]);
        assert_eq!(vertices[2].position, [75.0, 20.0]);
        assert_eq!(vertices[3].position, [25.0, 20.0]);
        assert_eq!(vertices[0].tex_coords, [0.25, 0.0]);
        assert_eq!(vertices[1].tex_coords, [0.75, 0.0]);
        assert_eq!(vertices[2].tex_coords, [0.75, 1.0]);
        assert_eq!(vertices[3].tex_coords, [0.25, 1.0]);
    }

    #[test]
    fn compute_mask_uvs_for_partial_overlap_tracks_clipped_rect() {
        let mask = Rectangle::new(iced::Point::new(50.0, 50.0), iced::Size::new(20.0, 20.0));
        let clipped = Rectangle::new(iced::Point::new(55.0, 50.0), iced::Size::new(10.0, 20.0));
        let uvs = compute_mask_uvs_from_rects(mask, clipped, 0.0, 1.0, 0.0, 1.0);
        assert_eq!(uvs, [[0.25, 0.0], [0.75, 0.0], [0.75, 1.0], [0.25, 1.0]]);
    }

    #[test]
    fn apply_mask_texture_marks_vertices_and_adds_mask_request() {
        let mut batch = QuadBatch::new();
        let bounds = Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(20.0, 20.0));
        batch.vertices.extend(quad(bounds));
        batch.indices.extend([0, 1, 2, 0, 2, 3]);

        let mut registry = WidgetRegistry::new();
        let mut mask = Frame::default();
        mask.id = 1;
        mask.texture = Some("Interface/Mask".to_string());
        mask.layout_rect = Some(crate::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 20.0 / UI_SCALE,
            height: 20.0 / UI_SCALE,
        });
        registry.register(mask);

        apply_mask_texture(&mut batch, 0, bounds, &[1], &registry);

        assert_eq!(batch.mask_texture_requests.len(), 1);
        assert_eq!(batch.mask_texture_requests[0].path, "Interface/Mask");
        assert!(
            batch
                .vertices
                .iter()
                .all(|vertex| vertex.mask_tex_index == -2)
        );
    }
}
