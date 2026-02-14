//! Tooltip visual rendering — sizing, data collection, and quad emission.

use std::collections::HashMap;
use std::sync::OnceLock;

use iced::{Point, Rectangle, Size};

use crate::atlas::{get_nine_slice_atlas_info, NineSliceAtlasInfo};
use crate::lua_api::SimState;
use crate::render::font::WowFontSystem;
use crate::render::glyph::{emit_text_quads, GlyphAtlas};
use crate::render::shader::GLYPH_ATLAS_TEX_INDEX;
use crate::render::QuadBatch;
use crate::widget::{TextJustify, TextOutline};

/// Cached nine-slice atlas info for the default tooltip border.
fn tooltip_nine_slice() -> Option<&'static NineSliceAtlasInfo> {
    static CACHE: OnceLock<Option<NineSliceAtlasInfo>> = OnceLock::new();
    CACHE.get_or_init(|| get_nine_slice_atlas_info("Tooltip")).as_ref()
}

const TOOLTIP_PADDING_H: f32 = 12.0;
const TOOLTIP_PADDING_V: f32 = 12.0;
const TOOLTIP_LINE_SPACING: f32 = 2.0;
const TOOLTIP_HEADER_FONT_SIZE: f32 = 14.0;
const TOOLTIP_BODY_FONT_SIZE: f32 = 12.0;

/// Pre-collected tooltip render data for a single tooltip frame.
pub struct TooltipRenderData {
    pub lines: Vec<TooltipLineRender>,
}

/// A single line ready for rendering.
pub struct TooltipLineRender {
    pub left_text: String,
    pub left_color: [f32; 4],
    pub right_text: Option<String>,
    pub right_color: [f32; 4],
    pub font_size: f32,
    pub wrap: bool,
}

/// Update tooltip frame sizes based on their text content.
///
/// Must be called before layout computation so anchors resolve with correct dimensions.
pub fn update_tooltip_sizes(state: &mut SimState, font_system: &mut WowFontSystem) {
    let tooltip_ids: Vec<u64> = state.tooltips.keys().copied().collect();
    for id in tooltip_ids {
        let (lines_empty, visible) = {
            let td = match state.tooltips.get(&id) {
                Some(td) => td,
                None => continue,
            };
            let visible = state.widgets.get(id).map(|f| f.visible).unwrap_or(false);
            (td.lines.is_empty(), visible)
        };
        if lines_empty || !visible {
            continue;
        }
        let (width, height) = measure_tooltip(state, id, font_system);
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.width = width;
            frame.height = height;
        }
    }
}

/// Measure a tooltip's required width and height from its lines.
fn measure_tooltip(state: &SimState, id: u64, font_system: &mut WowFontSystem) -> (f32, f32) {
    let td = match state.tooltips.get(&id) {
        Some(td) => td,
        None => return (0.0, 0.0),
    };

    let mut max_width: f32 = td.min_width;
    let mut total_height: f32 = 0.0;

    for (i, line) in td.lines.iter().enumerate() {
        let font_size = if i == 0 { TOOLTIP_HEADER_FONT_SIZE } else { TOOLTIP_BODY_FONT_SIZE };
        let left_w = font_system.measure_text_width(&line.left_text, None, font_size);
        let right_w = line.right_text.as_ref()
            .map(|t| font_system.measure_text_width(t, None, font_size))
            .unwrap_or(0.0);

        let line_width = if right_w > 0.0 {
            left_w + right_w + 20.0 // gap between left and right
        } else {
            left_w
        };
        max_width = max_width.max(line_width);

        let line_height = (font_size * 1.2).ceil();
        if i > 0 {
            total_height += TOOLTIP_LINE_SPACING;
        }
        total_height += line_height;
    }

    let width = max_width + TOOLTIP_PADDING_H * 2.0;
    let height = total_height + TOOLTIP_PADDING_V * 2.0;
    (width, height)
}

/// Collect render data for all visible tooltips with lines.
pub fn collect_tooltip_data(
    state: &SimState,
) -> HashMap<u64, TooltipRenderData> {
    let mut result = HashMap::new();
    for (&id, td) in &state.tooltips {
        if td.lines.is_empty() {
            continue;
        }
        let visible = state.widgets.get(id).map(|f| f.visible).unwrap_or(false);
        if !visible {
            continue;
        }
        let alpha = state.widgets.get(id).map(|f| f.alpha).unwrap_or(1.0);
        let lines = td.lines.iter().enumerate().map(|(i, line)| {
            let font_size = if i == 0 { TOOLTIP_HEADER_FONT_SIZE } else { TOOLTIP_BODY_FONT_SIZE };
            TooltipLineRender {
                left_text: line.left_text.clone(),
                left_color: [line.left_color.0, line.left_color.1, line.left_color.2, alpha],
                right_text: line.right_text.clone(),
                right_color: [line.right_color.0, line.right_color.1, line.right_color.2, alpha],
                font_size,
                wrap: line.wrap,
            }
        }).collect();
        result.insert(id, TooltipRenderData { lines });
    }
    result
}

/// Emit quads for a GameTooltip frame: background, border, and text lines.
#[allow(clippy::too_many_arguments)]
pub fn build_tooltip_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    _f: &crate::widget::Frame,
    tooltip_data: Option<&HashMap<u64, TooltipRenderData>>,
    id: u64,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    eff_alpha: f32,
) {
    // Only render when there are lines to display — otherwise the tooltip is
    // "owned" but has no content yet (e.g. during addon init).
    let data = tooltip_data.and_then(|map| map.get(&id));
    let Some(data) = data else { return };

    let alpha = eff_alpha;

    // Tooltip border and background via nine-slice atlas (rounded corners).
    // WoW calls SetCenterColor(0, 0, 0, 1) — TOOLTIP_DEFAULT_BACKGROUND_COLOR is black.
    if let Some(ns) = tooltip_nine_slice() {
        let center = [0.0, 0.0, 0.0, alpha];
        // WoW's TooltipDefaultLayout anchors center with (-4,4,4,-4) offsets,
        // extending the fill 4px into each corner to cover transparent inner areas.
        super::nine_slice::emit_nine_slice_with_center_color(batch, bounds, ns, alpha, center, 4.0);
    } else {
        // Fallback if atlas entries are missing
        batch.push_solid(bounds, [0.0, 0.0, 0.0, alpha]);
        batch.push_border(bounds, 1.0, [0.6, 0.5, 0.15, alpha]);
    }

    let Some((font_sys, glyph_atlas)) = text_ctx else { return };

    let content_x = bounds.x + TOOLTIP_PADDING_H;
    let content_width = bounds.width - TOOLTIP_PADDING_H * 2.0;
    let mut y = bounds.y + TOOLTIP_PADDING_V;

    for line in &data.lines {
        let line_height = (line.font_size * 1.2).ceil();

        emit_tooltip_line(
            batch, font_sys, glyph_atlas,
            line, content_x, y, content_width, line_height,
        );

        y += line_height + TOOLTIP_LINE_SPACING;
    }
}

/// Emit quads for a single tooltip line (left text, optional right text).
#[allow(clippy::too_many_arguments)]
fn emit_tooltip_line(
    batch: &mut QuadBatch,
    font_sys: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    line: &TooltipLineRender,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    // Left-aligned text
    let left_bounds = Rectangle::new(Point::new(x, y), Size::new(width, height));
    emit_text_quads(
        batch, font_sys, glyph_atlas,
        &line.left_text, left_bounds,
        None, line.font_size, line.left_color,
        TextJustify::Left, TextJustify::Center,
        GLYPH_ATLAS_TEX_INDEX,
        None, (0.0, 0.0), TextOutline::None,
        line.wrap, 0,
    );

    // Right-aligned text (for double lines)
    if let Some(ref right_text) = line.right_text {
        let right_bounds = Rectangle::new(Point::new(x, y), Size::new(width, height));
        emit_text_quads(
            batch, font_sys, glyph_atlas,
            right_text, right_bounds,
            None, line.font_size, line.right_color,
            TextJustify::Right, TextJustify::Center,
            GLYPH_ATLAS_TEX_INDEX,
            None, (0.0, 0.0), TextOutline::None,
            false, 0,
        );
    }
}
