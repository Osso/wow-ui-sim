//! Shader rendering implementation.

use iced::mouse;
use iced::widget::shader;
use iced::{Event, Rectangle, Size, window};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::render::font::WowFontSystem;
use crate::render::glyph::GlyphAtlas;
use crate::render::{GpuBcTextureData, GpuTextureData, QuadBatch, WowUiPrimitive};

use super::Message;
use super::app::App;
use super::frame_collect::collect_hittable_frames;
use super::state::CanvasMessage;
use super::strata_emit::build_hittable_rects;
use super::update_helpers::apply_hit_grid_batch;

#[path = "render_draw_frame.rs"]
mod draw_frame;
#[path = "render_draw_log.rs"]
mod draw_log;
#[path = "render_mouse_events.rs"]
mod mouse_events;
#[path = "render_preload.rs"]
mod preload;
mod rebuild;

#[path = "render_textures.rs"]
mod textures;

#[cfg(test)]
mod test_support;

use draw_log::{DrawLogMetrics, log_draw_metrics};
use mouse_events::handle_mouse_event;
pub(crate) use preload::preload_texture_request_source;
#[cfg(test)]
use preload::{TexturePreloadPassTelemetry, format_texture_preload_log, texture_preload_reason};
use preload::{
    prune_completed_texture_requests_by_strata, sort_pending_texture_paths,
    update_ready_texture_path_cache,
};
use rebuild::prune_irrelevant_dirty_strata;
pub use rebuild::{DirtyStrataRebuildParams, rebuild_dirty_strata_batches_for_registry};

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
            Event::Mouse(me) => handle_mouse_event(me, bounds, cursor),
            Event::Window(window::Event::Unfocused | window::Event::Closed) => Some(
                shader::Action::publish(Message::CanvasEvent(CanvasMessage::MouseLeave)),
            ),
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        self.draw_wow_ui_primitive(bounds)
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

use crate::widget::FrameStrata;
use std::sync::Arc;

type PendingTextureRequestsByPath = FxHashMap<String, Vec<crate::render::TextureRequest>>;
type PendingTextureRequestsByStrata = [PendingTextureRequestsByPath; FrameStrata::COUNT];

struct DrawQuadRebuild {
    dirty_strata: [Option<Arc<QuadBatch>>; FrameStrata::COUNT],
    dirty_before: u16,
    quad_dur: std::time::Duration,
    had_textures_pending: bool,
}

impl App {
    /// Scan cached (non-dirty) strata for unresolved texture requests and
    /// include them in the primitive so `prepare()` re-resolves their refs
    /// against the updated GPU atlas.
    fn recover_pending_textures(
        &self,
        dirty_strata: &mut [Option<Arc<QuadBatch>>; FrameStrata::COUNT],
        texture_requests: &Arc<
            std::sync::Mutex<crate::render::shader::primitive::TextureRequestTracker>,
        >,
    ) {
        self.prune_completed_pending_texture_paths();
        let cached = self.cached_strata_quads.borrow();
        let strata_pending = self.strata_pending_texture_requests.borrow();
        let mut texture_requests = texture_requests.lock().unwrap();
        let mut reinjected = false;
        for i in 0..dirty_strata.len() {
            if dirty_strata[i].is_none()
                && let Some(batch) = &cached[i]
                && strata_pending[i]
                    .values()
                    .any(|requests| requests.iter().any(|request| request.handle.is_pending()))
            {
                texture_requests.register_batch(batch);
                dirty_strata[i] = Some(batch.clone());
                reinjected = true;
            }
        }
        if reinjected {
            self.textures_pending.set(true);
        }
    }

    fn rebuild_draw_quads(&self, size: Size) -> DrawQuadRebuild {
        let had_textures_pending = self.textures_pending.get();
        let dirty_before = self.strata_dirty.get();
        let started = std::time::Instant::now();
        let (dirty_strata, _) = self.get_or_rebuild_quads(size);
        DrawQuadRebuild {
            dirty_strata,
            dirty_before,
            quad_dur: started.elapsed(),
            had_textures_pending,
        }
    }

    fn build_draw_primitive(
        &self,
        dirty_strata: [Option<Arc<QuadBatch>>; FrameStrata::COUNT],
        overlay: QuadBatch,
        textures: Vec<GpuTextureData>,
        bc_textures: Vec<GpuBcTextureData>,
        texture_requests: Arc<
            std::sync::Mutex<crate::render::shader::primitive::TextureRequestTracker>,
        >,
    ) -> WowUiPrimitive {
        let mut primitive = WowUiPrimitive {
            strata_batches: dirty_strata,
            overlay,
            clear_color: [0.10, 0.11, 0.14, 1.0],
            textures,
            bc_textures,
            glyph_atlas_data: None,
            glyph_atlas_size: 0,
            texture_requests: Some(texture_requests),
        };
        self.attach_dirty_glyph_atlas(&mut primitive);
        primitive
    }

    fn record_draw_time(&self, elapsed: std::time::Duration) {
        let elapsed_ms = elapsed.as_secs_f32() * 1000.0;
        self.draw_time_accum_ms
            .set(self.draw_time_accum_ms.get() + elapsed_ms);
    }

    pub(crate) fn cached_render_requests_still_pending(&self) -> bool {
        self.prune_completed_pending_texture_paths();
        if self.pending_texture_path_set.borrow().is_empty() && self.textures_pending.get() {
            self.seed_pending_texture_paths_from_cached_strata();
            self.prune_completed_pending_texture_paths();
        }
        !self.pending_texture_path_set.borrow().is_empty()
    }

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
            // Invalidate hit grid — frame positions change with screen size.
            *self.cached_hittable.borrow_mut() = None;
        }

        let dirty = self.strata_dirty.get();
        if dirty == 0 {
            return (std::array::from_fn(|_| None), false);
        }

        let rebuilt = self.rebuild_dirty_strata(size, dirty);
        self.strata_dirty.set(0);
        // Record current size so next frame detects resize.
        *size_cache = Some((size, Arc::new(QuadBatch::new())));

        let strata = self.cached_strata_quads.borrow();
        let result = std::array::from_fn(|i| {
            if rebuilt & (1 << i) != 0 {
                strata[i].clone()
            } else {
                None
            }
        });
        (result, rebuilt != 0)
    }

    /// Rebuild only the strata whose bits are set in `dirty`.
    ///
    /// Stores results in `cached_strata_quads`. Also updates the hittable
    /// grid on first build and syncs layout caches.
    fn rebuild_dirty_strata(&self, size: Size, dirty: u16) -> u16 {
        let dirty_ids = {
            let mut pending_dirty_ids = self.pending_dirty_ids.borrow_mut();
            let drained = pending_dirty_ids.take();
            // A full-rebuild sentinel (`None`) only applies to this rebuild pass.
            // Reset to an empty concrete set so subsequent ticks can recover
            // incremental dirty IDs instead of remaining in permanent full mode.
            *pending_dirty_ids = Some(FxHashSet::default());
            drained
        };
        let effective_dirty = self.prune_dirty_strata(dirty, dirty_ids.as_ref());
        if effective_dirty == 0 {
            return self.finish_without_strata_rebuild(size);
        }

        let env = self.env.borrow();
        let mut font_sys = self.font_system.borrow_mut();
        let (strata_buckets, layout_roots) = self.resolve_layout_and_buckets(&env, &mut font_sys);
        let state = env.state().borrow();

        self.emit_dirty_strata_batches(
            effective_dirty,
            dirty_ids.as_ref(),
            size,
            &strata_buckets,
            &state,
            &mut font_sys,
        );
        drop(state);
        self.update_hit_grid_after_render(&env, strata_buckets, layout_roots, size);
        self.refresh_pending_texture_requests_for_rebuilt_strata(effective_dirty);
        effective_dirty
    }

    fn refresh_pending_texture_requests_for_rebuilt_strata(&self, rebuilt: u16) {
        self.refresh_pending_texture_requests_for_strata(Some(rebuilt));
    }

    pub(crate) fn seed_pending_texture_paths_from_cached_strata(&self) {
        self.refresh_pending_texture_requests_for_strata(None);
    }

    fn refresh_pending_texture_requests_for_strata(&self, rebuilt_strata_mask: Option<u16>) {
        if rebuilt_strata_mask == Some(0) {
            return;
        }
        let strata_cache = self.cached_strata_quads.borrow();
        let mut strata_pending = self.strata_pending_texture_requests.borrow_mut();
        for strata_idx in 0..FrameStrata::COUNT {
            let refresh_strata = match rebuilt_strata_mask {
                Some(mask) => mask & (1 << strata_idx) != 0,
                None => true,
            };
            if !refresh_strata {
                continue;
            }
            strata_pending[strata_idx] = strata_cache[strata_idx]
                .as_deref()
                .map_or_else(FxHashMap::default, |batch| {
                    self.collect_pending_texture_requests_for_batch(batch)
                });
        }
        drop(strata_pending);
        drop(strata_cache);
        self.rebuild_pending_texture_queue_from_strata_maps();
    }

    fn rebuild_pending_texture_queue_from_strata_maps(&self) {
        let strata_pending = self.strata_pending_texture_requests.borrow();
        let mut pending_by_path: FxHashMap<String, Vec<crate::render::TextureRequest>> =
            FxHashMap::default();
        for strata_map in strata_pending.iter() {
            for (path, requests) in strata_map {
                pending_by_path.entry(path.clone()).or_default().extend(
                    requests
                        .iter()
                        .filter(|request| request.handle.is_pending())
                        .cloned(),
                );
            }
        }
        pending_by_path.retain(|_, requests| !requests.is_empty());
        let mut ordered_paths: Vec<String> = pending_by_path.keys().cloned().collect();
        sort_pending_texture_paths(&mut ordered_paths);

        *self.pending_texture_requests_by_path.borrow_mut() = pending_by_path;
        *self.pending_texture_path_set.borrow_mut() = ordered_paths.iter().cloned().collect();
        *self.pending_texture_path_queue.borrow_mut() =
            std::collections::VecDeque::from(ordered_paths);
    }

    fn prune_completed_pending_texture_paths(&self) {
        let (changed, resolved_paths, unresolved_paths) = {
            let mut strata_pending = self.strata_pending_texture_requests.borrow_mut();
            prune_completed_texture_requests_by_strata(&mut strata_pending)
        };

        update_ready_texture_path_cache(
            &mut self.ready_texture_path_cache.borrow_mut(),
            resolved_paths,
            unresolved_paths,
        );

        if changed {
            self.rebuild_pending_texture_queue_from_strata_maps();
        }
    }

    fn pop_next_pending_texture_path(&self) -> Option<String> {
        self.pending_texture_path_queue.borrow_mut().pop_front()
    }

    fn requeue_pending_texture_path(&self, path: String) {
        if self.pending_texture_path_set.borrow().contains(&path) {
            self.pending_texture_path_queue.borrow_mut().push_back(path);
        }
    }

    fn pending_path_state(&self, path: &str) -> (bool, bool) {
        let pending_by_path = self.pending_texture_requests_by_path.borrow();
        let Some(requests) = pending_by_path.get(path) else {
            return (false, false);
        };
        let mut has_pending = false;
        let mut should_load = false;
        for request in requests {
            if request.handle.is_pending() {
                has_pending = true;
                if request.handle.should_load() {
                    should_load = true;
                    break;
                }
            }
        }
        (has_pending, should_load)
    }

    fn register_pending_texture_requests_for_path(
        &self,
        path: &str,
        texture_requests: &mut crate::render::shader::primitive::TextureRequestTracker,
    ) {
        let pending_by_path = self.pending_texture_requests_by_path.borrow();
        let Some(requests) = pending_by_path.get(path) else {
            return;
        };
        for request in requests {
            texture_requests.register_request(request);
        }
    }

    fn remove_pending_texture_path(&self, path: &str) {
        if !self.pending_texture_path_set.borrow_mut().remove(path) {
            return;
        }
        let had_ready = self
            .pending_texture_requests_by_path
            .borrow()
            .get(path)
            .is_some_and(|requests| requests.iter().any(|request| request.handle.is_ready()));
        if had_ready {
            self.ready_texture_path_cache
                .borrow_mut()
                .insert(path.to_string());
        } else {
            self.ready_texture_path_cache.borrow_mut().remove(path);
        }
        self.pending_texture_requests_by_path
            .borrow_mut()
            .remove(path);
        for strata_map in self.strata_pending_texture_requests.borrow_mut().iter_mut() {
            strata_map.remove(path);
        }
        self.pending_texture_path_queue
            .borrow_mut()
            .retain(|queued_path| queued_path != path);
    }

    fn collect_pending_texture_requests_for_batch(
        &self,
        batch: &crate::render::QuadBatch,
    ) -> FxHashMap<String, Vec<crate::render::TextureRequest>> {
        let ready_paths = self.ready_texture_path_cache.borrow();
        let mut pending: FxHashMap<String, Vec<crate::render::TextureRequest>> =
            FxHashMap::default();
        for request in batch
            .texture_requests
            .iter()
            .chain(&batch.mask_texture_requests)
        {
            if !request.handle.is_pending() {
                continue;
            }
            if ready_paths.contains(&request.path) {
                request.handle.mark_ready();
                continue;
            }
            pending
                .entry(request.path.clone())
                .or_default()
                .push(request.clone());
        }
        pending
    }

    fn prune_dirty_strata(&self, dirty: u16, dirty_ids: Option<&FxHashSet<u64>>) -> u16 {
        let env = self.env.borrow();
        let state = env.state().borrow();
        let strata_cache = self.cached_strata_quads.borrow();
        let snapshot_cache = self.cached_frame_snapshots.borrow();
        prune_irrelevant_dirty_strata(
            dirty,
            dirty_ids,
            &state.widgets,
            state.strata_buckets.as_deref(),
            &strata_cache,
            &snapshot_cache,
        )
    }

    fn finish_without_strata_rebuild(&self, size: Size) -> u16 {
        self.apply_hit_grid_changes();
        self.initialize_missing_hit_grid(size);
        0
    }

    fn initialize_missing_hit_grid(&self, size: Size) {
        if self.cached_hittable.borrow().is_some() {
            return;
        }
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let strata_buckets = state
            .get_strata_buckets()
            .expect("hit-grid initialization requires render buckets")
            .clone();
        self.rebuild_hit_grid_if_needed(&state, &strata_buckets, size);
    }

    /// Emit quads for dirty strata into the cache.
    fn emit_dirty_strata_batches(
        &self,
        dirty: u16,
        dirty_ids: Option<&FxHashSet<u64>>,
        size: Size,
        strata_buckets: &[Vec<u64>],
        state: &crate::lua_api::SimState,
        font_sys: &mut WowFontSystem,
    ) {
        let elapsed_secs = state.start_time.elapsed().as_secs_f64();
        let tooltip_data = super::tooltip::collect_tooltip_data(state);
        let mut glyph_atlas = self.glyph_atlas.borrow_mut();
        glyph_atlas.advance_generation();
        let mut text_ctx: Option<(&mut WowFontSystem, &mut GlyphAtlas)> =
            Some((font_sys, &mut glyph_atlas));

        let mut strata_cache = self.cached_strata_quads.borrow_mut();
        let mut snap_cache = self.cached_frame_snapshots.borrow_mut();
        rebuild_dirty_strata_batches_for_registry(
            &mut strata_cache,
            &mut snap_cache,
            &mut text_ctx,
            DirtyStrataRebuildParams {
                dirty,
                dirty_ids,
                size,
                strata_buckets,
                widgets: &state.widgets,
                pressed_frame: self.pressed_frame,
                hovered_frame: self.hovered_frame,
                message_frames: &state.message_frames,
                tooltip_data: &tooltip_data,
                quest_blobs: &state.quest_blobs,
                elapsed_secs,
            },
        );
    }

    fn update_hit_grid_after_render(
        &self,
        env: &crate::lua_api::WowLuaEnv,
        strata_buckets: Vec<Vec<u64>>,
        layout_roots: Vec<u64>,
        size: Size,
    ) {
        let mut state = env.state().borrow_mut();
        let hit_roots = state.widgets.drain_hit_grid_dirty();
        let changes = std::mem::take(&mut state.pending_hit_grid_changes);
        if let Some(grid) = self.cached_hittable.borrow_mut().as_mut() {
            apply_hit_grid_batch(
                grid,
                &state.widgets,
                &strata_buckets,
                hit_roots.into_iter().chain(layout_roots),
                &changes,
            );
        }
        self.rebuild_hit_grid_if_needed(&state, &strata_buckets, size);
        state.strata_buckets = Some(strata_buckets);
    }

    /// Resolve layout rects and build strata buckets, logging slow phases.
    fn resolve_layout_and_buckets(
        &self,
        env: &crate::lua_api::WowLuaEnv,
        font_sys: &mut WowFontSystem,
    ) -> (Vec<Vec<u64>>, Vec<u64>) {
        let mut state = env.state().borrow_mut();
        let t0 = std::time::Instant::now();
        super::tooltip::update_tooltip_sizes(&mut state, font_sys);
        let layout_roots = state.widgets.pending_layout_roots();
        state.ensure_layout_rects();
        let layout_dur = t0.elapsed();
        let t1 = std::time::Instant::now();
        let _ = state.get_strata_buckets();
        let bucket_dur = t1.elapsed();
        if layout_dur.as_millis() > 5 || bucket_dur.as_millis() > 5 {
            eprintln!(
                "{} [rebuild] layout={layout_dur:.1?} buckets={bucket_dur:.1?}",
                crate::logging::global_elapsed_prefix()
            );
        }
        (state.strata_buckets.take().unwrap(), layout_roots)
    }

    fn rebuild_hit_grid_if_needed(
        &self,
        state: &crate::lua_api::SimState,
        buckets: &[Vec<u64>],
        size: Size,
    ) {
        if self.cached_hittable.borrow().is_some() {
            return;
        }
        let t = std::time::Instant::now();
        let collected = collect_hittable_frames(&state.widgets, buckets);
        let hittable = build_hittable_rects(&collected, &state.widgets);
        let grid = super::hit_grid::HitGrid::new(hittable, size.width, size.height);
        *self.cached_hittable.borrow_mut() = Some(grid);
        let dur = t.elapsed();
        if dur.as_millis() > 5 {
            eprintln!(
                "{} [rebuild] hit_grid={dur:.1?}",
                crate::logging::global_elapsed_prefix()
            );
        }
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
}

#[cfg(test)]
include!("render_tests.rs");
#[cfg(test)]
include!("render_hit_grid_tests.rs");
