use super::env::WowLuaAppData;
use super::env::WowLuaEnv;
use super::env::next_timer_id;
use super::env_init::{record_addon_time, update_threshold_counters};
use super::state::{AddonInfo, AppFrameMetrics, LoadDiagnostics, PendingTimer, SimState};
use super::timer_processing::{reschedule_timer, timer_should_wait};
use crate::Result;
use crate::font::WowFontSystem;
use crate::lua_api::methods::{call_function as call_rilua_function, registry_get};
use crate::lua_api::script_helpers::call_error_handler;
use crate::screen::ScreenKind;
use rilua::{LuaApi, LuaApiMut, Val};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

const RECENT_FRAME_WINDOW_SIZE: usize = 60;
// Blizzard SharedXML PixelUtil uses this reference height for pixel/UI conversion.
const UI_REFERENCE_HEIGHT: f32 = 768.0;

#[cfg(feature = "timed-signal-maps")]
fn next_timer_or_signal_delay(timer_delay: Option<Duration>, state: &SimState) -> Option<Duration> {
    match (
        timer_delay,
        crate::c_api::timed_signal_map::next_signal_delay(state),
    ) {
        (Some(timer), Some(signal)) => Some(timer.min(signal)),
        (Some(delay), None) | (None, Some(delay)) => Some(delay),
        (None, None) => None,
    }
}

#[cfg(not(feature = "timed-signal-maps"))]
fn next_timer_or_signal_delay(
    timer_delay: Option<Duration>,
    _state: &SimState,
) -> Option<Duration> {
    timer_delay
}

const EDIT_MODE_LAYOUTS_INFO_LUA: &str = r#"
    local source = (EditModeManagerFrame and EditModeManagerFrame.layoutInfo) or C_EditMode.GetLayouts()
    if type(source) ~= "table" then
        return source
    end

    local filtered = {
        layouts = {},
        activeLayout = source.activeLayout or 1,
    }
    local editModeLayoutType = type(Enum) == "table" and Enum.EditModeLayoutType or nil

    if type(source.layouts) ~= "table" then
        return filtered
    end

    for _, layoutInfo in ipairs(source.layouts) do
        local layoutType = type(layoutInfo) == "table" and layoutInfo.layoutType or nil
        if editModeLayoutType == nil
            or layoutType == editModeLayoutType.Account
            or layoutType == editModeLayoutType.Character then
            table.insert(filtered.layouts, layoutInfo)
        end
    end

    return filtered
"#;

impl WowLuaEnv {
    /// Call a named global Lua function.
    pub fn call_global(&self, name: &str, args: &[Val]) -> Result<Vec<Val>> {
        let mut lua = self.lua.borrow_mut();
        let func = LuaApiMut::get_global_val(&mut *lua, name);
        let Val::Function(func_ref) = func else {
            return Ok(Vec::new());
        };
        let func_handle = rilua::Function::from_gc_ref(func_ref);
        lua.call_function(&func_handle, args).map_err(Into::into)
    }

    /// Get access to the simulator state.
    pub fn state(&self) -> &Rc<RefCell<SimState>> {
        &self.state
    }

    /// Drain finalized failures and nil-symbol diagnostics from runtime addon loads.
    pub fn drain_runtime_addon_diagnostics(&self) -> LoadDiagnostics {
        std::mem::take(&mut self.state.borrow_mut().runtime_addon_diagnostics)
    }

    /// Set the font system for text measurement from Lua API methods.
    pub fn set_font_system(&self, font_system: Rc<RefCell<WowFontSystem>>) {
        let mut rilua = self.rilua_mut();
        let app_data = rilua
            .state_mut()
            .app_data_mut::<WowLuaAppData>()
            .expect("WowLuaEnv rilua app_data should always exist");
        app_data.font_system = Some(font_system);
    }

    pub(crate) fn install_initial_screen_size_globals(&self) {
        install_screen_size_globals(self);
    }

    /// Set an explicit one-to-one layout canvas and physical output size.
    pub fn set_screen_size(&self, width: f32, height: f32) {
        self.apply_screen_dimensions((width, height), (width, height));
    }

    /// Set physical display pixels using WoW's 768-unit base UI canvas.
    /// UIParent scale is applied separately by the existing frame/CVar model.
    pub fn set_display_size(&self, width: f32, height: f32) -> Result<()> {
        let valid = [width, height]
            .into_iter()
            .all(|dimension| dimension.is_finite() && dimension > 0.0);
        if !valid {
            return Err(crate::Error::Other(
                "display dimensions must be finite and positive".into(),
            ));
        }
        let canvas_width = width / height * UI_REFERENCE_HEIGHT;
        if !canvas_width.is_finite() {
            return Err(crate::Error::Other(
                "display aspect ratio exceeds UI canvas range".into(),
            ));
        }
        self.apply_screen_dimensions((canvas_width, UI_REFERENCE_HEIGHT), (width, height));
        Ok(())
    }

    fn apply_screen_dimensions(&self, canvas: (f32, f32), physical: (f32, f32)) {
        let (width, height) = canvas;
        {
            let mut state = self.state.borrow_mut();
            state.screen_width = width;
            state.screen_height = height;
            state.physical_screen_width = physical.0;
            state.physical_screen_height = physical.1;
            state.invalidate_strata_buckets();
            state.widgets.set_layout_physical_height(physical.1);
            update_screen_widgets_for_dimensions(&mut state, width, height);
        }
        install_screen_size_globals(self);
        self.notify_screen_size_changed();
    }

    fn notify_screen_size_changed(&self) {
        // Retail fires these as an ordered pair on every display/scale
        // recalculation (resize, scale slider, resolution change) — never one
        // alone. Ground truth: docs/wiki/investigations/display-size-ui-scale-events.md
        let _ = self.fire_event("DISPLAY_SIZE_CHANGED");
        let _ = self.fire_event("UI_SCALE_CHANGED");
        // Our EditMode anchor apply runs before the GUI knows its real window
        // size, so position-dependent system layouts (MicroMenu picks its
        // anchor corner from the container's screen quadrant) are computed
        // against the default dimensions. Replay Blizzard's anchor-changed
        // broadcast now that frames sit at their final screen positions.
        crate::lua_api::workarounds_editmode::invoke_anchor_changed_hooks(self);
    }

    /// Select which UI surface should be loaded.
    pub fn set_screen_mode(&self, screen_kind: ScreenKind) {
        self.state.borrow_mut().set_screen_kind(screen_kind);
        install_screen_mode_globals(self, screen_kind);
    }

    /// Toggle whether the simulated player is logged into the world.
    pub fn set_logged_in(&self, is_logged_in: bool) {
        self.state.borrow_mut().is_logged_in = is_logged_in;
    }

    /// Register an addon in the addon list.
    pub fn register_addon(&self, info: AddonInfo) {
        self.state.borrow_mut().addons.push(info);
    }

    /// Scan an addons directory and register all found addons (metadata only, no loading).
    pub fn scan_and_register_addons(&self, addons_path: &std::path::Path) {
        let mut addons = super::addon_scan::scan_addon_entries(addons_path);
        addons.sort_by(|a, b| {
            a.folder_name
                .to_lowercase()
                .cmp(&b.folder_name.to_lowercase())
        });
        let mut state = self.state.borrow_mut();
        for addon in addons {
            if !state
                .addons
                .iter()
                .any(|a| a.folder_name == addon.folder_name)
            {
                state.addons.push(addon);
            }
        }
    }

    /// Schedule a timer callback.
    pub fn schedule_timer(
        &self,
        seconds: f64,
        callback: Val,
        interval: Option<Duration>,
        iterations: Option<i32>,
    ) -> Result<u64> {
        let id = next_timer_id();
        {
            let mut lua = self.lua.borrow_mut();
            crate::lua_api::timer_layout::store_timer_callback(lua.state_mut(), id, callback);
        }
        let owner_addon = {
            let state = self.state.borrow();
            state.loading_addon_index.or(state.executing_addon_index)
        };
        let timer = PendingTimer {
            id,
            fire_at: Instant::now() + Duration::from_secs_f64(seconds),
            interval,
            remaining: iterations,
            cancelled: false,
            owner_addon,
            callback_receives_timer: false,
            callback_arg: None,
        };
        self.state.borrow_mut().rilua_timers.push_back(timer);
        Ok(id)
    }

    /// Run ready timers and return how many callbacks fired.
    pub fn process_timers(&self) -> Result<usize> {
        let signal_fired = self.process_due_timed_signals();

        let now = Instant::now();
        let timers = take_pending_timers(self);
        let (timer_fired, mut requeue) = process_timer_queue(self, timers, now);
        let mut state = self.state.borrow_mut();
        requeue.append(&mut state.rilua_timers);
        state.rilua_timers = requeue;
        Ok(signal_fired + timer_fired)
    }

    #[cfg(feature = "timed-signal-maps")]
    fn process_due_timed_signals(&self) -> usize {
        let due_signals = {
            let mut state = self.state.borrow_mut();
            crate::c_api::timed_signal_map::take_due_signals(&mut state)
        };
        let mut fired = 0;
        for signal in due_signals {
            if let Some(callback) = self.timer_callback(signal.id) {
                self.fire_timer_callback(
                    signal.owner_addon,
                    callback,
                    signal.id,
                    false,
                    Some(Val::Num(signal.key as f64)),
                );
                fired += 1;
            }
        }
        fired
    }

    #[cfg(not(feature = "timed-signal-maps"))]
    fn process_due_timed_signals(&self) -> usize {
        0
    }

    /// Fire OnUpdate handlers for all frames that have them registered.
    pub fn fire_on_update(&self, elapsed: f64) -> Result<()> {
        self.fire_on_update_timed(elapsed).map(|_| ())
    }

    /// Fire OnUpdate handlers and return stage timings for profiling.
    pub(crate) fn fire_on_update_timed(
        &self,
        elapsed: f64,
    ) -> Result<super::on_update::OnUpdateStageTimings> {
        super::on_update::fire(self, elapsed)
    }

    fn drain_addon_timing(&self) {
        let mut lua = self.lua.borrow_mut();
        let state = lua.state_mut();
        let timing = registry_get(state, "__addon_timing");
        let Val::Table(timing_ref) = timing else {
            return;
        };
        let entries = state
            .gc
            .tables
            .get(timing_ref)
            .map(|table| table.hash_entries())
            .unwrap_or_default();
        let mut consumed_keys = Vec::new();
        {
            let mut sim = self.state.borrow_mut();
            for (key, value) in entries {
                let Val::Num(idx) = key else {
                    continue;
                };
                let Val::Num(ms) = value else {
                    continue;
                };
                consumed_keys.push(idx);
                if let Some(addon) = sim.addons.get_mut(idx as usize) {
                    addon.runtime.current_frame_ms += ms;
                }
            }
        }
        for idx in consumed_keys {
            if let Some(table) = state.gc.tables.get_mut(timing_ref) {
                let _ = table.raw_set(Val::Num(idx), Val::Nil, &state.gc.string_arena);
            }
        }
    }

    pub(crate) fn finalize_frame_metrics(&self, frame_elapsed_ms: f64) {
        self.drain_addon_timing();
        let mut state = self.state.borrow_mut();
        update_app_frame_metrics(&mut state.app_frame_metrics, frame_elapsed_ms);
        update_addon_frame_metrics(&mut state.addons);
    }

    /// Fire `EDIT_MODE_LAYOUTS_UPDATED` with layout info from `C_EditMode.GetLayouts()`.
    pub fn fire_edit_mode_layouts_updated(&self) -> Result<()> {
        let Some(info) = build_edit_mode_layouts_info(self)? else {
            return Ok(());
        };
        self.fire_event_with_args("EDIT_MODE_LAYOUTS_UPDATED", &[info, Val::Bool(true)])
    }

    /// Get the time until the next timer fires, if any.
    pub fn next_timer_delay(&self) -> Option<Duration> {
        let state = self.state.borrow();
        let now = Instant::now();
        let timer_delay = state
            .rilua_timers
            .iter()
            .filter(|timer| !timer.cancelled)
            .map(|timer| timer.fire_at.saturating_duration_since(now))
            .min();
        next_timer_or_signal_delay(timer_delay, &state)
    }

    /// Dump all frame positions for debugging.
    pub fn dump_frames(&self) -> String {
        let state = self.state.borrow();
        super::diagnostics::dump_frames(&state)
    }

    fn timer_callback(&self, timer_id: u64) -> Option<Val> {
        let mut lua = self.lua.borrow_mut();
        let callback = crate::lua_api::timer_layout::get_timer_callback(lua.state_mut(), timer_id);
        (!matches!(callback, Val::Nil)).then_some(callback)
    }

    fn fire_timer_callback(
        &self,
        owner_addon: Option<u16>,
        callback: Val,
        timer_id: u64,
        callback_receives_timer: bool,
        callback_arg: Option<Val>,
    ) {
        let start = Instant::now();
        let addon_name = self.addon_folder_name(owner_addon);
        self.state.borrow_mut().executing_addon_index = owner_addon;
        let call_result = {
            let mut lua = self.lua.borrow_mut();
            let args = if let Some(arg) = callback_arg {
                vec![arg]
            } else if callback_receives_timer {
                match crate::lua_api::timer_layout::create_timer_handle_table(
                    lua.state_mut(),
                    timer_id,
                ) {
                    Ok(handle) => vec![handle],
                    Err(error) => {
                        call_error_handler(&mut lua, &error.to_string());
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };
            call_rilua_function(&mut lua, callback, &args)
        };
        if let Err(error) = call_result {
            let mut lua = self.lua.borrow_mut();
            call_error_handler(&mut lua, &error.to_string());
        }
        self.state.borrow_mut().executing_addon_index = None;
        let source = self.timer_callback_source_label(callback);
        crate::lua_api::handler_timing::log_with_source(
            addon_name.as_deref(),
            "C_Timer",
            None,
            0,
            start.elapsed(),
            source.as_deref(),
        );
        record_addon_time(&self.state, owner_addon, &start);
    }

    fn timer_callback_source_label(&self, callback: Val) -> Option<String> {
        let Val::Function(func_ref) = callback else {
            return None;
        };
        let lua = self.lua.borrow();
        crate::lua_api::handler_timing::lua_closure_source_label(lua.state(), func_ref)
    }

    fn addon_folder_name(&self, owner_addon: Option<u16>) -> Option<String> {
        let addon_index = owner_addon? as usize;
        self.state
            .borrow()
            .addons
            .get(addon_index)
            .map(|addon| addon.folder_name.clone())
    }

    fn remove_timer_callback(&self, timer_id: u64) {
        let mut lua = self.lua.borrow_mut();
        crate::lua_api::timer_layout::remove_timer_callback(lua.state_mut(), timer_id);
    }
}

fn update_screen_widgets_for_dimensions(state: &mut SimState, width: f32, height: f32) {
    for name in &["UIParent", "WorldFrame"] {
        if let Some(id) = state.widgets.get_id_by_name(name)
            && let Some(frame) = state.widgets.get_mut_visual(id)
        {
            frame.width = width;
            frame.height = height;
        }
    }
}

fn install_screen_size_globals(env: &WowLuaEnv) {
    let (width, height, physical_width, physical_height) = {
        let state = env.state.borrow();
        (
            state.screen_width,
            state.screen_height,
            state.physical_screen_width,
            state.physical_screen_height,
        )
    };
    let _ = env.exec(&format!(
        r#"
        function GetScreenWidth()
            local scale = UIParent and UIParent.GetEffectiveScale and UIParent:GetEffectiveScale() or 1
            return {width} / scale
        end

        function GetScreenHeight()
            local scale = UIParent and UIParent.GetEffectiveScale and UIParent:GetEffectiveScale() or 1
            return {height} / scale
        end

        function GetPhysicalScreenSize()
            return {physical_width}, {physical_height}
        end

        function GetScreenDPIScale()
            return 1
        end

        function ConvertPixelsToUI(pixels, scale)
            return pixels * {UI_REFERENCE_HEIGHT} / ({physical_height} * scale)
        end
        "#
    ));
}

fn install_screen_mode_globals(env: &WowLuaEnv, screen_kind: ScreenKind) {
    let is_glue = if screen_kind.is_glue() {
        "true"
    } else {
        "false"
    };
    let _ = env.exec(&format!(
        r#"
        __wow_screen_mode_is_glue = {is_glue}
        function InGlue()
            return {is_glue}
        end

        "#
    ));
}

fn take_pending_timers(env: &WowLuaEnv) -> VecDeque<PendingTimer> {
    let mut state = env.state.borrow_mut();
    let mut pending = VecDeque::new();
    std::mem::swap(&mut pending, &mut state.rilua_timers);
    pending
}

fn process_timer_queue(
    env: &WowLuaEnv,
    mut timers: VecDeque<PendingTimer>,
    now: Instant,
) -> (usize, VecDeque<PendingTimer>) {
    let mut fired = 0usize;
    let mut requeue = VecDeque::new();
    while let Some(mut timer) = timers.pop_front() {
        if timer_should_wait(&timer, now) {
            requeue.push_back(timer);
            continue;
        }

        let Some(callback) = env.timer_callback(timer.id) else {
            continue;
        };

        env.fire_timer_callback(
            timer.owner_addon,
            callback,
            timer.id,
            timer.callback_receives_timer,
            timer.callback_arg,
        );
        fired += 1;

        if reschedule_timer(&mut timer, now) {
            requeue.push_back(timer);
            continue;
        }

        env.remove_timer_callback(timer.id);
    }

    (fired, requeue)
}

fn update_app_frame_metrics(metrics: &mut AppFrameMetrics, frame_elapsed_ms: f64) {
    metrics.recent_frame_ms.push_back(frame_elapsed_ms);
    if metrics.recent_frame_ms.len() > RECENT_FRAME_WINDOW_SIZE {
        metrics.recent_frame_ms.pop_front();
    }
    if frame_elapsed_ms > metrics.peak_ms {
        metrics.peak_ms = frame_elapsed_ms;
    }
    metrics.session_total_ms += frame_elapsed_ms;
    metrics.session_frame_count += 1;
}

fn update_addon_frame_metrics(addons: &mut [AddonInfo]) {
    for addon in addons {
        let ms = addon.runtime.current_frame_ms;
        if ms > 0.0 {
            addon.runtime.recent_frames.push_back(ms);
            if addon.runtime.recent_frames.len() > RECENT_FRAME_WINDOW_SIZE {
                addon.runtime.recent_frames.pop_front();
            }
            if ms > addon.runtime.peak_ms {
                addon.runtime.peak_ms = ms;
            }
            addon.runtime.session_total_ms += ms;
            addon.runtime.session_frame_count += 1;
            update_threshold_counters(&mut addon.runtime, ms);
        }
        addon.runtime.current_frame_ms = 0.0;
    }
}

fn build_edit_mode_layouts_info(env: &WowLuaEnv) -> Result<Option<Val>> {
    let Ok(true) = env.eval::<bool>("return C_EditMode ~= nil and C_EditMode.GetLayouts ~= nil")
    else {
        return Ok(None);
    };

    let info = env.eval::<Val>(EDIT_MODE_LAYOUTS_INFO_LUA)?;

    Ok(Some(info))
}
