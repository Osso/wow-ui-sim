use crate::lua_api::WowLuaEnv;
use iced::Task;
use rustc_hash::FxHashSet;
use std::time::Instant;

use super::Message;
use super::app::App;
use super::update::{
    TickStageTimings, log_slow_tick, request_redraw_task, should_drop_stale_timer_tick,
};
use super::update_helpers::{
    apply_hit_grid_batch, get_checked_attribute, is_toggleable_checkbutton,
};

struct TimerTickOutcome {
    combined: u16,
    redraw_needed: bool,
}

impl App {
    pub(super) fn save_config(&self) {
        let mut config = crate::config::SimConfig::load();
        config.player_class = self.selected_class.clone();
        config.player_race = self.selected_race.clone();
        config.rot_damage_level = self.selected_rot_level.clone();
        config.xp_level = self.selected_xp_level.clone();
        config.party_size = self.selected_party_size.parse::<u8>().unwrap_or(0).min(4);
        config.movement = self.movement.clone();
        config.save();
    }

    pub(super) fn flush_post_script_updates(&mut self) {
        self.env.borrow().state().borrow_mut().ensure_layout_rects();
        if let Err(e) = self.env.borrow().fire_on_update(0.016) {
            crate::logging::eprintln_elapsed(&format!("[OnUpdate] post-script flush error: {e}"));
        }
        self.invalidate_after_lua_mutation();
    }

    pub(super) fn invalidate_after_lua_mutation(&mut self) {
        self.drain_console();
        if !self.has_queued_texture_preloads() && !self.textures_pending.get() {
            self.preload_visible_textures();
        }
        self.clear_failed_texture_requests();
        self.merge_widget_dirty_into_render_state();
        self.preload_render_texture_requests_preserving_dirty(Some(
            std::time::Duration::from_millis(25),
        ));
    }

    fn merge_widget_dirty_into_render_state(&self) {
        let (dirty_mask, dirty_ids) = self.take_render_dirty_with_ids();
        if dirty_mask == 0 {
            return;
        }
        self.mark_strata_dirty(dirty_mask);
        self.merge_pending_dirty_ids(dirty_ids);
    }

    pub(super) fn invalidate(&mut self) {
        self.drain_console();
        if !self.has_queued_texture_preloads() && !self.textures_pending.get() {
            self.preload_visible_textures();
        }
        self.clear_failed_texture_requests();
        self.mark_all_strata_dirty();
        self.preload_render_texture_requests_preserving_dirty(Some(
            std::time::Duration::from_millis(25),
        ));
    }

    pub(super) fn preload_visible_textures(&self) {
        let _ = self.preload_visible_textures_with_budget(std::time::Duration::from_millis(50));
    }

    pub(super) fn preload_visible_textures_with_budget(&self, budget: std::time::Duration) -> bool {
        let paths = self.warmup_texture_paths();
        let deadline = std::time::Instant::now() + budget;
        let pending_before = self.textures_pending.get();
        let loaded = self.preload_missing_texture_paths(&paths, deadline);
        let remaining_pending =
            self.remaining_texture_work_after_warmup(&paths, loaded, pending_before);
        self.textures_pending.set(remaining_pending);
        if loaded > 0 {
            crate::logging::eprintln_elapsed(&format!(
                "[texture-cache-warmup] {loaded} new textures ({} requested)",
                paths.len()
            ));
        }
        loaded != 0 || (!pending_before && remaining_pending)
    }

    fn preload_missing_texture_paths(
        &self,
        paths: &[String],
        deadline: std::time::Instant,
    ) -> usize {
        let mut loaded = 0usize;
        let mut tex_mgr = self.texture_manager.borrow_mut();
        for path in paths {
            if texture_request_is_cached(&tex_mgr, path) {
                continue;
            }
            super::render::preload_texture_request_source(&mut tex_mgr, path);
            if texture_request_is_cached(&tex_mgr, path) {
                loaded += 1;
            }
            if std::time::Instant::now() >= deadline {
                break;
            }
        }
        loaded
    }

    fn remaining_texture_work_after_warmup(
        &self,
        paths: &[String],
        loaded: usize,
        pending_before: bool,
    ) -> bool {
        let cached_request_paths = self.cached_render_request_paths();
        let has_warmup_paths = !paths.is_empty();
        let cpu_decode_pending = self.has_uncached_warmup_paths(paths);
        let draw_upload_pending =
            self.cached_render_requests_pending_after_warmup(&cached_request_paths);

        cpu_decode_pending
            || draw_upload_pending
            || loaded != 0
            || (pending_before && has_warmup_paths)
    }

    fn cached_render_requests_pending_after_warmup(&self, cached_request_paths: &[String]) -> bool {
        if cached_request_paths.is_empty() {
            return false;
        }

        self.seed_pending_texture_paths_from_cached_strata();
        self.cached_render_requests_still_pending()
    }

    fn has_uncached_warmup_paths(&self, paths: &[String]) -> bool {
        let tex_mgr = self.texture_manager.borrow();
        paths
            .iter()
            .any(|path| !texture_request_is_cached(&tex_mgr, path))
    }

    fn warmup_texture_paths(&self) -> Vec<String> {
        let mut cached_paths = self.cached_render_request_paths();
        if !cached_paths.is_empty() {
            Self::sort_texture_request_paths(&mut cached_paths);
            return cached_paths;
        }

        let env = self.env.borrow();
        let mut visible_paths = env.state().borrow().widgets.visible_texture_paths();
        Self::sort_texture_request_paths(&mut visible_paths);
        visible_paths
    }

    pub(super) fn has_queued_texture_preloads(&self) -> bool {
        let env = self.env.borrow();
        !env.state().borrow().pending_texture_preloads.is_empty()
    }

    fn cached_render_request_paths(&self) -> Vec<String> {
        let mut paths = FxHashSet::default();
        let strata = self.cached_strata_quads.borrow();
        for batch in strata.iter().flatten() {
            for request in batch
                .texture_requests
                .iter()
                .chain(&batch.mask_texture_requests)
            {
                paths.insert(request.path.clone());
            }
        }
        paths.into_iter().collect()
    }

    pub(super) fn apply_hit_grid_changes(&self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        let layout_roots = state.widgets.pending_layout_roots();
        if !layout_roots.is_empty() {
            state.ensure_layout_rects();
        }
        let hit_grid_roots = state.widgets.drain_hit_grid_dirty();
        let changes = std::mem::take(&mut state.pending_hit_grid_changes);
        if changes.is_empty() && layout_roots.is_empty() && hit_grid_roots.is_empty() {
            return;
        }
        let mut grid_ref = self.cached_hittable.borrow_mut();
        let Some(grid) = grid_ref.as_mut() else {
            return;
        };
        let buckets = state
            .get_strata_buckets()
            .expect("hit-grid updates require current render buckets")
            .clone();
        apply_hit_grid_batch(
            grid,
            &state.widgets,
            &buckets,
            hit_grid_roots.into_iter().chain(layout_roots),
            &changes,
        );
    }

    pub(super) fn is_frame_enabled(&self, frame_id: u64) -> bool {
        let env = self.env.borrow();
        let state = env.state().borrow();
        state
            .widgets
            .get(frame_id)
            .and_then(|f| f.attributes.get("__enabled"))
            .and_then(|v| match v {
                crate::widget::AttributeValue::Boolean(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(true)
    }

    pub(super) fn update_editbox_focus(&self, clicked_frame: Option<u64>) {
        let env = self.env.borrow();
        let Some((old_focus, new_focus)) = resolve_editbox_focus_change(&env, clicked_frame) else {
            return;
        };

        apply_editbox_focus_state(&env, old_focus, new_focus);
        fire_editbox_focus_scripts(&env, old_focus, new_focus);
    }

    pub(super) fn toggle_checkbutton_if_needed(&self, frame_id: u64, env: &WowLuaEnv) {
        let mut state = env.state().borrow_mut();
        if !is_toggleable_checkbutton(&state, frame_id) {
            return;
        }
        let new_checked = !get_checked_attribute(&state, frame_id);
        if let Some(frame) = state.widgets.get_mut(frame_id) {
            frame.attributes.insert(
                "__checked".to_string(),
                crate::widget::AttributeValue::Boolean(new_checked),
            );
        }
        let tex_id = state
            .widgets
            .get(frame_id)
            .and_then(|f| f.children_keys.get("CheckedTexture").copied());
        if let Some(tex_id) = tex_id {
            state.set_frame_visible(tex_id, new_checked);
        }
    }

    pub(crate) fn sync_screen_size_to_state(&self, size: iced::Size) {
        let env = self.env.borrow();
        let state = env.state().borrow();
        if (state.screen_width - size.width).abs() > 0.5
            || (state.screen_height - size.height).abs() > 0.5
        {
            crate::logging::println_elapsed(&format!(
                "Window size: {}x{} (was {}x{})",
                size.width as i32,
                size.height as i32,
                state.screen_width as i32,
                state.screen_height as i32
            ));
            drop(state);
            env.set_screen_size(size.width, size.height);
            *self.cached_hittable.borrow_mut() = None;
            self.mark_all_strata_dirty();
            self.rebuild_hit_grid_after_screen_resize(size);
        }
    }

    fn rebuild_hit_grid_after_screen_resize(&self, size: iced::Size) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let Some(strata_buckets) = state.get_strata_buckets().cloned() else {
            return;
        };
        let collected =
            super::frame_collect::collect_hittable_frames(&state.widgets, &strata_buckets);
        let hittable = super::strata_emit::build_hittable_rects(&collected, &state.widgets);
        let grid = super::hit_grid::HitGrid::new(hittable, size.width, size.height);
        *self.cached_hittable.borrow_mut() = Some(grid);
    }

    pub(crate) fn drain_console(&mut self) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        self.log_messages.append(&mut state.console_output);
    }

    fn sort_texture_request_paths(paths: &mut [String]) {
        paths.sort_by(|a, b| {
            texture_request_priority(a)
                .cmp(&texture_request_priority(b))
                .then_with(|| a.cmp(b))
        });
    }

    pub(super) fn cached_render_request_ready_count(&self) -> usize {
        self.cached_strata_quads
            .borrow()
            .iter()
            .flatten()
            .map(|batch| {
                batch
                    .texture_requests
                    .iter()
                    .chain(&batch.mask_texture_requests)
                    .filter(|request| request.handle.is_ready())
                    .count()
            })
            .sum()
    }

    pub(super) fn clear_failed_texture_requests(&self) {
        let mut retried = false;
        for batch in self.cached_strata_quads.borrow().iter().flatten() {
            for request in batch
                .texture_requests
                .iter()
                .chain(&batch.mask_texture_requests)
            {
                if request.handle.is_failed() {
                    request.handle.mark_retry();
                    retried = true;
                }
            }
        }
        if retried {
            self.seed_pending_texture_paths_from_cached_strata();
        }
    }

    pub(super) fn handle_process_timers(&mut self, captured_at: Instant) -> Task<Message> {
        if self.drop_stale_timer_tick(captured_at) {
            return Task::none();
        }
        if !self.gui_startup_complete.get() {
            return request_redraw_task();
        }

        self.set_main_thread_phase("process_timers");
        let tick_started = std::time::Instant::now();
        let mut stage_timings = TickStageTimings::default();
        let outcome = self.run_timer_tick_stages(&mut stage_timings);

        self.finish_timer_tick(
            tick_started,
            outcome.combined,
            outcome.redraw_needed,
            &mut stage_timings,
        )
    }

    fn run_timer_tick_stages(&mut self, stage_timings: &mut TickStageTimings) -> TimerTickOutcome {
        let mut redraw_needed = self.preload_visible_textures_for_tick(stage_timings);
        redraw_needed |= self.run_pending_lua_for_tick(stage_timings);

        let combined = self.collect_tick_work(stage_timings);
        redraw_needed |= self.mark_tick_dirty(combined, stage_timings);
        redraw_needed |= self.preload_pending_render_requests_for_tick(stage_timings);

        TimerTickOutcome {
            combined,
            redraw_needed,
        }
    }

    /// Maximum time ticks may be dropped as stale before one is force-processed.
    /// Stale-dropping is a backlog optimization, not a license to starve: under
    /// sustained main-thread saturation (heavy draws plus input-driven redraws)
    /// every tick arrives older than the stale threshold. If all of them were
    /// dropped, OnUpdate handlers and C_Timer callbacks would never run and the
    /// FPS display (refreshed in finish_timer_tick) would freeze at its last
    /// value while the app looks alive.
    const STALE_TICK_PROGRESS_BUDGET: std::time::Duration = std::time::Duration::from_millis(250);

    fn drop_stale_timer_tick(&self, captured_at: Instant) -> bool {
        let age = captured_at.elapsed();
        let interval = self
            .compute_tick_interval()
            .unwrap_or(std::time::Duration::from_secs(1));
        if !should_drop_stale_timer_tick(age, interval) {
            self.last_timer_tick_processed.set(Instant::now());
            return false;
        }

        if self.last_timer_tick_processed.get().elapsed() >= Self::STALE_TICK_PROGRESS_BUDGET {
            self.last_timer_tick_processed.set(Instant::now());
            return false;
        }

        let dropped = self.dropped_stale_timer_ticks.get().saturating_add(1);
        self.dropped_stale_timer_ticks.set(dropped);
        let oldest = self.oldest_dropped_timer_tick_age.get().max(age);
        self.oldest_dropped_timer_tick_age.set(oldest);
        if crate::logging::gui_trace_enabled() {
            crate::logging::eprintln_gui_trace(&format!(
                "tick dropped stale age={age:.1?} interval={interval:.1?} dropped_total={dropped}"
            ));
        }
        true
    }

    fn preload_visible_textures_for_tick(&self, stage_timings: &mut TickStageTimings) -> bool {
        if self.textures_pending.get() || self.has_pending_render_work() {
            return false;
        }

        let started = std::time::Instant::now();
        let loaded =
            self.preload_visible_textures_with_budget(std::time::Duration::from_millis(10));
        stage_timings.preload += started.elapsed();
        loaded
    }

    fn has_pending_render_work(&self) -> bool {
        self.strata_dirty.get() != 0
            || self
                .pending_dirty_ids
                .borrow()
                .as_ref()
                .is_some_and(|ids| !ids.is_empty())
    }

    fn run_pending_lua_for_tick(&mut self, stage_timings: &mut TickStageTimings) -> bool {
        let started = std::time::Instant::now();
        self.run_pending_exec_lua();
        stage_timings.exec_lua = started.elapsed();
        self.strata_dirty.get() != 0
    }

    fn collect_tick_work(&mut self, stage_timings: &mut TickStageTimings) -> u16 {
        let (combined, collect_timings) = self.collect_tick_dirty();
        stage_timings.timers = collect_timings.timers;
        stage_timings.layout = collect_timings.layout;
        stage_timings.on_update = collect_timings.on_update;
        stage_timings.party_health = collect_timings.party_health;
        stage_timings.casting = collect_timings.casting;
        combined
    }

    fn mark_tick_dirty(&mut self, combined: u16, stage_timings: &mut TickStageTimings) -> bool {
        let started = std::time::Instant::now();
        self.drain_console();
        stage_timings.console = started.elapsed();
        if combined != 0 {
            let started = std::time::Instant::now();
            self.mark_strata_dirty(combined);
            stage_timings.mark_dirty = started.elapsed();
            return true;
        }
        false
    }

    fn preload_pending_render_requests_for_tick(
        &self,
        stage_timings: &mut TickStageTimings,
    ) -> bool {
        let has_cached_render_requests = !self.cached_render_request_paths().is_empty();
        if self.strata_dirty.get() != 0 || has_cached_render_requests {
            let started = std::time::Instant::now();
            let preloaded = self.preload_render_texture_requests_preserving_dirty(Some(
                std::time::Duration::from_millis(25),
            ));
            stage_timings.preload += started.elapsed();
            return preloaded;
        }
        false
    }

    fn finish_timer_tick(
        &mut self,
        tick_started: std::time::Instant,
        combined: u16,
        redraw_needed: bool,
        stage_timings: &mut TickStageTimings,
    ) -> Task<Message> {
        let tick_elapsed = tick_started.elapsed();
        stage_timings.total = tick_elapsed;
        self.record_tick_time(tick_elapsed);
        self.update_fps_counter();
        log_slow_tick(&stage_timings, combined, self);
        if crate::logging::gui_trace_enabled() {
            let ready_count = self.cached_render_request_ready_count();
            crate::logging::eprintln_gui_trace(&format!(
                "tick redraw_request={} dirty=0x{:x} pending={} ready={ready_count}",
                redraw_needed,
                self.strata_dirty.get(),
                self.textures_pending.get()
            ));
        }
        if redraw_needed {
            request_redraw_task()
        } else {
            Task::none()
        }
    }
}

fn resolve_editbox_focus_change(
    env: &WowLuaEnv,
    clicked_frame: Option<u64>,
) -> Option<(Option<u64>, Option<u64>)> {
    let new_focus = env.resolve_editbox_focus_target(clicked_frame);
    let old_focus = env.state().borrow().focused_frame_id;

    (new_focus != old_focus).then_some((old_focus, new_focus))
}

fn apply_editbox_focus_state(env: &WowLuaEnv, old_focus: Option<u64>, new_focus: Option<u64>) {
    let mut state = env.state().borrow_mut();
    state.focused_frame_id = new_focus;
    set_editbox_visual_focus(&mut state, old_focus, false);
    set_editbox_visual_focus(&mut state, new_focus, true);
}

fn set_editbox_visual_focus(
    state: &mut crate::lua_api::state::SimState,
    frame_id: Option<u64>,
    focused: bool,
) {
    let Some(frame_id) = frame_id else {
        return;
    };
    if let Some(frame) = state.widgets.get_mut_visual(frame_id) {
        frame.editbox_focused = focused;
    }
    state.widgets.mark_visual_dirty(frame_id);
}

fn fire_editbox_focus_scripts(env: &WowLuaEnv, old_focus: Option<u64>, new_focus: Option<u64>) {
    if let Some(old_id) = old_focus {
        let _ = env.fire_script_handler(old_id, "OnEditFocusLost", vec![]);
    }
    if let Some(new_id) = new_focus {
        let _ = env.fire_script_handler(new_id, "OnEditFocusGained", vec![]);
    }
}

fn texture_request_is_cached(tex_mgr: &crate::texture::TextureManager, path: &str) -> bool {
    if path.contains("@crop:") {
        return tex_mgr.get_cached_crop_request(path).is_some();
    }
    tex_mgr.is_cached(path)
}

fn texture_request_priority(path: &str) -> (u8, u8) {
    let is_world_map = path
        .get(..19)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Interface\\WorldMap\\"))
        || path
            .get(..19)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Interface/WorldMap/"));
    let is_crop = path.contains("@crop:");
    (u8::from(!is_world_map), u8::from(is_crop))
}
