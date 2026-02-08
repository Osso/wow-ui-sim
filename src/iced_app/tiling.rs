//! Tiled texture rendering — horizontal, vertical, and grid tiling.

use iced::{Point, Rectangle, Size};
use crate::render::{BlendMode, QuadBatch};

/// Compute tile dimensions from frame size or UV region as fallback.
fn tile_dimensions(f: &crate::widget::Frame, uv_w: f32, uv_h: f32) -> (f32, f32) {
    let tile_w = if f.width > 1.0 { f.width } else { (uv_w * 128.0).max(8.0) };
    let tile_h = if f.height > 1.0 { f.height } else { (uv_h * 128.0).max(8.0) };
    (tile_w, tile_h)
}

/// Emit tiled texture quads (horizontal, vertical, or both).
pub(super) fn emit_tiled_texture(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    uvs: &Rectangle,
    tex_path: &str,
    f: &crate::widget::Frame,
) {
    let (left, right, top, bottom) = (uvs.x, uvs.x + uvs.width, uvs.y, uvs.y + uvs.height);
    let (tile_w, tile_h) = tile_dimensions(f, right - left, bottom - top);

    if f.horiz_tile && !f.vert_tile {
        emit_horiz_tiles(batch, bounds, uvs, tex_path, tile_w, f.alpha, f.blend_mode);
    } else if f.vert_tile && !f.horiz_tile {
        emit_vert_tiles(batch, bounds, uvs, tex_path, tile_h, f.alpha, f.blend_mode);
    } else {
        emit_grid_tiles(batch, bounds, uvs, tex_path, tile_w, tile_h, f.alpha, f.blend_mode);
    }
}

/// Emit horizontally tiled texture quads.
pub(super) fn emit_horiz_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_w: f32, alpha: f32, blend: BlendMode) {
    let mut x = bounds.x;
    while x < bounds.x + bounds.width {
        let w = (bounds.x + bounds.width - x).min(tile_w);
        let tile_bounds = Rectangle::new(Point::new(x, bounds.y), Size::new(w, bounds.height));
        let uv_w = if w < tile_w { uvs.width * (w / tile_w) } else { uvs.width };
        let tile_uvs = Rectangle::new(uvs.position(), Size::new(uv_w, uvs.height));
        batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, alpha], blend);
        x += tile_w;
    }
}

/// Emit vertically tiled texture quads.
pub(super) fn emit_vert_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_h: f32, alpha: f32, blend: BlendMode) {
    let mut y = bounds.y;
    while y < bounds.y + bounds.height {
        let h = (bounds.y + bounds.height - y).min(tile_h);
        let tile_bounds = Rectangle::new(Point::new(bounds.x, y), Size::new(bounds.width, h));
        let uv_h = if h < tile_h { uvs.height * (h / tile_h) } else { uvs.height };
        let tile_uvs = Rectangle::new(uvs.position(), Size::new(uvs.width, uv_h));
        batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, alpha], blend);
        y += tile_h;
    }
}

/// Emit grid-tiled texture quads (both horizontal and vertical).
fn emit_grid_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_w: f32, tile_h: f32, alpha: f32, blend: BlendMode) {
    let mut y = bounds.y;
    while y < bounds.y + bounds.height {
        let h = (bounds.y + bounds.height - y).min(tile_h);
        let mut x = bounds.x;
        while x < bounds.x + bounds.width {
            let w = (bounds.x + bounds.width - x).min(tile_w);
            let tile_bounds = Rectangle::new(Point::new(x, y), Size::new(w, h));
            let uv_w = if w < tile_w { uvs.width * (w / tile_w) } else { uvs.width };
            let uv_h = if h < tile_h { uvs.height * (h / tile_h) } else { uvs.height };
            let tile_uvs = Rectangle::new(uvs.position(), Size::new(uv_w, uv_h));
            batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, [1.0, 1.0, 1.0, alpha], blend);
            x += tile_w;
        }
        y += tile_h;
    }
}
