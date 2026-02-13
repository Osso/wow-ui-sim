//! Shader rendering implementation.

use iced::mouse;
use iced::widget::shader;
use iced::{Event, Point, Rectangle, Size};

use crate::render::font::WowFontSystem;
use crate::render::glyph::GlyphAtlas;
use crate::render::texture::UI_SCALE;
use crate::render::{GpuTextureData, QuadBatch, WowUiPrimitive};
use crate::widget::{WidgetType};

use super::app::App;
use super::frame_collect::{CollectedFrames, collect_subtree_ids, collect_ancestor_visible_ids, collect_sorted_frames};
use super::layout::LayoutCache;
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
                crate::render::BlendMode::Alpha,
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

/// Emit quads for all frames in a pre-built render list.
///
/// Looks up frame data from the registry by ID. This is the inner emit loop
/// shared by both the cached and uncached paths.
#[allow(clippy::too_many_arguments)]
fn emit_all_frames(
    batch: &mut QuadBatch,
    render_list: &[(u64, crate::LayoutRect, f32)],
    registry: &crate::widget::WidgetRegistry,
    screen_size: (f32, f32),
    visible_ids: &Option<std::collections::HashSet<u64>>,
    pressed_frame: Option<u64>,
    hovered_frame: Option<u64>,
    text_ctx: &mut Option<(&mut WowFontSystem, &mut GlyphAtlas)>,
    message_frames: Option<&std::collections::HashMap<u64, crate::lua_api::message_frame::MessageFrameData>>,
    tooltip_data: Option<&std::collections::HashMap<u64, TooltipRenderData>>,
    cache: &mut LayoutCache,
    elapsed_secs: f64,
) {
    let statusbar_fills = collect_statusbar_fills(render_list, registry);

    for &(id, rect, eff_alpha) in render_list {
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
        emit_frame_quads(batch, id, f, bounds, bar_fill, pressed_frame, hovered_frame, text_ctx, message_frames, tooltip_data, registry, screen_size, cache, elapsed_secs, eff_alpha);
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
) -> QuadBatch {
    let mut cache = LayoutCache::new();
    let ancestor_visible = collect_ancestor_visible_ids(registry);

    let (batch, _collected) = build_quad_batch_with_cache(
        registry, screen_size, root_name, pressed_frame, hovered_frame,
        &mut text_ctx, message_frames, tooltip_data, &mut cache,
        &ancestor_visible, None, None, 0.0,
    );
    batch
}

/// Build a QuadBatch, populating the shared layout cache for reuse by hit testing.
///
/// When `cached_render_list` is provided, skips the per-frame collection pass
/// (`collect_sorted_frames`) and re-emits quads from the cached list. Otherwise
/// builds the list from scratch and returns it for caching.
///
/// Returns the quad batch and the `CollectedFrames` used (for caller to cache).
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
    cached_render_list: Option<CollectedFrames>,
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
    let collected = if let Some(c) = cached_render_list {
        c
    } else {
        collect_sorted_frames(registry, screen_width, screen_height, ancestor_visible, strata_buckets, cache)
    };

    emit_all_frames(
        &mut batch, &collected.render, registry, screen_size,
        &visible_ids, pressed_frame, hovered_frame,
        text_ctx, message_frames, tooltip_data, cache, elapsed_secs,
    );
    (batch, collected)
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
        let (ancestor_visible, strata_buckets, mut cache, cached_render) = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            super::tooltip::update_tooltip_sizes(&mut state, &mut font_sys);
            let vis = state.get_ancestor_visible().clone();
            let buckets = state.get_strata_buckets().cloned();
            let layout = state.take_layout_cache();
            let render = state.cached_render_list.take();
            (vis, buckets, layout, render)
        };
        let state = env.state().borrow();
        let elapsed_secs = state.start_time.elapsed().as_secs_f64();
        let tooltip_data = super::tooltip::collect_tooltip_data(&state);
        let mut glyph_atlas = self.glyph_atlas.borrow_mut();
        let (batch, collected) = build_quad_batch_with_cache(
            &state.widgets, (size.width, size.height), None,
            self.pressed_frame, None,
            &mut Some((&mut font_sys, &mut glyph_atlas)),
            Some(&state.message_frames), Some(&tooltip_data),
            &mut cache, &ancestor_visible, strata_buckets.as_ref(),
            cached_render, elapsed_secs,
        );
        *self.cached_layout_rects.borrow_mut() = Some(cache.clone());
        *self.cached_hittable.borrow_mut() = Some(
            collected.hittable.iter().map(|&(id, r)| {
                (id, Rectangle::new(
                    Point::new(r.x * UI_SCALE, r.y * UI_SCALE),
                    Size::new(r.width * UI_SCALE, r.height * UI_SCALE),
                ))
            }).collect(),
        );
        drop(state);
        let mut state = env.state().borrow_mut();
        state.set_layout_cache(cache);
        state.cached_render_list = Some(collected);
        batch
    }

    /// Append hover highlight quads for the currently hovered button.
    fn append_hover_highlight(&self, quads: &mut QuadBatch, screen_size: Size) {
        let Some(hovered_id) = self.hovered_frame else { return };
        let env = self.env.borrow();
        let state = env.state().borrow();
        let registry = &state.widgets;
        let Some(f) = registry.get(hovered_id) else { return };

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

        let has_highlight_child = f.children_keys.contains_key("HighlightTexture");
        let is_pressed = self.pressed_frame == Some(hovered_id) || f.button_state == 1;
        if !is_pressed && !has_highlight_child {
            emit_button_highlight(quads, bounds, f, f.alpha);
        }

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
                    build_texture_quads(quads, ht_bounds, ht, None, ht.alpha);
                }
            }
    }
}
