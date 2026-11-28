use iced::Rectangle;

use crate::render::{QuadBatch, QuadVertex};

pub(in crate::iced_app) fn clip_recent_quads(
    batch: &mut QuadBatch,
    vert_before: usize,
    clip: Rectangle,
) {
    for chunk in batch.vertices[vert_before..].chunks_exact_mut(4) {
        let original = [chunk[0], chunk[1], chunk[2], chunk[3]];
        let left = chunk[0].position[0].min(chunk[3].position[0]);
        let right = chunk[1].position[0].max(chunk[2].position[0]);
        let top = chunk[0].position[1].min(chunk[1].position[1]);
        let bottom = chunk[2].position[1].max(chunk[3].position[1]);

        let clipped_left = left.max(clip.x);
        let clipped_right = right.min(clip.x + clip.width);
        let clipped_top = top.max(clip.y);
        let clipped_bottom = bottom.min(clip.y + clip.height);

        if clipped_left >= clipped_right || clipped_top >= clipped_bottom {
            for vertex in chunk.iter_mut() {
                vertex.color[3] = 0.0;
            }
            continue;
        }

        let x0 = inv_lerp(left, right, clipped_left);
        let x1 = inv_lerp(left, right, clipped_right);
        let y0 = inv_lerp(top, bottom, clipped_top);
        let y1 = inv_lerp(top, bottom, clipped_bottom);

        clip_vertex(chunk, &original, 0, clipped_left, clipped_top, x0, y0);
        clip_vertex(chunk, &original, 1, clipped_right, clipped_top, x1, y0);
        clip_vertex(chunk, &original, 2, clipped_right, clipped_bottom, x1, y1);
        clip_vertex(chunk, &original, 3, clipped_left, clipped_bottom, x0, y1);
    }
}

/// The client's two triangles of a quad: upper left, upper right, lower left
/// and upper right, lower right, lower left (SetVertexOffset's corners 1-3-2
/// and 3-4-2), split along the upper right - lower left diagonal. Measured in
/// client 1.60.1 with moved-corner textures: a quad with its lower right
/// corner pulled in past that diagonal draws as a plain triangle, one with its
/// upper right corner pulled in draws dented, and a texture bends along that
/// diagonal. Indices are into the quad's vertices in the simulator's order:
/// 0 upper left, 1 upper right, 2 lower right, 3 lower left.
pub(in crate::iced_app) const CLIENT_QUAD_TRIANGLES: [[usize; 3]; 2] = [[0, 1, 3], [1, 2, 3]];

/// Re-split the quads appended since `index_before` along the client's
/// diagonal (`CLIENT_QUAD_TRIANGLES`) instead of the simulator's upper left -
/// lower right one. Only quads whose corners were moved need it: a rectangle
/// draws the same either way, but a moved-corner quad changes shape (a corner
/// pulled in past a diagonal) and texture mapping (each triangle maps its
/// texture affinely, so the texture bends along the split).
pub(in crate::iced_app) fn split_quads_on_client_diagonal(
    batch: &mut QuadBatch,
    index_before: usize,
) {
    for chunk in batch.indices[index_before..].as_chunks_mut::<6>().0 {
        let b = chunk[0];
        if *chunk == [b, b + 1, b + 2, b, b + 2, b + 3] {
            let [t0, t1] = CLIENT_QUAD_TRIANGLES;
            for (slot, &corner) in chunk.iter_mut().zip(t0.iter().chain(t1.iter())) {
                *slot = b + corner as u32;
            }
        }
    }
}

fn clip_vertex(
    chunk: &mut [QuadVertex],
    original: &[QuadVertex; 4],
    index: usize,
    x: f32,
    y: f32,
    tx: f32,
    ty: f32,
) {
    const FLAG_COOLDOWN_SWIPE: u32 = 0x200;
    chunk[index].position = [x, y];
    if (chunk[index].flags & FLAG_COOLDOWN_SWIPE) == 0 {
        chunk[index].tex_coords = [
            lerp(original[0].tex_coords[0], original[1].tex_coords[0], tx),
            lerp(original[0].tex_coords[1], original[3].tex_coords[1], ty),
        ];
        chunk[index].local_uv = [
            lerp(original[0].local_uv[0], original[1].local_uv[0], tx),
            lerp(original[0].local_uv[1], original[3].local_uv[1], ty),
        ];
        if original[index].mask_tex_index != -1 || original[index].mask_tex_coords != [0.0, 0.0] {
            chunk[index].mask_tex_coords = [
                lerp(
                    original[0].mask_tex_coords[0],
                    original[1].mask_tex_coords[0],
                    tx,
                ),
                lerp(
                    original[0].mask_tex_coords[1],
                    original[3].mask_tex_coords[1],
                    ty,
                ),
            ];
        }
    }
}

fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    let denom = b - a;
    if denom.abs() < f32::EPSILON {
        0.0
    } else {
        ((v - a) / denom).clamp(0.0, 1.0)
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod client_diagonal_tests {
    use super::split_quads_on_client_diagonal;
    use crate::render::QuadBatch;
    use iced::{Point, Rectangle, Size};

    #[test]
    fn quads_are_split_upper_right_to_lower_left() {
        let mut batch = QuadBatch::new();
        batch.push_solid(
            Rectangle::new(Point::ORIGIN, Size::new(10.0, 10.0)),
            [1.0; 4],
        );
        batch.push_solid(
            Rectangle::new(Point::new(20.0, 0.0), Size::new(10.0, 10.0)),
            [1.0; 4],
        );
        assert_eq!(batch.indices, vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]);

        split_quads_on_client_diagonal(&mut batch, 6);

        // Only the quads from index 6 on: the first keeps its split.
        assert_eq!(batch.indices, vec![0, 1, 2, 0, 2, 3, 4, 5, 7, 5, 6, 7]);
    }
}
