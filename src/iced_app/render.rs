//! Shader rendering implementation.

use iced::mouse;
use iced::widget::shader;
use iced::{Event, Point, Rectangle, Size};

use crate::render::font::WowFontSystem;
use crate::render::glyph::{emit_text_quads, GlyphAtlas};
use crate::render::shader::GLYPH_ATLAS_TEX_INDEX;
use crate::render::texture::UI_SCALE;
use crate::render::{BlendMode, GpuTextureData, QuadBatch, WowUiPrimitive};
use crate::widget::{TextJustify, WidgetType};

use super::app::App;
use super::frame_collect::{collect_subtree_ids, collect_ancestor_visible_ids, collect_sorted_frames};
use super::layout::LayoutCache;
use super::message_frame_render::emit_message_frame_text;
use super::statusbar::{StatusBarFill, collect_statusbar_fills};
use super::state::CanvasMessage;
use super::tooltip::TooltipRenderData;
use super::Message;

/// Shader program implementation for GPU rendering of WoW frames.
impl shader::Program<Message> for &App {
    type State = ();
    type Primitive = WowUiPrimitive;

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { position } => {
                    if bounds.contains(*position) {
                        let local = Point::new(position.x - bounds.x, position.y - bounds.y);
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseMove(local),
                        )));
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseDown(pos),
                        )));
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MouseUp(pos),
                        )));
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::MiddleClick(pos),
                        )));
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    let dy = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => *y,
                        mouse::ScrollDelta::Pixels { y, .. } => *y / 30.0,
                    };
                    return Some(shader::Action::publish(Message::Scroll(0.0, dy)));
                }
                _ => {}
            },
            Event::Keyboard(_) => {}
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        let start = std::time::Instant::now();
        self.frame_count.set(self.frame_count.get() + 1);

        let size = bounds.size();
        self.screen_size.set(size);
        self.sync_screen_size_to_state(size);
        let (quads, rebuilt) = self.get_or_rebuild_quads(size);

        // Build overlay (hover highlight + cursor) as a separate small batch.
        // This avoids cloning the entire world quad batch every frame.
        let mut overlay = QuadBatch::new();
        self.append_hover_highlight(&mut overlay, size);
        if let Some(pos) = self.mouse_position {
            const CURSOR_SIZE: f32 = 32.0;
            overlay.push_textured_path(
                Rectangle::new(Point::new(pos.x, pos.y), Size::new(CURSOR_SIZE, CURSOR_SIZE)),
                r"Interface\Cursor\Point",
                [1.0, 1.0, 1.0, 1.0],
                BlendMode::Alpha,
            );
        }

        let mut textures = self.load_new_textures(&quads);
        textures.extend(self.load_new_textures(&overlay));

        // Only update frame time when quads were actually rebuilt.
        // Cache-hit draws are trivial (~0.1ms) and would drown out real rebuild costs.
        if rebuilt {
            let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
            self.frame_time_ms.set(elapsed_ms);
            let avg = self.frame_time_avg.get();
            self.frame_time_avg.set(0.33 * elapsed_ms + 0.67 * avg);
        }

        let mut primitive = WowUiPrimitive::with_textures(quads, textures);
        primitive.overlay = overlay;
        self.attach_dirty_glyph_atlas(&mut primitive);
        primitive
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.position_in(bounds).is_some() {
            mouse::Interaction::Hidden
        } else {
            mouse::Interaction::default()
        }
    }
}

/// Build quads for a Frame widget (backdrop).
pub fn build_frame_quads(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    // Draw backdrop if enabled
    if f.backdrop.enabled {
        let bg = &f.backdrop.bg_color;
        batch.push_solid(bounds, [bg.r, bg.g, bg.b, bg.a * f.alpha]);

        // Border
        if f.backdrop.edge_size > 0.0 {
            let bc = &f.backdrop.border_color;
            batch.push_border(bounds, f.backdrop.edge_size.max(1.0), [bc.r, bc.g, bc.b, bc.a * f.alpha]);
        }
    }

    // NineSlice rendering - for now, just draw a placeholder border
    if f.nine_slice_layout.is_some() {
        batch.push_border(bounds, 2.0, [0.6, 0.45, 0.15, f.alpha]);
    }
}

/// Build quads for a Button widget.
pub fn build_button_quads(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    f: &crate::widget::Frame,
    is_pressed: bool,
    is_hovered: bool,
) {
    // If this button has child Texture widgets for NormalTexture/PushedTexture,
    // those children render themselves. Skip rendering here to avoid double-draw.
    let has_normal_child = f.children_keys.contains_key("NormalTexture");
    let has_pushed_child = f.children_keys.contains_key("PushedTexture");

    let (texture_path, tex_coords, skip) = if is_pressed {
        (
            f.pushed_texture.as_ref().or(f.normal_texture.as_ref()),
            f.pushed_tex_coords.or(f.normal_tex_coords),
            if f.pushed_texture.is_some() { has_pushed_child } else { has_normal_child },
        )
    } else {
        (f.normal_texture.as_ref(), f.normal_tex_coords, has_normal_child)
    };

    if !skip {
        emit_button_texture(batch, bounds, texture_path, tex_coords, f.alpha);
    }

    let has_highlight_child = f.children_keys.contains_key("HighlightTexture");
    if is_hovered && !is_pressed && !has_highlight_child {
        emit_button_highlight(batch, bounds, f);
    }
}

/// Render the button's normal/pushed texture (atlas UV or 3-slice).
fn emit_button_texture(
    batch: &mut QuadBatch,
    bounds: Rectangle,
    texture_path: Option<&String>,
    tex_coords: Option<(f32, f32, f32, f32)>,
    alpha: f32,
) {
    let Some(tex_path) = texture_path else { return };
    if let Some((left, right, top, bottom)) = tex_coords {
        let uvs = Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top));
        batch.push_textured_path_uv(bounds, uvs, tex_path, [1.0, 1.0, 1.0, alpha], BlendMode::Alpha);
    } else {
        const BUTTON_TEX_WIDTH: f32 = 128.0;
        const BUTTON_CAP_WIDTH: f32 = 4.0;
        batch.push_three_slice_h_path(
            bounds, BUTTON_CAP_WIDTH, BUTTON_CAP_WIDTH,
            tex_path, BUTTON_TEX_WIDTH, [1.0, 1.0, 1.0, alpha],
        );
    }
}

/// Render the button highlight overlay on hover.
fn emit_button_highlight(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    if let Some(highlight_path) = &f.highlight_texture {
        if let Some((left, right, top, bottom)) = f.highlight_tex_coords {
            let uvs = Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top));
            batch.push_textured_path_uv(
                bounds, uvs, highlight_path,
                [1.0, 1.0, 1.0, 0.5 * f.alpha], BlendMode::Additive,
            );
        } else {
            const BUTTON_TEX_WIDTH: f32 = 128.0;
            const BUTTON_CAP_WIDTH: f32 = 4.0;
            batch.push_three_slice_h_path_blend(
                bounds, BUTTON_CAP_WIDTH, BUTTON_CAP_WIDTH,
                highlight_path, BUTTON_TEX_WIDTH,
                [1.0, 1.0, 1.0, 0.5 * f.alpha], BlendMode::Additive,
            );
        }
    }
}

/// Build quads for a Texture widget, optionally clipped by a StatusBar fill.
pub fn build_texture_quads(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame, bar_fill: Option<&StatusBarFill>) {
    if let Some(ref ns) = f.nine_slice_atlas {
        super::nine_slice::emit_nine_slice_atlas(batch, bounds, ns, f.alpha);
        return;
    }

    // Base tint from vertex color (SetVertexColor), defaulting to white
    let vc = f.vertex_color.as_ref();
    let base_tint = [
        vc.map_or(1.0, |c| c.r),
        vc.map_or(1.0, |c| c.g),
        vc.map_or(1.0, |c| c.b),
        vc.map_or(1.0, |c| c.a) * f.alpha,
    ];

    // StatusBar fill color overrides vertex color tint
    let tint = if let Some(fill) = bar_fill
        && let Some(c) = &fill.color {
            [c.r, c.g, c.b, c.a * f.alpha]
        } else {
            base_tint
        };

    if let Some(color) = f.color_texture {
        let fill_bounds = apply_bar_fill(bounds, bar_fill);
        batch.push_solid(fill_bounds, [color.r * tint[0], color.g * tint[1], color.b * tint[2], color.a * f.alpha]);
        return;
    }

    let Some(tex_path) = &f.texture else {
        // No texture — if there's a bar fill color, render as solid fill
        if let Some(fill) = bar_fill
            && let Some(c) = &fill.color {
                let fill_bounds = apply_bar_fill(bounds, bar_fill);
                batch.push_solid(fill_bounds, [c.r, c.g, c.b, c.a * f.alpha]);
            }
        return;
    };
    let (fill_bounds, fill_uvs) = apply_bar_fill_with_uvs(bounds, f.tex_coords, bar_fill);

    let vert_before = batch.vertices.len();
    if let Some((left, right, top, bottom)) = fill_uvs {
        let uvs = Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top));
        if f.horiz_tile || f.vert_tile {
            emit_tiled_texture(batch, fill_bounds, &uvs, tex_path, f);
        } else {
            batch.push_textured_path_uv(fill_bounds, uvs, tex_path, tint, f.blend_mode);
        }
    } else {
        batch.push_textured_path(fill_bounds, tex_path, tint, f.blend_mode);
    }

    if f.rotation != 0.0 {
        apply_uv_rotation(batch, vert_before, f.rotation);
    }
}

/// Apply StatusBar fill clipping to bounds.
fn apply_bar_fill(bounds: Rectangle, bar_fill: Option<&StatusBarFill>) -> Rectangle {
    let Some(fill) = bar_fill else { return bounds };
    let fill_width = bounds.width * fill.fraction;
    if fill.reverse {
        Rectangle::new(
            Point::new(bounds.x + bounds.width - fill_width, bounds.y),
            Size::new(fill_width, bounds.height),
        )
    } else {
        Rectangle::new(bounds.position(), Size::new(fill_width, bounds.height))
    }
}

/// Apply StatusBar fill clipping to bounds and UV coordinates.
fn apply_bar_fill_with_uvs(
    bounds: Rectangle,
    tex_coords: Option<(f32, f32, f32, f32)>,
    bar_fill: Option<&StatusBarFill>,
) -> (Rectangle, Option<(f32, f32, f32, f32)>) {
    let Some(fill) = bar_fill else { return (bounds, tex_coords) };
    let fill_bounds = apply_bar_fill(bounds, bar_fill);
    let (uv_left, uv_right, uv_top, uv_bottom) = tex_coords.unwrap_or((0.0, 1.0, 0.0, 1.0));
    let uv_range = uv_right - uv_left;
    let fill_uvs = if fill.reverse {
        (uv_left + uv_range * (1.0 - fill.fraction), uv_right, uv_top, uv_bottom)
    } else {
        (uv_left, uv_left + uv_range * fill.fraction, uv_top, uv_bottom)
    };
    (fill_bounds, Some(fill_uvs))
}

use super::tiling::emit_tiled_texture;
use super::masking::apply_mask_texture;

/// Rotate texture UV coordinates around their center for vertices added after `vert_before`.
///
/// WoW's Texture:SetRotation(radians) rotates the texture image within the quad bounds
/// by rotating UV coordinates around the center of the UV region.
/// Rotation is applied per-quad (4 vertices) to handle tiled textures correctly.
fn apply_uv_rotation(batch: &mut QuadBatch, vert_before: usize, radians: f32) {
    let verts = &mut batch.vertices[vert_before..];
    if verts.len() < 4 {
        return;
    }
    let (sin_r, cos_r) = radians.sin_cos();
    // Process each quad (4 vertices) independently
    for chunk in verts.chunks_exact_mut(4) {
        let cx = (chunk[0].tex_coords[0] + chunk[1].tex_coords[0]
            + chunk[2].tex_coords[0] + chunk[3].tex_coords[0]) * 0.25;
        let cy = (chunk[0].tex_coords[1] + chunk[1].tex_coords[1]
            + chunk[2].tex_coords[1] + chunk[3].tex_coords[1]) * 0.25;
        for v in chunk.iter_mut() {
            let du = v.tex_coords[0] - cx;
            let dv = v.tex_coords[1] - cy;
            v.tex_coords[0] = cx + du * cos_r - dv * sin_r;
            v.tex_coords[1] = cy + du * sin_r + dv * cos_r;
        }
    }
}

/// Build quads for a Minimap widget - map texture clipped to a circle.
pub fn build_minimap_quads(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    use crate::render::shader::FLAG_CIRCLE_CLIP;
    batch.push_textured_path(
        bounds,
        r"Interface\Minimap\placeholder-map",
        [1.0, 1.0, 1.0, f.alpha],
        BlendMode::Alpha,
    );
    batch.set_extra_flags(4, FLAG_CIRCLE_CLIP);
}

/// Build quads for an EditBox widget.
pub fn build_editbox_quads(batch: &mut QuadBatch, bounds: Rectangle, f: &crate::widget::Frame) {
    // Skip placeholder if child textures provide the border (e.g. SearchBoxTemplate)
    if !f.children_keys.is_empty() {
        return;
    }
    // Background
    batch.push_solid(bounds, [0.06, 0.06, 0.08, 0.9 * f.alpha]);
    // Border
    batch.push_border(bounds, 1.0, [0.3, 0.25, 0.15, f.alpha]);
}

/// Emit text quads for a widget, extracting color/shadow from the frame.
#[allow(clippy::too_many_arguments)]
fn emit_widget_text_quads(
    batch: &mut QuadBatch,
    font_sys: &mut WowFontSystem,
    glyph_atlas: &mut GlyphAtlas,
    f: &crate::widget::Frame,
    text: &str,
    text_bounds: Rectangle,
    justify_h: TextJustify,
    justify_v: TextJustify,
    word_wrap: bool,
    max_lines: u32,
) {
    let color = [
        f.text_color.r,
        f.text_color.g,
        f.text_color.b,
        f.text_color.a * f.alpha,
    ];
    let shadow = if f.shadow_color.a > 0.0 {
        Some([f.shadow_color.r, f.shadow_color.g, f.shadow_color.b, f.shadow_color.a * f.alpha])
    } else {
        None
    };
    emit_text_quads(
        batch, font_sys, glyph_atlas, text, text_bounds,
        f.font.as_deref(), f.font_size, color,
        justify_h, justify_v,
        GLYPH_ATLAS_TEX_INDEX,
        shadow, f.shadow_offset,
        f.font_outline,
        word_wrap, max_lines,
    );
}

/// Emit quads for a single visible frame based on its widget type.
#[allow(clippy::too_many_arguments)]
/// Check if a button is visually pressed (mouse or Lua SetButtonState).
fn is_button_pressed(f: &crate::widget::Frame, id: u64, pressed_frame: Option<u64>) -> bool {
    pressed_frame == Some(id) || f.button_state == 1
}

fn emit_frame_quads(
    batch: &mut QuadBatch,
    id: u64,
    f: &crate::widget::Frame,
    bounds: Rectangle,
    bar_fill: Option<&StatusBarFill>,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    message_frames: Option<&std::collections::HashMap<u64, crate::lua_api::message_frame::MessageFrameData>>,
    tooltip_data: Option<&std::collections::HashMap<u64, TooltipRenderData>>,
    registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
    cache: &mut LayoutCache,
) {
    match f.widget_type {
        WidgetType::Frame | WidgetType::StatusBar => build_frame_quads(batch, bounds, f),
        WidgetType::MessageFrame => {
            build_frame_quads(batch, bounds, f);
            if let Some((fs, ga)) = text_ctx
                && let Some(mf_map) = message_frames {
                    emit_message_frame_text(batch, fs, ga, f, id, bounds, mf_map);
                }
        }
        WidgetType::GameTooltip => {
            super::tooltip::build_tooltip_quads(batch, bounds, f, tooltip_data, id, text_ctx);
        }
        WidgetType::Minimap => build_minimap_quads(batch, bounds, f),
        WidgetType::Button => {
            build_button_quads(batch, bounds, f, is_button_pressed(f, id, pressed_frame), hovered_frame == Some(id));
            if !f.children_keys.contains_key("Text")
                && let Some((fs, ga)) = text_ctx
                && let Some(ref txt) = f.text {
                    emit_widget_text_quads(batch, fs, ga, f, txt, bounds, f.justify_h, f.justify_v, false, 0);
                }
        }
        WidgetType::Texture => {
            if !f.is_mask {
                let vert_before = batch.vertices.len();
                build_texture_quads(batch, bounds, f, bar_fill);
                if !f.mask_textures.is_empty() {
                    apply_mask_texture(batch, vert_before, bounds, &f.mask_textures, registry, screen_size, cache);
                }
            }
        }
        WidgetType::FontString => {
            if let Some((fs, ga)) = text_ctx
                && let Some(ref txt) = f.text {
                    emit_widget_text_quads(batch, fs, ga, f, txt, bounds, f.justify_h, f.justify_v, f.word_wrap, f.max_lines);
                }
        }
        WidgetType::CheckButton => {
            build_button_quads(batch, bounds, f, is_button_pressed(f, id, pressed_frame), hovered_frame == Some(id));
            if let Some((fs, ga)) = text_ctx
                && let Some(ref txt) = f.text {
                    let label_bounds = Rectangle::new(
                        Point::new(bounds.x + 20.0, bounds.y),
                        Size::new(bounds.width - 20.0, bounds.height),
                    );
                    emit_widget_text_quads(batch, fs, ga, f, txt, label_bounds, TextJustify::Left, TextJustify::Center, false, 0);
                }
        }
        WidgetType::EditBox => {
            build_editbox_quads(batch, bounds, f);
            if let Some((fs, ga)) = text_ctx
                && let Some(ref txt) = f.text {
                    let (left_inset, right_inset, top_inset, bottom_inset) = f.editbox_text_insets;
                    let left_pad = if left_inset > 0.0 { left_inset } else { 4.0 };
                    let right_pad = if right_inset > 0.0 { right_inset } else { 4.0 };
                    let text_bounds = Rectangle::new(
                        Point::new(bounds.x + left_pad, bounds.y + top_inset),
                        Size::new((bounds.width - left_pad - right_pad).max(0.0), (bounds.height - top_inset - bottom_inset).max(0.0)),
                    );
                    emit_widget_text_quads(batch, fs, ga, f, txt, text_bounds, TextJustify::Left, TextJustify::Center, false, 0);
                }
        }
        _ => {}
    }
}

/// Build a QuadBatch from a WidgetRegistry without needing an App instance.
///
/// This contains the sorting/filtering logic from `App::build_quad_batch` but
/// takes a `WidgetRegistry` directly.
///
/// When `text_ctx` is provided, FontString and button/editbox/checkbox text is
/// rendered as glyph quads interleaved with texture quads (correct draw order).
/// When `None`, text is skipped (legacy behavior for callers without fonts).
#[allow(clippy::too_many_arguments)]
/// Check if a frame should be skipped during rendering (visibility, alpha, subtree filter).
pub fn build_quad_batch_for_registry(
    registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
    root_name: Option<&str>,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    mut text_ctx: Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    message_frames: Option<&std::collections::HashMap<u64, crate::lua_api::message_frame::MessageFrameData>>,
    tooltip_data: Option<&std::collections::HashMap<u64, TooltipRenderData>>,
) -> QuadBatch {
    let mut cache = LayoutCache::new();
    let ancestor_visible = collect_ancestor_visible_ids(registry);
    let (batch, _hittable) = build_quad_batch_with_cache(
        registry, screen_size, root_name, pressed_frame, hovered_frame,
        &mut text_ctx, message_frames, tooltip_data, &mut cache,
        &ancestor_visible, None,
    );
    batch
}

/// Build a QuadBatch, populating the shared layout cache for reuse by hit testing.
///
/// Returns the quad batch and a hittable frame list (unscaled rects sorted by
/// strata/level/id) as a side output of the same collection pass.
#[allow(clippy::too_many_arguments)]
pub fn build_quad_batch_with_cache(
    registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
    root_name: Option<&str>,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    message_frames: Option<&std::collections::HashMap<u64, crate::lua_api::message_frame::MessageFrameData>>,
    tooltip_data: Option<&std::collections::HashMap<u64, TooltipRenderData>>,
    cache: &mut LayoutCache,
    ancestor_visible: &std::collections::HashMap<u64, f32>,
    strata_buckets: Option<&Vec<Vec<u64>>>,
) -> (QuadBatch, Vec<(u64, crate::LayoutRect)>) {
    let t0 = std::time::Instant::now();
    let mut batch = QuadBatch::with_capacity(1000);
    let (screen_width, screen_height) = screen_size;
    let size = Size::new(screen_width, screen_height);

    // Tiled marble background
    batch.push_tiled_path(
        Rectangle::new(Point::ORIGIN, size),
        256.0,
        256.0,
        "framegeneral/ui-background-marble",
        [0.55, 0.55, 0.55, 1.0],
    );

    let visible_ids = root_name.map(|name| collect_subtree_ids(registry, name));
    let t1 = std::time::Instant::now();
    let collected = collect_sorted_frames(registry, screen_width, screen_height, &ancestor_visible, strata_buckets, cache);
    let t2 = std::time::Instant::now();

    // Collect StatusBar fill info: bar_texture_id -> fill fraction
    let statusbar_fills = collect_statusbar_fills(registry);

    for (id, f, rect, eff_alpha) in &collected.render {
        if super::button_vis::should_skip_frame(f, *id, *eff_alpha, &visible_ids, registry, pressed_frame, hovered_frame) {
            continue;
        }
        let is_fontstring = matches!(f.widget_type, WidgetType::FontString);
        if rect.height <= 0.0 || (rect.width <= 0.0 && !is_fontstring) {
            continue;
        }
        let bounds = Rectangle::new(Point::new(rect.x * UI_SCALE, rect.y * UI_SCALE), Size::new(rect.width * UI_SCALE, rect.height * UI_SCALE));
        let bar_fill = statusbar_fills.get(id);
        emit_frame_quads(&mut batch, *id, f, bounds, bar_fill, pressed_frame, hovered_frame, text_ctx, message_frames, tooltip_data, registry, screen_size, cache);
    }
    let t3 = std::time::Instant::now();
    eprintln!(
        "[quads] visible={:.0}µs layout={:.0}µs emit({})={:.0}µs total={:.0}µs",
        (t1 - t0).as_micros(), (t2 - t1).as_micros(),
        collected.render.len(), (t3 - t2).as_micros(), (t3 - t0).as_micros(),
    );
    (batch, collected.hittable)
}


impl App {
    /// Return cached quads or rebuild if dirty/resized. Returns (quads, rebuilt).
    fn get_or_rebuild_quads(&self, size: Size) -> (std::sync::Arc<QuadBatch>, bool) {
        let mut cache = self.cached_quads.borrow_mut();
        let size_changed = cache.as_ref().map(|(s, _)| *s != size).unwrap_or(true);

        if size_changed || self.quads_dirty.get() {
            let new_quads = std::sync::Arc::new(self.build_quad_batch(size));
            *cache = Some((size, std::sync::Arc::clone(&new_quads)));
            self.quads_dirty.set(false);
            (new_quads, true)
        } else {
            (std::sync::Arc::clone(&cache.as_ref().unwrap().1), false)
        }
    }

    /// Load textures not yet uploaded to the GPU atlas.
    fn load_new_textures(&self, quads: &QuadBatch) -> Vec<GpuTextureData> {
        let mut textures = Vec::new();
        let mut uploaded = self.gpu_uploaded_textures.borrow_mut();
        let mut tex_mgr = self.texture_manager.borrow_mut();
        let all_requests = quads.texture_requests.iter().chain(&quads.mask_texture_requests);
        for request in all_requests {
            if uploaded.contains(&request.path) {
                continue;
            }
            if textures.iter().any(|t: &GpuTextureData| t.path == request.path) {
                continue;
            }
            if let Some(tex_data) = tex_mgr.load(&request.path) {
                textures.push(GpuTextureData {
                    path: request.path.clone(),
                    width: tex_data.width,
                    height: tex_data.height,
                    rgba: tex_data.pixels.clone(),
                });
                uploaded.insert(request.path.clone());
            }
        }
        textures
    }

    /// Attach glyph atlas data to the primitive if there are new glyphs.
    fn attach_dirty_glyph_atlas(&self, primitive: &mut WowUiPrimitive) {
        let mut ga = self.glyph_atlas.borrow_mut();
        if ga.is_dirty() {
            let (data, size, _) = ga.texture_data();
            primitive.glyph_atlas_data = Some(data.to_vec());
            primitive.glyph_atlas_size = size;
            ga.mark_clean();
        }
    }

    /// Build a QuadBatch for GPU shader rendering.
    ///
    /// Hover highlights are NOT baked in — they're appended dynamically in
    /// `draw()` so that hover changes don't force a full quad rebuild.
    pub(crate) fn build_quad_batch(&self, size: Size) -> QuadBatch {
        let env = self.env.borrow();
        let mut font_sys = self.font_system.borrow_mut();
        // Mutable phase: update tooltips + build/get caches.
        let (ancestor_visible, strata_buckets, mut cache) = {
            let mut state = env.state().borrow_mut();
            super::tooltip::update_tooltip_sizes(&mut state, &mut font_sys);
            let vis = state.get_ancestor_visible().clone();
            let buckets = state.get_strata_buckets().cloned();
            let layout = state.take_layout_cache();
            (vis, buckets, layout)
        };
        let state = env.state().borrow();
        let tooltip_data = super::tooltip::collect_tooltip_data(&state);
        let mut glyph_atlas = self.glyph_atlas.borrow_mut();
        let (batch, hittable) = build_quad_batch_with_cache(
            &state.widgets, (size.width, size.height), None,
            self.pressed_frame, None,
            &mut Some((&mut font_sys, &mut glyph_atlas)),
            Some(&state.message_frames), Some(&tooltip_data),
            &mut cache, &ancestor_visible, strata_buckets.as_ref(),
        );
        *self.cached_layout_rects.borrow_mut() = Some(cache.clone());
        drop(state);
        env.state().borrow_mut().set_layout_cache(cache);
        *self.cached_hittable.borrow_mut() = Some(
            hittable.into_iter().map(|(id, r)| {
                (id, Rectangle::new(
                    Point::new(r.x * UI_SCALE, r.y * UI_SCALE),
                    Size::new(r.width * UI_SCALE, r.height * UI_SCALE),
                ))
            }).collect(),
        );
        batch
    }

    /// Append hover highlight quads for the currently hovered button.
    fn append_hover_highlight(&self, quads: &mut QuadBatch, screen_size: Size) {
        let Some(hovered_id) = self.hovered_frame else { return };
        let env = self.env.borrow();
        let state = env.state().borrow();
        let registry = &state.widgets;
        let Some(f) = registry.get(hovered_id) else { return };

        // Only buttons/checkbuttons have hover highlights
        if !matches!(f.widget_type, WidgetType::Button | WidgetType::CheckButton) {
            return;
        }

        let rect = {
            let layout_cache = self.cached_layout_rects.borrow();
            if let Some(cache) = layout_cache.as_ref()
                && let Some(cached) = cache.get(&hovered_id) {
                    cached.rect
                } else {
                    drop(layout_cache);
                    super::layout::compute_frame_rect(registry, hovered_id, screen_size.width, screen_size.height)
                }
        };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let bounds = Rectangle::new(
            Point::new(rect.x * UI_SCALE, rect.y * UI_SCALE),
            Size::new(rect.width * UI_SCALE, rect.height * UI_SCALE),
        );

        // Emit the button's own highlight (from build_button_quads hover path)
        let has_highlight_child = f.children_keys.contains_key("HighlightTexture");
        let is_pressed = is_button_pressed(f, hovered_id, self.pressed_frame);
        if !is_pressed && !has_highlight_child {
            emit_button_highlight(quads, bounds, f);
        }

        // Emit HighlightTexture child if it exists
        if let Some(&ht_id) = f.children_keys.get("HighlightTexture")
            && let Some(ht) = registry.get(ht_id) {
                let ht_rect = {
                    let layout_cache = self.cached_layout_rects.borrow();
                    if let Some(cache) = layout_cache.as_ref()
                        && let Some(cached) = cache.get(&ht_id) {
                            cached.rect
                        } else {
                            drop(layout_cache);
                            super::layout::compute_frame_rect(registry, ht_id, screen_size.width, screen_size.height)
                        }
                };
                if ht_rect.width > 0.0 && ht_rect.height > 0.0 {
                    let ht_bounds = Rectangle::new(
                        Point::new(ht_rect.x * UI_SCALE, ht_rect.y * UI_SCALE),
                        Size::new(ht_rect.width * UI_SCALE, ht_rect.height * UI_SCALE),
                    );
                    build_texture_quads(quads, ht_bounds, ht, None);
                }
            }
    }

}

