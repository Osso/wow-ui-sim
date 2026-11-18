//! Mask texture UV computation and application.

use iced::Rectangle;

use crate::render::texture::UI_SCALE;
use crate::render::{QuadBatch, TextureRequest};

/// Apply mask texture to recently emitted quads by setting mask_tex_index/mask_tex_coords
/// for GPU alpha sampling (resolved to atlas coords during prepare).
///
/// Uses the mask frame's computed layout bounds to determine the UV mapping. The mask
/// texture is stretched to fill the mask's layout area; the icon UVs are computed based
/// on where the icon falls within that area. A mask with slice margins
/// (`SetTextureSliceMargins`, stretched mode) keeps its margins at their texel size,
/// as the client draws it: each masked quad is split at the slice lines first.
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
    // A sliced mask has split the quads: the request covers the new ones.
    let count = batch.vertices.len() - vert_before;
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
        slice: None,
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
    slice: Option<MaskSlice>,
}

/// Slice margins of a mask texture in texels of its source, which are also
/// their size on screen (times `UI_SCALE`), and the source size in texels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct MaskSlice {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    width: f32,
    height: f32,
}

const TEXTURE_SLICE_MODE_STRETCHED: i32 = 0;

/// The slice layout of a mask frame, when it has margins in the stretched
/// mode, its source size is known and both the source and the mask's screen
/// rect leave a centre between the margins. Otherwise the mask stretches whole.
fn mask_slice(mask_frame: &crate::widget::Frame, mask_screen: Rectangle) -> Option<MaskSlice> {
    let (left, right, top, bottom) = mask_frame.texture_slice_margins;
    let margins = [left, right, top, bottom];
    if margins.iter().all(|m| *m == 0.0)
        || margins.iter().any(|m| !m.is_finite() || *m < 0.0)
        || mask_frame.texture_slice_mode != TEXTURE_SLICE_MODE_STRETCHED
        || mask_frame.tex_coords.is_some()
    {
        return None;
    }
    let (width, height) = mask_frame.texture_source_dimensions?;
    let (width, height) = (width as f32, height as f32);
    let source_has_center = width > left + right && height > top + bottom;
    let screen_has_center = mask_screen.width > (left + right) * UI_SCALE
        && mask_screen.height > (top + bottom) * UI_SCALE;
    (source_has_center && screen_has_center).then_some(MaskSlice {
        left,
        right,
        top,
        bottom,
        width,
        height,
    })
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
    let mask_rect =
        crate::layout::apply_scroll_offsets(registry, mask_frame.id, mask_frame.layout_rect?);
    let mask_screen = mask_to_screen_rect(mask_rect);
    if !rects_overlap(icon_bounds, mask_screen) {
        truncate_masked_vertices(batch, vert_before);
        return None;
    }
    Some(MaskInfo {
        path,
        screen_rect: mask_screen,
        tex_coords: mask_frame.tex_coords.unwrap_or((0.0, 1.0, 0.0, 1.0)),
        slice: mask_slice(mask_frame, mask_screen),
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
    if let Some(slice) = mask_info.slice {
        apply_sliced_mask_to_quads(batch, vert_before, mask_info.screen_rect, slice);
        return;
    }
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

/// Splits each quad from `vert_before` on at the mask's slice lines, so the
/// mask UVs, interpolated linearly per quad, follow the piecewise mapping of
/// the nine slices. Texture requests of those quads move with their vertices.
fn apply_sliced_mask_to_quads(
    batch: &mut QuadBatch,
    vert_before: usize,
    mask_screen: Rectangle,
    slice: MaskSlice,
) {
    let cells = slice_cells(mask_screen, slice);
    let mut out = Vec::with_capacity(batch.vertices.len() - vert_before);
    let mut starts = Vec::new();
    for quad in batch.vertices[vert_before..].as_chunks::<4>().0 {
        starts.push(out.len());
        append_sliced_quad(*quad, &cells, mask_screen, slice, &mut out);
    }
    starts.push(out.len());
    remap_tail_requests(&mut batch.texture_requests, vert_before, &starts);
    replace_quad_tail(batch, vert_before, out.as_chunks::<4>().0);
}

fn append_sliced_quad(
    quad: [crate::render::shader::QuadVertex; 4],
    cells: &[Rectangle],
    mask_screen: Rectangle,
    slice: MaskSlice,
    out: &mut Vec<crate::render::shader::QuadVertex>,
) {
    let bounds = quad_rect(&quad);
    let before = out.len();
    for cell in cells {
        let Some(clipped) = rect_intersection(bounds, *cell) else {
            continue;
        };
        let mut piece = quad;
        clip_quad_to_rect(&mut piece, bounds, clipped);
        for (vertex, uv) in piece
            .iter_mut()
            .zip(sliced_mask_uvs(mask_screen, slice, clipped))
        {
            vertex.mask_tex_index = -2;
            vertex.mask_tex_coords = uv;
        }
        out.extend(piece);
    }
    if out.len() == before {
        let mut hidden = quad;
        hide_quad(&mut hidden);
        out.extend(hidden);
    }
}

fn replace_quad_tail(
    batch: &mut QuadBatch,
    vert_before: usize,
    replacement: &[[crate::render::shader::QuadVertex; 4]],
) {
    batch.vertices.truncate(vert_before);
    batch.indices.truncate(vert_before / 4 * 6);
    for (index, piece) in replacement.iter().enumerate() {
        let base = (vert_before + index * 4) as u32;
        batch.vertices.extend_from_slice(piece);
        batch
            .indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// The nine slice rects of the mask on screen, empty ones left out.
fn slice_cells(mask: Rectangle, slice: MaskSlice) -> Vec<Rectangle> {
    let xs = [
        mask.x,
        mask.x + slice.left * UI_SCALE,
        mask.x + mask.width - slice.right * UI_SCALE,
        mask.x + mask.width,
    ];
    let ys = [
        mask.y,
        mask.y + slice.top * UI_SCALE,
        mask.y + mask.height - slice.bottom * UI_SCALE,
        mask.y + mask.height,
    ];
    let mut cells = Vec::with_capacity(9);
    for row in ys.windows(2) {
        for col in xs.windows(2) {
            if col[1] > col[0] && row[1] > row[0] {
                cells.push(Rectangle::new(
                    iced::Point::new(col[0], row[0]),
                    iced::Size::new(col[1] - col[0], row[1] - row[0]),
                ));
            }
        }
    }
    cells
}

/// Mask UVs of a rect that lies within one slice cell.
fn sliced_mask_uvs(mask: Rectangle, slice: MaskSlice, rect: Rectangle) -> [[f32; 2]; 4] {
    let u = |x: f32| {
        sliced_texel(x - mask.x, mask.width, slice.left, slice.right, slice.width) / slice.width
    };
    let v = |y: f32| {
        sliced_texel(
            y - mask.y,
            mask.height,
            slice.top,
            slice.bottom,
            slice.height,
        ) / slice.height
    };
    let (u0, u1) = (u(rect.x), u(rect.x + rect.width));
    let (v0, v1) = (v(rect.y), v(rect.y + rect.height));
    [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
}

/// The source texel at `offset` screen pixels into a span of `span` pixels:
/// the margins map one texel to `UI_SCALE` pixels, the centre stretches.
fn sliced_texel(offset: f32, span: f32, near: f32, far: f32, source: f32) -> f32 {
    let (near_px, far_px) = (near * UI_SCALE, far * UI_SCALE);
    let texel = if offset <= near_px {
        offset / UI_SCALE
    } else if offset >= span - far_px {
        source - (span - offset) / UI_SCALE
    } else {
        near + (offset - near_px) * (source - near - far) / (span - near_px - far_px)
    };
    texel.clamp(0.0, source)
}

/// Moves the texture requests at or after `vert_before` onto the quads that
/// replaced theirs: `starts[i]` is where old quad i begins in the new tail,
/// the last entry its end.
fn remap_tail_requests(requests: &mut [TextureRequest], vert_before: usize, starts: &[usize]) {
    let quads = starts.len() - 1;
    for request in requests
        .iter_mut()
        .filter(|r| r.vertex_start as usize >= vert_before)
    {
        let first = ((request.vertex_start as usize - vert_before) / 4).min(quads);
        let last = (first + request.vertex_count as usize / 4).min(quads);
        request.vertex_start = (vert_before + starts[first]) as u32;
        request.vertex_count = (starts[last] - starts[first]) as u32;
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
    // mask's atlas sub-region. Keep interpolation in the original orientation.
    let (min_u, max_u) = (tl.min(tr), tl.max(tr));
    let (min_v, max_v) = (tt.min(tb), tt.max(tb));
    let ul = (tl + u0 * (tr - tl)).clamp(min_u, max_u);
    let ur = (tl + u1 * (tr - tl)).clamp(min_u, max_u);
    let ut = (tt + v0 * (tb - tt)).clamp(min_v, max_v);
    let ub = (tt + v1 * (tb - tt)).clamp(min_v, max_v);
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
    fn clipped_quad_preserves_flipped_mask_uvs() {
        let mask = Rectangle::new(iced::Point::new(50.0, 50.0), iced::Size::new(20.0, 20.0));
        let bounds = Rectangle::new(iced::Point::new(45.0, 55.0), iced::Size::new(15.0, 20.0));
        let mut vertices = quad(bounds);

        apply_mask_to_quad(&mut vertices, mask, 1.0, 0.0, 1.0, 0.0);

        assert_eq!(
            vertices.map(|vertex| vertex.mask_tex_coords),
            [[1.0, 0.75], [0.5, 0.75], [0.5, 0.0], [1.0, 0.0]]
        );
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

    fn sliced_mask(id: u64, rect: Rectangle) -> Frame {
        Frame {
            id,
            texture: Some("Interface/Rounded".to_string()),
            texture_slice_margins: (4.0, 4.0, 4.0, 4.0),
            texture_source_dimensions: Some((9, 9)),
            layout_rect: Some(crate::LayoutRect {
                x: rect.x / UI_SCALE,
                y: rect.y / UI_SCALE,
                width: rect.width / UI_SCALE,
                height: rect.height / UI_SCALE,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn sliced_texel_keeps_margins_and_stretches_the_centre() {
        assert_eq!(sliced_texel(2.0 * UI_SCALE, 100.0, 4.0, 4.0, 9.0), 2.0);
        assert_eq!(sliced_texel(50.0, 100.0, 4.0, 4.0, 9.0), 4.5);
        assert_eq!(
            sliced_texel(100.0 - 2.0 * UI_SCALE, 100.0, 4.0, 4.0, 9.0),
            7.0
        );
    }

    #[test]
    fn sliced_mask_splits_quads_at_the_slice_lines() {
        let bounds = Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(100.0, 40.0));
        let mut batch = QuadBatch::new();
        batch.vertices.extend(quad(bounds));
        batch.indices.extend([0, 1, 2, 0, 2, 3]);
        batch
            .texture_requests
            .push(TextureRequest::new("Interface/Body", 0, 4));
        let mut registry = WidgetRegistry::new();
        registry.register(sliced_mask(1, bounds));

        apply_mask_texture(&mut batch, 0, bounds, &[1], &registry);

        assert_eq!(batch.vertices.len(), 9 * 4);
        assert_eq!(batch.indices.len(), 9 * 6);
        assert_eq!(batch.texture_requests[0].vertex_start, 0);
        assert_eq!(batch.texture_requests[0].vertex_count, 36);
        assert_eq!(batch.mask_texture_requests[0].vertex_count, 36);
        // The top-left corner piece maps its 4 pixels onto the mask's first 4 texels.
        let corner = &batch.vertices[0..4];
        assert_eq!(corner[0].position, [0.0, 0.0]);
        assert_eq!(corner[2].position, [4.0 * UI_SCALE, 4.0 * UI_SCALE]);
        assert_eq!(corner[0].mask_tex_coords, [0.0, 0.0]);
        assert_eq!(corner[2].mask_tex_coords, [4.0 / 9.0, 4.0 / 9.0]);
        // The centre piece samples only the middle texel column and row.
        let centre = &batch.vertices[16..20];
        assert_eq!(centre[0].mask_tex_coords, [4.0 / 9.0, 4.0 / 9.0]);
        assert_eq!(centre[2].mask_tex_coords, [5.0 / 9.0, 5.0 / 9.0]);
    }

    #[test]
    fn sliced_mask_on_a_quad_inside_one_slice_keeps_one_quad() {
        let mask = Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(100.0, 40.0));
        let bounds = Rectangle::new(iced::Point::new(10.0, 10.0), iced::Size::new(20.0, 20.0));
        let mut batch = QuadBatch::new();
        batch.vertices.extend(quad(bounds));
        batch.indices.extend([0, 1, 2, 0, 2, 3]);
        let mut registry = WidgetRegistry::new();
        registry.register(sliced_mask(1, mask));

        apply_mask_texture(&mut batch, 0, bounds, &[1], &registry);

        // Inside the centre slice, which maps 92 x 32 pixels onto one texel.
        assert_eq!(batch.vertices.len(), 4);
        let u = |x: f32| (4.0 + (x - 4.0) / 92.0) / 9.0;
        let v = |y: f32| (4.0 + (y - 4.0) / 32.0) / 9.0;
        assert_eq!(batch.vertices[0].mask_tex_coords, [u(10.0), v(10.0)]);
        assert_eq!(batch.vertices[2].mask_tex_coords, [u(30.0), v(30.0)]);
    }

    #[test]
    fn mask_without_room_for_its_margins_stretches_whole() {
        let bounds = Rectangle::new(iced::Point::new(0.0, 0.0), iced::Size::new(6.0, 6.0));
        let mut batch = QuadBatch::new();
        batch.vertices.extend(quad(bounds));
        batch.indices.extend([0, 1, 2, 0, 2, 3]);
        let mut registry = WidgetRegistry::new();
        registry.register(sliced_mask(1, bounds));

        apply_mask_texture(&mut batch, 0, bounds, &[1], &registry);

        assert_eq!(batch.vertices.len(), 4);
        assert_eq!(batch.vertices[0].mask_tex_coords, [0.0, 0.0]);
        assert_eq!(batch.vertices[2].mask_tex_coords, [1.0, 1.0]);
    }
}
