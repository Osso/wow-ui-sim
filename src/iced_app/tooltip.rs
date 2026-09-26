//! Tooltip visual rendering — sizing, data collection, and quad emission.

use std::collections::HashMap;
use std::sync::OnceLock;

use iced::{Point, Rectangle, Size};

use crate::atlas::{NineSliceAtlasInfo, get_nine_slice_atlas_info};
use crate::lua_api::SimState;
use crate::render::QuadBatch;
use crate::render::font::WowFontSystem;
use crate::render::glyph::{GlyphAtlas, emit_text_quads};
use crate::render::shader::GLYPH_ATLAS_TEX_INDEX;
use crate::widget::{TextJustify, TextOutline};

/// Cached nine-slice atlas info for the default tooltip border.
fn tooltip_nine_slice() -> Option<&'static NineSliceAtlasInfo> {
    static CACHE: OnceLock<Option<NineSliceAtlasInfo>> = OnceLock::new();
    CACHE
        .get_or_init(|| get_nine_slice_atlas_info("Tooltip"))
        .as_ref()
}

const TOOLTIP_PADDING_H: f32 = 12.0;
const TOOLTIP_PADDING_V: f32 = 12.0;
const TOOLTIP_LINE_SPACING: f32 = 2.0;
const TOOLTIP_HEADER_FONT_SIZE: f32 = 14.0;
const TOOLTIP_BODY_FONT_SIZE: f32 = 12.0;
const DOUBLE_LINE_GAP: f32 = 20.0;
const TOOLTIP_CENTER_OVERLAP: f32 = 4.0;

/// Pre-collected tooltip render data for a single tooltip frame.
pub struct TooltipRenderData {
    pub lines: Vec<TooltipLineRender>,
    pub line_spacing: f32,
}

/// A single line ready for rendering.
pub struct TooltipLineRender {
    pub left_text: String,
    pub left_color: [f32; 4],
    pub left_segments: Vec<TooltipTextSegmentRender>,
    pub right_text: Option<String>,
    pub right_color: [f32; 4],
    pub right_segments: Vec<TooltipTextSegmentRender>,
    pub font_size: f32,
    pub wrap: bool,
    /// Measured height for this line (accounts for word-wrap).
    pub measured_height: f32,
}

pub struct TooltipTextSegmentRender {
    pub text: String,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TooltipTextInsets {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

/// Update tooltip frame sizes based on their text content.
///
/// Must be called before layout computation so anchors resolve with correct dimensions.
pub fn update_tooltip_sizes(state: &mut SimState, font_system: &mut WowFontSystem) {
    let tooltip_ids: Vec<u64> = state.tooltips.keys().copied().collect();
    for id in tooltip_ids {
        let (has_render_content, visible) = {
            let td = match state.tooltips.get(&id) {
                Some(td) => td,
                None => continue,
            };
            let visible = state.widgets.get(id).map(|f| f.visible).unwrap_or(false);
            (tooltip_has_render_content(td), visible)
        };
        if !has_render_content || !visible {
            continue;
        }
        let (width, height) = measure_tooltip(state, id, font_system);
        let changed = if let Some(frame) = state.widgets.get_mut_visual(id) {
            let changed = frame.width != width || frame.height != height;
            frame.width = width;
            frame.height = height;
            changed
        } else {
            false
        };
        if changed {
            state.widgets.mark_rect_dirty(id);
        }
    }
}

/// Measure a tooltip's required width and height from its lines.
///
/// Two-pass approach: first determine width from non-wrapping lines,
/// then measure wrapped lines' heights against that width.
fn measure_tooltip(state: &SimState, id: u64, font_system: &mut WowFontSystem) -> (f32, f32) {
    let td = match state.tooltips.get(&id) {
        Some(td) => td,
        None => return (0.0, 0.0),
    };

    let line_spacing = td.line_spacing.unwrap_or(TOOLTIP_LINE_SPACING);

    // Pass 1: determine content width from non-wrapping lines.
    let content_width = measure_tooltip_content_width(td, font_system);

    // Pass 2: sum line heights, using text measurement for wrapped lines.
    let mut total_height: f32 = 0.0;
    for (i, line) in td.lines.iter().enumerate() {
        let font_size = tooltip_line_font_size(i);
        let line_height = if line.wrap && !line.left_text.is_empty() {
            measure_tooltip_text_height(
                font_system,
                &line.left_text,
                font_size,
                Some(content_width),
            )
        } else {
            (font_size * 1.2).ceil()
        };
        if i > 0 {
            total_height += line_spacing;
        }
        total_height += line_height;
    }

    let insets = tooltip_text_insets();
    let width = content_width + insets.left + insets.right;
    let height = total_height + insets.top + insets.bottom;
    (width, height)
}

/// Measure the content width from non-wrapping tooltip lines.
/// Wrapping lines don't contribute to width — they wrap within it.
fn measure_tooltip_content_width(
    td: &crate::lua_api::tooltip::TooltipData,
    font_system: &mut WowFontSystem,
) -> f32 {
    let mut max_width: f32 = td.min_width;
    let mut wrapped_only_max_width: f32 = 0.0;
    let mut measured_non_wrapped_line = false;
    if tooltip_has_wrapped_lines(td) {
        max_width = max_width.max(tooltip_wrapped_min_width(td));
    }
    for (i, line) in td.lines.iter().enumerate() {
        let font_size = tooltip_line_font_size(i);
        let left_w = measure_tooltip_text_width(font_system, &line.left_text, font_size);
        let right_w = line
            .right_text
            .as_ref()
            .map(|t| measure_tooltip_text_width(font_system, t, font_size))
            .unwrap_or(0.0);
        let line_width = if right_w > 0.0 {
            left_w + right_w + DOUBLE_LINE_GAP
        } else {
            left_w
        };
        let has_width_text = tooltip_line_has_width_text(line);
        if line.wrap && td.shrink_to_fit_wrapped {
            if has_width_text {
                wrapped_only_max_width = wrapped_only_max_width.max(line_width);
            }
            continue;
        }
        if !has_width_text {
            continue;
        }
        measured_non_wrapped_line = true;
        max_width = max_width.max(line_width);
    }
    if !measured_non_wrapped_line {
        max_width = max_width.max(wrapped_only_max_width);
    }
    max_width
}

fn tooltip_wrapped_min_width(td: &crate::lua_api::tooltip::TooltipData) -> f32 {
    td.custom_word_wrap_min_width.unwrap_or(0.0)
}

fn tooltip_has_wrapped_lines(td: &crate::lua_api::tooltip::TooltipData) -> bool {
    td.lines.iter().any(|line| line.wrap)
}

fn tooltip_line_has_width_text(line: &crate::lua_api::tooltip::TooltipLine) -> bool {
    !tooltip_visible_text(&line.left_text).trim().is_empty()
        || line
            .right_text
            .as_ref()
            .is_some_and(|text| !tooltip_visible_text(text).trim().is_empty())
}

fn measure_tooltip_text_width(font_system: &mut WowFontSystem, text: &str, font_size: f32) -> f32 {
    font_system.measure_text_width(&tooltip_visible_text(text), None, font_size)
}

fn measure_tooltip_text_height(
    font_system: &mut WowFontSystem,
    text: &str,
    font_size: f32,
    wrap_width: Option<f32>,
) -> f32 {
    font_system.measure_text_height(
        &tooltip_visible_text(text),
        None,
        font_size,
        wrap_width,
        0.0,
    )
}

fn tooltip_visible_text(text: &str) -> String {
    crate::render::strip_wow_markup(text)
}

/// Collect render data for all visible tooltips with lines.
pub fn collect_tooltip_data(state: &SimState) -> HashMap<u64, TooltipRenderData> {
    let mut result = HashMap::new();
    for (&id, td) in &state.tooltips {
        let Some(alpha) = tooltip_alpha(state, id, td) else {
            continue;
        };
        let lines = collect_tooltip_lines(td, alpha);
        let line_spacing = td.line_spacing.unwrap_or(TOOLTIP_LINE_SPACING);
        result.insert(
            id,
            TooltipRenderData {
                lines,
                line_spacing,
            },
        );
    }
    result
}

fn tooltip_alpha(
    state: &SimState,
    id: u64,
    td: &crate::lua_api::tooltip::TooltipData,
) -> Option<f32> {
    if !tooltip_has_render_content(td) {
        return None;
    }

    let frame = state.widgets.get(id)?;
    if !frame.visible {
        return None;
    }

    Some(frame.alpha)
}

fn tooltip_has_render_content(td: &crate::lua_api::tooltip::TooltipData) -> bool {
    !td.lines.is_empty() || td.allow_show_with_no_lines
}

fn collect_tooltip_lines(
    td: &crate::lua_api::tooltip::TooltipData,
    alpha: f32,
) -> Vec<TooltipLineRender> {
    td.lines
        .iter()
        .enumerate()
        .map(|(i, line)| tooltip_line_render(i, line, alpha))
        .collect()
}

fn tooltip_line_render(
    index: usize,
    line: &crate::lua_api::tooltip::TooltipLine,
    alpha: f32,
) -> TooltipLineRender {
    let left_color = [
        line.left_color.0,
        line.left_color.1,
        line.left_color.2,
        alpha,
    ];
    let right_color = [
        line.right_color.0,
        line.right_color.1,
        line.right_color.2,
        alpha,
    ];
    TooltipLineRender {
        left_text: line.left_text.clone(),
        left_color,
        left_segments: tooltip_text_segment_renders(&line.left_segments, alpha),
        right_text: line.right_text.clone(),
        right_color,
        right_segments: tooltip_text_segment_renders(&line.right_segments, alpha),
        font_size: tooltip_line_font_size(index),
        wrap: line.wrap,
        measured_height: (tooltip_line_font_size(index) * 1.2).ceil(),
    }
}

fn tooltip_text_segment_renders(
    segments: &[crate::lua_api::tooltip::TooltipTextSegment],
    alpha: f32,
) -> Vec<TooltipTextSegmentRender> {
    segments
        .iter()
        .map(|segment| TooltipTextSegmentRender {
            text: segment.text.clone(),
            color: [segment.color.0, segment.color.1, segment.color.2, alpha],
        })
        .collect()
}

fn tooltip_line_font_size(index: usize) -> f32 {
    if index == 0 {
        TOOLTIP_HEADER_FONT_SIZE
    } else {
        TOOLTIP_BODY_FONT_SIZE
    }
}

/// Emit quads for a GameTooltip frame: background, border, and text lines.
pub fn build_tooltip_quads(
    tooltip: TooltipRender<'_>,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
) {
    let data = tooltip.tooltip_data.and_then(|map| map.get(&tooltip.id));
    let Some(data) = data else { return };

    if tooltip.draw_background {
        emit_tooltip_background(tooltip.batch, tooltip.bounds, tooltip.eff_alpha);
    } else {
        emit_tooltip_center_fill(tooltip.batch, tooltip.bounds, tooltip.eff_alpha);
    }

    let Some((font_sys, glyph_atlas)) = text_ctx else {
        return;
    };
    let mut text_renderer = TooltipTextRenderer {
        batch: tooltip.batch,
        font_sys,
        glyph_atlas,
    };
    emit_tooltip_lines(&mut text_renderer, data, tooltip.bounds, tooltip.eff_scale);
}

/// Render tooltip background: nine-slice border with black fill, or solid fallback.
fn emit_tooltip_background(batch: &mut QuadBatch, bounds: Rectangle, alpha: f32) {
    if let Some(ns) = tooltip_nine_slice() {
        let center = [0.0, 0.0, 0.0, alpha];
        super::nine_slice::emit_nine_slice_with_center_color(batch, bounds, ns, alpha, center, 4.0);
    } else {
        batch.push_solid(bounds, [0.0, 0.0, 0.0, alpha]);
        batch.push_border(bounds, 1.0, [0.6, 0.5, 0.15, alpha]);
    }
}

fn emit_tooltip_center_fill(batch: &mut QuadBatch, bounds: Rectangle, alpha: f32) {
    let center_color = [0.0, 0.0, 0.0, alpha];
    let Some(ns) = tooltip_nine_slice() else {
        batch.push_solid(bounds, center_color);
        return;
    };

    let x = bounds.x + ns.corner_tl.width as f32;
    let y = bounds.y + ns.corner_tl.height as f32;
    let width = bounds.width - ns.corner_tl.width as f32 - ns.corner_tr.width as f32;
    let height = bounds.height - ns.corner_tl.height as f32 - ns.corner_bl.height as f32;
    if width > 0.0 && height > 0.0 {
        batch.push_solid(
            Rectangle::new(Point::new(x, y), Size::new(width, height)),
            center_color,
        );
    }
}

/// Render tooltip text lines with wrapping-aware height measurement.
fn emit_tooltip_lines(
    text_renderer: &mut TooltipTextRenderer<'_>,
    data: &TooltipRenderData,
    bounds: Rectangle,
    eff_scale: f32,
) {
    let insets = scaled_tooltip_text_insets(eff_scale);
    let content_x = bounds.x + insets.left;
    let content_width = bounds.width - insets.left - insets.right;
    let mut y = bounds.y + insets.top;

    for line in &data.lines {
        let font_size = scaled_tooltip_font_size(line.font_size, eff_scale);
        let line_height = if line.wrap && !line.left_text.is_empty() {
            text_renderer.font_sys.measure_text_height(
                &line.left_text,
                None,
                font_size,
                Some(content_width),
                0.0,
            )
        } else {
            line.measured_height * eff_scale
        };

        emit_tooltip_line(
            text_renderer,
            line,
            TooltipLinePlacement {
                x: content_x,
                y,
                width: content_width,
                height: line_height,
            },
            eff_scale,
        );

        y += line_height + data.line_spacing * eff_scale;
    }
}

fn tooltip_text_insets() -> TooltipTextInsets {
    match tooltip_nine_slice() {
        Some(ns) => TooltipTextInsets {
            left: TOOLTIP_PADDING_H + tooltip_center_inset(ns.corner_tl.width),
            right: TOOLTIP_PADDING_H + tooltip_center_inset(ns.corner_tr.width),
            top: TOOLTIP_PADDING_V + tooltip_center_inset(ns.corner_tl.height),
            bottom: TOOLTIP_PADDING_V + tooltip_center_inset(ns.corner_bl.height),
        },
        None => TooltipTextInsets {
            left: TOOLTIP_PADDING_H,
            right: TOOLTIP_PADDING_H,
            top: TOOLTIP_PADDING_V,
            bottom: TOOLTIP_PADDING_V,
        },
    }
}

fn scaled_tooltip_text_insets(eff_scale: f32) -> TooltipTextInsets {
    let insets = tooltip_text_insets();
    TooltipTextInsets {
        left: insets.left * eff_scale,
        right: insets.right * eff_scale,
        top: insets.top * eff_scale,
        bottom: insets.bottom * eff_scale,
    }
}

fn scaled_tooltip_font_size(font_size: f32, eff_scale: f32) -> f32 {
    font_size * eff_scale
}

fn tooltip_center_inset(piece_extent: u32) -> f32 {
    (piece_extent as f32 - TOOLTIP_CENTER_OVERLAP).max(0.0)
}

pub struct TooltipRender<'a> {
    pub batch: &'a mut QuadBatch,
    pub bounds: Rectangle,
    pub tooltip_data: Option<&'a HashMap<u64, TooltipRenderData>>,
    pub id: u64,
    pub eff_alpha: f32,
    pub eff_scale: f32,
    pub draw_background: bool,
}

struct TooltipTextRenderer<'a> {
    batch: &'a mut QuadBatch,
    font_sys: &'a mut WowFontSystem,
    glyph_atlas: &'a mut GlyphAtlas,
}

impl TooltipTextRenderer<'_> {
    fn emit(
        &mut self,
        text: &str,
        bounds: Rectangle,
        justify: TextJustify,
        font_size: f32,
        color: [f32; 4],
        wrap: bool,
    ) {
        let run = TooltipTextRun {
            text,
            bounds,
            justify,
            font_size,
            color,
            wrap,
        };
        emit_tooltip_text_run(self.batch, self.font_sys, self.glyph_atlas, run);
    }
}

/// Emit quads for a single tooltip line (left text, optional right text).
fn emit_tooltip_line(
    tr: &mut TooltipTextRenderer<'_>,
    line: &TooltipLineRender,
    placement: TooltipLinePlacement,
    eff_scale: f32,
) {
    let font_size = scaled_tooltip_font_size(line.font_size, eff_scale);
    let right_width = line
        .right_text
        .as_ref()
        .map(|t| measure_tooltip_text_width(tr.font_sys, t, font_size));

    let left_width = match right_width {
        Some(rw) if rw > 0.0 => (placement.width - rw - DOUBLE_LINE_GAP).max(0.0),
        _ => placement.width,
    };
    emit_left_tooltip_text(tr, line, placement, left_width, font_size);

    if let Some(ref right_text) = line.right_text {
        emit_right_tooltip_text(tr, line, placement, left_width, right_text, font_size);
    }
}

fn emit_left_tooltip_text(
    tr: &mut TooltipTextRenderer<'_>,
    line: &TooltipLineRender,
    placement: TooltipLinePlacement,
    left_width: f32,
    font_size: f32,
) {
    let left_bounds = tooltip_line_bounds(placement.x, placement.y, left_width, placement.height);
    emit_tooltip_text_segments(
        tr,
        TooltipTextSegments {
            text: &line.left_text,
            segments: &line.left_segments,
            bounds: left_bounds,
            justify: TextJustify::Left,
            font_size,
            color: line.left_color,
            wrap: line.wrap,
        },
    );
}

fn emit_right_tooltip_text(
    tr: &mut TooltipTextRenderer<'_>,
    line: &TooltipLineRender,
    placement: TooltipLinePlacement,
    left_width: f32,
    right_text: &str,
    font_size: f32,
) {
    let right_x = placement.x + left_width + DOUBLE_LINE_GAP;
    let right_w = (placement.width - left_width - DOUBLE_LINE_GAP).max(0.0);
    let right_bounds = tooltip_line_bounds(right_x, placement.y, right_w, placement.height);
    emit_tooltip_text_segments(
        tr,
        TooltipTextSegments {
            text: right_text,
            segments: &line.right_segments,
            bounds: right_bounds,
            justify: TextJustify::Right,
            font_size,
            color: line.right_color,
            wrap: false,
        },
    );
}

#[derive(Clone, Copy)]
struct TooltipTextSegments<'a> {
    text: &'a str,
    segments: &'a [TooltipTextSegmentRender],
    bounds: Rectangle,
    justify: TextJustify,
    font_size: f32,
    color: [f32; 4],
    wrap: bool,
}

struct TooltipSegmentFlow {
    x: f32,
    y: f32,
    line_height: f32,
    right: f32,
}

fn emit_tooltip_text_segments(
    tr: &mut TooltipTextRenderer<'_>,
    segment_run: TooltipTextSegments<'_>,
) {
    if segment_run.segments.is_empty() {
        emit_unsegmented_tooltip_text(tr, segment_run);
        return;
    }

    let mut flow = tooltip_segment_flow(tr, segment_run);
    for segment in segment_run.segments {
        emit_tooltip_segment_chunks(tr, segment_run, segment, &mut flow);
    }
}

fn emit_unsegmented_tooltip_text(
    tr: &mut TooltipTextRenderer<'_>,
    segment_run: TooltipTextSegments<'_>,
) {
    tr.emit(
        segment_run.text,
        segment_run.bounds,
        segment_run.justify,
        segment_run.font_size,
        segment_run.color,
        segment_run.wrap,
    );
}

fn tooltip_segment_flow(
    tr: &mut TooltipTextRenderer<'_>,
    segment_run: TooltipTextSegments<'_>,
) -> TooltipSegmentFlow {
    let line_height = (segment_run.font_size * 1.2).ceil();
    let total_width =
        measure_tooltip_segments_width(tr, segment_run.segments, segment_run.font_size);
    let x = tooltip_segment_start_x(
        segment_run.bounds,
        segment_run.justify,
        total_width,
        segment_run.wrap,
    );
    let right = segment_run.bounds.x + segment_run.bounds.width;

    TooltipSegmentFlow {
        x,
        y: segment_run.bounds.y,
        line_height,
        right,
    }
}

fn emit_tooltip_segment_chunks(
    tr: &mut TooltipTextRenderer<'_>,
    segment_run: TooltipTextSegments<'_>,
    segment: &TooltipTextSegmentRender,
    flow: &mut TooltipSegmentFlow,
) {
    for chunk in tooltip_segment_chunks(&segment.text, segment_run.wrap) {
        let chunk_width = tr
            .font_sys
            .measure_text_width(chunk, None, segment_run.font_size);
        if starts_wrapped_tooltip_segment_line(
            segment_run.wrap,
            chunk,
            flow.x,
            chunk_width,
            segment_run.bounds.x,
            flow.right,
        ) {
            flow.x = segment_run.bounds.x;
            flow.y += flow.line_height;
        }
        let chunk_bounds = tooltip_line_bounds(
            flow.x,
            flow.y,
            chunk_width.max(1.0),
            flow.line_height.min(segment_run.bounds.height),
        );
        tr.emit(
            chunk,
            chunk_bounds,
            TextJustify::Left,
            segment_run.font_size,
            segment.color,
            false,
        );
        flow.x += chunk_width;
    }
}

fn measure_tooltip_segments_width(
    tr: &mut TooltipTextRenderer<'_>,
    segments: &[TooltipTextSegmentRender],
    font_size: f32,
) -> f32 {
    segments
        .iter()
        .map(|segment| {
            tr.font_sys
                .measure_text_width(&segment.text, None, font_size)
        })
        .sum()
}

fn tooltip_segment_start_x(
    bounds: Rectangle,
    justify: TextJustify,
    total_width: f32,
    wrap: bool,
) -> f32 {
    if wrap {
        return bounds.x;
    }
    match justify {
        TextJustify::Center => bounds.x + ((bounds.width - total_width) / 2.0).max(0.0),
        TextJustify::Right => bounds.x + (bounds.width - total_width).max(0.0),
        _ => bounds.x,
    }
}

fn tooltip_segment_chunks(text: &str, wrap: bool) -> Box<dyn Iterator<Item = &str> + '_> {
    if wrap {
        Box::new(text.split_inclusive(char::is_whitespace))
    } else {
        Box::new(std::iter::once(text))
    }
}

fn starts_wrapped_tooltip_segment_line(
    wrap: bool,
    chunk: &str,
    x: f32,
    chunk_width: f32,
    left: f32,
    right: f32,
) -> bool {
    wrap && x > left && x + chunk_width > right && !chunk.trim().is_empty()
}

#[derive(Clone, Copy)]
struct TooltipLinePlacement {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct TooltipTextRun<'a> {
    text: &'a str,
    bounds: Rectangle,
    justify: TextJustify,
    font_size: f32,
    color: [f32; 4],
    wrap: bool,
}

fn tooltip_line_bounds(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
    Rectangle::new(Point::new(x, y), Size::new(width, height))
}

fn emit_tooltip_text_run(
    batch: &mut QuadBatch,
    font_sys: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    run: TooltipTextRun<'_>,
) {
    emit_text_quads(
        batch,
        font_sys,
        glyph_atlas,
        run.text,
        run.bounds,
        None,
        run.font_size,
        run.color,
        run.justify,
        TextJustify::Center,
        GLYPH_ATLAS_TEX_INDEX,
        None,
        (0.0, 0.0),
        TextOutline::None,
        run.wrap,
        0,
        0.0,
        None,
    );
}

#[cfg(test)]
#[path = "tooltip_tests.rs"]
mod tooltip_tests;
