use std::borrow::Cow;
use std::collections::hash_map::Entry;

use iced::Rectangle;

use super::{
    CachedLayoutRun, GlyphAtlas, GlyphCacheEmitRequest, ShapeCacheEntry, TextShapeRequest,
    emit_glyphs_from_cache, extract_layout_runs, shape_cache_hash, shape_text_to_runs,
    shaped_text_total_height,
};
use crate::render::font::{WowFontSystem, line_height_for_font_size};
use crate::render::shader::QuadBatch;
use crate::widget::TextJustify;

/// Emit text quads into a QuadBatch.
///
/// Shapes the text, rasterizes glyphs into the atlas, and pushes textured quads.
/// The glyph atlas texture must be uploaded separately via `GlyphAtlas::texture_data()`.
#[allow(clippy::too_many_arguments)]
pub fn emit_text_quads(
    batch: &mut QuadBatch,
    font_system: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    text: &str,
    bounds: Rectangle,
    font_path: Option<&str>,
    font_size: f32,
    color: [f32; 4],
    justify_h: TextJustify,
    justify_v: TextJustify,
    glyph_tex_index: i32,
    shadow_color: Option<[f32; 4]>,
    shadow_offset: (f32, f32),
    outline: crate::widget::TextOutline,
    word_wrap: bool,
    max_lines: u32,
    spacing: f32,
    smooth_scaling: bool,
    pre_stripped: Option<&str>,
    alpha_gradient: Option<crate::widget::AlphaGradient>,
) {
    if !can_emit_text(text, bounds, font_size) {
        return;
    }

    let stripped = stripped_text(text, pre_stripped);
    if stripped.is_empty() {
        return;
    }

    let layout = cached_text_layout(
        glyph_atlas,
        font_system,
        TextLayoutCacheRequest {
            text: &stripped,
            font_path,
            font_size,
            bounds,
            word_wrap,
            max_lines,
            spacing,
            smooth_scaling,
        },
    );
    let pass = TextPassContext {
        runs: &layout.runs,
        bounds,
        y_offset: vertical_text_offset(justify_v, bounds.height, layout.total_height),
        justify_h,
        glyph_tex_index,
        alpha_gradient,
    };

    emit_outline_text_pass(batch, glyph_atlas, font_system, &pass, outline, color[3]);
    emit_shadow_text_pass(
        batch,
        glyph_atlas,
        font_system,
        &pass,
        shadow_color,
        shadow_offset,
    );
    emit_text_pass(batch, glyph_atlas, font_system, &pass, color, (0.0, 0.0));
}

/// Geometry settings and current paint for an already prepared full-content buffer.
pub(crate) struct PreparedBufferTextRequest<'a> {
    pub(crate) buffer: &'a cosmic_text::Buffer,
    pub(crate) bounds: Rectangle,
    pub(crate) spacing: f32,
    pub(crate) justify_h: TextJustify,
    pub(crate) justify_v: TextJustify,
    pub(crate) color: [f32; 4],
    pub(crate) glyph_tex_index: i32,
    pub(crate) shadow_color: Option<[f32; 4]>,
    pub(crate) shadow_offset: (f32, f32),
    pub(crate) outline: crate::widget::TextOutline,
}

/// Emit every prepared run without reshaping or limiting it to the viewport.
/// The caller clips the emitted quads after full-content alignment.
pub(crate) fn emit_prepared_buffer_text_quads(
    batch: &mut QuadBatch,
    font_system: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    request: PreparedBufferTextRequest<'_>,
) {
    let runs = extract_layout_runs(request.buffer, 0, request.spacing, glyph_atlas.raster_scale);
    let total_height = shaped_text_total_height(
        request.buffer,
        0,
        request.buffer.metrics().line_height,
        request.spacing,
    );
    let pass = TextPassContext {
        runs: &runs,
        bounds: request.bounds,
        y_offset: vertical_text_offset(request.justify_v, request.bounds.height, total_height),
        justify_h: request.justify_h,
        glyph_tex_index: request.glyph_tex_index,
        alpha_gradient: None,
    };
    paint_prepared_text(batch, glyph_atlas, font_system, &pass, &request);
}

fn paint_prepared_text(
    batch: &mut QuadBatch,
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    pass: &TextPassContext<'_>,
    request: &PreparedBufferTextRequest<'_>,
) {
    emit_outline_text_pass(
        batch,
        glyph_atlas,
        font_system,
        pass,
        request.outline,
        request.color[3],
    );
    emit_shadow_text_pass(
        batch,
        glyph_atlas,
        font_system,
        pass,
        request.shadow_color,
        request.shadow_offset,
    );
    emit_text_pass(
        batch,
        glyph_atlas,
        font_system,
        pass,
        request.color,
        (0.0, 0.0),
    );
}

fn can_emit_text(text: &str, bounds: Rectangle, font_size: f32) -> bool {
    !text.is_empty() && bounds.height > 0.0 && line_height_for_font_size(font_size, false).is_some()
}

fn stripped_text<'a>(text: &'a str, pre_stripped: Option<&'a str>) -> Cow<'a, str> {
    match pre_stripped {
        Some(stripped) => Cow::Borrowed(stripped),
        None => Cow::Owned(crate::render::strip_wow_markup(text)),
    }
}

struct CachedTextLayout {
    runs: Vec<CachedLayoutRun>,
    total_height: f32,
}

struct TextLayoutCacheRequest<'a> {
    text: &'a str,
    font_path: Option<&'a str>,
    font_size: f32,
    bounds: Rectangle,
    word_wrap: bool,
    max_lines: u32,
    spacing: f32,
    smooth_scaling: bool,
}

fn cached_text_layout(
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    request: TextLayoutCacheRequest<'_>,
) -> CachedTextLayout {
    let key = shape_cache_hash(
        request.text,
        request.font_path,
        request.font_size,
        shape_width_for_bounds(request.word_wrap, request.bounds.width),
        request.bounds.height,
        request.max_lines,
        request.spacing,
        request.smooth_scaling,
    );
    populate_text_layout_cache(glyph_atlas, font_system, key, request);
    let entry = glyph_atlas
        .shape_cache
        .get_mut(&key)
        .expect("text cache entry");
    entry.last_used = glyph_atlas.shape_cache_generation;
    CachedTextLayout {
        runs: entry.runs.clone(),
        total_height: entry.total_height,
    }
}

fn populate_text_layout_cache(
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    key: u64,
    request: TextLayoutCacheRequest<'_>,
) {
    let Entry::Vacant(entry) = glyph_atlas.shape_cache.entry(key) else {
        return;
    };
    let (buffer, total_height) = shape_text_to_runs(
        font_system,
        TextShapeRequest {
            text: request.text,
            font_path: request.font_path,
            font_size: request.font_size,
            bounds_width: request.bounds.width,
            bounds_height: request.bounds.height,
            word_wrap: request.word_wrap,
            max_lines: request.max_lines,
            spacing: request.spacing,
            smooth_scaling: request.smooth_scaling,
        },
    );
    entry.insert(ShapeCacheEntry {
        runs: extract_layout_runs(
            &buffer,
            request.max_lines,
            request.spacing,
            glyph_atlas.raster_scale,
        ),
        total_height,
        last_used: glyph_atlas.shape_cache_generation,
    });
}

fn shape_width_for_bounds(word_wrap: bool, bounds_width: f32) -> f32 {
    if word_wrap && bounds_width > 0.0 {
        bounds_width
    } else {
        10000.0
    }
}

fn vertical_text_offset(justify_v: TextJustify, bounds_height: f32, total_height: f32) -> f32 {
    match justify_v {
        TextJustify::Left => 0.0, // TOP
        TextJustify::Center => (bounds_height - total_height) / 2.0,
        TextJustify::Right => bounds_height - total_height, // BOTTOM
    }
}

struct TextPassContext<'a> {
    runs: &'a [CachedLayoutRun],
    bounds: Rectangle,
    y_offset: f32,
    justify_h: TextJustify,
    glyph_tex_index: i32,
    alpha_gradient: Option<crate::widget::AlphaGradient>,
}

fn emit_outline_text_pass(
    batch: &mut QuadBatch,
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    pass: &TextPassContext<'_>,
    outline: crate::widget::TextOutline,
    alpha: f32,
) {
    let Some(offsets) = outline_offsets(outline) else {
        return;
    };
    let outline_color = [0.0_f32, 0.0, 0.0, alpha];
    for offset in offsets {
        emit_text_pass(
            batch,
            glyph_atlas,
            font_system,
            pass,
            outline_color,
            *offset,
        );
    }
}

fn outline_offsets(outline: crate::widget::TextOutline) -> Option<&'static [(f32, f32); 8]> {
    match outline {
        crate::widget::TextOutline::None => None,
        crate::widget::TextOutline::Outline => Some(&OUTLINE_OFFSETS),
        crate::widget::TextOutline::ThickOutline => Some(&THICK_OUTLINE_OFFSETS),
    }
}

const OUTLINE_OFFSETS: [(f32, f32); 8] = [
    (-1.0, 0.0),
    (1.0, 0.0),
    (0.0, -1.0),
    (0.0, 1.0),
    (-1.0, -1.0),
    (1.0, -1.0),
    (-1.0, 1.0),
    (1.0, 1.0),
];

const THICK_OUTLINE_OFFSETS: [(f32, f32); 8] = [
    (-2.0, 0.0),
    (2.0, 0.0),
    (0.0, -2.0),
    (0.0, 2.0),
    (-2.0, -2.0),
    (2.0, -2.0),
    (-2.0, 2.0),
    (2.0, 2.0),
];

fn emit_shadow_text_pass(
    batch: &mut QuadBatch,
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    pass: &TextPassContext<'_>,
    shadow_color: Option<[f32; 4]>,
    shadow_offset: (f32, f32),
) {
    if let Some(color) = visible_shadow_color(shadow_color) {
        emit_text_pass(batch, glyph_atlas, font_system, pass, color, shadow_offset);
    }
}

fn visible_shadow_color(shadow_color: Option<[f32; 4]>) -> Option<[f32; 4]> {
    shadow_color.filter(|color| color[3] > 0.0)
}

fn emit_text_pass(
    batch: &mut QuadBatch,
    glyph_atlas: &mut GlyphAtlas,
    font_system: &mut WowFontSystem,
    pass: &TextPassContext<'_>,
    color: [f32; 4],
    offset: (f32, f32),
) {
    emit_glyphs_from_cache(
        batch,
        glyph_atlas,
        font_system,
        GlyphCacheEmitRequest {
            runs: pass.runs,
            bounds: pass.bounds,
            y_offset: pass.y_offset,
            justify_h: pass.justify_h,
            glyph_color: color,
            offset,
            glyph_tex_index: pass.glyph_tex_index,
            alpha_gradient: pass.alpha_gradient,
        },
    );
}
