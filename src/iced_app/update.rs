//! App::update() method and related logic.

#[cfg(not(target_os = "linux"))]
use crate::inspector_server_stub::ScreenshotData;
use iced::Task;
#[cfg(target_os = "linux")]
use iced_layout_inspector::server::ScreenshotData;
use rilua::Val;
use rustc_hash::FxHashSet;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::lua_api::WowLuaEnv;

use super::Message;
use super::app::App;
use super::state::CanvasMessage;
use super::update_helpers::merge_dirty_ids;

pub(super) fn request_redraw_task<T>() -> Task<T> {
    iced_runtime::task::effect(iced_runtime::Action::Window(
        iced_runtime::window::Action::RedrawAll,
    ))
}

fn request_runtime_exit_task() -> Task<Message> {
    iced_runtime::exit()
}

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let ipc_task = self.process_ipc();
        let task = match message {
            Message::CanvasEvent(cm) => self.handle_canvas_event(cm),
            Message::ProcessTimers(captured_at) => self.handle_process_timers(captured_at),
            msg => self.dispatch_simple_message(msg),
        };
        let exit_task = self.take_simulator_exit_task();
        Task::batch([exit_task, task, ipc_task])
    }

    fn dispatch_simple_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyPress(key, text, captured_at) => {
                self.handle_key_press_message(&key, text.as_deref(), captured_at)
            }
            Message::KeyRelease(key) => {
                self.env.borrow().send_key_up(&key);
                Task::none()
            }
            message => {
                self.dispatch_non_key_message(message);
                Task::none()
            }
        }
    }

    fn dispatch_non_key_message(&mut self, message: Message) {
        match message {
            Message::FireEvent(event) => self.handle_fire_event(&event),
            Message::Scroll(dx, dy) => self.handle_scroll(dx, dy),
            Message::ReloadUI => self.handle_reload_ui(),
            Message::CommandInputChanged(input) => self.command_input = input,
            Message::ExecuteCommand => self.handle_execute_command(),
            Message::ScreenshotTaken(ss) => self.handle_screenshot_taken(ss),
            Message::FpsTick => {}
            Message::InspectorClose
            | Message::InspectorWidthChanged(_)
            | Message::InspectorHeightChanged(_)
            | Message::InspectorAlphaChanged(_)
            | Message::InspectorLevelChanged(_)
            | Message::InspectorVisibleToggled(_)
            | Message::InspectorMouseEnabledToggled(_)
            | Message::InspectorApply => self.handle_inspector_message(message),
            Message::ToggleFramesPanel => self.toggle_frames_panel(),
            Message::XpLevelChanged(ref label) => self.handle_xp_level_changed(label),
            Message::PartySizeChanged(ref label) => self.handle_party_size_changed(label),
            Message::PlayerClassChanged(ref name) => self.handle_player_class_changed(name),
            Message::PlayerRaceChanged(ref name) => self.handle_player_race_changed(name),
            Message::RotDamageLevelChanged(ref label) => {
                self.handle_rot_damage_level_changed(label)
            }
            Message::ToggleOptionsModal => self.options_modal_visible = !self.options_modal_visible,
            Message::CloseOptionsModal => self.options_modal_visible = false,
            Message::MovementToggled(field, val) => self.handle_movement_toggled(field, val),
            Message::ModifiersChanged(modifiers) => self.handle_modifiers_changed(modifiers),
            // Handled before dispatch_non_key_message() or directly in update():
            Message::KeyPress(_, _, _)
            | Message::KeyRelease(_)
            | Message::CanvasEvent(_)
            | Message::ProcessTimers(_) => {
                unreachable!()
            }
        }
    }

    fn toggle_frames_panel(&mut self) {
        self.frames_panel_collapsed = !self.frames_panel_collapsed;
    }

    fn take_simulator_exit_task(&self) -> Task<Message> {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        if !state.simulator_exit_requested {
            return Task::none();
        }
        state.simulator_exit_requested = false;
        request_runtime_exit_task()
    }

    fn handle_key_press_message(
        &mut self,
        key: &str,
        text: Option<&str>,
        captured_at: Instant,
    ) -> Task<Message> {
        if key == "CTRL-Q" {
            return request_runtime_exit_task();
        }

        self.handle_simple_key_press(key, text, captured_at)
    }

    fn handle_inspector_message(&mut self, message: Message) {
        match message {
            Message::InspectorClose => self.handle_inspector_close(),
            Message::InspectorWidthChanged(v) => self.inspector_state.width = v,
            Message::InspectorHeightChanged(v) => self.inspector_state.height = v,
            Message::InspectorAlphaChanged(v) => self.inspector_state.alpha = v,
            Message::InspectorLevelChanged(v) => self.inspector_state.frame_level = v,
            Message::InspectorVisibleToggled(v) => self.inspector_state.visible = v,
            Message::InspectorMouseEnabledToggled(v) => self.inspector_state.mouse_enabled = v,
            Message::InspectorApply => self.handle_inspector_apply(),
            _ => unreachable!(),
        }
    }

    fn handle_simple_key_press(
        &mut self,
        key: &str,
        text: Option<&str>,
        captured_at: Instant,
    ) -> Task<Message> {
        let (phase, phase_elapsed) = crate::logging::blocking_phase_snapshot();
        let dropped_stale_ticks = self.dropped_stale_timer_ticks.take();
        let oldest_stale_tick_age = self.oldest_dropped_timer_tick_age.take();
        let mut message = format!(
            "[key] {key} reached app in {:.1?} (last phase={phase} for {:.1?})",
            captured_at.elapsed(),
            phase_elapsed
        );
        if dropped_stale_ticks > 0 {
            message.push_str(&format!(
                " after dropping {dropped_stale_ticks} stale timer ticks (oldest {:.1?})",
                oldest_stale_tick_age
            ));
        }
        crate::logging::eprintln_elapsed(&message);
        if key == "ESCAPE" && self.options_modal_visible {
            self.options_modal_visible = false;
            return Task::none();
        }

        self.handle_key_press(key, text, captured_at)
    }

    fn handle_modifiers_changed(&self, modifiers: iced::keyboard::Modifiers) {
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        state.modifier_keys.shift = modifiers.shift();
        state.modifier_keys.control = modifiers.control();
        state.modifier_keys.alt = modifiers.alt();
        state.modifier_keys.meta = modifiers.logo();
    }

    // ── Event handlers ──────────────────────────────────────────────────

    fn handle_fire_event(&mut self, event: &str) {
        {
            let env = self.env.borrow();
            if let Err(e) = env.fire_event(event) {
                self.log_messages.push(format!("Event error: {}", e));
            } else {
                self.log_messages.push(format!("Fired: {}", event));
            }
        }
        self.flush_post_script_updates();
    }

    fn handle_canvas_event(&mut self, canvas_msg: CanvasMessage) -> Task<Message> {
        let is_mouse_move = matches!(canvas_msg, CanvasMessage::MouseMove(_));
        match canvas_msg {
            CanvasMessage::MouseMove(pos) => self.handle_mouse_move(pos),
            CanvasMessage::MouseLeave => self.handle_mouse_leave(),
            CanvasMessage::MouseDown(pos) => self.handle_mouse_down(pos),
            CanvasMessage::MouseUp(pos) => self.handle_mouse_up(pos),
            CanvasMessage::RightMouseDown(pos) => self.handle_right_mouse_down(pos),
            CanvasMessage::RightMouseUp(pos) => self.handle_right_mouse_up(pos),
            CanvasMessage::MiddleClick(pos) => self.handle_middle_click(pos),
        }
        if self.canvas_event_needs_redraw(is_mouse_move) {
            request_redraw_task()
        } else {
            Task::none()
        }
    }

    fn canvas_event_needs_redraw(&self, is_mouse_move: bool) -> bool {
        if self.strata_dirty.get() != 0 {
            return true;
        }
        if is_mouse_move {
            return self.env.borrow().state().borrow().cursor_item.is_some();
        }
        self.textures_pending.get()
    }

    pub(super) fn handle_key_press(
        &mut self,
        key: &str,
        text: Option<&str>,
        captured_at: Instant,
    ) -> Task<Message> {
        let dispatch_started = Instant::now();
        let env = self.env.borrow();
        if let Err(e) = env.send_key_down(key, text) {
            self.log_messages
                .push(format!("KeyPress({}) error: {}", key, e));
        }
        drop(env);
        crate::logging::eprintln_elapsed(&format!(
            "[key] {key} app->lua dispatch took {:.1?} ({:.1?} since capture)",
            dispatch_started.elapsed(),
            captured_at.elapsed()
        ));
        self.flush_post_script_updates();
        if self.key_press_needs_redraw() {
            request_redraw_task()
        } else {
            Task::none()
        }
    }

    fn key_press_needs_redraw(&self) -> bool {
        self.strata_dirty.get() != 0
            || self.textures_pending.get()
            || self.env.borrow().state().borrow().casting.is_some()
    }

    fn handle_xp_level_changed(&mut self, label: &str) {
        use crate::lua_api::state::XP_LEVELS;
        self.selected_xp_level = label.to_string();
        let fraction = XP_LEVELS
            .iter()
            .find(|(l, _)| *l == label)
            .map(|(_, f)| *f)
            .unwrap_or(0.0);
        let at_max = fraction == 0.0;
        let event = if at_max {
            "DISABLE_XP_GAIN"
        } else {
            "ENABLE_XP_GAIN"
        };
        {
            let env = self.env.borrow();
            let xp_max = 89_750i32;
            let xp_current = (xp_max as f64 * fraction) as i32;
            let lua_code = format!(
                "IsPlayerAtEffectiveMaxLevel = function() return {} end; \
                 UnitXP = function() return {} end; \
                 UnitXPMax = function() return {} end",
                at_max, xp_current, xp_max
            );
            if let Err(e) = env.exec(&lua_code) {
                self.log_messages.push(format!("XP level error: {}", e));
            }
            if let Err(e) = env.fire_event(event) {
                self.log_messages.push(format!("XP event error: {}", e));
            }
        }
        self.save_config();
        self.invalidate();
    }

    fn handle_party_size_changed(&mut self, label: &str) {
        let size = label.parse::<usize>().unwrap_or(0).min(4);
        self.selected_party_size = size.to_string();
        {
            let env = self.env.borrow();
            let mut state = env.state().borrow_mut();
            crate::startup::resize_party_state(&mut state, size);
            drop(state);
            crate::startup::refresh_party_frames(&env);
        }
        self.save_config();
        self.invalidate();
    }

    fn handle_player_class_changed(&mut self, class_name: &str) {
        use crate::lua_api::state::CLASS_LABELS;
        let index = CLASS_LABELS
            .iter()
            .position(|&n| n == class_name)
            .map(|i| (i + 1) as i32)
            .unwrap_or(1);
        self.selected_class = class_name.to_string();
        {
            let env = self.env.borrow();
            env.state().borrow_mut().player.class_index = index;
            self.fire_portrait_update(&env);
        }
        self.save_config();
        self.invalidate();
    }

    fn handle_player_race_changed(&mut self, race_name: &str) {
        use crate::lua_api::state::RACE_DATA;
        let index = RACE_DATA
            .iter()
            .position(|(name, _, _)| *name == race_name)
            .unwrap_or(0);
        self.selected_race = race_name.to_string();
        {
            let env = self.env.borrow();
            env.state().borrow_mut().player.race_index = index;
            self.fire_portrait_update(&env);
        }
        self.save_config();
        self.invalidate();
    }

    fn handle_rot_damage_level_changed(&mut self, label: &str) {
        use crate::lua_api::state::ROT_DAMAGE_LEVELS;
        let index = ROT_DAMAGE_LEVELS
            .iter()
            .position(|(l, _)| *l == label)
            .unwrap_or(0);
        self.selected_rot_level = label.to_string();
        self.env.borrow().state().borrow_mut().rot_damage_level = index;
        self.save_config();
    }

    fn handle_movement_toggled(&mut self, field: &str, val: bool) {
        match field {
            "moving" => self.movement.moving = val,
            "mounted" => self.movement.mounted = val,
            "flying" => self.movement.flying = val,
            "falling" => self.movement.falling = val,
            "swimming" => self.movement.swimming = val,
            _ => return,
        }
        let env = self.env.borrow();
        let mut state = env.state().borrow_mut();
        match field {
            "moving" => state.player.movement.moving = val,
            "mounted" => state.player.movement.mounted = val,
            "flying" => state.player.movement.flying = val,
            "falling" => state.player.movement.falling = val,
            "swimming" => state.player.movement.swimming = val,
            _ => {}
        }
        drop(state);
        drop(env);
        self.save_config();
    }

    fn fire_portrait_update(&self, env: &WowLuaEnv) {
        let _ = env.fire_event_with_args("UNIT_PORTRAIT_UPDATE", &[env.lua_string("player")]);
        let _ = env.fire_event_with_args(
            "PLAYER_ENTERING_WORLD",
            &[Val::Bool(false), Val::Bool(false)],
        );
    }

    fn handle_reload_ui(&mut self) {
        self.log_messages.push("Reloading UI...".to_string());
        {
            let env = self.env.borrow();
            let _ = env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("WoWUISim")]);
            let _ = env.fire_event("VARIABLES_LOADED");
            let _ = env.fire_event_with_args(
                "PLAYER_ENTERING_WORLD",
                &[Val::Bool(false), Val::Bool(true)],
            );
            let _ = env.fire_event("UPDATE_BINDINGS");
            let _ = env.fire_event("DISPLAY_SIZE_CHANGED");
            let _ = env.fire_event("UI_SCALE_CHANGED");
        }
        self.drain_console();
        self.log_messages.push("UI reloaded.".to_string());
        self.mark_all_strata_dirty();
    }

    fn handle_execute_command(&mut self) {
        let cmd = self.command_input.clone();
        if cmd.is_empty() {
            return;
        }
        self.log_messages.push(format!("> {}", cmd));
        self.execute_command_inner(&cmd);
        self.command_input.clear();
        self.invalidate_after_lua_mutation();
    }

    fn execute_command_inner(&mut self, cmd: &str) {
        let cmd_lower = cmd.to_lowercase();
        if cmd_lower == "/frames" || cmd_lower == "/f" {
            let env = self.env.borrow();
            let dump = env.dump_frames();
            eprintln!("{}", dump);
            let line_count = dump.lines().count();
            self.log_messages
                .push(format!("Dumped {} frames to stderr", line_count / 2));
        } else {
            let env = self.env.borrow();
            match env.dispatch_slash_command(cmd) {
                Ok(true) => {}
                Ok(false) => self.log_messages.push(format!("Unknown command: {}", cmd)),
                Err(e) => self.log_messages.push(format!("Command error: {}", e)),
            }
        }
    }

    /// Run timers, layout, OnUpdate, health/casting and collect dirty mask + IDs.
    pub(super) fn collect_tick_dirty(&mut self) -> (u16, TickStageTimings) {
        let mut timings = TickStageTimings::default();
        let pending_before = self.pending_dirty_ids.borrow_mut().take();
        let (m0, ids0) = self.take_render_dirty_with_ids();

        let started = std::time::Instant::now();
        self.run_wow_timers();
        timings.timers = started.elapsed();
        let (m1, ids1) = self.take_render_dirty_with_ids();

        let t_layout = std::time::Instant::now();
        self.env.borrow().state().borrow_mut().ensure_layout_rects();
        timings.layout = t_layout.elapsed();

        timings.on_update = self.fire_on_update();
        let (m2, ids2) = self.take_render_dirty_with_ids();

        let started = std::time::Instant::now();
        self.tick_party_health();
        timings.party_health = started.elapsed();

        let started = std::time::Instant::now();
        self.tick_casting();
        timings.casting = started.elapsed();
        let (m3, ids3) = self.take_render_dirty_with_ids();

        self.mark_active_cooldown_widgets_dirty();
        let (m4, ids4) = self.take_render_dirty_with_ids();

        *self.pending_dirty_ids.borrow_mut() =
            merge_dirty_ids([pending_before, ids0, ids1, ids2, ids3, ids4]);
        (m0 | m1 | m2 | m3 | m4, timings)
    }

    pub(super) fn take_render_dirty_with_ids(&self) -> (u16, Option<FxHashSet<u64>>) {
        self.env
            .borrow()
            .state()
            .borrow()
            .widgets
            .take_render_dirty_with_ids()
    }

    pub(super) fn merge_pending_dirty_ids(&self, ids: Option<FxHashSet<u64>>) {
        let current = self.pending_dirty_ids.borrow_mut().take();
        *self.pending_dirty_ids.borrow_mut() = merge_dirty_ids([current, ids]);
    }

    pub(super) fn update_fps_counter(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.fps_last_time);
        if elapsed >= std::time::Duration::from_secs(1) {
            let main_thread_cpu = main_thread_cpu_time();
            let metrics = sample_display_metrics(MetricsWindow {
                elapsed,
                frames: self.frame_count.get(),
                ticks: self.tick_count.get(),
                tick_total_ms: self.tick_time_accum_ms.get(),
                draw_total_ms: self.draw_time_accum_ms.get(),
                prepare_total_ms: duration_ms(crate::render::shader::primitive::take_prepare_time())
                    as f32,
                main_thread_cpu: main_thread_cpu
                    .zip(self.fps_last_main_thread_cpu)
                    .map(|(now, before)| now.saturating_sub(before)),
            });
            self.fps = metrics.fps;
            if super::perf_logging_enabled() {
                crate::logging::eprintln_elapsed(&format!("[fps] {}", metrics.summary()));
            }
            self.display_metrics = metrics;
            self.fps_last_main_thread_cpu = main_thread_cpu;
            self.frame_count.set(0);
            self.draw_time_accum_ms.set(0.0);
            self.tick_time_accum_ms.set(0.0);
            self.tick_count.set(0);
            self.fps_last_time = now;
            let env = self.env.borrow();
            env.state().borrow_mut().fps = self.fps;
        }
    }

    pub(super) fn record_tick_time(&self, elapsed: std::time::Duration) {
        let elapsed_ms = elapsed.as_secs_f32() * 1000.0;
        self.tick_time_accum_ms
            .set(self.tick_time_accum_ms.get() + elapsed_ms);
        self.tick_count.set(self.tick_count.get().saturating_add(1));
    }

    fn run_wow_timers(&self) {
        let env = self.env.borrow();
        if let Err(e) = env.process_timers() {
            eprintln!("Timer error: {}", e);
        }
    }

    pub(super) fn fire_on_update(&mut self) -> crate::lua_api::on_update::OnUpdateStageTimings {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_on_update_time);
        if elapsed.as_millis() < 16 {
            return crate::lua_api::on_update::OnUpdateStageTimings::default();
        }
        self.last_on_update_time = now;
        let env = self.env.borrow();
        match env.fire_on_update_timed(elapsed.as_secs_f64()) {
            Ok(timings) => {
                self.last_on_update_time = std::time::Instant::now();
                timings
            }
            Err(e) => {
                crate::logging::eprintln_elapsed(&format!("[OnUpdate] error: {e}"));
                self.last_on_update_time = std::time::Instant::now();
                crate::lua_api::on_update::OnUpdateStageTimings::default()
            }
        }
    }

    fn tick_party_health(&mut self) {
        if self.selected_rot_level == "Off" {
            return;
        }
        let now = std::time::Instant::now();
        if now.duration_since(self.last_party_health_tick) < std::time::Duration::from_secs(2) {
            return;
        }
        self.last_party_health_tick = now;
        let env = self.env.borrow();
        let changed = {
            let mut state = env.state().borrow_mut();
            let (_, pct) = crate::lua_api::state::ROT_DAMAGE_LEVELS
                .get(state.rot_damage_level)
                .copied()
                .unwrap_or(("Light (1%)", 0.01));
            crate::lua_api::tick_party_health(&mut state.party_members, pct)
        };
        for idx in changed {
            let unit_id = format!("party{idx}");
            let _ = env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string(&unit_id)]);
        }
    }

    fn tick_casting(&mut self) {
        let env = self.env.borrow();
        let completed = super::casting::extract_completed_cast(env.state());
        if let Some((cast_id, spell_id)) = completed {
            super::casting::fire_cast_complete_events(&env, cast_id, spell_id);
            super::casting::apply_spell_effect(env.state(), &env, spell_id);
            super::casting::apply_spec_change(env.state(), &env);
        }
    }

    pub(super) fn run_pending_exec_lua(&mut self) {
        if let Some((code, secure)) = self.pending_exec_lua.take() {
            eprintln!(
                "[exec-lua{}] Running: {}",
                if secure { "-secure" } else { "" },
                code
            );
            let env = self.env.borrow();
            if let Err(e) = env.exec_maybe_secure(&code, secure) {
                eprintln!("[exec-lua] Error: {}", e);
            }
            drop(env);
            self.flush_post_script_updates();
        }
    }

    fn handle_screenshot_taken(&mut self, screenshot: iced::window::screenshot::Screenshot) {
        if let Some(respond) = self.pending_screenshot.take() {
            let data = ScreenshotData {
                width: screenshot.size.width,
                height: screenshot.size.height,
                pixels: screenshot.rgba.to_vec(),
            };
            let _ = respond.send(Ok(data));
        }
    }

    fn handle_inspector_close(&mut self) {
        self.inspector_visible = false;
        self.inspected_frame = None;
    }

    fn handle_inspector_apply(&mut self) {
        if let Some(frame_id) = self.inspected_frame {
            self.apply_inspector_changes(frame_id);
            self.mark_all_strata_dirty();
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct TickStageTimings {
    pub(super) total: Duration,
    pub(super) exec_lua: Duration,
    pub(super) timers: Duration,
    pub(super) layout: Duration,
    pub(super) on_update: crate::lua_api::on_update::OnUpdateStageTimings,
    pub(super) party_health: Duration,
    pub(super) casting: Duration,
    pub(super) console: Duration,
    pub(super) mark_dirty: Duration,
    pub(super) preload: Duration,
}

pub(super) fn log_slow_tick(stage_timings: &TickStageTimings, combined: u16, app: &App) {
    let atlas_ready = app.cached_render_request_ready_count();
    if super::perf_logging_enabled() && stage_timings.total.as_millis() > 10 {
        let n = app.pending_dirty_ids.borrow().as_ref().map(|s| s.len());
        eprintln!(
            "[tick] {:.1?} (layout={:.1?} dirty=0x{combined:x} ids={n:?} pending={})",
            stage_timings.total,
            stage_timings.layout,
            app.textures_pending.get()
        );
    }
    if tick_stage_logging_enabled() {
        let n = app.pending_dirty_ids.borrow().as_ref().map(|s| s.len());
        eprintln!(
            "{}",
            format_tick_stage_log(
                stage_timings,
                combined,
                n,
                app.textures_pending.get(),
                atlas_ready,
            )
        );
    }
}

fn tick_stage_logging_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("WOW_SIM_LOG_TICK_STAGES").is_some())
}

fn format_tick_stage_log(
    stage_timings: &TickStageTimings,
    combined: u16,
    dirty_ids: Option<usize>,
    textures_pending: bool,
    atlas_ready: usize,
) -> String {
    format!(
        "[tick-stage] total={:.3}ms exec_lua={:.3} timers={:.3} layout={:.3} on_update={:.3} handlers={:.3} anim={:.3} post={:.3} metrics={:.3} gc={:.3} party={:.3} casting={:.3} console={:.3} mark={:.3} preload={:.3} dirty=0x{combined:x} ids={dirty_ids:?} pending={textures_pending} ready={atlas_ready}",
        duration_ms(stage_timings.total),
        duration_ms(stage_timings.exec_lua),
        duration_ms(stage_timings.timers),
        duration_ms(stage_timings.layout),
        duration_ms(stage_timings.on_update.total),
        duration_ms(stage_timings.on_update.dispatch_handlers),
        duration_ms(stage_timings.on_update.animation_groups),
        duration_ms(stage_timings.on_update.on_post_update),
        duration_ms(stage_timings.on_update.finalize_metrics),
        duration_ms(stage_timings.on_update.gc_step),
        duration_ms(stage_timings.party_health),
        duration_ms(stage_timings.casting),
        duration_ms(stage_timings.console),
        duration_ms(stage_timings.mark_dirty),
        duration_ms(stage_timings.preload),
    )
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

pub(super) fn should_drop_stale_timer_tick(
    age: std::time::Duration,
    interval: std::time::Duration,
) -> bool {
    let stale_threshold = interval
        .saturating_mul(2)
        .max(std::time::Duration::from_millis(100));
    age > stale_threshold
}

/// Title-bar metrics for one sample window.
///
/// Costs are averaged per event (per tick, per drawn frame), not spread over
/// frames, so they stay meaningful when the app redraws only on change.
/// `unmeasured_ms_per_sec` is main-thread CPU time not spent in tick, draw or
/// GPU prepare (iced view/layout/present, input); wall time outside the busy
/// share is the thread waiting for events.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct DisplayMetrics {
    pub(crate) fps: f32,
    pub(crate) tick_ms: f32,
    pub(crate) ticks_per_sec: f32,
    pub(crate) draw_ms: f32,
    pub(crate) prepare_ms: f32,
    pub(crate) main_thread_busy_pct: Option<f32>,
    pub(crate) unmeasured_ms_per_sec: Option<f32>,
}

impl DisplayMetrics {
    /// e.g. `4.0 FPS | tick:1.20ms x62/s | draw:2.08ms | prep:0.50ms | main:32% busy, 180ms/s unmeasured`
    pub(crate) fn summary(&self) -> String {
        let main_thread = match (self.main_thread_busy_pct, self.unmeasured_ms_per_sec) {
            (Some(busy), Some(unmeasured)) => {
                format!("main:{busy:.0}% busy, {unmeasured:.0}ms/s unmeasured")
            }
            _ => "main:n/a".to_string(),
        };
        format!(
            "{:.1} FPS | tick:{:.2}ms x{:.0}/s | draw:{:.2}ms | prep:{:.2}ms | {main_thread}",
            self.fps, self.tick_ms, self.ticks_per_sec, self.draw_ms, self.prepare_ms
        )
    }
}

pub(super) struct MetricsWindow {
    pub(super) elapsed: std::time::Duration,
    pub(super) frames: u32,
    pub(super) ticks: u32,
    pub(super) tick_total_ms: f32,
    pub(super) draw_total_ms: f32,
    pub(super) prepare_total_ms: f32,
    pub(super) main_thread_cpu: Option<std::time::Duration>,
}

fn sample_display_metrics(window: MetricsWindow) -> DisplayMetrics {
    let elapsed_secs = window.elapsed.as_secs_f32();
    let per_sec = |value: f32| {
        if elapsed_secs > 0.0 {
            value / elapsed_secs
        } else {
            0.0
        }
    };
    let frames = window.frames.max(1) as f32;
    let measured_ms = window.tick_total_ms + window.draw_total_ms + window.prepare_total_ms;
    let cpu_ms = window.main_thread_cpu.map(|cpu| duration_ms(cpu) as f32);
    DisplayMetrics {
        fps: per_sec(window.frames as f32),
        tick_ms: window.tick_total_ms / window.ticks.max(1) as f32,
        ticks_per_sec: per_sec(window.ticks as f32),
        draw_ms: window.draw_total_ms / frames,
        prepare_ms: window.prepare_total_ms / frames,
        main_thread_busy_pct: cpu_ms.map(|cpu| per_sec(cpu) / 10.0),
        unmeasured_ms_per_sec: cpu_ms.map(|cpu| per_sec((cpu - measured_ms).max(0.0))),
    }
}

/// CPU time consumed by the calling thread (the GUI main thread when called
/// from `update`), or `None` where the platform does not report it.
#[cfg(unix)]
pub(super) fn main_thread_cpu_time() -> Option<std::time::Duration> {
    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `spec` is a valid, writable timespec for the duration of the call.
    let status = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut spec) };
    if status != 0 {
        return None;
    }
    let secs = u64::try_from(spec.tv_sec).ok()?;
    let nanos = u32::try_from(spec.tv_nsec).ok()?;
    Some(std::time::Duration::new(secs, nanos))
}

#[cfg(not(unix))]
pub(super) fn main_thread_cpu_time() -> Option<std::time::Duration> {
    None
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod update_tests;

#[cfg(test)]
#[path = "update_cooldown_tests.rs"]
mod update_cooldown_tests;

#[cfg(test)]
#[path = "update_key_tests.rs"]
mod update_key_tests;

#[cfg(test)]
#[path = "resize_event_tests.rs"]
mod resize_event_tests;
