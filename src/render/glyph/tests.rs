use super::{
    GLYPH_ATLAS_SIZE, GlyphAtlas, SwashContent, build_glyph_entry, emit_text_quads,
    glyph_atlas_pixel_offset, subpixel_mask_alpha, write_glyph_pixels,
};
use crate::font::WowFontSystem;
use crate::render::shader::QuadBatch;
use crate::widget::{TextJustify, TextOutline};
use iced::{Point, Rectangle, Size};

#[test]
fn subpixel_mask_alpha_averages_rgb_channels() {
    assert_eq!(subpixel_mask_alpha(0, 0, 0), 0);
    assert_eq!(subpixel_mask_alpha(255, 255, 255), 255);
    assert_eq!(subpixel_mask_alpha(255, 0, 0), 85);
    assert_eq!(subpixel_mask_alpha(0, 255, 128), 127);
}

#[test]
fn glyph_entry_keeps_left_and_top_placement_offsets() {
    let entry = build_glyph_entry(8, 16, 20, 30, 4, 7);
    assert_eq!(entry.left, 4);
    assert_eq!(entry.top, 7);
}

#[test]
fn write_mask_glyph_pixels_writes_white_with_source_alpha() {
    let mut pixels = blank_atlas_pixels();
    write_glyph_pixels(&mut pixels, 2, 3, 2, 1, &[12, 240], SwashContent::Mask);

    assert_eq!(rgba_at(&pixels, 2, 3), [255, 255, 255, 12]);
    assert_eq!(rgba_at(&pixels, 3, 3), [255, 255, 255, 240]);
}

#[test]
fn write_color_glyph_pixels_copies_source_rgba() {
    let mut pixels = blank_atlas_pixels();
    write_glyph_pixels(
        &mut pixels,
        4,
        5,
        1,
        2,
        &[10, 20, 30, 40, 50, 60, 70, 80],
        SwashContent::Color,
    );

    assert_eq!(rgba_at(&pixels, 4, 5), [10, 20, 30, 40]);
    assert_eq!(rgba_at(&pixels, 4, 6), [50, 60, 70, 80]);
}

#[test]
fn write_subpixel_glyph_pixels_writes_white_with_averaged_alpha() {
    let mut pixels = blank_atlas_pixels();
    write_glyph_pixels(
        &mut pixels,
        6,
        7,
        1,
        2,
        &[255, 0, 0, 0, 255, 128],
        SwashContent::SubpixelMask,
    );

    assert_eq!(rgba_at(&pixels, 6, 7), [255, 255, 255, 85]);
    assert_eq!(rgba_at(&pixels, 6, 8), [255, 255, 255, 127]);
}

#[test]
fn zero_font_size_emits_no_text_quads() {
    let mut batch = QuadBatch::new();
    let mut fonts = WowFontSystem::new();
    let mut glyphs = GlyphAtlas::new();

    emit_text_quads(
        &mut batch,
        &mut fonts,
        &mut glyphs,
        "Collections",
        Rectangle::new(Point::ORIGIN, Size::new(120.0, 20.0)),
        None,
        0.0,
        [1.0, 1.0, 1.0, 1.0],
        TextJustify::Left,
        TextJustify::Center,
        0,
        None,
        (0.0, 0.0),
        TextOutline::None,
        false,
        0,
        None,
    );

    assert_eq!(batch.quad_count(), 0);
}

#[test]
fn glyph_quads_land_on_whole_pixels() {
    // Frame origin, line position and shadow offset are fractional at any UI
    // scale; a glyph bitmap drawn at a fractional position is smeared over
    // two pixels by the bilinear glyph sampler.
    let mut batch = QuadBatch::new();
    let mut fonts = WowFontSystem::new();
    let mut glyphs = GlyphAtlas::new();

    emit_text_quads(
        &mut batch,
        &mut fonts,
        &mut glyphs,
        "Quests",
        Rectangle::new(Point::new(10.3, 5.7), Size::new(200.0, 40.0)),
        None,
        14.0,
        [1.0, 1.0, 1.0, 1.0],
        TextJustify::Left,
        TextJustify::Center,
        0,
        Some([0.0, 0.0, 0.0, 1.0]),
        (1.6875, -1.6875),
        TextOutline::None,
        false,
        0,
        None,
    );

    assert!(batch.quad_count() > 0, "the text must render");
    for vertex in &batch.vertices {
        let [x, y] = vertex.position;
        assert_eq!(x, x.round(), "glyph quad x {x} is not on the pixel grid");
        assert_eq!(y, y.round(), "glyph quad y {y} is not on the pixel grid");
    }
}

fn blank_atlas_pixels() -> Vec<u8> {
    vec![0; (GLYPH_ATLAS_SIZE * GLYPH_ATLAS_SIZE * 4) as usize]
}

fn rgba_at(pixels: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = glyph_atlas_pixel_offset(x, y);
    pixels[offset..offset + 4].try_into().expect("rgba pixel")
}
