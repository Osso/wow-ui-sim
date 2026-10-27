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
    for y in [3, 12] {
        let row: Vec<u8> = (8..24).map(|x| image.get_pixel(x, y).0[0]).collect();
        assert!(
            row.iter()
                .filter(|&&value| value > 24 && value < 231)
                .count()
                >= 3,
            "row {y} must interpolate between black and white: {row:?}"
        );
    }
}
