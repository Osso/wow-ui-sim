use iced::{Point, Rectangle, Size};
use std::borrow::Cow;

use crate::atlas::{AtlasSliceMode, get_atlas_slice_info};
use crate::render::{BlendMode, QuadBatch};

use super::super::slice_render::{
    StretchSliceRender, TextureUvs, TexturedSlice, ThreeSliceRender, TileSliceRender,
    emit_stretch_slice_atlas, emit_three_slice_h_atlas, emit_tile_slice_atlas,
};
use super::super::statusbar::StatusBarFill;
use super::super::tiling::{emit_tiled_texture, has_uv_repeat};

/// Build quads for a Texture widget, optionally clipped by a StatusBar fill.
pub(crate) fn build_texture_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    bar_fill: Option<&StatusBarFill>,
    alpha: f32,
) {
    if let Some(ref ns) = f.nine_slice_atlas {
        super::super::nine_slice::emit_nine_slice_atlas(batch, bounds, ns, alpha);
        return;
    }

    let tint = resolve_tint(f, bar_fill, alpha);

    if let Some(color) = f.color_texture {
        let fill_bounds = apply_bar_fill(bounds, bar_fill);
        if let Some(ref grad) = f.gradient {
            push_gradient_quad(batch, fill_bounds, color, grad, tint);
        } else {
            batch.push_solid(
                fill_bounds,
                [
                    color.r * tint[0],
                    color.g * tint[1],
                    color.b * tint[2],
                    color.a * alpha,
                ],
            );
        }
        return;
    }

    let Some(tex_path) = resolve_texture_path(f) else {
        emit_bar_fill_fallback(batch, bar_fill, bounds, alpha);
        return;
    };
    emit_textured_quad(batch, bounds, f, bar_fill, tex_path.as_ref(), tint, alpha);
}

fn resolve_texture_path(f: &crate::widget::Frame) -> Option<Cow<'_, str>> {
    if let Some(path) = f.texture.as_deref() {
        return Some(Cow::Borrowed(path));
    }

    let file_data_id = u32::try_from(f.texture_file_data_id?).ok()?;
    let path = crate::manifest_interface_data::get_texture_path(file_data_id)?;
    Some(Cow::Owned(format!(
        "Interface\\{}",
        path.replace('/', "\\")
    )))
}

/// Compute the vertex color tint from vertex_color and bar fill override.
fn resolve_tint(
    f: &crate::widget::Frame,
    bar_fill: Option<&StatusBarFill>,
    alpha: f32,
) -> [f32; 4] {
    if let Some(fill) = bar_fill
        && let Some(c) = &fill.color
    {
        return [c.r, c.g, c.b, c.a * alpha];
    }
    let vc = f.vertex_color.as_ref();
    [
        vc.map_or(1.0, |c| c.r),
        vc.map_or(1.0, |c| c.g),
        vc.map_or(1.0, |c| c.b),
        vc.map_or(1.0, |c| c.a) * alpha,
    ]
}

/// Emit a solid color quad when no texture path exists but a bar fill has a color.
fn emit_bar_fill_fallback(
    batch: &mut QuadBatch,
    bar_fill: Option<&StatusBarFill>,
    bounds: Rectangle,
    alpha: f32,
) {
    if let Some(fill) = bar_fill
        && let Some(c) = &fill.color
    {
        let fill_bounds = apply_bar_fill(bounds, bar_fill);
        batch.push_solid(fill_bounds, [c.r, c.g, c.b, c.a * alpha]);
    }
}

/// Emit a gradient quad with per-vertex colors (VERTICAL or HORIZONTAL).
fn push_gradient_quad(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    base: crate::widget::Color,
    grad: &crate::widget::Gradient,
    tint: [f32; 4],
) {
    let min = tinted_gradient_color(base, grad.min_color, tint);
    let max = tinted_gradient_color(base, grad.max_color, tint);
    let (top_color, bottom_color) = if grad.vertical {
        (max, min)
    } else {
        (min, min)
    };
    let colors = if grad.vertical {
        [top_color, top_color, bottom_color, bottom_color]
    } else {
        let right = max;
        let left = min;
        [left, right, right, left]
    };
    batch.push_gradient(bounds, colors);
}

fn tinted_gradient_color(
    base: crate::widget::Color,
    stop: crate::widget::Color,
    tint: [f32; 4],
) -> [f32; 4] {
    [
        base.r * stop.r * tint[0],
        base.g * stop.g * tint[1],
        base.b * stop.b * tint[2],
        base.a * stop.a * tint[3],
    ]
}

/// Emit a textured quad with atlas cropping, three-slice, tiling, rotation, desaturation.
fn emit_textured_quad(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    bar_fill: Option<&StatusBarFill>,
    tex_path: &str,
    tint: [f32; 4],
    alpha: f32,
) {
    if bar_fill.is_none()
        && let Some(uv4) = rotated_quad_uvs(f)
    {
        let (effective_path, effective_uv4) =
            remap_atlas_crop_uv4(tex_path, uv4, f.atlas_tex_coords);
        let vert_before = batch.vertices.len();
        batch.push_textured_path_uv4(bounds, effective_uv4, &effective_path, tint, f.blend_mode);
        finalize_textured_quad(batch, vert_before, f);
        return;
    }

    let (fill_bounds, fill_uvs) = apply_bar_fill_with_uvs(bounds, f.tex_coords, bar_fill);
    let (effective_path, effective_uvs) = remap_atlas_crop(tex_path, fill_uvs, f.atlas_tex_coords);
    let vert_before = batch.vertices.len();
    emit_texture_fill(
        batch,
        fill_bounds,
        effective_uvs,
        &effective_path,
        f,
        tint,
        alpha,
    );
    finalize_textured_quad(batch, vert_before, f);
}

/// Returns the 4 corner UVs (TL, TR, BR, BL) when the frame has an 8-arg
/// SetTexCoord that can't be represented as an axis-aligned rect. Excludes
/// UV-repeat tiling (any value > 1.0) which is handled separately.
fn rotated_quad_uvs(f: &crate::widget::Frame) -> Option<[[f32; 2]; 4]> {
    let raw = f.tex_coords_quad?;
    if raw.iter().any(|&v| v > 1.0) {
        return None;
    }
    let tl = [raw[0], raw[1]];
    let bl = [raw[2], raw[3]];
    let tr = [raw[4], raw[5]];
    let br = [raw[6], raw[7]];
    let axis_aligned = (tl[0] - bl[0]).abs() < f32::EPSILON
        && (tr[0] - br[0]).abs() < f32::EPSILON
        && (tl[1] - tr[1]).abs() < f32::EPSILON
        && (bl[1] - br[1]).abs() < f32::EPSILON;
    if axis_aligned {
        return None;
    }
    Some([tl, tr, br, bl])
}

/// Apply atlas-slot cropping to 4-corner UVs. Returns the rewritten path
/// (with `@crop:` key when the texture is a sub-region) and corner UVs in
/// [0,1] of the slot's local space.
fn remap_atlas_crop_uv4(
    tex_path: &str,
    uv4: [[f32; 2]; 4],
    atlas_tex_coords: Option<TextureUvs>,
) -> (String, [[f32; 2]; 4]) {
    let Some((cl, cr, ct, cb)) = atlas_tex_coords else {
        return (tex_path.to_string(), uv4);
    };
    let is_full = (cl - 0.0).abs() < 0.001
        && (cr - 1.0).abs() < 0.001
        && (ct - 0.0).abs() < 0.001
        && (cb - 1.0).abs() < 0.001;
    if is_full {
        return (tex_path.to_string(), uv4);
    }
    let crop_key = format!("{tex_path}@crop:{cl:.6},{cr:.6},{ct:.6},{cb:.6}");
    let cw = cr - cl;
    let ch = cb - ct;
    if cw <= 0.0 || ch <= 0.0 {
        return (crop_key, [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    }
    let remapped = uv4.map(|[u, v]| [(u - cl) / cw, (v - ct) / ch]);
    (crop_key, remapped)
}

fn emit_texture_fill(
    batch: &mut QuadBatch,
    fill_bounds: Rectangle,
    effective_uvs: Option<TextureUvs>,
    effective_path: &str,
    f: &crate::widget::Frame,
    tint: [f32; 4],
    alpha: f32,
) {
    let Some(uvs) = effective_uvs else {
        batch.push_textured_path(fill_bounds, &effective_path, tint, f.blend_mode);
        return;
    };

    let texture = TexturedSlice {
        path: effective_path,
        uvs,
        tint,
        blend: f.blend_mode,
    };

    if emit_specialized_textured_fill(batch, fill_bounds, f, texture) {
        return;
    }

    emit_basic_textured_fill(batch, fill_bounds, texture, f, alpha);
}

fn emit_specialized_textured_fill(
    batch: &mut QuadBatch,
    fill_bounds: Rectangle,
    f: &crate::widget::Frame,
    texture: TexturedSlice<'_>,
) -> bool {
    if let Some(render) = stretch_slice_render(f, fill_bounds, texture) {
        emit_stretch_slice_atlas(batch, render);
        return true;
    }

    if let Some(render) = tile_slice_render(f, fill_bounds, texture) {
        emit_tile_slice_atlas(batch, render);
        return true;
    }

    if let Some(render) = three_slice_render(f, fill_bounds, texture) {
        emit_three_slice_h_atlas(batch, render);
        return true;
    }

    false
}

fn stretch_slice_render<'a>(
    f: &crate::widget::Frame,
    bounds: Rectangle,
    texture: TexturedSlice<'a>,
) -> Option<StretchSliceRender<'a>> {
    let atlas_name = f.atlas.as_deref()?;
    let slice = get_atlas_slice_info(atlas_name)?;
    if slice.mode != AtlasSliceMode::Stretch {
        return None;
    }

    if bounds.width <= (slice.left + slice.right) as f32
        || bounds.height <= (slice.top + slice.bottom) as f32
    {
        return None;
    }

    let atlas_info = crate::atlas::get_atlas_info(atlas_name)?;
    Some(StretchSliceRender {
        bounds,
        texture,
        left_px: slice.left as f32,
        top_px: slice.top as f32,
        right_px: slice.right as f32,
        bottom_px: slice.bottom as f32,
        atlas_width_px: atlas_info.width() as f32,
        atlas_height_px: atlas_info.height() as f32,
    })
}

fn tile_slice_render<'a>(
    f: &crate::widget::Frame,
    bounds: Rectangle,
    texture: TexturedSlice<'a>,
) -> Option<TileSliceRender<'a>> {
    let atlas_name = f.atlas.as_deref()?;
    let slice = get_atlas_slice_info(atlas_name)?;
    if slice.mode != AtlasSliceMode::Tile {
        return None;
    }

    if bounds.width < (slice.left + slice.right) as f32
        || bounds.height < (slice.top + slice.bottom) as f32
    {
        return None;
    }

    let atlas_info = crate::atlas::get_atlas_info(atlas_name)?;
    Some(TileSliceRender {
        bounds,
        texture,
        left_px: slice.left as f32,
        top_px: slice.top as f32,
        right_px: slice.right as f32,
        bottom_px: slice.bottom as f32,
        atlas_width_px: atlas_info.width() as f32,
        atlas_height_px: atlas_info.height() as f32,
    })
}

fn three_slice_render<'a>(
    f: &crate::widget::Frame,
    bounds: Rectangle,
    texture: TexturedSlice<'a>,
) -> Option<ThreeSliceRender<'a>> {
    let (left_cap_px, right_cap_px, atlas_width_px) = f.three_slice_h?;
    if bounds.width <= left_cap_px + right_cap_px {
        return None;
    }

    Some(ThreeSliceRender {
        bounds,
        texture,
        left_cap_px,
        right_cap_px,
        atlas_width_px,
    })
}

fn emit_basic_textured_fill(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    texture: TexturedSlice<'_>,
    f: &crate::widget::Frame,
    alpha: f32,
) {
    let uvs = uv_rect(texture.uvs);
    if f.horiz_tile || f.vert_tile || has_uv_repeat(f) {
        emit_tiled_texture(batch, bounds, &uvs, texture.path, f, alpha);
        return;
    }

    batch.push_textured_path_uv(bounds, uvs, texture.path, texture.tint, texture.blend);
}

fn uv_rect((left, right, top, bottom): TextureUvs) -> Rectangle {
    Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top))
}

fn finalize_textured_quad(batch: &mut QuadBatch, vert_before: usize, f: &crate::widget::Frame) {
    if f.rotation != 0.0 {
        apply_uv_rotation(batch, vert_before, f.rotation);
    }
    if f.desaturated {
        apply_desaturate_flag(batch, vert_before);
    }
}

/// Apply StatusBar fill clipping to bounds.
fn apply_bar_fill(bounds: Rectangle, bar_fill: Option<&StatusBarFill>) -> Rectangle {
    let Some(fill) = bar_fill else { return bounds };
    let fill_width = bounds.width * fill.fraction;
    if fill.reverse {
        Rectangle::new(
            Point::new(bounds.x + bounds.width - fill_width, bounds.y),
            Size::new(fill_width, bounds.height),
        )
    } else {
        Rectangle::new(bounds.position(), Size::new(fill_width, bounds.height))
    }
}

/// Remap atlas sub-region textures: encode crop coords in path, remap UVs to [0,1].
pub(super) fn remap_atlas_crop(
    tex_path: &str,
    fill_uvs: Option<TextureUvs>,
    atlas_tex_coords: Option<TextureUvs>,
) -> (String, Option<TextureUvs>) {
    let Some((cl, cr, ct, cb)) = atlas_tex_coords else {
        return (tex_path.to_string(), fill_uvs);
    };
    let is_full = (cl - 0.0).abs() < 0.001
        && (cr - 1.0).abs() < 0.001
        && (ct - 0.0).abs() < 0.001
        && (cb - 1.0).abs() < 0.001;
    if is_full {
        return (tex_path.to_string(), fill_uvs);
    }

    let crop_key = format!("{tex_path}@crop:{cl:.6},{cr:.6},{ct:.6},{cb:.6}");
    let remapped_uvs = fill_uvs.map(|(fl, fr, ft, fb)| {
        let cw = cr - cl;
        let ch = cb - ct;
        if cw <= 0.0 || ch <= 0.0 {
            return (0.0, 1.0, 0.0, 1.0);
        }
        (
            (fl - cl) / cw,
            (fr - cl) / cw,
            (ft - ct) / ch,
            (fb - ct) / ch,
        )
    });

    (crop_key, remapped_uvs)
}

/// Apply StatusBar fill clipping to bounds and UV coordinates.
fn apply_bar_fill_with_uvs(
    bounds: Rectangle,
    tex_coords: Option<TextureUvs>,
    bar_fill: Option<&StatusBarFill>,
) -> (Rectangle, Option<TextureUvs>) {
    let Some(fill) = bar_fill else {
        return (bounds, tex_coords);
    };
    let fill_bounds = apply_bar_fill(bounds, bar_fill);
    let (uv_left, uv_right, uv_top, uv_bottom) = tex_coords.unwrap_or((0.0, 1.0, 0.0, 1.0));
    let uv_range = uv_right - uv_left;
    let fill_uvs = if fill.reverse {
        (
            uv_left + uv_range * (1.0 - fill.fraction),
            uv_right,
            uv_top,
            uv_bottom,
        )
    } else {
        (
            uv_left,
            uv_left + uv_range * fill.fraction,
            uv_top,
            uv_bottom,
        )
    };
    (fill_bounds, Some(fill_uvs))
}

/// Rotate texture UV coordinates around their center for vertices added after `vert_before`.
fn apply_uv_rotation(batch: &mut QuadBatch, vert_before: usize, radians: f32) {
    let verts = &mut batch.vertices[vert_before..];
    if verts.len() < 4 {
        return;
    }
    let (sin_r, cos_r) = radians.sin_cos();
    for chunk in verts.chunks_exact_mut(4) {
        let cx = (chunk[0].tex_coords[0]
            + chunk[1].tex_coords[0]
            + chunk[2].tex_coords[0]
            + chunk[3].tex_coords[0])
            * 0.25;
        let cy = (chunk[0].tex_coords[1]
            + chunk[1].tex_coords[1]
            + chunk[2].tex_coords[1]
            + chunk[3].tex_coords[1])
            * 0.25;
        for v in chunk.iter_mut() {
            let du = v.tex_coords[0] - cx;
            let dv = v.tex_coords[1] - cy;
            v.tex_coords[0] = cx + du * cos_r - dv * sin_r;
            v.tex_coords[1] = cy + du * sin_r + dv * cos_r;
        }
    }
}

/// Apply the desaturation flag to vertices added after `vert_before`.
fn apply_desaturate_flag(batch: &mut QuadBatch, vert_before: usize) {
    use crate::render::shader::FLAG_DESATURATE;
    for v in &mut batch.vertices[vert_before..] {
        v.flags |= FLAG_DESATURATE;
    }
}

const DEFAULT_MINIMAP_MASK_TEXTURE: &str = r"Interface\HUD\UIMinimapMask";

const DEFAULT_MINIMAP_MASK_TEXTURE_2X: &str = r"Interface\HUD\UIMinimapMask2x";

/// The Minimap frame is 198 units (Minimap.xml) and the built-in mask is a
/// 256-texel canvas (BLP2 uncompressed BGRA, alpha uniformly 255, coverage
/// in RGB) whose opaque disc is a circle of r = 96.7 texels centred on the
/// canvas, with four compass notches and the outline of the N chevron
/// around it. Decoding the compass frame art gives the ring's inner edge at
/// r = 95.9 units and the client capture shows the terrain reaching that
/// edge everywhere, so the client maps one mask texel to one UI unit,
/// centred on the frame: the disc then ends within a pixel of the ring and
/// the notch interiors sit under the ring's cusps and chevron. Stretching
/// the disc's bounding box (203x210, notches included) over the frame,
/// as before, squashed the disc into a 94x91-unit ellipse 5 px low and
/// left a dark gap of 5-9 px inside the ring's NW half and the N notch's
/// black interior 6 px inside the ring at the top.
const MINIMAP_MASK_CANVAS: f32 = 256.0;
const MINIMAP_FRAME_UNITS: f32 = 198.0;

/// Screen rectangle to draw the built-in minimap mask into: the canvas at
/// one texel per UI unit, centred on `bounds`.
fn default_minimap_mask_rect(bounds: Rectangle) -> Rectangle {
    let sx = bounds.width / MINIMAP_FRAME_UNITS;
    let sy = bounds.height / MINIMAP_FRAME_UNITS;
    let width = MINIMAP_MASK_CANVAS * sx;
    let height = MINIMAP_MASK_CANVAS * sy;
    Rectangle::new(
        iced::Point::new(
            bounds.x + (bounds.width - width) / 2.0,
            bounds.y + (bounds.height - height) / 2.0,
        ),
        iced::Size::new(width, height),
    )
}

/// The 2x mask (512 texels, the same disc) once the render draws the 2x
/// compass art.
fn default_minimap_mask_texture() -> &'static str {
    if crate::atlas::prefer_hires_atlases() {
        DEFAULT_MINIMAP_MASK_TEXTURE_2X
    } else {
        DEFAULT_MINIMAP_MASK_TEXTURE
    }
}

/// Build quads for a Minimap widget - map texture clipped by the active minimap mask.
pub(crate) fn build_minimap_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    alpha: f32,
) {
    let vert_before = batch.vertices.len();
    batch.push_textured_path(
        bounds,
        r"Interface\AddOns\SimCommands\textures\minimap-placeholder",
        [1.0, 1.0, 1.0, alpha],
        BlendMode::Alpha,
    );
    match f.minimap_mask_texture.as_deref() {
        // An addon-supplied mask is a MaskTexture stretched over the frame,
        // which is what the plain path does.
        Some(mask_texture) => {
            crate::iced_app::masking::apply_mask_path(batch, vert_before, bounds, mask_texture)
        }
        None => crate::iced_app::masking::apply_mask_path_with_rect(
            batch,
            vert_before,
            default_minimap_mask_rect(bounds),
            default_minimap_mask_texture(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BlendMode, DEFAULT_MINIMAP_MASK_TEXTURE, TexturedSlice, build_minimap_quads,
        build_texture_quads, emit_texture_fill, remap_atlas_crop, stretch_slice_render,
        tile_slice_render,
    };
    use crate::atlas::get_render_atlas_info;
    use crate::iced_app::slice_render::{tile_slice_center_height, tile_slice_center_width};
    use crate::render::QuadBatch;
    use crate::widget::{Color, Frame, Gradient, WidgetType};
    use iced::{Point, Rectangle, Size};

    fn texture_frame_with_atlas(name: &str) -> Frame {
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.atlas = Some(name.to_string());
        frame
    }

    fn render_texture_slice(name: &str) -> TexturedSlice<'static> {
        let lookup = get_render_atlas_info(name)
            .unwrap_or_else(|| panic!("missing render atlas info for {name}"));
        TexturedSlice {
            path: lookup.info.file,
            uvs: (
                lookup.info.left_tex_coord,
                lookup.info.right_tex_coord,
                lookup.info.top_tex_coord,
                lookup.info.bottom_tex_coord,
            ),
            tint: [1.0, 1.0, 1.0, 1.0],
            blend: BlendMode::Alpha,
        }
    }

    #[test]
    fn minimap_uses_default_mask_texture() {
        let mut batch = QuadBatch::new();
        let frame = Frame::new(WidgetType::Minimap, Some("Minimap".to_string()), None);
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(140.0, 140.0));

        build_minimap_quads(&mut batch, bounds, &frame, 1.0);

        assert_eq!(batch.mask_texture_requests.len(), 1);
        assert_eq!(
            batch.mask_texture_requests[0].path,
            DEFAULT_MINIMAP_MASK_TEXTURE
        );
        assert!(
            batch
                .vertices
                .iter()
                .all(|vertex| vertex.mask_tex_index == -2)
        );
    }

    /// The built-in mask is a 256-texel canvas whose disc (r = 96.7 texels,
    /// centred) coincides with the compass ring's inner edge when one texel
    /// is one UI unit centred on the 198-unit frame: the quad's mask UVs are
    /// the central 198/256 of the canvas on both axes and the canvas centre
    /// lands on the frame centre. Stretching the disc's bounding box (which
    /// includes the compass notches) over the frame squashed the disc into
    /// a 94x91-unit ellipse 5 px low and left a dark gap inside the ring.
    #[test]
    fn default_minimap_mask_maps_one_texel_to_one_unit_about_the_frame_centre() {
        let mut batch = QuadBatch::new();
        let frame = Frame::new(WidgetType::Minimap, Some("Minimap".to_string()), None);
        let bounds = Rectangle::new(Point::new(10.0, 20.0), Size::new(198.0, 198.0));

        build_minimap_quads(&mut batch, bounds, &frame, 1.0);

        let us: Vec<f32> = batch
            .vertices
            .iter()
            .map(|v| v.mask_tex_coords[0])
            .collect();
        let vs: Vec<f32> = batch
            .vertices
            .iter()
            .map(|v| v.mask_tex_coords[1])
            .collect();
        let (umin, umax) = (
            us.iter().cloned().fold(1.0, f32::min),
            us.iter().cloned().fold(0.0, f32::max),
        );
        let (vmin, vmax) = (
            vs.iter().cloned().fold(1.0, f32::min),
            vs.iter().cloned().fold(0.0, f32::max),
        );
        let close = |a: f32, b: f32| (a - b).abs() < 0.002;
        assert!(
            close(umin, 29.0 / 256.0) && close(umax, 227.0 / 256.0),
            "mask U range {umin}..{umax} should be the central 198 of 256 texels"
        );
        assert!(
            close(vmin, 29.0 / 256.0) && close(vmax, 227.0 / 256.0),
            "mask V range {vmin}..{vmax} should be the central 198 of 256 texels"
        );
        assert!(
            close((umin + umax) / 2.0, 0.5) && close((vmin + vmax) / 2.0, 0.5),
            "the canvas centre must land on the frame centre"
        );
    }

    #[test]
    fn minimap_respects_set_mask_texture_state() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Minimap, Some("Minimap".to_string()), None);
        frame.minimap_mask_texture = Some(r"Interface\CharacterFrame\TempPortraitAlphaMask".into());
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(140.0, 140.0));

        build_minimap_quads(&mut batch, bounds, &frame, 1.0);

        assert_eq!(batch.mask_texture_requests.len(), 1);
        assert_eq!(
            batch.mask_texture_requests[0].path,
            r"Interface\CharacterFrame\TempPortraitAlphaMask"
        );
    }

    #[test]
    fn color_texture_gradient_tints_stops_with_base_color() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.color_texture = Some(Color::new(0.306, 0.133, 0.031, 0.5));
        frame.gradient = Some(Gradient {
            vertical: true,
            min_color: Color::new(1.0, 1.0, 1.0, 0.0),
            max_color: Color::new(1.0, 1.0, 1.0, 0.8),
        });

        build_texture_quads(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(430.0, 200.0)),
            &frame,
            None,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(batch.vertices[0].color, [0.306, 0.133, 0.031, 0.4]);
        assert_eq!(batch.vertices[2].color, [0.306, 0.133, 0.031, 0.0]);
    }

    #[test]
    fn remap_atlas_crop_rewrites_subregion_to_crop_key() {
        let (path, uvs) = remap_atlas_crop(
            r"Interface\Glues\CharacterSelect\Glues-AddOn-Icons",
            Some((0.25, 0.5, 0.125, 0.625)),
            Some((0.25, 0.5, 0.125, 0.625)),
        );

        assert_eq!(
            path,
            r"Interface\Glues\CharacterSelect\Glues-AddOn-Icons@crop:0.250000,0.500000,0.125000,0.625000"
        );
        assert_eq!(uvs, Some((0.0, 1.0, 0.0, 1.0)));
    }

    #[test]
    fn stretch_atlas_slices_emit_nine_quads() {
        let mut batch = QuadBatch::new();
        let frame = texture_frame_with_atlas("common-button-tertiary-normal");

        emit_texture_fill(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(160.0, 32.0)),
            Some((0.0, 1.0, 0.0, 1.0)),
            "stretch-path",
            &frame,
            [1.0, 1.0, 1.0, 1.0],
            1.0,
        );

        assert_eq!(batch.vertices.len(), 36);
        assert_eq!(batch.texture_requests.len(), 9);
    }

    #[test]
    fn tile_atlas_slices_collapse_unit_repeat_regions() {
        let mut batch = QuadBatch::new();
        let frame = texture_frame_with_atlas("questlog-frame");

        emit_texture_fill(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(120.0, 120.0)),
            Some((0.0, 1.0, 0.0, 1.0)),
            "tile-path@crop:0.001953,0.210938,0.076172,0.285156",
            &frame,
            [1.0, 1.0, 1.0, 1.0],
            1.0,
        );

        assert_eq!(batch.vertices.len(), 36);
        assert_eq!(batch.texture_requests.len(), 9);
        assert!(
            batch
                .texture_requests
                .iter()
                .all(|request| request.path.matches("@crop:").count() == 1),
            "tile atlas slices should flatten crop paths, got: {:?}",
            batch.texture_requests
        );
    }

    #[test]
    fn tile_slice_render_uses_logical_dimensions_for_2x_fallback_atlas() {
        let frame = texture_frame_with_atlas("questlog-frame");
        let texture = render_texture_slice("questlog-frame");
        let render = tile_slice_render(
            &frame,
            Rectangle::new(Point::ORIGIN, Size::new(314.0, 436.0)),
            texture,
        )
        .expect("questlog-frame should use tile slice rendering");

        assert_eq!(render.atlas_width_px, 107.0);
        assert_eq!(render.atlas_height_px, 107.0);
        assert_eq!(tile_slice_center_width(render), Some((208.0, 1.0)));
        assert_eq!(tile_slice_center_height(render), Some((330.0, 1.0)));
    }

    #[test]
    fn uv_repeat_texcoords_emit_tiled_quads_without_tile_flags() {
        let mut batch = QuadBatch::new();
        let mut frame = Frame::new(WidgetType::Texture, None, None);
        frame.texture = Some(r"Interface\AddOns\Details\images\background".to_string());
        frame.tex_coords = Some((0.0, 2.109, 0.0, 0.872));
        frame.tex_coords_quad = Some([0.0, 0.0, 0.0, 0.872, 2.109, 0.0, 2.109, 0.872]);

        build_texture_quads(
            &mut batch,
            Rectangle::new(Point::ORIGIN, Size::new(270.0, 112.0)),
            &frame,
            None,
            1.0,
        );

        assert_eq!(batch.vertices.len(), 12);
        assert_eq!(batch.texture_requests.len(), 3);
        assert!(
            batch
                .vertices
                .iter()
                .all(|vertex| vertex.local_uv[0] <= 1.0 && vertex.local_uv[1] <= 1.0),
            "UV-repeat quads must not sample beyond their atlas slot: {:?}",
            batch.vertices
        );
        assert_eq!(batch.vertices[1].position[0], 128.02277);
        assert_eq!(batch.vertices[5].position[0], 256.04553);
        assert_eq!(batch.vertices[9].position[0], 270.0);
    }

    #[test]
    fn stretch_slice_render_uses_logical_dimensions_for_2x_fallback_atlas() {
        let frame = texture_frame_with_atlas("common-button-tertiary-normal");
        let texture = render_texture_slice("common-button-tertiary-normal");
        let render = stretch_slice_render(
            &frame,
            Rectangle::new(Point::ORIGIN, Size::new(160.0, 32.0)),
            texture,
        )
        .expect("common-button-tertiary-normal should use stretch slice rendering");

        assert_eq!(render.atlas_width_px, 46.0);
        assert_eq!(render.atlas_height_px, 34.0);
    }
}
