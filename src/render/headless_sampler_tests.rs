use super::HeadlessRenderContext;
use crate::render::QuadBatch;
use crate::render::shader::{GpuTextureData, WowUiPrimitive};
use iced::{Point, Rectangle, Size};
use std::sync::Arc;

#[test]
fn magnified_texture_interpolates_both_colour_transitions() {
    let path = "sampler-checker";
    let mut batch = QuadBatch::default();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(32.0, 16.0)),
        path,
        [1.0; 4],
        crate::BlendMode::Alpha,
    );
    let pixels = [
        [0u8, 0, 0, 255],
        [255, 255, 255, 255],
        [255, 255, 255, 255],
        [0, 0, 0, 255],
    ]
    .concat();
    let texture = GpuTextureData {
        path: path.into(),
        width: 2,
        height: 2,
        rgba: pixels.into(),
    };
    let mut primitive =
        WowUiPrimitive::new_merged_with_textures(Arc::new(batch), vec![texture], Vec::new());
    let image = HeadlessRenderContext::new(32, 16).render(&mut primitive);
    // The half-texel inset maps quad edges to texel centers. Linear filtering
    // interpolates the checker in linear light, then the target encodes sRGB.
    for y in [3, 12] {
        let expected = |x: u32| {
            let u = (x as f32 + 0.5) / 32.0;
            let v = (y as f32 + 0.5) / 16.0;
            let linear = u * (1.0 - v) + (1.0 - u) * v;
            ((1.055 * linear.powf(1.0 / 2.4) - 0.055) * 255.0).round() as u8
        };
        let endpoints = [expected(7), expected(24)];
        let row: Vec<u8> = (7..=24).map(|x| image.get_pixel(x, y).0[0]).collect();
        assert!(
            row[0].abs_diff(endpoints[0]) <= 2,
            "row {y}: {row:?}, expected {endpoints:?}"
        );
        assert!(
            row[17].abs_diff(endpoints[1]) <= 2,
            "row {y}: {row:?}, expected {endpoints:?}"
        );
        assert!(
            row.iter()
                .filter(|&&value| value > 24 && value < 231)
                .count()
                >= 3,
            "row {y}: {row:?}"
        );
        assert!(
            row.windows(2).all(|pair| if endpoints[0] < endpoints[1] {
                pair[0] <= pair[1]
            } else {
                pair[0] >= pair[1]
            }),
            "row {y}: {row:?}"
        );
    }
}
