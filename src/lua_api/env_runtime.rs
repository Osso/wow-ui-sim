use super::env::WowLuaAppData;
use super::env::WowLuaEnv;
use super::env_init::update_threshold_counters;
use super::state::{AddonInfo, AppFrameMetrics, LoadDiagnostics, PendingTimer, SimState};
use super::timer_processing::{reschedule_timer, timer_should_wait};
use crate::Result;
use crate::font::WowFontSystem;
use crate::lua_api::methods::{borrow_state_mut, registry_get};
use crate::lua_bridge::table_set_rust_fn_static;
use crate::screen::ScreenKind;
use rilua::{LuaApiMut, LuaResult, Val};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

const RECENT_FRAME_WINDOW_SIZE: usize = 60;

struct TimerReplySnapshot {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    raid_info_replies: usize,
    #[cfg(feature = "client-retail")]
    owned_house_replies: VecDeque<Vec<crate::c_api::c_housing::current_house::HouseInfo>>,
    #[cfg(feature = "client-retail")]
    house_finder_replies: VecDeque<crate::c_api::c_housing::house_finder::HouseFinderReply>,
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pending_completions: usize,
}
// Blizzard SharedXML PixelUtil uses this reference height for pixel/UI conversion.
pub(crate) const UI_REFERENCE_HEIGHT: f32 = 768.0;
const SCREEN_SIZE_GLOBALS_LUA: &str = r#"
function GetScreenWidth()
    local scale = UIParent and UIParent.GetEffectiveScale and UIParent:GetEffectiveScale() or 1
    local width = __wow_get_screen_metrics()
    return width / scale
end

function GetScreenHeight()
    local scale = UIParent and UIParent.GetEffectiveScale and UIParent:GetEffectiveScale() or 1
    local _, height = __wow_get_screen_metrics()
    return height / scale
end

function GetPhysicalScreenSize()
    local _, _, width, height = __wow_get_screen_metrics()
    return width, height
end

function GetScreenDPIScale()
    return 1
end

function ConvertPixelsToUI(pixels, scale)
    local _, _, _, physicalHeight = __wow_get_screen_metrics()
    return pixels * {UI_REFERENCE_HEIGHT} / (physicalHeight * scale)
end
"#;

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

fn publish_r88y_after_screen_registration(env: &WowLuaEnv, screen_kind: ScreenKind) {
    let mut lua = env.lua.borrow_mut();
    crate::c_api::publish_verified_secure_namespace_members(lua.state_mut(), screen_kind)
        .expect("R88y publication should follow screen-specific registration");
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

    /// The TOC files parsed in this UI session.
    pub fn toc_cache(&self) -> Rc<crate::toc::TocCache> {
        self.state.borrow().toc_cache()
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
        drop(rilua);
        self.state()
            .borrow_mut()
            .widgets
            .invalidate_editbox_presentations();
    }

    pub(crate) fn install_initial_screen_size_globals(&self) {
        install_screen_size_globals(self);
    }

    /// Set the host default frame scale and minimum/maximum render scales.
    pub fn set_display_scale_inputs(&self, default: f64, min: f64, max: f64) -> Result<()> {
        if ![default, min, max]
            .into_iter()
            .all(|value| value.is_finite() && value > 0.0)
            || min > max
        {
            return Err(crate::Error::Other(
                "display scale inputs must be finite, positive, and have ordered render bounds"
                    .into(),
            ));
        }
        self.state.borrow_mut().display_scale_inputs = (default, min, max);
        Ok(())
    }

    /// Set an explicit one-to-one layout canvas and physical output size.
    pub fn set_screen_size(&self, width: f32, height: f32) {
        self.apply_screen_dimensions((width, height), (width, height));
    }

    /// Supply host graphics adapters for the Retail and Forever video model.
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pub fn set_gx_adapter_info(&self, adapters: &[(&str, bool, bool)]) {
        let mut state = self.state.borrow_mut();
        state.gx_adapter_info = adapters
            .iter()
            .map(|(name, is_low_power, is_external)| {
                crate::c_api::video_window::GxAdapterInfoDetails {
                    name: (*name).to_owned(),
                    is_low_power: *is_low_power,
                    is_external: *is_external,
                }
            })
            .collect();
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
        update_screen_dimensions(&mut self.state.borrow_mut(), (width, height), physical);
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
        self.update_screen_mode_workarounds(screen_kind);
        self.update_screen_mode_registrations(screen_kind);
    }

    fn update_screen_mode_workarounds(&self, screen_kind: ScreenKind) {
        crate::lua_api::workarounds::update_adventureguide_for_screen(
            &mut self.lua.borrow_mut(),
            screen_kind,
        )
        .expect("AdventureGuide availability workaround should follow screen mode");
        crate::lua_api::workarounds::update_player_location_for_screen(
            &mut self.lua.borrow_mut(),
            screen_kind,
        )
        .expect("PlayerLocation workaround should follow screen mode");
        crate::c_api::c_character_services::update_queued_upgrade_registration(
            self.lua.borrow_mut().state_mut(),
            screen_kind,
        )
        .expect("queued-upgrade registration should follow screen mode");
    }

    fn update_screen_mode_registrations(&self, screen_kind: ScreenKind) {
        {
            let mut lua = self.lua.borrow_mut();
            crate::c_api::update_screen_registration(lua.state_mut())
                .expect("screen-specific C_* registration should succeed");
        }
        install_screen_mode_globals(self, screen_kind);
        self.update_screen_specific_registrations(screen_kind);
        publish_r88y_after_screen_registration(self, screen_kind);
    }

    fn update_screen_specific_registrations(&self, screen_kind: ScreenKind) {
        crate::c_api::c_character_creation::update_screen_registration(self, screen_kind);
        #[cfg(feature = "client-wowforever")]
        crate::lua_api::globals::real::name_reservation::update_screen_registration(
            self,
            screen_kind.is_glue(),
        );
        crate::c_api::c_login::update_screen_registration(self, screen_kind);
        crate::c_api::c_district_utils::update_screen_registration(self, screen_kind.is_glue());
        crate::c_api::c_player_info_guid::update_screen_registration(self, screen_kind.is_glue());
        crate::lua_api::globals::real::bank_reagents::update_screen_registration(
            self,
            screen_kind.is_glue(),
        );
        #[cfg(any(
            feature = "client-retail",
            feature = "client-ptr",
            feature = "client-wowforever"
        ))]
        crate::lua_api::globals::real::quest_faction_group::update_screen_registration(
            self,
            screen_kind.is_glue(),
        );
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
        let mut addons = super::addon_scan::scan_addon_entries(&self.toc_cache(), addons_path);
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
        let id = self.state.borrow_mut().allocate_timer_id();
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
            fire_at: self.state.borrow().clock_now() + Duration::from_secs_f64(seconds),
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
        let replies = self.take_timer_reply_snapshot();
        let timer_fired = self.run_due_timer_callbacks()?;
        self.dispatch_timer_replies(timer_fired, replies)
    }

    fn run_due_timer_callbacks(&self) -> Result<usize> {
        if self.state.borrow().manual_clock.is_some() {
            super::manual_timers::drain(self.lua.borrow_mut().state_mut())
        } else {
            Ok(self.process_regular_timers())
        }
    }

    fn take_timer_reply_snapshot(&self) -> TimerReplySnapshot {
        #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
        let raid_info_replies =
            std::mem::take(&mut self.state.borrow_mut().pending_raid_info_replies).len();
        #[cfg(feature = "client-retail")]
        let owned_house_replies =
            std::mem::take(&mut self.state.borrow_mut().pending_owned_house_replies);
        #[cfg(feature = "client-retail")]
        let house_finder_replies =
            std::mem::take(&mut self.state.borrow_mut().pending_house_finder_replies);
        #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
        let pending_completions = self.state.borrow().pending_combat_audio_utterance_ids.len();
        TimerReplySnapshot {
            #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
            raid_info_replies,
            #[cfg(feature = "client-retail")]
            owned_house_replies,
            #[cfg(feature = "client-retail")]
            house_finder_replies,
            #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
            pending_completions,
        }
    }

    fn dispatch_timer_replies(
        &self,
        timer_fired: usize,
        replies: TimerReplySnapshot,
    ) -> Result<usize> {
        #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
        return self.dispatch_profile_timer_replies(timer_fired, replies);
        #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
        {
            let _ = replies;
            Ok(timer_fired)
        }
    }

    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    fn dispatch_profile_timer_replies(
        &self,
        timer_fired: usize,
        replies: TimerReplySnapshot,
    ) -> Result<usize> {
        #[cfg(feature = "client-retail")]
        let housing_replies = self.fire_owned_house_replies(replies.owned_house_replies)?
            + self.fire_house_finder_replies(replies.house_finder_replies)?;
        #[cfg(not(feature = "client-retail"))]
        let housing_replies = 0;
        self.fire_raid_info_replies(replies.raid_info_replies)?;
        let completions = self.dispatch_combat_audio_completions(replies.pending_completions)?;
        Ok(timer_fired + housing_replies + completions)
    }

    fn process_regular_timers(&self) -> usize {
        let signal_fired = self.process_due_timed_signals();
        let now = self.state.borrow().clock_now();
        let timers = take_pending_timers(self);
        let (timer_fired, mut requeue) = process_timer_queue(self, timers, now);
        let mut state = self.state.borrow_mut();
        requeue.append(&mut state.rilua_timers);
        state.rilua_timers = requeue;
        signal_fired + timer_fired
    }

    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    fn dispatch_combat_audio_completions(&self, count: usize) -> Result<usize> {
        let mut fired = 0;
        for _ in 0..count {
            let Some(utterance_id) = self
                .state
                .borrow_mut()
                .pending_combat_audio_utterance_ids
                .pop_front()
            else {
                break;
            };
            let args = [Val::Num(f64::from(utterance_id))];
            if let Err(error) = self.fire_event_with_args("VOICE_CHAT_TTS_PLAYBACK_FINISHED", &args)
            {
                self.state
                    .borrow_mut()
                    .pending_combat_audio_utterance_ids
                    .push_front(utterance_id);
                return Err(error);
            }
            fired += 1;
        }
        Ok(fired)
    }

    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    fn fire_raid_info_replies(&self, count: usize) -> Result<()> {
        let mut first_error = None;
        for _ in 0..count {
            if let Err(error) = self.fire_event("UPDATE_INSTANCE_INFO")
                && first_error.is_none()
            {
                first_error = Some(error);
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    #[cfg(feature = "client-retail")]
    fn fire_house_finder_replies(
        &self,
        mut replies: std::collections::VecDeque<
            crate::c_api::c_housing::house_finder::HouseFinderReply,
        >,
    ) -> Result<usize> {
        let count = replies.len();
        while let Some(reply) = replies.pop_front() {
            if let Err(error) = self.dispatch_house_finder_reply(reply.clone()) {
                let mut state = self.state.borrow_mut();
                state.pending_house_finder_replies.push_front(reply);
                state.pending_house_finder_replies.append(&mut replies);
                return Err(error);
            }
        }
        Ok(count)
    }

    #[cfg(feature = "client-retail")]
    fn dispatch_house_finder_reply(
        &self,
        reply: crate::c_api::c_housing::house_finder::HouseFinderReply,
    ) -> Result<()> {
        use crate::c_api::c_housing::house_finder::{
            HouseFinderReply, encode_neighborhoods, encode_plots,
        };
        match reply {
            HouseFinderReply::Neighborhoods { success, rows } => {
                let payload = encode_neighborhoods(self.lua.borrow_mut().state_mut(), rows);
                self.fire_event_with_args("NEIGHBORHOOD_LIST_UPDATED", &[success, payload])
            }
            HouseFinderReply::Plots(plots) => {
                let payload = encode_plots(self.lua.borrow_mut().state_mut(), plots);
                self.fire_event_with_args("HOUSE_FINDER_NEIGHBORHOOD_DATA_RECIEVED", &[payload])
            }
        }
    }

    #[cfg(feature = "client-retail")]
    fn fire_owned_house_replies(
        &self,
        mut replies: std::collections::VecDeque<
            Vec<crate::c_api::c_housing::current_house::HouseInfo>,
        >,
    ) -> Result<usize> {
        let count = replies.len();
        while let Some(houses) = replies.pop_front() {
            let payload = crate::c_api::c_housing::owned_houses::encode_house_list(
                self.lua.borrow_mut().state_mut(),
                houses.clone(),
            );
            if let Err(error) = self.fire_event_with_args("PLAYER_HOUSE_LIST_UPDATED", &[payload]) {
                let mut state = self.state.borrow_mut();
                state.pending_owned_house_replies.push_front(houses);
                state.pending_owned_house_replies.append(&mut replies);
                return Err(error);
            }
        }
        Ok(count)
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
        crate::c_api::c_login::advance(self)?;
        // INFERRED (simulator policy): map the native queued notification to this
        // dispatch boundary, before frame OnUpdate handlers; native delivery timing is unknown.
        crate::c_api::store_catalog_model::flush_purchase_list_notification(self)?;
        let timings = super::on_update::fire(self, elapsed)?;
        #[cfg(feature = "client-ptr")]
        {
            let pending = {
                let mut sim = self.state.borrow_mut();
                std::mem::take(&mut sim.actionbar_cooldown_update_pending)
            };
            if pending {
                self.fire_event("ACTIONBAR_UPDATE_COOLDOWN")?;
            }
        }
        Ok(timings)
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
        let now = state.clock_now();
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
        super::timer_dispatch::fire(
            self.lua.borrow_mut().state_mut(),
            owner_addon,
            callback,
            timer_id,
            callback_receives_timer,
            callback_arg,
        );
    }

    fn remove_timer_callback(&self, timer_id: u64) {
        let mut lua = self.lua.borrow_mut();
        crate::lua_api::timer_layout::remove_timer_callback(lua.state_mut(), timer_id);
    }
}

fn drain_addon_timing(state: &mut rilua::vm::state::LuaState) {
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
        let mut sim = crate::lua_api::methods::borrow_state_mut(state)
            .expect("frame metrics require simulator state");
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

pub(crate) fn finalize_frame_metrics_state(
    state: &mut rilua::vm::state::LuaState,
    frame_elapsed_ms: f64,
) {
    drain_addon_timing(state);
    let mut state = crate::lua_api::methods::borrow_state_mut(state)
        .expect("frame metrics require simulator state");
    update_app_frame_metrics(&mut state.app_frame_metrics, frame_elapsed_ms);
    update_addon_frame_metrics(&mut state.addons);
}

fn update_screen_dimensions(state: &mut SimState, canvas: (f32, f32), physical: (f32, f32)) {
    let (width, height) = canvas;
    state.screen_width = width;
    state.screen_height = height;
    state.physical_screen_width = physical.0;
    state.physical_screen_height = physical.1;
    state.invalidate_strata_buckets();
    state.widgets.set_layout_physical_height(physical.1);
    state.widgets.set_layout_canvas_size(width, height);
    update_screen_widgets_for_dimensions(state, width, height);
}

/// INFERRED: a requested resolution updates simulator display state and emits its display event.
pub(crate) fn set_display_size_from_lua(
    lua_state: &mut rilua::vm::state::LuaState,
    width: f32,
    height: f32,
) -> LuaResult<()> {
    if ![width, height]
        .into_iter()
        .all(|value| value.is_finite() && value > 0.0)
    {
        return Err(rilua::runtime_error(
            "window dimensions must be finite and positive",
        ));
    }
    let canvas = (width / height * UI_REFERENCE_HEIGHT, UI_REFERENCE_HEIGHT);
    if !canvas.0.is_finite() {
        return Err(rilua::runtime_error(
            "window aspect ratio exceeds UI canvas range",
        ));
    }
    let mut model = borrow_state_mut(lua_state)?;
    update_screen_dimensions(&mut model, canvas, (width, height));
    drop(model);
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        lua_state,
        "DISPLAY_SIZE_CHANGED",
        &[],
    )?;
    Ok(())
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
    let reference_height = UI_REFERENCE_HEIGHT.to_string();
    let script = SCREEN_SIZE_GLOBALS_LUA.replace("{UI_REFERENCE_HEIGHT}", &reference_height);
    let _ = env.exec(&script);
    install_screen_metrics_helper(env);
}

fn install_screen_metrics_helper(env: &WowLuaEnv) {
    let mut lua = env.lua.borrow_mut();
    let state = lua.state_mut();
    table_set_rust_fn_static(
        state,
        state.global,
        "__wow_get_screen_metrics",
        screen_metrics,
    )
    .expect("register dynamic screen metrics");
}

fn screen_metrics(state: &mut rilua::vm::state::LuaState) -> LuaResult<u32> {
    let metrics = {
        let model = crate::lua_api::methods::borrow_state(state)?;
        (
            model.screen_width,
            model.screen_height,
            model.physical_screen_width,
            model.physical_screen_height,
        )
    };
    for value in [metrics.0, metrics.1, metrics.2, metrics.3] {
        state.push(Val::Num(f64::from(value)));
    }
    Ok(4)
}

fn install_screen_mode_globals(env: &WowLuaEnv, screen_kind: ScreenKind) {
    let _ = crate::c_api::credits::register(env.lua.borrow_mut().state_mut());
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    let _ = super::globals::real::glue_selection::register_season_query(
        env.lua.borrow_mut().state_mut(),
    );
    #[cfg(feature = "client-ptr")]
    register_ptr_screen_queries(env);
    #[cfg(feature = "client-mists")]
    super::globals::real::mouse_probes::update_screen_registration(env, screen_kind.is_glue())
        .expect("Mists cursor visibility globals should follow Glue screen mode");
    #[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
    let _ = super::globals::real::glue_feature_status::register_all(&mut env.lua.borrow_mut());
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

#[cfg(feature = "client-ptr")]
fn register_ptr_screen_queries(env: &WowLuaEnv) {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr {
        let _ =
            super::globals::real::glue_character_season::register(env.lua.borrow_mut().state_mut());
        let _ = super::globals::real::glue_background::register_background_only(
            env.lua.borrow_mut().state_mut(),
        );
        let _ = super::globals::real::glue_background::register_model_ffx_only(
            env.lua.borrow_mut().state_mut(),
        );
    }
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
