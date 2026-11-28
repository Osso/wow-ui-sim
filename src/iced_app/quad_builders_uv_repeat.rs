//! Textures whose coordinates go past one texture: each face mapped from its
//! own corners, with the texture repeating.
//!
//! The client draws a texture region as two triangles over its four corners
//! (after SetVertexOffset), each mapping the corners' SetTexCoord values
//! affinely, and a texture set with a repeating wrap mode samples each value
//! modulo one texture. The simulator's textures sit in shared atlases with
//! clamped addressing, so the repeat cannot come from the sampler: each
//! triangle is cut along the whole-number texture lines instead, and every
//! piece samples its own repeat with coordinates inside one texture.

use iced::Rectangle;

use crate::render::{BlendMode, QuadBatch};

use super::clipping::CLIENT_QUAD_TRIANGLES;

/// Most texture repeats one triangle is cut into; a face asking for more is
/// drawn with its coordinates clamped instead of being cut into that many
/// pieces.
const MAX_REPEAT_CELLS: f32 = 4096.0;

type Corner = ([f32; 2], [f32; 2]);

/// The face's four corners in the simulator's quad order (upper left, upper
/// right, lower right, lower left): screen position after the corner offsets,
/// and SetTexCoord value.
pub(super) fn face_corners(
    bounds: Rectangle,
    tex_coords: [f32; 8],
    offsets: Option<[(f32, f32); 4]>,
    pixels_per_unit: f32,
) -> [Corner; 4] {
    let [ul_u, ul_v, ll_u, ll_v, ur_u, ur_v, lr_u, lr_v] = tex_coords;
    let [ul, ll, ur, lr] = offsets.unwrap_or_default();
    let (left, top) = (bounds.x, bounds.y);
    let (right, bottom) = (bounds.x + bounds.width, bounds.y + bounds.height);
    let at =
        |x: f32, y: f32, (dx, dy): (f32, f32)| [x + dx * pixels_per_unit, y - dy * pixels_per_unit];
    [
        (at(left, top, ul), [ul_u, ul_v]),
        (at(right, top, ur), [ur_u, ur_v]),
        (at(right, bottom, lr), [lr_u, lr_v]),
        (at(left, bottom, ll), [ll_u, ll_v]),
    ]
}

/// Draw the face: its two triangles (the client's split), each cut into one
/// piece per texture repeat it covers.
pub(super) fn emit_repeating_face(
    batch: &mut QuadBatch,
    corners: [Corner; 4],
    path: &str,
    color: [f32; 4],
    blend: BlendMode,
) {
    let mut quads = Vec::new();
    for triangle in CLIENT_QUAD_TRIANGLES {
        cut_into_repeats(&triangle.map(|i| corners[i]), &mut quads);
    }
    batch.push_textured_path_corner_quads(&quads, path, color, blend);
}

fn cut_into_repeats(triangle: &[Corner; 3], quads: &mut Vec<[Corner; 4]>) {
    let range = |axis: usize| {
        let values = triangle.map(|(_, uv)| uv[axis]);
        let low = values.iter().copied().fold(f32::INFINITY, f32::min).floor();
        let high = values
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil();
        (low, (high - low).max(1.0))
    };
    let ((u0, u_cells), (v0, v_cells)) = (range(0), range(1));
    if !(u_cells * v_cells <= MAX_REPEAT_CELLS) {
        let clamped = triangle.map(|(p, uv)| (p, uv.map(|c| c.clamp(0.0, 1.0))));
        push_fan(&clamped, quads);
        return;
    }
    for i in 0..u_cells as i32 {
        let u = u0 + i as f32;
        let column = clip(triangle.to_vec(), 0, u, u + 1.0);
        if column.len() < 3 {
            continue;
        }
        for j in 0..v_cells as i32 {
            let v = v0 + j as f32;
            let piece = clip(column.clone(), 1, v, v + 1.0);
            if piece.len() < 3 {
                continue;
            }
            let local: Vec<Corner> = piece
                .into_iter()
                .map(|(p, uv)| {
                    (
                        p,
                        [(uv[0] - u).clamp(0.0, 1.0), (uv[1] - v).clamp(0.0, 1.0)],
                    )
                })
                .collect();
            push_fan(&local, quads);
        }
    }
}

/// A convex polygon as quads of a fan from its first corner; a quad whose
/// last corner repeats its third covers a single triangle.
fn push_fan(polygon: &[Corner], quads: &mut Vec<[Corner; 4]>) {
    let mut i = 1;
    while i + 1 < polygon.len() {
        let last = polygon[(i + 2).min(polygon.len() - 1)];
        quads.push([polygon[0], polygon[i], polygon[i + 1], last]);
        i += 2;
    }
}

/// Keep the part of a convex polygon whose texture coordinate `axis` lies in
/// `[low, high]`, interpolating position and coordinates on the cut edges.
fn clip(polygon: Vec<Corner>, axis: usize, low: f32, high: f32) -> Vec<Corner> {
    let polygon = clip_side(polygon, |c| c.1[axis] - low);
    clip_side(polygon, |c| high - c.1[axis])
}

fn clip_side(polygon: Vec<Corner>, inside: impl Fn(&Corner) -> f32) -> Vec<Corner> {
    let mut out = Vec::with_capacity(polygon.len() + 2);
    for (k, &current) in polygon.iter().enumerate() {
        let next = polygon[(k + 1) % polygon.len()];
        let (a, b) = (inside(&current), inside(&next));
        if a >= 0.0 {
            out.push(current);
        }
        if (a >= 0.0) != (b >= 0.0) {
            let t = a / (a - b);
            let lerp =
                |x: [f32; 2], y: [f32; 2]| [x[0] + (y[0] - x[0]) * t, x[1] + (y[1] - x[1]) * t];
            out.push((lerp(current.0, next.0), lerp(current.1, next.1)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::{Point, Size};

    /// The texture coordinate drawn at (x, y), from whichever piece triangle
    /// holds the point.
    fn sample(quads: &[[Corner; 4]], x: f32, y: f32) -> Option<[f32; 2]> {
        for q in quads {
            for t in [[0, 1, 2], [0, 2, 3]] {
                let [a, b, c] = t.map(|i| q[i]);
                let d =
                    (b.0[1] - c.0[1]) * (a.0[0] - c.0[0]) + (c.0[0] - b.0[0]) * (a.0[1] - c.0[1]);
                if d.abs() < 1e-6 {
                    continue;
                }
                let w0 = ((b.0[1] - c.0[1]) * (x - c.0[0]) + (c.0[0] - b.0[0]) * (y - c.0[1])) / d;
                let w1 = ((c.0[1] - a.0[1]) * (x - c.0[0]) + (a.0[0] - c.0[0]) * (y - c.0[1])) / d;
                let w2 = 1.0 - w0 - w1;
                if w0 >= -1e-5 && w1 >= -1e-5 && w2 >= -1e-5 {
                    return Some([0, 1].map(|k| w0 * a.1[k] + w1 * b.1[k] + w2 * c.1[k]));
                }
            }
        }
        None
    }

    fn cut(corners: [Corner; 4]) -> Vec<[Corner; 4]> {
        let mut quads = Vec::new();
        for triangle in CLIENT_QUAD_TRIANGLES {
            cut_into_repeats(&triangle.map(|i| corners[i]), &mut quads);
        }
        quads
    }

    /// Equal as texture coordinates within one repeat (0 and 1 are the same
    /// place on a repeating texture's seam).
    fn same_place(a: [f32; 2], b: [f32; 2]) -> bool {
        a.iter().zip(b).all(|(x, y)| {
            let d = (x - y).abs();
            d < 1e-3 || (d - 1.0).abs() < 1e-3
        })
    }

    #[test]
    fn a_face_samples_its_corner_mapping_modulo_one_texture() {
        // coordinates running 0..3.5 across and 0..2.25 down, sheared so no
        // axis-aligned tiling reproduces them
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(300.0, 200.0));
        let raw = [0.0, 0.0, 0.5, 2.0, 3.0, 0.25, 3.5, 2.25];
        let corners = face_corners(bounds, raw, None, 1.0);
        let quads = cut(corners);
        let [ul, ur, lr, ll] = corners;
        for (x, y) in [
            (10.0, 10.0),
            (150.0, 37.0),
            (280.0, 190.0),
            (77.0, 160.0),
            (210.0, 120.0),
        ] {
            // the client's split: upper left - upper right - lower left, and
            // upper right - lower right - lower left
            let (s, t) = (x / 300.0, y / 200.0);
            let full = if s + t <= 1.0 {
                [0, 1].map(|k| ul.1[k] + (ur.1[k] - ul.1[k]) * s + (ll.1[k] - ul.1[k]) * t)
            } else {
                [0, 1].map(|k| {
                    lr.1[k] + (ll.1[k] - lr.1[k]) * (1.0 - s) + (ur.1[k] - lr.1[k]) * (1.0 - t)
                })
            };
            let want = full.map(|c| c - c.floor());
            let got = sample(&quads, x, y).expect("the point is drawn");
            assert!(same_place(got, want), "at {x},{y}: {got:?} want {want:?}");
        }
        for q in &quads {
            for (_, uv) in q {
                assert!(uv.iter().all(|c| (0.0..=1.0).contains(c)), "{uv:?}");
            }
        }
    }

    #[test]
    fn a_triangle_face_is_cut_along_its_own_mapping() {
        // the lower right corner moved onto the lower left: one triangle
        let bounds = Rectangle::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0));
        let offsets = Some([(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (-100.0, 0.0)]);
        let raw = [0.0, 0.0, 0.0, 2.0, 2.0, 0.0, 2.0, 2.0];
        let quads = cut(face_corners(bounds, raw, offsets, 1.0));
        // in the upper left - upper right - lower left triangle, (x, y) maps
        // to (x / 50, y / 50)
        let got = sample(&quads, 30.0, 40.0).expect("inside the triangle");
        assert!(same_place(got, [0.6, 0.8]), "{got:?}");
        let got = sample(&quads, 60.0, 20.0).expect("inside the triangle");
        assert!(same_place(got, [0.2, 0.4]), "{got:?}");
        assert!(
            sample(&quads, 90.0, 90.0).is_none(),
            "outside the triangle stays empty"
        );
    }
}
