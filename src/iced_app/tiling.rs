//! Tiled texture rendering — horizontal, vertical, and grid tiling.

use iced::{Point, Rectangle, Size};
use crate::render::{BlendMode, QuadBatch};

#[derive(Debug, Clone, Copy)]
enum TileDir {
    Horizontal,
    Vertical,
    Grid,
}

/// Result of analyzing raw 8-arg SetTexCoord for UV-repeat tiling.
struct UvRepeatInfo {
    /// Base UV region for one tile (U range of the texture strip).
    u_min: f32,
    u_max: f32,
    /// V range for one tile (coordStart to coordEnd or 0.0 to 1.0).
    v_start: f32,
    /// Tile direction on screen.
    dir: TileDir,
    /// Whether UV mapping is rotated (U→vertical, V→horizontal on screen).
    rotated: bool,
}

/// Compute tile dimensions from frame size or UV region as fallback.
fn tile_dimensions(f: &crate::widget::Frame, uv_w: f32, uv_h: f32) -> (f32, f32) {
    let tile_w = if f.width > 1.0 { f.width } else { (uv_w * 128.0).max(8.0) };
    let tile_h = if f.height > 1.0 { f.height } else { (uv_h * 128.0).max(8.0) };
    (tile_w, tile_h)
}

/// Analyze raw 8-arg SetTexCoord values to determine tiling parameters.
///
/// BackdropTemplateMixin encodes repeat counts as UV values >1.0.
/// TopEdge/BottomEdge: Y coords on left corners have repeats, UV is rotated.
/// LeftEdge/RightEdge: Y coords on bottom corners have repeats, UV is standard.
fn analyze_uv_repeat(raw: &[f32; 8]) -> UvRepeatInfo {
    let [ul_x, ul_y, ll_x, ll_y, ur_x, ur_y, lr_x, lr_y] = *raw;

    let left_y_repeats = ul_y > 1.0 || ll_y > 1.0;
    let right_y_repeats = ur_y > 1.0 || lr_y > 1.0;
    let bottom_y_repeats = ll_y > 1.0 || lr_y > 1.0;
    let top_y_repeats = ul_y > 1.0 || ur_y > 1.0;
    let any_x_repeats = ul_x > 1.0 || ll_x > 1.0 || ur_x > 1.0 || lr_x > 1.0;

    let u_min = ul_x.min(ll_x).min(ur_x).min(lr_x);
    let u_max = ul_x.max(ll_x).max(ur_x).max(lr_x).min(1.0);
    let v_start = ul_y.min(ll_y).min(ur_y).min(lr_y);

    // Y repeats on left XOR right → TopEdge/BottomEdge (rotated: V→horizontal)
    if (left_y_repeats ^ right_y_repeats) && !any_x_repeats {
        return UvRepeatInfo { u_min, u_max, v_start, dir: TileDir::Horizontal, rotated: true };
    }

    // Y repeats on bottom XOR top → LeftEdge/RightEdge (standard: V→vertical)
    if (bottom_y_repeats ^ top_y_repeats) && !any_x_repeats {
        return UvRepeatInfo { u_min, u_max, v_start, dir: TileDir::Vertical, rotated: false };
    }

    // Both axes repeat or unknown → grid
    UvRepeatInfo { u_min, u_max, v_start, dir: TileDir::Grid, rotated: false }
}

/// Determine tile pixel size for UV-repeat tiling.
fn uv_repeat_tile_size(f: &crate::widget::Frame) -> (f32, f32) {
    if f.width > 1.0 && f.height > 1.0 {
        (f.width, f.height)
    } else if f.height > 1.0 {
        (f.height, f.height)
    } else if f.width > 1.0 {
        (f.width, f.width)
    } else {
        (32.0, 32.0) // fallback for backdrop edges
    }
}

fn frame_tint(f: &crate::widget::Frame) -> [f32; 4] {
    let vc = f.vertex_color.as_ref();
    [
        vc.map_or(1.0, |c| c.r),
        vc.map_or(1.0, |c| c.g),
        vc.map_or(1.0, |c| c.b),
        vc.map_or(1.0, |c| c.a) * f.alpha,
    ]
}

/// Emit tiled texture quads (horizontal, vertical, or both).
pub(super) fn emit_tiled_texture(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    uvs: &Rectangle,
    tex_path: &str,
    f: &crate::widget::Frame,
) {
    // Check for UV-based repeat tiling (BackdropTemplateMixin pattern):
    // 8-arg SetTexCoord with values >1.0 encoding repeat counts.
    if let Some(raw) = &f.tex_coords_quad {
        if raw.iter().any(|&v| v > 1.0) {
            emit_uv_repeat_tiled(batch, bounds, raw, tex_path, f);
            return;
        }
    }

    let (left, right, top, bottom) = (uvs.x, uvs.x + uvs.width, uvs.y, uvs.y + uvs.height);
    let (tile_w, tile_h) = tile_dimensions(f, right - left, bottom - top);
    let tint = frame_tint(f);

    if f.horiz_tile && !f.vert_tile {
        emit_horiz_tiles(batch, bounds, uvs, tex_path, tile_w, tint, f.blend_mode);
    } else if f.vert_tile && !f.horiz_tile {
        emit_vert_tiles(batch, bounds, uvs, tex_path, tile_h, tint, f.blend_mode);
    } else {
        emit_grid_tiles(batch, bounds, uvs, tex_path, tile_w, tile_h, tint, f.blend_mode);
    }
}

/// Handle UV-based repeat tiling from BackdropTemplateMixin.
fn emit_uv_repeat_tiled(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    raw: &[f32; 8],
    tex_path: &str,
    f: &crate::widget::Frame,
) {
    let tint = frame_tint(f);
    let info = analyze_uv_repeat(raw);
    let tile_size = uv_repeat_tile_size(f);

    if info.rotated {
        emit_rotated_horiz_tiles(batch, bounds, &info, tex_path, tile_size.0, tint, f.blend_mode);
    } else {
        let base_uvs = Rectangle::new(
            Point::new(info.u_min, info.v_start),
            Size::new(info.u_max - info.u_min, 1.0 - info.v_start),
        );
        match info.dir {
            TileDir::Vertical => {
                emit_vert_tiles(batch, bounds, &base_uvs, tex_path, tile_size.1, tint, f.blend_mode);
            }
            _ => {
                emit_grid_tiles(batch, bounds, &base_uvs, tex_path, tile_size.0, tile_size.1, tint, f.blend_mode);
            }
        }
    }
}

/// Emit horizontally tiled quads with rotated UV mapping (U→vertical, V→horizontal).
/// Used for BackdropTemplateMixin TopEdge/BottomEdge where V maps to screen horizontal.
#[allow(clippy::too_many_arguments)]
fn emit_rotated_horiz_tiles(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    info: &UvRepeatInfo,
    tex_path: &str,
    tile_w: f32,
    tint: [f32; 4],
    blend: BlendMode,
) {
    let mut x = bounds.x;
    while x < bounds.x + bounds.width {
        let w = (bounds.x + bounds.width - x).min(tile_w);
        let tile_bounds = Rectangle::new(Point::new(x, bounds.y), Size::new(w, bounds.height));
        let v_extent = if w < tile_w { (1.0 - info.v_start) * (w / tile_w) } else { 1.0 - info.v_start };
        // Rotated: U maps to screen Y (top→bottom), V maps to screen X (left→right)
        let uvs = [
            [info.u_min, info.v_start + v_extent], // TL: top of strip, right side of V tile
            [info.u_min, info.v_start],             // TR: top of strip, left side of V tile
            [info.u_max, info.v_start],             // BR: bottom of strip, left side of V tile
            [info.u_max, info.v_start + v_extent],  // BL: bottom of strip, right side of V tile
        ];
        batch.push_textured_path_uv4(tile_bounds, uvs, tex_path, tint, blend);
        x += tile_w;
    }
}

/// Emit horizontally tiled texture quads.
pub(super) fn emit_horiz_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_w: f32, tint: [f32; 4], blend: BlendMode) {
    let mut x = bounds.x;
    while x < bounds.x + bounds.width {
        let w = (bounds.x + bounds.width - x).min(tile_w);
        let tile_bounds = Rectangle::new(Point::new(x, bounds.y), Size::new(w, bounds.height));
        let uv_w = if w < tile_w { uvs.width * (w / tile_w) } else { uvs.width };
        let tile_uvs = Rectangle::new(uvs.position(), Size::new(uv_w, uvs.height));
        batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, tint, blend);
        x += tile_w;
    }
}

/// Emit vertically tiled texture quads.
pub(super) fn emit_vert_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_h: f32, tint: [f32; 4], blend: BlendMode) {
    let mut y = bounds.y;
    while y < bounds.y + bounds.height {
        let h = (bounds.y + bounds.height - y).min(tile_h);
        let tile_bounds = Rectangle::new(Point::new(bounds.x, y), Size::new(bounds.width, h));
        let uv_h = if h < tile_h { uvs.height * (h / tile_h) } else { uvs.height };
        let tile_uvs = Rectangle::new(uvs.position(), Size::new(uvs.width, uv_h));
        batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, tint, blend);
        y += tile_h;
    }
}

/// Emit grid-tiled texture quads (both horizontal and vertical).
#[allow(clippy::too_many_arguments)]
fn emit_grid_tiles(batch: &mut QuadBatch, bounds: Rectangle, uvs: &Rectangle, tex_path: &str, tile_w: f32, tile_h: f32, tint: [f32; 4], blend: BlendMode) {
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
            batch.push_textured_path_uv(tile_bounds, tile_uvs, tex_path, tint, blend);
            x += tile_w;
        }
        y += tile_h;
    }
}
