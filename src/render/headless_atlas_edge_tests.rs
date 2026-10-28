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
        // The texture is sRGB, and alpha blending occurs in linear space over
        // the headless clear color before the sRGB target encodes the result.
        for (edge, rgb) in [(127, [255u8, 0, 0]), (128, [0u8, 255, 0])] {
            let edge_pixel = image.get_pixel(edge, 64).0;
            let opacity = f32::from(alpha) / 255.0;
            let mut expected = [0u8; 4];
            for channel in 0..3 {
                let source = f32::from(rgb[channel]) / 255.0;
                let background = [0.05, 0.05, 0.08][channel];
                let linear = source * opacity + background * (1.0 - opacity);
                let encoded = if linear <= 0.0031308 {
                    12.92 * linear
                } else {
                    1.055 * linear.powf(1.0 / 2.4) - 0.055
                };
                expected[channel] = (encoded * 255.0).round() as u8;
            }
            expected[3] = 255;
            for channel in 0..4 {
                assert!(
                    edge_pixel[channel].abs_diff(expected[channel]) <= 2,
                    "alpha={alpha} edge={edge}: {edge_pixel:?}, expected {expected:?}"
                );
            }
        }
    }
}
