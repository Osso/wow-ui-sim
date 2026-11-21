use super::HeadlessRenderContext;
use crate::render::QuadBatch;
use crate::render::shader::{GpuTextureData, WowUiPrimitive};
use iced::{Point, Rectangle, Size};
use std::sync::Arc;

#[test]
fn magnified_texture_interpolates_both_colour_transitions() {
    let image = render_checker();
    for y in [3, 12] {
        assert_checker_row(&image, y);
    }
}

fn render_checker() -> image::RgbaImage {
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
    HeadlessRenderContext::new(32, 16).render(&mut primitive)
}

fn assert_checker_row(image: &image::RgbaImage, y: u32) {
    let expected = |x: u32| {
        let u = (x as f32 + 0.5) / 32.0;
        let v = (y as f32 + 0.5) / 16.0;
        let linear = u * (1.0 - v) + (1.0 - u) * v;
        ((1.055 * linear.powf(1.0 / 2.4) - 0.055) * 255.0).round() as u8
    };
    let endpoints = [expected(7), expected(24)];
    let row: Vec<u8> = (7..=24).map(|x| image.get_pixel(x, y).0[0]).collect();
    assert!(row[0].abs_diff(endpoints[0]) <= 2, "row {y}: {row:?}");
    assert!(row[17].abs_diff(endpoints[1]) <= 2, "row {y}: {row:?}");
    assert!(
        row.iter().filter(|&&v| v > 24 && v < 231).count() >= 3,
        "row {y}: {row:?}"
    );
    assert!(
        row.windows(2).all(|pair| {
            if endpoints[0] < endpoints[1] {
                pair[0] <= pair[1]
            } else {
                pair[0] >= pair[1]
            }
        }),
        "row {y}: {row:?}"
    );
}

#[test]
fn minified_thin_lines_stay_continuous_with_mipmaps() {
    let image = render_minified_cross();
    let background = image.get_pixel(16, 16);
    assert_eq!(image.get_pixel(0, 0), background, "transparent corner");
    assert_cross_row_is_visible(&image, background.0[0]);
}

fn render_minified_cross() -> image::RgbaImage {
    let path = "minified-cross-128";
    let mut batch = QuadBatch::default();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(16.0, 16.0)),
        path,
        [1.0; 4],
        crate::BlendMode::Alpha,
    );
    let texture = GpuTextureData {
        path: path.into(),
        width: 128,
        height: 128,
        rgba: minified_cross_pixels().into(),
    };
    let mut primitive =
        WowUiPrimitive::new_merged_with_textures(Arc::new(batch), vec![texture], Vec::new());
    HeadlessRenderContext::new(17, 17).render(&mut primitive)
}

fn minified_cross_pixels() -> Vec<u8> {
    let mut pixels = vec![0u8; 128 * 128 * 4];
    for i in 16..112 {
        for band in [63, 64] {
            for offset in [(band * 128 + i) * 4, (i * 128 + band) * 4] {
                pixels[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }
    pixels
}

fn assert_cross_row_is_visible(image: &image::RgbaImage, background: u8) {
    for x in 3..13 {
        let peak = image.get_pixel(x, 7).0[0].max(image.get_pixel(x, 8).0[0]);
        assert!(
            peak.saturating_sub(background) >= 20,
            "minified 2px line at x={x}: peak red={peak}, background red={background}"
        );
    }
}

#[test]
fn downscaled_white_stroke_does_not_darken_white_background() {
    let path = "white-stroke-on-transparent";
    let mut batch = QuadBatch::default();
    for (x, background) in [(0.0, 1.0), (24.0, 0.0)] {
        let bounds = Rectangle::new(Point::new(x, 0.0), Size::new(24.0, 8.0));
        batch.push_solid(bounds, [background, background, background, 1.0]);
        batch.push_textured_path(bounds, path, [1.0; 4], crate::BlendMode::Alpha);
    }
    let row = [[0u8; 4]; 24]
        .into_iter()
        .chain([[255; 4]; 4])
        .chain([[0; 4]; 36])
        .flatten()
        .collect::<Vec<_>>();
    let texture = GpuTextureData {
        path: path.into(),
        width: 64,
        height: 64,
        rgba: row.repeat(64).into(),
    };
    let mut primitive =
        WowUiPrimitive::new_merged_with_textures(Arc::new(batch), vec![texture], Vec::new());
    let image = HeadlessRenderContext::new(48, 8).render(&mut primitive);

    // Confirm a partially covered stroke over black, not just a white background.
    let stroke = image.get_pixel(34, 4).0[0];
    assert!((80..200).contains(&stroke), "stroke on black: {stroke}");
    for x in 0..24 {
        let pixel = image.get_pixel(x, 4).0;
        assert!(pixel[0] >= 252, "white on white at x={x}: {pixel:?}");
    }
}

#[test]
fn minified_adjacent_atlas_slots_never_bleed_across_cell_edges() {
    let image = render_adjacent_slots();
    assert_green_slot_without_bleed(&image);
}

fn render_adjacent_slots() -> image::RgbaImage {
    let target_path = "minified-slot-green";
    let mut batch = QuadBatch::default();
    batch.push_textured_path(
        Rectangle::new(Point::ORIGIN, Size::new(7.0, 7.0)),
        target_path,
        [1.0; 4],
        crate::BlendMode::Alpha,
    );
    let textures = vec![
        solid_texture("minified-slot-red", [255, 0, 0, 255]),
        solid_texture(target_path, [0, 255, 0, 255]),
    ];
    let mut primitive =
        WowUiPrimitive::new_merged_with_textures(Arc::new(batch), textures, Vec::new());
    HeadlessRenderContext::new(7, 7).render(&mut primitive)
}

fn solid_texture(path: &str, rgba: [u8; 4]) -> GpuTextureData {
    GpuTextureData {
        path: path.into(),
        width: 64,
        height: 64,
        rgba: rgba.repeat(64 * 64).into(),
    }
}

fn assert_green_slot_without_bleed(image: &image::RgbaImage) {
    for y in 0..7 {
        for x in 0..7 {
            let px = image.get_pixel(x, y).0;
            assert!(
                px[0] <= 2 && px[2] <= 2 && px[1] >= 253,
                "minified green slot at ({x}, {y}) bled from red neighbor: {px:?}"
            );
        }
    }
}
