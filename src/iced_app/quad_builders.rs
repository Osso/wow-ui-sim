//! Widget-specific quad emitters for rendering WoW frames.

use iced::{Point, Rectangle, Size};

use crate::render::font::WowFontSystem;
use crate::render::glyph::{GlyphAtlas, emit_text_quads};
use crate::render::shader::GLYPH_ATLAS_TEX_INDEX;
use crate::render::{BlendMode, QuadBatch};
use crate::widget::{TextJustify, WidgetType};

use super::masking::apply_mask_texture;
use super::message_frame_render::{MessageFrameTextRenderer, emit_message_frame_text};
use super::statusbar::StatusBarFill;
use super::tooltip::TooltipRenderData;
#[path = "quad_builders_clipping.rs"]
mod clipping;

#[path = "quad_builders_textures.rs"]
mod textures;

#[path = "quad_builders_cooldown.rs"]
mod cooldown;

#[path = "quad_builders_button.rs"]
mod button;

pub(super) use button::emit_button_highlight;
use clipping::clip_recent_quads;
pub(super) use textures::{build_minimap_quads, build_texture_quads};

/// Build quads for a Frame widget (backdrop).
pub fn build_frame_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    alpha: f32,
) {
    emit_fog_of_war_quads(batch, bounds, f, alpha);

    if f.backdrop.enabled {
        let bg = &f.backdrop.bg_color;
        batch.push_solid(bounds, [bg.r, bg.g, bg.b, bg.a * alpha]);

        if f.backdrop.edge_size > 0.0 {
            let bc = &f.backdrop.border_color;
            batch.push_border(
                bounds,
                f.backdrop.edge_size.max(1.0),
                [bc.r, bc.g, bc.b, bc.a * alpha],
            );
        }
    }

    // Nine-slice frames render their pieces through the child Texture pass
    // (see `quad_builders_textures.rs` → `emit_nine_slice_atlas`), so the
    // parent Frame must not add an extra border here. The previous 2 px gold
    // fallback used to paint a visible box behind any frame that had a
    // nine-slice layout registered — it's the "offset border box behind the
    // tooltip" artifact reported in PLAN.md #53.
}

const FOG_OPACITY: f32 = 0.6;
const FOG_EDGE_FRACTION: f32 = 0.05;
const FOG_EDGE_MIN_WIDTH: f32 = 16.0;
const FOG_EDGE_MAX_WIDTH: f32 = 48.0;

struct FogOverlayRects {
    fade: Option<Rectangle>,
    solid: Option<Rectangle>,
}

fn emit_fog_of_war_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    frame: &crate::widget::Frame,
    alpha: f32,
) {
    if !is_fog_of_war_frame(frame) {
        return;
    }
    let Some(fog_alpha) = fog_overlay_alpha(frame, alpha) else {
        return;
    };
    let Some(rects) = fog_overlay_rects(bounds) else {
        return;
    };

    if let Some(fade_bounds) = rects.fade {
        batch.push_gradient(
            fade_bounds,
            [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, fog_alpha],
                [0.0, 0.0, 0.0, fog_alpha],
                [0.0, 0.0, 0.0, 0.0],
            ],
        );
    }
    if let Some(solid_bounds) = rects.solid {
        batch.push_solid(solid_bounds, [0.0, 0.0, 0.0, fog_alpha]);
    }
}

fn is_fog_of_war_frame(frame: &crate::widget::Frame) -> bool {
    frame
        .object_type_name
        .as_deref()
        .is_some_and(|name| name.eq_ignore_ascii_case("FogOfWarFrame"))
        && frame.fog_of_war_background_atlas.is_some()
}

fn fog_overlay_alpha(frame: &crate::widget::Frame, alpha: f32) -> Option<f32> {
    let fog_alpha =
        alpha * FOG_OPACITY * frame.fog_of_war_mask_scalar.unwrap_or(1.0).clamp(0.0, 1.0);
    (fog_alpha > f32::EPSILON).then_some(fog_alpha)
}

fn fog_overlay_rects(bounds: Rectangle) -> Option<FogOverlayRects> {
    const EXPLORED_LEFT_FRACTION: f32 = 0.5;
    let explored_left_fraction = EXPLORED_LEFT_FRACTION.clamp(0.0, 1.0);
    let fog_fraction = 1.0 - explored_left_fraction;
    if fog_fraction <= f32::EPSILON {
        return None;
    }

    let fog_start_x = bounds.x + bounds.width * explored_left_fraction;
    let fog_width = bounds.width * fog_fraction;
    let fade_width = (bounds.width * FOG_EDGE_FRACTION)
        .clamp(FOG_EDGE_MIN_WIDTH, FOG_EDGE_MAX_WIDTH)
        .min(fog_width);
    let fade = (fade_width > f32::EPSILON).then(|| {
        Rectangle::new(
            Point::new(fog_start_x, bounds.y),
            Size::new(fade_width, bounds.height),
        )
    });

    let solid_start_x = fog_start_x + fade_width;
    let solid_width = bounds.x + bounds.width - solid_start_x;
    let solid = (solid_width > f32::EPSILON).then(|| {
        Rectangle::new(
            Point::new(solid_start_x, bounds.y),
            Size::new(solid_width, bounds.height),
        )
    });

    Some(FogOverlayRects { fade, solid })
}

pub(super) fn color_with_alpha(c: &crate::widget::Color, alpha: f32) -> [f32; 4] {
    [c.r, c.g, c.b, c.a * alpha]
}

pub(super) struct WidgetTextRenderer<'a> {
    pub(super) batch: &'a mut QuadBatch,
    pub(super) font_sys: &'a mut WowFontSystem,
    pub(super) glyph_atlas: &'a mut GlyphAtlas,
}

pub(super) struct WidgetTextLayout<'a> {
    pub(super) text: &'a str,
    pub(super) bounds: Rectangle,
    pub(super) justify_h: TextJustify,
    pub(super) justify_v: TextJustify,
    pub(super) word_wrap: bool,
    pub(super) max_lines: u32,
    pub(super) alpha: f32,
}

fn effective_word_wrap(f: &crate::widget::Frame, requested_word_wrap: bool) -> bool {
    requested_word_wrap && !f.width_is_text_auto
}

/// The shadow offset a FontString stores is in WoW's UI units with y pointing
/// up (`SetShadowOffset(1, -1)` is one unit right and one DOWN), while the quad
/// builders work in screen pixels with y pointing down. Flip y and scale both
/// axes by the frame's effective scale; passing the raw pair drew the shadow
/// one unscaled pixel ABOVE the glyphs, so the bottom edge of every text lost
/// its shadow at any UI scale.
fn screen_shadow_offset(f: &crate::widget::Frame) -> (f32, f32) {
    (
        f.shadow_offset.0 * f.effective_scale,
        -f.shadow_offset.1 * f.effective_scale,
    )
}

/// Emit text quads for a widget, extracting color/shadow from the frame.
pub(super) fn emit_widget_text_quads(
    text_renderer: &mut WidgetTextRenderer<'_>,
    f: &crate::widget::Frame,
    mut layout: WidgetTextLayout<'_>,
) {
    layout.word_wrap = effective_word_wrap(f, layout.word_wrap);
    let color = color_with_alpha(&f.text_color, layout.alpha);
    let shadow = (f.shadow_color.a > 0.0).then(|| color_with_alpha(&f.shadow_color, layout.alpha));
    let clip_bounds = layout.bounds;
    let vert_before = text_renderer.batch.vertices.len();
    if !f.text_segments.is_empty() {
        emit_widget_text_segment_quads(text_renderer, f, layout, shadow);
        clip_recent_quads(text_renderer.batch, vert_before, clip_bounds);
        return;
    }
    emit_text_quads(
        text_renderer.batch,
        text_renderer.font_sys,
        text_renderer.glyph_atlas,
        layout.text,
        layout.bounds,
        f.font.as_deref(),
        f.font_size * f.effective_scale,
        color,
        layout.justify_h,
        layout.justify_v,
        GLYPH_ATLAS_TEX_INDEX,
        shadow,
        screen_shadow_offset(f),
        f.font_outline,
        layout.word_wrap,
        layout.max_lines,
        f.text_stripped.as_deref(),
    );
    clip_recent_quads(text_renderer.batch, vert_before, clip_bounds);
}

fn emit_widget_text_segment_quads(
    text_renderer: &mut WidgetTextRenderer<'_>,
    f: &crate::widget::Frame,
    layout: WidgetTextLayout<'_>,
    shadow: Option<[f32; 4]>,
) {
    let font_size = f.font_size * f.effective_scale;
    let line_height = (font_size * 1.2).ceil();
    let mut x = layout.bounds.x;
    let mut y = layout.bounds.y;
    let right = layout.bounds.x + layout.bounds.width;

    for segment in &f.text_segments {
        let color = color_with_alpha(&segment.color, layout.alpha);
        for chunk in text_chunks(&segment.text) {
            let width =
                text_renderer
                    .font_sys
                    .measure_text_width(chunk, f.font.as_deref(), font_size);
            if starts_new_segment_line(&layout, chunk, x, width, right) {
                x = layout.bounds.x;
                y += line_height;
            }
            let bounds = Rectangle::new(Point::new(x, y), Size::new(width.max(1.0), line_height));
            emit_text_segment_chunk(text_renderer, f, chunk, bounds, font_size, color, shadow);
            x += width;
        }
    }
}

fn starts_new_segment_line(
    layout: &WidgetTextLayout<'_>,
    chunk: &str,
    x: f32,
    width: f32,
    right: f32,
) -> bool {
    layout.word_wrap && x > layout.bounds.x && x + width > right && !chunk.trim().is_empty()
}

fn emit_text_segment_chunk(
    text_renderer: &mut WidgetTextRenderer<'_>,
    f: &crate::widget::Frame,
    chunk: &str,
    bounds: Rectangle,
    font_size: f32,
    color: [f32; 4],
    shadow: Option<[f32; 4]>,
) {
    emit_text_quads(
        text_renderer.batch,
        text_renderer.font_sys,
        text_renderer.glyph_atlas,
        chunk,
        bounds,
        f.font.as_deref(),
        font_size,
        color,
        TextJustify::Left,
        TextJustify::Center,
        GLYPH_ATLAS_TEX_INDEX,
        shadow,
        screen_shadow_offset(f),
        f.font_outline,
        false,
        0,
        None,
    );
}

fn text_chunks(text: &str) -> impl Iterator<Item = &str> {
    text.split_inclusive(char::is_whitespace)
}

pub struct FrameQuadEmit<'a> {
    pub id: u64,
    pub widget: &'a crate::widget::Frame,
    pub bounds: Rectangle,
    pub clip_bounds: Option<Rectangle>,
    pub bar_fill: Option<&'a StatusBarFill>,
    pub pressed_frame: Option<u64>,
    pub hovered_frame: Option<u64>,
    pub message_frames:
        Option<&'a std::collections::HashMap<u64, crate::lua_api::message_frame::MessageFrameData>>,
    pub tooltip_data: Option<&'a std::collections::HashMap<u64, TooltipRenderData>>,
    pub quest_blobs:
        Option<&'a std::collections::HashMap<u64, crate::lua_api::state::QuestBlobState>>,
    pub registry: &'a crate::widget::WidgetRegistry,
    pub elapsed_secs: f64,
    pub eff_alpha: f32,
}

/// Emit quads for a single visible frame based on its widget type.
///
/// `eff_alpha` is the effective alpha from the ancestor chain (`parent_alpha * f.alpha`),
/// matching WoW's `GetEffectiveAlpha()` behavior where parent alpha dims all descendants.
pub fn emit_frame_quads(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    frame: FrameQuadEmit<'_>,
) {
    let vert_before = batch.vertices.len();
    dispatch_widget_quads(batch, text_ctx, &frame);

    if let Some(clip_bounds) = frame.clip_bounds
        && frame.widget.rotation == 0.0
        // clip_recent_quads assumes axis-aligned quad vertex ordering.
        // Line widgets emit rotated quads and need a different clip path.
        && frame.widget.widget_type != WidgetType::Line
    {
        clip_recent_quads(batch, vert_before, clip_bounds);
    }

    emit_quest_blob_quads(batch, &frame);
}

fn dispatch_widget_quads(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    frame: &FrameQuadEmit<'_>,
) {
    match frame.widget.widget_type {
        WidgetType::Frame | WidgetType::StatusBar => {
            build_frame_quads(batch, frame.bounds, frame.widget, frame.eff_alpha)
        }
        WidgetType::MessageFrame => emit_message_frame_quads(batch, text_ctx, frame),
        WidgetType::GameTooltip => emit_tooltip_quads(batch, text_ctx, frame),
        WidgetType::Minimap => {
            build_minimap_quads(batch, frame.bounds, frame.widget, frame.eff_alpha)
        }
        WidgetType::Button => button::emit_button_quads_with_text(batch, text_ctx, frame),
        WidgetType::Texture => emit_texture_quads_with_mask(batch, frame),
        WidgetType::FontString | WidgetType::SimpleHTML => {
            emit_fontstring_quads(batch, text_ctx, frame)
        }
        WidgetType::CheckButton => button::emit_checkbutton_quads(batch, text_ctx, frame),
        WidgetType::EditBox => button::emit_editbox_with_text(
            batch,
            frame.bounds,
            frame.widget,
            text_ctx,
            frame.eff_alpha,
        ),
        WidgetType::Cooldown => cooldown::emit_cooldown_quads(batch, text_ctx, frame),
        WidgetType::Line => super::quad_builders_line::build_line_quads(
            batch,
            frame.widget,
            frame.registry,
            frame.eff_alpha,
        ),
        _ => {}
    }
}

fn emit_tooltip_quads(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    frame: &FrameQuadEmit<'_>,
) {
    super::tooltip::build_tooltip_quads(
        super::tooltip::TooltipRender {
            batch,
            bounds: frame.bounds,
            tooltip_data: frame.tooltip_data,
            id: frame.id,
            eff_alpha: frame.eff_alpha,
            eff_scale: frame.widget.effective_scale,
            draw_background: !frame.widget.children_keys.contains_key("NineSlice"),
        },
        text_ctx,
    );
}

fn emit_message_frame_quads(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    frame: &FrameQuadEmit<'_>,
) {
    build_frame_quads(batch, frame.bounds, frame.widget, frame.eff_alpha);
    if let Some((fs, ga)) = text_ctx
        && let Some(mf_map) = frame.message_frames
    {
        let mut render = MessageFrameTextRenderer {
            batch,
            font_sys: fs,
            glyph_atlas: ga,
        };
        emit_message_frame_text(
            &mut render,
            frame.widget,
            frame.id,
            frame.bounds,
            mf_map,
            frame.eff_alpha,
            frame.elapsed_secs,
        );
    }
}

fn emit_texture_quads_with_mask(batch: &mut QuadBatch, frame: &FrameQuadEmit<'_>) {
    if frame.widget.is_mask {
        return;
    }
    if is_inactive_button_highlight_texture(frame) {
        return;
    }
    let vert_before = batch.vertices.len();
    build_texture_quads(
        batch,
        frame.bounds,
        frame.widget,
        frame.bar_fill,
        frame.eff_alpha,
    );
    if !frame.widget.mask_textures.is_empty() {
        apply_mask_texture(
            batch,
            vert_before,
            frame.bounds,
            &frame.widget.mask_textures,
            frame.registry,
        );
    }
}

fn is_inactive_button_highlight_texture(frame: &FrameQuadEmit<'_>) -> bool {
    if frame.widget.parent_key.as_deref() != Some("HighlightTexture") {
        return false;
    }
    let Some(parent_id) = frame.widget.parent_id else {
        return false;
    };
    let Some(parent) = frame.registry.get(parent_id) else {
        return false;
    };
    if !matches!(
        parent.widget_type,
        WidgetType::Button | WidgetType::CheckButton
    ) {
        return false;
    }
    // Hover render goes through `append_hover_highlight` (overlay batch); the
    // generic draw loop must treat the slot child as inactive so additive
    // blending is applied once. Locked highlights bypass the suppression and
    // render through the generic loop because no hover state will toggle them.
    !parent.highlight_locked
}

fn emit_fontstring_quads(
    batch: &mut QuadBatch,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    frame: &FrameQuadEmit<'_>,
) {
    if is_tooltip_line_frame(frame) {
        return;
    }
    if let Some((fs, ga)) = text_ctx
        && let Some(ref txt) = frame.widget.text
    {
        let text_bounds = button::button_text_bounds(frame);
        let mut text_renderer = WidgetTextRenderer {
            batch,
            font_sys: fs,
            glyph_atlas: ga,
        };
        emit_widget_text_quads(
            &mut text_renderer,
            frame.widget,
            WidgetTextLayout {
                text: txt,
                bounds: text_bounds,
                justify_h: frame.widget.justify_h,
                justify_v: frame.widget.justify_v,
                word_wrap: frame.widget.word_wrap,
                max_lines: frame.widget.max_lines,
                alpha: frame.eff_alpha,
            },
        );
    }
}

fn is_tooltip_line_frame(frame: &FrameQuadEmit<'_>) -> bool {
    let Some(parent_id) = frame.widget.parent_id else {
        return false;
    };
    let Some(parent) = frame.registry.get(parent_id) else {
        return false;
    };
    parent.widget_type == WidgetType::GameTooltip
        && frame
            .widget
            .parent_key
            .as_deref()
            .is_some_and(is_tooltip_line_parent_key)
}

fn is_tooltip_line_parent_key(parent_key: &str) -> bool {
    parent_key
        .strip_prefix("TextLeft")
        .or_else(|| parent_key.strip_prefix("TextRight"))
        .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit()))
}

fn emit_quest_blob_quads(batch: &mut QuadBatch, frame: &FrameQuadEmit<'_>) {
    let Some(quest_blobs) = frame.quest_blobs else {
        return;
    };
    let Some(blob_state) = quest_blobs.get(&frame.id) else {
        return;
    };
    if blob_state.active_quests.is_empty() || blob_state.map_id == 0 {
        return;
    }

    let alpha = blob_state.fill_alpha.unwrap_or(1.0) as f32 * frame.eff_alpha;
    if alpha <= 0.0 {
        return;
    }

    for &quest_id in &blob_state.active_quests {
        for blob in crate::quest_poi_blobs::get_quest_blobs_for_map(quest_id, blob_state.map_id) {
            emit_blob_polygon(
                batch,
                frame.bounds,
                blob.vertices,
                blob_state.fill_texture.as_deref(),
                alpha,
            );
        }
    }
}

fn emit_blob_polygon(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    vertices: &[(f32, f32)],
    fill_texture: Option<&str>,
    alpha: f32,
) {
    if vertices.len() < 3 {
        return;
    }

    let color = [1.0, 1.0, 1.0, alpha];
    for i in 1..vertices.len() - 1 {
        let triangle = [vertices[0], vertices[i], vertices[i + 1]];
        let positions =
            triangle.map(|(u, v)| [bounds.x + u * bounds.width, bounds.y + v * bounds.height]);
        let uvs = triangle.map(|(u, v)| [u, v]);
        if let Some(path) = fill_texture {
            batch.push_textured_triangle_path(positions, uvs, path, color, BlendMode::Alpha);
        } else {
            batch.push_solid_triangle(positions, color);
        }
    }
}

#[cfg(test)]
#[path = "quad_builders_tests.rs"]
mod tests;
