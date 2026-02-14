//! Shader rendering implementation.

use iced::mouse;
use iced::widget::shader;
use iced::{Event, Point, Rectangle, Size};

use crate::render::font::WowFontSystem;
use crate::render::glyph::GlyphAtlas;
use crate::render::texture::UI_SCALE;
use crate::render::{GpuTextureData, QuadBatch, WowUiPrimitive, load_texture_or_crop};
use crate::widget::{WidgetType};

use super::app::App;
use super::frame_collect::{CollectedFrames, collect_subtree_ids, collect_hittable_frames};
use super::quad_builders::{build_texture_quads, emit_button_highlight, emit_frame_quads};
use super::statusbar::collect_statusbar_fills;
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
                mouse::Event::ButtonPressed(mouse::Button::Right) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::RightMouseDown(pos),
                        )));
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Right) => {
                    if let Some(pos) = cursor.position_in(bounds) {
                        return Some(shader::Action::publish(Message::CanvasEvent(
                            CanvasMessage::RightMouseUp(pos),
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
        let (dirty_strata, rebuilt) = self.get_or_rebuild_quads(size);

        // Build overlay (hover highlight + cursor) as a separate small batch.
        let mut overlay = QuadBatch::new();
        self.append_hover_highlight(&mut overlay);
        if let Some(pos) = self.mouse_position {
            self.append_cursor_item_icon(&mut overlay, pos);

            const CURSOR_SIZE: f32 = 32.0;
            overlay.push_textured_path(
                Rectangle::new(Point::new(pos.x, pos.y), Size::new(CURSOR_SIZE, CURSOR_SIZE)),
                r"Interface\Cursor\Point",
                [1.0, 1.0, 1.0, 1.0],
                crate::render::BlendMode::Alpha,
            );
        }

        // Load textures from dirty strata batches.
        let mut textures = Vec::new();
        for batch_opt in &dirty_strata {
            if let Some(batch) = batch_opt {
                textures.extend(self.load_new_textures(batch));
            }
        }
        textures.extend(self.load_new_textures(&overlay));

        if rebuilt {
            let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
            self.frame_time_ms.set(elapsed_ms);
            let avg = self.frame_time_avg.get();
            self.frame_time_avg.set(0.33 * elapsed_ms + 0.67 * avg);
        }

        let mut primitive = WowUiPrimitive {
            strata_batches: dirty_strata,
            overlay,
            clear_color: [0.10, 0.11, 0.14, 1.0],
            textures,
            glyph_atlas_data: None,
            glyph_atlas_size: 0,
        };
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

/// Emit quads for a single strata bucket.
///
/// Reads rect and effective_alpha fresh from the registry for each frame.
/// Button state textures use parent's effective_alpha as fallback.
#[allow(clippy::too_many_arguments)]
fn emit_single_strata(
    batch: &mut QuadBatch,
    bucket: &[u64],
    registry: &crate::widget::WidgetRegistry,
    visible_ids: &Option<std::collections::HashSet<u64>>,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    message_frames: Option<&std::collections::HashMap<u64, crate::lua_api::message_frame::MessageFrameData>>,
    tooltip_data: Option<&std::collections::HashMap<u64, TooltipRenderData>>,
    elapsed_secs: f64,
) {
    let mut render_list: Vec<(u64, crate::LayoutRect, f32)> = Vec::new();
    for &id in bucket {
        let Some(f) = registry.get(id) else { continue };
        let Some(rect) = f.layout_rect else { continue };
        let eff_alpha = if f.effective_alpha > 0.0 {
            f.effective_alpha
        } else {
            f.parent_id
                .and_then(|pid| registry.get(pid))
                .map(|p| p.effective_alpha)
                .unwrap_or(0.0)
        };
        if eff_alpha <= 0.0 { continue; }
        render_list.push((id, rect, eff_alpha));
    }
    let statusbar_fills = collect_statusbar_fills(&render_list, registry);

    for &(id, rect, eff_alpha) in &render_list {
        let Some(f) = registry.get(id) else { continue };

        if super::button_vis::should_skip_frame(f, id, eff_alpha, visible_ids, registry, pressed_frame, hovered_frame) {
            continue;
        }
        let is_fontstring = matches!(f.widget_type, WidgetType::FontString);
        let is_line = matches!(f.widget_type, WidgetType::Line);
        if (rect.height <= 0.0 && !is_line) || (rect.width <= 0.0 && !is_fontstring && !is_line) {
            continue;
        }

        let bounds = Rectangle::new(
            Point::new(rect.x * UI_SCALE, rect.y * UI_SCALE),
            Size::new(rect.width * UI_SCALE, rect.height * UI_SCALE),
        );
        let bar_fill = statusbar_fills.get(&id);
        emit_frame_quads(batch, id, f, bounds, bar_fill, pressed_frame, hovered_frame, text_ctx, message_frames, tooltip_data, registry, elapsed_secs, eff_alpha);
    }
}

/// Build a QuadBatch from a WidgetRegistry without needing an App instance.
///
/// When `text_ctx` is provided, FontString and button/editbox/checkbox text is
/// rendered as glyph quads interleaved with texture quads (correct draw order).
/// When `None`, text is skipped (legacy behavior for callers without fonts).
#[allow(clippy::too_many_arguments)]
pub fn build_quad_batch_for_registry(
    registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
    root_name: Option<&str>,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    mut text_ctx: Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    message_frames: Option<&std::collections::HashMap<u64, crate::lua_api::message_frame::MessageFrameData>>,
    tooltip_data: Option<&std::collections::HashMap<u64, TooltipRenderData>>,
    strata_buckets: &Vec<Vec<u64>>,
) -> QuadBatch {
    let (batch, _collected) = build_quad_batch_with_cache(
        registry, screen_size, root_name, pressed_frame, hovered_frame,
        &mut text_ctx, message_frames, tooltip_data,
        strata_buckets, 0.0,
    );
    batch
}

/// Scale hittable layout rects to screen coordinates, applying hit rect insets.
pub fn build_hittable_rects(
    collected: &CollectedFrames,
    registry: &crate::widget::WidgetRegistry,
) -> Vec<(u64, Rectangle)> {
    collected.hittable.iter().map(|&(id, r)| {
        let (il, ir, it, ib) = registry.get(id)
            .map(|f| f.hit_rect_insets)
            .unwrap_or((0.0, 0.0, 0.0, 0.0));
        (id, Rectangle::new(
            Point::new((r.x + il) * UI_SCALE, (r.y + it) * UI_SCALE),
            Size::new((r.width - il - ir).max(0.0) * UI_SCALE,
                      (r.height - it - ib).max(0.0) * UI_SCALE),
        ))
    }).collect()
}

/// Build a QuadBatch by iterating visible-only strata buckets directly.
///
/// Also builds a hittable frame list as a side output for hit testing.
/// Returns the quad batch and the `CollectedFrames` (hittable list only).
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
    strata_buckets: &[Vec<u64>],
    elapsed_secs: f64,
) -> (QuadBatch, CollectedFrames) {
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
    let collected = collect_hittable_frames(registry, strata_buckets);

    for bucket in strata_buckets {
        emit_single_strata(
            &mut batch, bucket, registry,
            &visible_ids, pressed_frame, hovered_frame,
            text_ctx, message_frames, tooltip_data, elapsed_secs,
        );
    }
    (batch, collected)
}


use crate::widget::FrameStrata;
use std::sync::Arc;

impl App {
    /// Return per-strata dirty batches, rebuilding only strata whose bit is
    /// set in `strata_dirty`. Clean strata get `None` — the GPU pipeline
    /// keeps their buffers from the previous frame.
    ///
    /// Returns `(batches, rebuilt)` where `rebuilt` is true when any strata
    /// was re-emitted (used for frame-time measurement).
    fn get_or_rebuild_quads(
        &self,
        size: Size,
    ) -> ([Option<Arc<QuadBatch>>; FrameStrata::COUNT], bool) {
        let mut size_cache = self.cached_quads.borrow_mut();
        let size_changed = size_cache.as_ref().map(|(s, _)| *s != size).unwrap_or(true);

        if size_changed {
            self.mark_all_strata_dirty();
            // Invalidate per-strata cache — screen size changed.
            *self.cached_strata_quads.borrow_mut() = std::array::from_fn(|_| None);
        }

        let dirty = self.strata_dirty.get();
        if dirty == 0 {
            return (std::array::from_fn(|_| None), false);
        }

        self.rebuild_dirty_strata(size, dirty);
        self.strata_dirty.set(0);
        // Record current size so next frame detects resize.
        *size_cache = Some((size, Arc::new(QuadBatch::new())));

        let strata = self.cached_strata_quads.borrow();
        let result = std::array::from_fn(|i| {
            if dirty & (1 << i) != 0 {
                strata[i].clone()
            } else {
                None
            }
        });
        (result, true)
    }

    /// Rebuild only the strata whose bits are set in `dirty`.
    ///
    /// Stores results in `cached_strata_quads`. Also updates the hittable
    /// grid on first build and syncs layout caches.
    fn rebuild_dirty_strata(&self, size: Size, dirty: u16) {
        let env = self.env.borrow();
        let mut font_sys = self.font_system.borrow_mut();

        // Mutable phase: ensure strata buckets exist.
        let strata_buckets = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            super::tooltip::update_tooltip_sizes(&mut state, &mut font_sys);
            let _ = state.get_strata_buckets();
            state.strata_buckets.take().unwrap()
        };

        let state = env.state().borrow();
        let elapsed_secs = state.start_time.elapsed().as_secs_f64();
        let tooltip_data = super::tooltip::collect_tooltip_data(&state);
        let mut glyph_atlas = self.glyph_atlas.borrow_mut();
        glyph_atlas.advance_generation();
        let mut text_ctx: Option<(&mut WowFontSystem, &mut GlyphAtlas)> =
            Some((&mut font_sys, &mut glyph_atlas));

        let mut strata_cache = self.cached_strata_quads.borrow_mut();

        for i in 0..FrameStrata::COUNT {
            if dirty & (1 << i) == 0 && strata_cache[i].is_some() {
                continue;
            }
            let mut batch = QuadBatch::new();
            // World strata (index 0) gets the marble background.
            if i == 0 {
                batch.push_tiled_path(
                    Rectangle::new(Point::ORIGIN, size),
                    256.0,
                    256.0,
                    "framegeneral/ui-background-marble",
                    [0.55, 0.55, 0.55, 1.0],
                );
            }
            if let Some(bucket) = strata_buckets.get(i) {
                emit_single_strata(
                    &mut batch, bucket, &state.widgets,
                    &None,
                    self.pressed_frame, None,
                    &mut text_ctx, Some(&state.message_frames),
                    Some(&tooltip_data), elapsed_secs,
                );
            }
            strata_cache[i] = Some(Arc::new(batch));
        }
        drop(strata_cache);

        // Build hittable grid on first render.
        if self.cached_hittable.borrow().is_none() {
            let collected = collect_hittable_frames(&state.widgets, &strata_buckets);
            let hittable = build_hittable_rects(&collected, &state.widgets);
            let grid = super::hit_grid::HitGrid::new(hittable, size.width, size.height);
            *self.cached_hittable.borrow_mut() = Some(grid);
        }
        drop(state);
        self.apply_hit_grid_changes();

        let mut state = env.state().borrow_mut();
        state.strata_buckets = Some(strata_buckets);
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
            if let Some(gpu_data) = load_texture_or_crop(&mut tex_mgr, &request.path) {
                uploaded.insert(request.path.clone());
                textures.push(gpu_data);
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

    /// Append hover highlight quads for the currently hovered button.
    fn append_hover_highlight(&self, quads: &mut QuadBatch) {
        let Some(hovered_id) = self.hovered_frame else { return };
        let env = self.env.borrow();
        let state = env.state().borrow();
        let registry = &state.widgets;
        let Some(f) = registry.get(hovered_id) else { return };

        if !matches!(f.widget_type, WidgetType::Button | WidgetType::CheckButton) {
            return;
        }

        let Some(rect) = f.layout_rect else { return };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let bounds = Rectangle::new(
            Point::new(rect.x * UI_SCALE, rect.y * UI_SCALE),
            Size::new(rect.width * UI_SCALE, rect.height * UI_SCALE),
        );

        let has_highlight_child = f.children_keys.contains_key("HighlightTexture");
        let is_pressed = self.pressed_frame == Some(hovered_id) || f.button_state == 1;
        if !is_pressed && !has_highlight_child {
            emit_button_highlight(quads, bounds, f, f.alpha);
        }

        if let Some(&ht_id) = f.children_keys.get("HighlightTexture")
            && let Some(ht) = registry.get(ht_id) {
                let Some(ht_rect) = ht.layout_rect else { return };
                if ht_rect.width > 0.0 && ht_rect.height > 0.0 {
                    let ht_bounds = Rectangle::new(
                        Point::new(ht_rect.x * UI_SCALE, ht_rect.y * UI_SCALE),
                        Size::new(ht_rect.width * UI_SCALE, ht_rect.height * UI_SCALE),
                    );
                    build_texture_quads(quads, ht_bounds, ht, None, ht.alpha);
                }
            }
    }

    /// Render the spell icon attached to the cursor when dragging.
    fn append_cursor_item_icon(&self, overlay: &mut QuadBatch, pos: Point) {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let spell_id = match &state.cursor_item {
            Some(crate::lua_api::state::CursorInfo::Action { spell_id, .. }) => *spell_id,
            Some(crate::lua_api::state::CursorInfo::Spell { spell_id }) => *spell_id,
            None => return,
        };
        let Some(spell) = crate::spells::get_spell(spell_id) else { return };
        let Some(path) = crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id) else { return };
        let tex_path = format!("Interface\\{}", path.replace('/', "\\"));

        const ICON_SIZE: f32 = 32.0;
        // WoW centers the drag icon on the cursor position.
        let icon_bounds = Rectangle::new(
            Point::new(pos.x - ICON_SIZE * 0.5, pos.y - ICON_SIZE * 0.5),
            Size::new(ICON_SIZE, ICON_SIZE),
        );
        overlay.push_textured_path(
            icon_bounds,
            &tex_path,
            [1.0, 1.0, 1.0, 1.0],
            crate::render::BlendMode::Alpha,
        );
    }
}
