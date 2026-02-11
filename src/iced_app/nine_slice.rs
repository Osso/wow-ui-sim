//! Nine-slice atlas rendering: 4 corners + 4 tiled edges + optional center.

use iced::{Point, Rectangle, Size};

use crate::atlas::{NineSliceAtlasInfo, NineSlicePiece};
use crate::render::{BlendMode, QuadBatch};

use super::tiling::{emit_horiz_tiles, emit_vert_tiles};

/// Emit a single nine-slice piece as a textured quad.
fn emit_piece(batch: &mut QuadBatch, bounds: Rectangle, piece: &NineSlicePiece, alpha: f32) {
    let uvs = Rectangle::new(
        Point::new(piece.left, piece.top),
        Size::new(piece.right - piece.left, piece.bottom - piece.top),
    );
    batch.push_textured_path_uv(bounds, uvs, piece.file, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);
}

/// UV rectangle for a nine-slice piece.
fn piece_uvs(p: &NineSlicePiece) -> Rectangle {
    Rectangle::new(
        Point::new(p.left, p.top),
        Size::new(p.right - p.left, p.bottom - p.top),
    )
}

/// Emit all four corners of a nine-slice kit.
fn emit_corners(batch: &mut QuadBatch, bounds: Rectangle, ns: &NineSliceAtlasInfo, alpha: f32) {
    let (left_w, right_w) = (ns.corner_tl.width as f32, ns.corner_tr.width as f32);
    let (top_h, bottom_h) = (ns.corner_tl.height as f32, ns.corner_bl.height as f32);

    let tl = Rectangle::new(bounds.position(), Size::new(left_w, top_h));
    let tr = Rectangle::new(
        Point::new(bounds.x + bounds.width - right_w, bounds.y),
        Size::new(right_w, top_h),
    );
    let bl = Rectangle::new(
        Point::new(bounds.x, bounds.y + bounds.height - bottom_h),
        Size::new(left_w, bottom_h),
    );
    let br = Rectangle::new(
        Point::new(bounds.x + bounds.width - right_w, bounds.y + bounds.height - bottom_h),
        Size::new(right_w, bottom_h),
    );

    emit_piece(batch, tl, &ns.corner_tl, alpha);
    emit_piece(batch, tr, &ns.corner_tr, alpha);
    emit_piece(batch, bl, &ns.corner_bl, alpha);
    emit_piece(batch, br, &ns.corner_br, alpha);
}

/// Emit tiled horizontal edges (top and bottom) between corners.
fn emit_horiz_edges(batch: &mut QuadBatch, bounds: Rectangle, ns: &NineSliceAtlasInfo, alpha: f32) {
    let edge_x = bounds.x + ns.corner_tl.width as f32;
    let edge_w = bounds.width - ns.corner_tl.width as f32 - ns.corner_tr.width as f32;
    if edge_w <= 0.0 {
        return;
    }

    let top_bounds = Rectangle::new(
        Point::new(edge_x, bounds.y),
        Size::new(edge_w, ns.edge_top.height as f32),
    );
    emit_horiz_tiles(batch, top_bounds, &piece_uvs(&ns.edge_top), ns.edge_top.file, ns.edge_top.width as f32, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);

    let bot_bounds = Rectangle::new(
        Point::new(edge_x, bounds.y + bounds.height - ns.edge_bottom.height as f32),
        Size::new(edge_w, ns.edge_bottom.height as f32),
    );
    emit_horiz_tiles(batch, bot_bounds, &piece_uvs(&ns.edge_bottom), ns.edge_bottom.file, ns.edge_bottom.width as f32, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);
}

/// Emit tiled vertical edges (left and right) between corners.
fn emit_vert_edges(batch: &mut QuadBatch, bounds: Rectangle, ns: &NineSliceAtlasInfo, alpha: f32) {
    let edge_y = bounds.y + ns.corner_tl.height as f32;
    let edge_h = bounds.height - ns.corner_tl.height as f32 - ns.corner_bl.height as f32;
    if edge_h <= 0.0 {
        return;
    }

    let left_bounds = Rectangle::new(
        Point::new(bounds.x, edge_y),
        Size::new(ns.edge_left.width as f32, edge_h),
    );
    emit_vert_tiles(batch, left_bounds, &piece_uvs(&ns.edge_left), ns.edge_left.file, ns.edge_left.height as f32, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);

    let right_bounds = Rectangle::new(
        Point::new(bounds.x + bounds.width - ns.edge_right.width as f32, edge_y),
        Size::new(ns.edge_right.width as f32, edge_h),
    );
    emit_vert_tiles(batch, right_bounds, &piece_uvs(&ns.edge_right), ns.edge_right.file, ns.edge_right.height as f32, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);
}

/// Emit a nine-slice atlas kit: 4 corners, 4 tiled edges, optional stretched center.
pub fn emit_nine_slice_atlas(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    ns: &NineSliceAtlasInfo,
    alpha: f32,
) {
    emit_corners(batch, bounds, ns, alpha);
    emit_horiz_edges(batch, bounds, ns, alpha);
    emit_vert_edges(batch, bounds, ns, alpha);

    if let Some(ref center) = ns.center {
        let cx = bounds.x + ns.corner_tl.width as f32;
        let cy = bounds.y + ns.corner_tl.height as f32;
        let cw = bounds.width - ns.corner_tl.width as f32 - ns.corner_tr.width as f32;
        let ch = bounds.height - ns.corner_tl.height as f32 - ns.corner_bl.height as f32;
        if cw > 0.0 && ch > 0.0 {
            emit_piece(batch, Rectangle::new(Point::new(cx, cy), Size::new(cw, ch)), center, alpha);
        }
    }
}

/// Emit a nine-slice border (corners + edges) with a solid-color center fill.
///
/// Used for tooltips where `SetCenterColor` tints the center texture to a solid color.
/// `center_overlap` extends the center fill into the corners by the given number of pixels
/// (WoW's TooltipDefaultLayout uses 4px via anchor offsets `x=-4, y=4, x1=4, y1=-4`).
pub fn emit_nine_slice_with_center_color(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    ns: &NineSliceAtlasInfo,
    alpha: f32,
    center_color: [f32; 4],
    center_overlap: f32,
) {
    // Solid center fill (drawn first, behind the border pieces).
    // WoW anchors the center from TopLeftCorner.BOTTOMRIGHT to BottomRightCorner.TOPLEFT
    // with negative insets, so the fill extends under the corners to plug transparent areas.
    let cx = bounds.x + ns.corner_tl.width as f32 - center_overlap;
    let cy = bounds.y + ns.corner_tl.height as f32 - center_overlap;
    let cw = bounds.width - ns.corner_tl.width as f32 - ns.corner_tr.width as f32 + center_overlap * 2.0;
    let ch = bounds.height - ns.corner_tl.height as f32 - ns.corner_bl.height as f32 + center_overlap * 2.0;
    if cw > 0.0 && ch > 0.0 {
        batch.push_solid(Rectangle::new(Point::new(cx, cy), Size::new(cw, ch)), center_color);
    }

    emit_corners(batch, bounds, ns, alpha);
    emit_horiz_edges(batch, bounds, ns, alpha);
    emit_vert_edges(batch, bounds, ns, alpha);
}
