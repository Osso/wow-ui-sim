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
