use super::HeadlessRenderContext;
use crate::render::QuadBatch;
use crate::render::shader::{GpuTextureData, WowUiPrimitive};
use iced::{Point, Rectangle, Size};
use std::sync::Arc;

#[test]
fn adjacent_full_atlas_slots_preserve_edges_for_opaque_and_translucent_texels() {
    for alpha in [255u8, 96, 0] {
        let mut batch = QuadBatch::default();
        let mut textures = Vec::new();
        for (index, rgb) in [[255, 0, 0], [0, 255, 0]].into_iter().enumerate() {
            let path = format!("adjacent-{index}");
            batch.push_textured_path(
                Rectangle::new(
                    Point::new(index as f32 * 128.0, 0.0),
                    Size::new(128.0, 128.0),
                ),
                &path,
                [1.0; 4],
                crate::BlendMode::Alpha,
            );
            textures.push(GpuTextureData {
                path,
                width: 64,
                height: 64,
                rgba: [rgb[0], rgb[1], rgb[2], alpha].repeat(64 * 64).into(),
            });
        }
        let mut primitive =
            WowUiPrimitive::new_merged_with_textures(Arc::new(batch), textures, Vec::new());
        let image = HeadlessRenderContext::new(256, 128).render(&mut primitive);
        for (edge, center) in [(127, 64), (128, 192)] {
            let edge_pixel = image.get_pixel(edge, 64).0;
            let center_pixel = image.get_pixel(center, 64).0;
            for channel in 0..4 {
                assert!(
                    edge_pixel[channel].abs_diff(center_pixel[channel]) <= 2,
                    "alpha={alpha} edge={edge}: {edge_pixel:?} differs from interior {center_pixel:?}"
                );
            }
        }
    }
}
