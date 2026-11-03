use super::{
    GLYPH_ATLAS_SIZE, GlyphAtlas, PreparedBufferTextRequest, SwashContent, TextShapeRequest,
    build_glyph_entry, emit_prepared_buffer_text_quads, emit_text_quads, glyph_atlas_pixel_offset,
    shape_text_to_runs, subpixel_mask_alpha, write_glyph_pixels,
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
        0.0,
        false,
        None,
        None,
    );

    assert_eq!(batch.quad_count(), 0);
}

#[test]
fn glyph_raster_scale_uses_native_texels_without_resizing_layout() {
    let mut fonts = WowFontSystem::new();
    let bounds = Rectangle::new(Point::new(10.25, 20.25), Size::new(120.0, 40.0));
    let (buffer, expected_height) = shape_text_to_runs(
        &mut fonts,
        TextShapeRequest {
            text: "M",
            font_path: None,
            font_size: 16.0,
            bounds_width: bounds.width,
            bounds_height: bounds.height,
            word_wrap: false,
            max_lines: 0,
            spacing: 0.0,
            smooth_scaling: false,
        },
    );
    let logical_run = buffer.layout_runs().next().unwrap();

    let [base_span, native_span] = [1.0, 2.75].map(|scale| {
        let mut glyphs = GlyphAtlas::with_raster_scale(scale);
        let mut batch = QuadBatch::new();
        emit_text_quads(
            &mut batch,
            &mut fonts,
            &mut glyphs,
            "M",
            bounds,
            None,
            16.0,
            [1.0; 4],
            TextJustify::Left,
            TextJustify::Left,
            0,
            None,
            (0.0, 0.0),
            TextOutline::None,
            false,
            0,
            0.0,
            false,
            None,
            None,
        );

        let layout = glyphs.shape_cache.values().next().unwrap();
        assert_eq!(layout.total_height, expected_height);
        assert_eq!(layout.runs[0].line_y, logical_run.line_y);
        assert_eq!(layout.runs[0].line_w, logical_run.line_w);
        let (key, glyph) = glyphs.entries.iter().next().unwrap();
        assert_eq!(f32::from_bits(key.font_size_bits), 16.0 * scale);
        assert_eq!(batch.quad_count(), 1);

        let top_left = batch.vertices[0];
        let bottom_right = batch.vertices[2];
        let raster_span = [glyph.width as f32, glyph.height as f32];
        for (axis, texels) in raster_span.into_iter().enumerate() {
            let uv_span = bottom_right.tex_coords[axis] - top_left.tex_coords[axis];
            assert_eq!(uv_span * GLYPH_ATLAS_SIZE as f32, texels);
            let layout_span = bottom_right.position[axis] - top_left.position[axis];
            assert!((layout_span * scale - texels).abs() < 0.001);
            if scale != 1.0 {
                let start = top_left.position[axis] * scale;
                assert!((start - start.round()).abs() < 0.001);
            }
        }
        if scale == 1.0 {
            assert_eq!(top_left.position[0].fract(), 0.25);
        }

        let mut prepared = QuadBatch::new();
        emit_prepared_buffer_text_quads(
            &mut prepared,
            &mut fonts,
            &mut glyphs,
            PreparedBufferTextRequest {
                buffer: &buffer,
                bounds,
                spacing: 0.0,
                justify_h: TextJustify::Left,
                justify_v: TextJustify::Left,
                color: [1.0; 4],
                glyph_tex_index: 0,
                shadow_color: None,
                shadow_offset: (0.0, 0.0),
                outline: TextOutline::None,
            },
        );
        assert_eq!(prepared.quad_count(), batch.quad_count());
        for (prepared, cached) in prepared.vertices.iter().zip(batch.vertices) {
            assert_eq!(prepared.position, cached.position);
            assert_eq!(prepared.tex_coords, cached.tex_coords);
        }
        raster_span
    });
    assert!(native_span[0] > base_span[0] * 2.0);
    assert!(native_span[1] > base_span[1] * 2.0);

    let glyph = &logical_run.glyphs[0];
    let oversized_scale = (GLYPH_ATLAS_SIZE * 2) as f32 / glyph.font_size;
    let oversized_key = glyph.physical((0.0, 0.0), oversized_scale).cache_key;
    let mut oversized_atlas = GlyphAtlas::with_raster_scale(oversized_scale);
    assert!(
        oversized_atlas
            .ensure_glyph(&mut fonts, oversized_key)
            .is_none()
    );
    assert!(!fonts.swash_cache.image_cache.contains_key(&oversized_key));
}

fn blank_atlas_pixels() -> Vec<u8> {
    vec![0; (GLYPH_ATLAS_SIZE * GLYPH_ATLAS_SIZE * 4) as usize]
}

fn rgba_at(pixels: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = glyph_atlas_pixel_offset(x, y);
    pixels[offset..offset + 4].try_into().expect("rgba pixel")
}
