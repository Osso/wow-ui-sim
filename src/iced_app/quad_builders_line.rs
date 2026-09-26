//! Line widget quad rendering — rotated quads between two anchor points.

use crate::render::{BlendMode, QuadBatch};

fn is_talent_arrow_line(f: &crate::widget::Frame) -> bool {
    f.atlas
        .as_deref()
        .is_some_and(|atlas| atlas.starts_with("talents-arrow-line"))
}

/// Resolve a line anchor to screen-space pixel coordinates.
fn resolve_line_endpoint(
    anchor: &crate::widget::LineAnchor,
    registry: &crate::widget::WidgetRegistry,
) -> Option<(f32, f32)> {
    use super::layout::anchor_position;

    let target_id = anchor.target_id?;
    let r = registry.get(target_id)?.layout_rect?;
    let (ax, ay) = anchor_position(anchor.point, r.x, r.y, r.width, r.height);
    let ui_scale = crate::render::texture::UI_SCALE;
    Some((
        (ax + anchor.x_offset) * ui_scale,
        (ay - anchor.y_offset) * ui_scale,
    ))
}

fn adjust_talent_arrow_line_caps(
    f: &crate::widget::Frame,
    start: (f32, f32),
    end: (f32, f32),
    thickness: f32,
) -> ((f32, f32), (f32, f32)) {
    if !is_talent_arrow_line(f) {
        return (start, end);
    }
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return (start, end);
    }
    let ux = dx / len;
    let uy = dy / len;
    let start_overlap = (thickness * 0.5).clamp(1.0, 2.0);
    let end_overlap = (thickness * 1.25).clamp(2.0, 5.0);
    (
        (start.0 - ux * start_overlap, start.1 - uy * start_overlap),
        (end.0 + ux * end_overlap, end.1 + uy * end_overlap),
    )
}

/// Compute the 4 corner positions of a rotated line quad from endpoints and thickness.
fn line_quad_positions(
    start: (f32, f32),
    end: (f32, f32),
    thickness: f32,
) -> Option<[[f32; 2]; 4]> {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return None;
    }
    let half_t = thickness / 2.0;
    let px = -dy / len * half_t;
    let py = dx / len * half_t;
    Some([
        [start.0 + px, start.1 + py],
        [start.0 - px, start.1 - py],
        [end.0 - px, end.1 - py],
        [end.0 + px, end.1 + py],
    ])
}

/// Build quads for a Line widget — a rotated quad between two anchor points.
pub(super) fn build_line_quads(
    batch: &mut QuadBatch,
    f: &crate::widget::Frame,
    registry: &crate::widget::WidgetRegistry,
    alpha: f32,
) {
    let Some((positions, uvs, tint)) = resolve_line_quad_inputs(f, registry, alpha) else {
        return;
    };
    emit_resolved_line_quad(batch, f, alpha, &positions, &uvs, tint);
}

fn resolve_line_quad_inputs(
    f: &crate::widget::Frame,
    registry: &crate::widget::WidgetRegistry,
    alpha: f32,
) -> Option<([[f32; 2]; 4], [[f32; 2]; 4], [f32; 4])> {
    let (Some(start_anchor), Some(end_anchor)) = (&f.line_start, &f.line_end) else {
        return None;
    };
    let sp = resolve_line_endpoint(start_anchor, registry)?;
    let ep = resolve_line_endpoint(end_anchor, registry)?;
    // Endpoints anchor to logical geometry, but the line moves with its own subtree.
    let offset = crate::layout::apply_scroll_offsets(
        registry,
        f.id,
        crate::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
    );
    let scale = crate::render::texture::UI_SCALE;
    let sp = (sp.0 + offset.x * scale, sp.1 + offset.y * scale);
    let ep = (ep.0 + offset.x * scale, ep.1 + offset.y * scale);
    let thickness = f.line_thickness * scale;
    let (sp, ep) = adjust_talent_arrow_line_caps(f, sp, ep, thickness);
    let positions = line_quad_positions(sp, ep, thickness)?;
    Some((positions, line_uvs(f), line_tint(f, alpha)))
}

fn line_uvs(f: &crate::widget::Frame) -> [[f32; 2]; 4] {
    if let Some((left, right, top, bottom)) = f.tex_coords {
        [[left, top], [left, bottom], [right, bottom], [right, top]]
    } else {
        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
    }
}

fn line_tint(f: &crate::widget::Frame, alpha: f32) -> [f32; 4] {
    let vc = f.vertex_color.as_ref();
    [
        vc.map_or(1.0, |c| c.r),
        vc.map_or(1.0, |c| c.g),
        vc.map_or(1.0, |c| c.b),
        vc.map_or(1.0, |c| c.a) * alpha,
    ]
}

fn emit_resolved_line_quad(
    batch: &mut QuadBatch,
    f: &crate::widget::Frame,
    alpha: f32,
    positions: &[[f32; 2]; 4],
    uvs: &[[f32; 2]; 4],
    tint: [f32; 4],
) {
    if let Some(color) = f.color_texture {
        let color_tint = [
            color.r * tint[0],
            color.g * tint[1],
            color.b * tint[2],
            color.a * alpha,
        ];
        emit_line_vertices(batch, positions, uvs, color_tint, -1, f.blend_mode);
        return;
    }
    if let Some(ref tex_path) = f.texture {
        let vertex_start = batch.vertices.len() as u32;
        emit_line_vertices(batch, positions, uvs, tint, -2, f.blend_mode);
        batch
            .texture_requests
            .push(crate::render::shader::TextureRequest::new(
                tex_path.clone(),
                vertex_start,
                4,
            ));
        return;
    }
    emit_line_vertices(batch, positions, uvs, tint, -1, f.blend_mode);
}

/// Push 4 vertices and 6 indices for a line quad with arbitrary positions.
fn emit_line_vertices(
    batch: &mut QuadBatch,
    positions: &[[f32; 2]; 4],
    uvs: &[[f32; 2]; 4],
    color: [f32; 4],
    tex_index: i32,
    blend_mode: BlendMode,
) {
    use crate::render::shader::QuadVertex;

    let base = batch.vertices.len() as u32;
    let flags = blend_mode as u32;
    for i in 0..4 {
        batch.vertices.push(QuadVertex {
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
    // TL(0)-BL(1)-BR(2) and TL(0)-BR(2)-TR(3)
    batch
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{Color, Frame};

    #[test]
    fn line_uvs_uses_tex_coords_when_present() {
        let mut frame = Frame::default();
        frame.tex_coords = Some((0.1, 0.9, 0.2, 0.8));

        assert_eq!(
            line_uvs(&frame),
            [[0.1, 0.2], [0.1, 0.8], [0.9, 0.8], [0.9, 0.2]]
        );
    }

    #[test]
    fn line_tint_uses_vertex_color_and_alpha() {
        let mut frame = Frame::default();
        frame.vertex_color = Some(Color::new(0.2, 0.4, 0.6, 0.5));

        assert_eq!(line_tint(&frame, 0.8), [0.2, 0.4, 0.6, 0.4]);
    }

    #[test]
    fn talent_arrow_line_caps_extend_beyond_anchors() {
        let mut frame = Frame::default();
        frame.atlas = Some("talents-arrow-line-gray".to_string());
        let start = (10.0, 20.0);
        let end = (110.0, 20.0);

        let (extended_start, extended_end) = adjust_talent_arrow_line_caps(&frame, start, end, 4.0);

        assert!(
            extended_start.0 < start.0,
            "start cap should extend backward"
        );
        assert!(extended_end.0 > end.0, "end cap should extend forward");
        assert_eq!(extended_start.1, start.1);
        assert_eq!(extended_end.1, end.1);
    }
}
