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
    batch: &mut QuadBatch, vert_before: usize, icon_bounds: Rectangle,
    mask_textures: &[u64], registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
    cache: &mut super::layout::LayoutCache,
) {
    let count = batch.vertices.len() - vert_before;
    if count == 0 || mask_textures.is_empty() { return; }
    let mask_id = mask_textures[0];
    let Some(mask_frame) = registry.get(mask_id) else {
        return;
    };
    let Some(ref mask_path) = mask_frame.texture else {
        return;
    };
    // Check if icon overlaps the mask area at all. If not, remove the quads
    // entirely — the texture is fully outside the mask (e.g., animation-driven
    // textures at rest position).
    let mask_rect = super::layout::compute_frame_rect_cached(
        registry, mask_id, screen_size.0, screen_size.1, cache,
    ).rect;
    let mask_screen = mask_to_screen_rect(mask_rect);
    if !rects_overlap(icon_bounds, mask_screen) {
        batch.vertices.truncate(vert_before);
        batch.indices.truncate(vert_before / 4 * 6);
        return;
    }
    let (tl, tr, tt, tb) = mask_frame.tex_coords.unwrap_or((0.0, 1.0, 0.0, 1.0));
    let mask_uvs = compute_mask_uvs_from_rects(
        mask_screen, icon_bounds, tl, tr, tt, tb,
    );
    for i in (vert_before..batch.vertices.len()).step_by(4) {
        let end = (i + 4).min(batch.vertices.len());
        for (j, v) in batch.vertices[i..end].iter_mut().enumerate() {
            v.mask_tex_index = -2;
            v.mask_tex_coords = mask_uvs[j];
        }
    }
    batch.mask_texture_requests.push(TextureRequest {
        path: mask_path.clone(), vertex_start: vert_before as u32, vertex_count: count as u32,
    });
}

fn mask_to_screen_rect(r: crate::LayoutRect) -> Rectangle {
    Rectangle::new(
        iced::Point::new(r.x * UI_SCALE, r.y * UI_SCALE),
        iced::Size::new(r.width * UI_SCALE, r.height * UI_SCALE),
    )
}

fn rects_overlap(a: Rectangle, b: Rectangle) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x
        && a.y < b.y + b.height && a.y + a.height > b.y
}

/// Compute mask UVs from pre-computed screen-space rectangles.
///
/// Maps the icon position within the mask area to UV space, clamping to
/// the mask's tex_coord range to prevent atlas sampling artifacts.
fn compute_mask_uvs_from_rects(
    mask_screen: Rectangle, icon_bounds: Rectangle,
    tl: f32, tr: f32, tt: f32, tb: f32,
) -> [[f32; 2]; 4] {
    let (mw, mh) = (mask_screen.width, mask_screen.height);
    if mw <= 0.0 || mh <= 0.0 {
        return [[tl, tt], [tr, tt], [tr, tb], [tl, tb]];
    }
    let dx = icon_bounds.x - mask_screen.x;
    let dy = icon_bounds.y - mask_screen.y;
    let (u0, v0) = (dx / mw, dy / mh);
    let (u1, v1) = ((dx + icon_bounds.width) / mw, (dy + icon_bounds.height) / mh);
    // Clamp to mask's tex_coord range to avoid sampling outside the
    // mask's atlas sub-region (GPU ClampToEdge would hit unrelated pixels).
    let ul = (tl + u0 * (tr - tl)).clamp(tl, tr);
    let ur = (tl + u1 * (tr - tl)).clamp(tl, tr);
    let ut = (tt + v0 * (tb - tt)).clamp(tt, tb);
    let ub = (tt + v1 * (tb - tt)).clamp(tt, tb);
    [[ul, ut], [ur, ut], [ur, ub], [ul, ub]]
}
