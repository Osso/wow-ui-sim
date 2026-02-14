//! WoW Lua environment.

use super::builtin_frames::create_builtin_frames;
use super::state::{AddonRuntimeMetrics, AddonInfo, PendingTimer, SimState};
use crate::render::font::WowFontSystem;
use crate::Result;
use mlua::{Lua, MultiValue, Value};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);

/// Generate a unique timer ID.
pub(crate) fn next_timer_id() -> u64 {
    NEXT_TIMER_ID.fetch_add(1, Ordering::Relaxed)
}

/// The WoW Lua environment.
pub struct WowLuaEnv {
    pub(crate) lua: Lua,
    pub(crate) state: Rc<RefCell<SimState>>,
    /// OnUpdate handlers: maps frame ID → consecutive error count.
    /// Errors are logged but handlers keep firing (matching WoW behavior).
    /// Only suppressed after many consecutive errors to avoid infinite spam.
    on_update_errors: RefCell<std::collections::HashMap<u64, u32>>,
}

impl WowLuaEnv {
    /// Create a new WoW Lua environment with the API initialized.
    pub fn new() -> Result<Self> {
        // Use unsafe_new to get full standard library including debug
        // This is safe for our simulator since we control the Lua code
        let lua = unsafe { Lua::unsafe_new() };
        let state = Rc::new(RefCell::new(SimState::default()));

        // Create all built-in frames
        {
            let mut s = state.borrow_mut();
            let (w, h) = (s.screen_width, s.screen_height);
            create_builtin_frames(&mut s.widgets, w, h);
        }

        // Register global functions
        super::globals::register_globals(&lua, Rc::clone(&state))?;

        // Initialize keybinding tables with defaults
        super::keybindings::init_keybindings(&lua)?;

        // Precompile Lua helper functions for the XML loader
        crate::loader::precompiled::init(&lua)?;

        Ok(Self {
            lua,
            state,
            on_update_errors: RefCell::new(std::collections::HashMap::new()),
        })
    }

    /// Execute Lua code.
    pub fn exec(&self, code: &str) -> Result<()> {
        self.lua.load(code).exec()?;
        Ok(())
    }

    /// Execute Lua code with a custom chunk name (for better error messages and debugstack).
    pub fn exec_named(&self, code: &str, name: &str) -> Result<()> {
        self.lua.load(code).set_name(name).exec()?;
        Ok(())
    }

    /// Execute Lua code with varargs (like WoW addon loading).
    /// In WoW, each addon file receives (addonName, addonTable) as varargs.
    pub fn exec_with_varargs(
        &self,
        code: &str,
        name: &str,
        addon_name: &str,
        addon_table: mlua::Table,
    ) -> Result<()> {
        let chunk = self.lua.load(code).set_name(name);
        let func: mlua::Function = chunk.into_function()?;
        func.call::<()>((addon_name.to_string(), addon_table))?;
        Ok(())
    }

    /// Create a new empty table for addon private storage.
    /// Includes a default `unpack` method that returns values at numeric indices.
    pub fn create_addon_table(&self) -> Result<mlua::Table> {
        let table = self.lua.create_table()?;
        // Add default unpack method - returns values at indices 1, 2, 3, 4
        // Addons like OmniCD use this pattern: local E, L, C = select(2, ...):unpack()
        let unpack_fn = self.lua.create_function(|_, this: mlua::Table| {
            let v1: mlua::Value = this.get(1).unwrap_or(mlua::Value::Nil);
            let v2: mlua::Value = this.get(2).unwrap_or(mlua::Value::Nil);
            let v3: mlua::Value = this.get(3).unwrap_or(mlua::Value::Nil);
            let v4: mlua::Value = this.get(4).unwrap_or(mlua::Value::Nil);
            Ok((v1, v2, v3, v4))
        })?;
        table.set("unpack", unpack_fn)?;
        Ok(table)
    }

    /// Execute Lua code and return the result.
    pub fn eval<T: mlua::FromLuaMulti>(&self, code: &str) -> Result<T> {
        let result = self.lua.load(code).eval()?;
        Ok(result)
    }

    /// Apply post-load workarounds for Blizzard code that depends on
    /// unimplemented engine features (AnimationGroups, EditMode, etc.).
    /// Must be called after all addons are loaded and before firing events.
    pub fn apply_post_load_workarounds(&self) {
        super::workarounds::apply(self);
    }

    /// Apply workarounds that must run after startup events.
    ///
    /// Some fixes (like BagsBar anchoring) get undone by event handlers
    /// (e.g. EDIT_MODE_LAYOUTS_UPDATED repositions managed frames).
    pub fn apply_post_event_workarounds(&self) {
        super::workarounds::apply_post_event(self);
    }

    /// Fire an event to all registered frames.
    pub fn fire_event(&self, event: &str) -> Result<()> {
        self.fire_event_with_args(event, &[])
    }

    /// Fire an event with arguments to all registered frames.
    pub fn fire_event_with_args(&self, event: &str, args: &[Value]) -> Result<()> {
        use super::script_helpers::{call_error_handler, get_frame_ref, get_script};

        let listeners = {
            let state = self.state.borrow();
            state.widgets.get_event_listeners(event)
        };

        for widget_id in listeners {
            if let Some(handler) = get_script(&self.lua, widget_id, "OnEvent")
                && let Some(frame) = get_frame_ref(&self.lua, widget_id) {
                    let addon_idx = self.state.borrow().widgets.get(widget_id)
                        .and_then(|f| f.owner_addon);
                    let mut call_args =
                        vec![frame, Value::String(self.lua.create_string(event)?)];
                    call_args.extend(args.iter().cloned());

                    let start = Instant::now();
                    if let Err(e) = handler.call::<()>(MultiValue::from_vec(call_args)) {
                        call_error_handler(&self.lua, &e.to_string());
                    }
                    if let Some(idx) = addon_idx {
                        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                        let mut state = self.state.borrow_mut();
                        if let Some(addon) = state.addons.get_mut(idx as usize) {
                            addon.runtime.current_frame_ms += elapsed_ms;
                        }
                    }
                }
        }

        Ok(())
    }

    /// Fire a script handler for a specific widget.
    /// handler_name is like "OnClick", "OnEnter", etc.
    /// extra_args are passed after the frame (self) argument.
    /// Lua errors are routed through `call_error_handler` (same as event dispatch).
    pub fn fire_script_handler(
        &self,
        widget_id: u64,
        handler_name: &str,
        extra_args: Vec<Value>,
    ) -> Result<()> {
        use super::script_helpers::{call_error_handler, get_script};

        if let Some(handler) = get_script(&self.lua, widget_id, handler_name) {
            let frame = super::frame::frame_lud(widget_id);

            let mut call_args = vec![frame];
            call_args.extend(extra_args);

            if let Err(e) = handler.call::<()>(MultiValue::from_vec(call_args)) {
                call_error_handler(&self.lua, &e.to_string());
            }
        }

        Ok(())
    }

    /// Check if a script handler is registered for a widget.
    pub fn has_script_handler(&self, widget_id: u64, handler_name: &str) -> bool {
        super::script_helpers::get_script(&self.lua, widget_id, handler_name).is_some()
    }

    /// Simulate a left-click on a frame by ID.
    ///
    /// Handles EditBox focus management (focus/unfocus), then fires
    /// OnMouseDown, OnClick, and OnMouseUp in sequence.
    pub fn send_click(&self, frame_id: u64) -> Result<()> {
        use crate::widget::WidgetType;

        let is_editbox = self
            .state
            .borrow()
            .widgets
            .get(frame_id)
            .map(|f| f.widget_type == WidgetType::EditBox)
            .unwrap_or(false);

        let old_focus = self.state.borrow().focused_frame_id;

        // EditBox focus management (mirrors iced_app::update::update_editbox_focus)
        if is_editbox {
            if old_focus != Some(frame_id) {
                self.state.borrow_mut().focused_frame_id = Some(frame_id);
                if let Some(old_id) = old_focus {
                    self.fire_script_handler(old_id, "OnEditFocusLost", vec![])?;
                }
                self.fire_script_handler(frame_id, "OnEditFocusGained", vec![])?;
            }
        } else if let Some(old_id) = old_focus {
            self.state.borrow_mut().focused_frame_id = None;
            self.fire_script_handler(old_id, "OnEditFocusLost", vec![])?;
        }

        let button_val = Value::String(self.lua.create_string("LeftButton")?);
        self.fire_script_handler(frame_id, "OnMouseDown", vec![button_val.clone()])?;
        let down_val = Value::Boolean(false);
        self.fire_script_handler(frame_id, "OnClick", vec![button_val.clone(), down_val])?;
        self.fire_script_handler(frame_id, "OnMouseUp", vec![button_val])?;

        Ok(())
    }

    /// Dispatch a slash command (e.g., "/wa options").
    /// Returns Ok(true) if a handler was found and called, Ok(false) if no handler matched.
    pub fn dispatch_slash_command(&self, input: &str) -> Result<bool> {
        let input = input.trim();
        if !input.starts_with('/') {
            return Ok(false);
        }

        // Parse command and message: "/wa options" -> cmd="/wa", msg="options"
        let (cmd, msg) = match input.find(' ') {
            Some(pos) => (&input[..pos], input[pos + 1..].trim()),
            None => (input, ""),
        };
        let cmd_lower = cmd.to_lowercase();

        // Scan globals for SLASH_* variables to find a matching command
        let globals = self.lua.globals();
        let slash_cmd_list: mlua::Table = globals.get("SlashCmdList")?;

        // Iterate through all globals looking for SLASH_* patterns
        for pair in globals.pairs::<String, Value>() {
            let (key, value) = pair?;

            // Look for SLASH_NAME1, SLASH_NAME2, etc.
            if !key.starts_with("SLASH_") {
                continue;
            }

            // Extract the command name (e.g., "SLASH_WEAKAURAS1" -> "WEAKAURAS")
            let suffix = &key[6..]; // Skip "SLASH_"
            let name = suffix.trim_end_matches(|c: char| c.is_ascii_digit());
            if name.is_empty() {
                continue;
            }

            // Check if this SLASH_ variable matches our command
            if let Value::String(slash_str) = value
                && slash_str.to_str()?.to_lowercase() == cmd_lower {
                    // Found a match! Look up the handler in SlashCmdList
                    let handler: Option<mlua::Function> = slash_cmd_list.get(name).ok();
                    if let Some(handler) = handler {
                        let msg_value = self.lua.create_string(msg)?;
                        handler.call::<()>(msg_value)?;
                        return Ok(true);
                    }
                }
        }

        Ok(false)
    }

    /// Get access to the Lua state.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Get access to the simulator state.
    pub fn state(&self) -> &Rc<RefCell<SimState>> {
        &self.state
    }

    /// Create a loader environment borrowing from this environment.
    pub fn loader_env(&self) -> super::loader_env::LoaderEnv<'_> {
        super::loader_env::LoaderEnv::new(&self.lua, Rc::clone(&self.state))
    }

    /// Set the font system for text measurement from Lua API methods.
    ///
    /// This stores the font system as Lua app_data so that methods like
    /// `GetStringWidth()` can measure text accurately via cosmic-text.
    pub fn set_font_system(&self, font_system: Rc<RefCell<WowFontSystem>>) {
        self.lua.set_app_data(font_system);
    }

    /// Update screen dimensions in SimState and resize UIParent/WorldFrame to match.
    pub fn set_screen_size(&self, width: f32, height: f32) {
        let mut state = self.state.borrow_mut();
        state.screen_width = width;
        state.screen_height = height;
        // Screen resize invalidates all cached layout rects and strata buckets.
        state.strata_buckets = None;
        state.widgets.clear_all_layout_rects();
        for name in &["UIParent", "WorldFrame"] {
            if let Some(id) = state.widgets.get_id_by_name(name)
                && let Some(frame) = state.widgets.get_mut_visual(id) {
                    frame.width = width;
                    frame.height = height;
                }
        }
    }

    /// Register an addon in the addon list.
    pub fn register_addon(&self, info: AddonInfo) {
        self.state.borrow_mut().addons.push(info);
    }

    /// Scan an addons directory and register all found addons (metadata only, no loading).
    pub fn scan_and_register_addons(&self, addons_path: &std::path::Path) {
        use crate::toc::TocFile;
        let entries = match std::fs::read_dir(addons_path) {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut addons: Vec<AddonInfo> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if name.starts_with('.') || name == "BlizzardUI" {
                continue;
            }
            // Find TOC file
            let toc_path = crate::loader::find_toc_file(&path);
            let Some(toc_path) = toc_path else { continue };
            let toc = TocFile::from_file(&toc_path).ok();
            let (title, notes, load_on_demand) = toc
                .as_ref()
                .map(|t| {
                    let title = t.metadata.get("Title").cloned().unwrap_or_else(|| name.clone());
                    let notes = t.metadata.get("Notes").cloned().unwrap_or_default();
                    let lod = t.metadata.get("LoadOnDemand").map(|v| v == "1").unwrap_or(false);
                    (title, notes, lod)
                })
                .unwrap_or_else(|| (name.clone(), String::new(), false));
            addons.push(AddonInfo {
                folder_name: name, title, notes,
                enabled: true, loaded: false, load_on_demand,
                load_time_secs: 0.0, ..Default::default()
            });
        }
        addons.sort_by(|a, b| a.folder_name.to_lowercase().cmp(&b.folder_name.to_lowercase()));
        let mut state = self.state.borrow_mut();
        for addon in addons {
            // Don't register duplicates (Blizzard addons may already be registered)
            if !state.addons.iter().any(|a| a.folder_name == addon.folder_name) {
                state.addons.push(addon);
            }
        }
    }

    /// Schedule a timer callback.
    pub fn schedule_timer(
        &self,
        seconds: f64,
        callback: mlua::Function,
        interval: Option<std::time::Duration>,
        iterations: Option<i32>,
    ) -> Result<u64> {
        let id = next_timer_id();
        let callback_key = self.lua.create_registry_value(callback)?;
        let fire_at = Instant::now() + std::time::Duration::from_secs_f64(seconds);

        let owner_addon = self.state.borrow().loading_addon_index;
        let timer = PendingTimer {
            id, fire_at, callback_key, interval,
            remaining: iterations, cancelled: false,
            handle_key: None, owner_addon,
        };

        self.state.borrow_mut().timers.push_back(timer);
        Ok(id)
    }

    /// Cancel a timer by ID.
    pub fn cancel_timer(&self, timer_id: u64) {
        let mut state = self.state.borrow_mut();
        for timer in state.timers.iter_mut() {
            if timer.id == timer_id {
                timer.cancelled = true;
                break;
            }
        }
    }

    /// Remove registry keys for a finished or cancelled timer.
    fn cleanup_timer(&self, timer: PendingTimer) {
        self.lua.remove_registry_value(timer.callback_key).ok();
        if let Some(hk) = timer.handle_key {
            self.lua.remove_registry_value(hk).ok();
        }
    }

    /// Fire a single timer callback, returning true if it fired successfully.
    fn fire_timer_callback(&self, timer: &PendingTimer) -> bool {
        let Ok(callback) = self.lua.registry_value::<mlua::Function>(&timer.callback_key) else {
            return false;
        };
        let handle: Option<mlua::Table> = timer
            .handle_key
            .as_ref()
            .and_then(|k| self.lua.registry_value(k).ok());
        let result = match handle {
            Some(h) => callback.call::<()>(h),
            None => callback.call::<()>(()),
        };
        if let Err(e) = result {
            eprintln!("Timer callback error: {}", e);
        }
        true
    }

    /// Check if a ticker should repeat and decrement its remaining count.
    fn ticker_should_repeat(timer: &mut PendingTimer) -> bool {
        match &mut timer.remaining {
            Some(n) if *n > 1 => {
                *n -= 1;
                true
            }
            Some(_) => false,
            None => true,
        }
    }

    /// Process any timers that are ready to fire.
    /// Returns the number of callbacks invoked.
    pub fn process_timers(&self) -> Result<usize> {
        let now = Instant::now();
        let mut fired = 0;
        let mut to_reschedule = Vec::new();

        let mut state = self.state.borrow_mut();
        let mut i = 0;
        while i < state.timers.len() {
            if state.timers[i].cancelled {
                self.cleanup_timer(state.timers.remove(i).unwrap());
                continue;
            }

            if state.timers[i].fire_at <= now {
                let mut timer = state.timers.remove(i).unwrap();
                let timer_addon = timer.owner_addon;
                // Drop state borrow before calling Lua callback
                drop(state);

                let cb_start = Instant::now();
                if self.fire_timer_callback(&timer) {
                    let elapsed_ms = cb_start.elapsed().as_secs_f64() * 1000.0;
                    fired += 1;
                    state = self.state.borrow_mut();
                    if let Some(idx) = timer_addon {
                        if let Some(addon) = state.addons.get_mut(idx as usize) {
                            addon.runtime.current_frame_ms += elapsed_ms;
                        }
                    }

                    if let Some(interval) = timer.interval {
                        if Self::ticker_should_repeat(&mut timer) {
                            timer.fire_at = now + interval;
                            to_reschedule.push(timer);
                        } else {
                            self.cleanup_timer(timer);
                        }
                    } else {
                        self.cleanup_timer(timer);
                    }
                } else {
                    self.cleanup_timer(timer);
                    state = self.state.borrow_mut();
                }
                continue;
            }
            i += 1;
        }

        for timer in to_reschedule {
            state.timers.push_back(timer);
        }

        Ok(fired)
    }

    /// Check if there are any pending timers.
    pub fn has_pending_timers(&self) -> bool {
        !self.state.borrow().timers.is_empty()
    }


    /// Fire OnUpdate handlers for all frames that have them registered,
    /// then tick animation groups.
    /// `elapsed` is the time in seconds since the last frame.
    pub fn fire_on_update(&self, elapsed: f64) -> Result<()> {
        use super::script_helpers::{call_error_handler, get_frame_ref, get_script};

        let frame_ids = self.get_visible_on_update_frames();

        if !frame_ids.is_empty() {
            self.fire_on_update_handlers(&frame_ids, elapsed);
        }

        // Fire OnPostUpdate handlers
        if !frame_ids.is_empty() {
            let elapsed_val = Value::Number(elapsed);

            for widget_id in &frame_ids {
                let addon_idx = self.state.borrow().widgets.get(*widget_id)
                    .and_then(|f| f.owner_addon);
                let start = Instant::now();
                if let Some(handler) = get_script(&self.lua, *widget_id, "OnPostUpdate")
                    && let Some(frame) = get_frame_ref(&self.lua, *widget_id)
                        && let Err(e) = handler
                            .call::<()>(MultiValue::from_vec(vec![frame, elapsed_val.clone()]))
                        {
                            call_error_handler(&self.lua, &e.to_string());
                        }
                if let Some(idx) = addon_idx {
                    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                    let mut state = self.state.borrow_mut();
                    if let Some(addon) = state.addons.get_mut(idx as usize) {
                        addon.runtime.current_frame_ms += elapsed_ms;
                    }
                }
            }
        }

        // Tick animation groups
        super::animation::tick_animation_groups(&self.state, &self.lua, elapsed)?;

        // Finalize per-addon metrics for this frame.
        // elapsed is delta-time in seconds; convert to ms for metrics.
        self.finalize_frame_metrics(elapsed * 1000.0);

        Ok(())
    }

    /// Return the cached visible OnUpdate frame IDs.
    /// The cache is built on first call and kept up to date by `set_frame_visible`.
    fn get_visible_on_update_frames(&self) -> Vec<u64> {
        let mut state = self.state.borrow_mut();
        if let Some(ref cached) = state.visible_on_update_cache {
            return cached.clone();
        }
        // Initial build: filter all on_update_frames by ancestor visibility.
        let ids: Vec<u64> = state
            .on_update_frames
            .iter()
            .copied()
            .filter(|&id| state.widgets.is_ancestor_visible(id))
            .collect();
        state.visible_on_update_cache = Some(ids.clone());
        ids
    }

    /// Execute OnUpdate Lua handlers for the given visible frame IDs.
    ///
    /// Matches WoW behavior: handlers continue firing after errors (WoW shows
    /// an error popup but doesn't disable the handler). We suppress logging
    /// after 100 consecutive errors per frame to avoid infinite spam.
    fn fire_on_update_handlers(&self, frame_ids: &[u64], elapsed: f64) {
        use super::script_helpers::{call_error_handler, get_frame_ref, get_script};
        const SUPPRESS_THRESHOLD: u32 = 100;
        let elapsed_val = Value::Number(elapsed);
        let mut error_counts = self.on_update_errors.borrow_mut();
        for widget_id in frame_ids {
            let count = error_counts.get(widget_id).copied().unwrap_or(0);
            if count >= SUPPRESS_THRESHOLD {
                continue;
            }
            let addon_idx = self.state.borrow().widgets.get(*widget_id)
                .and_then(|f| f.owner_addon);
            let start = Instant::now();
            if let Some(handler) = get_script(&self.lua, *widget_id, "OnUpdate")
                && let Some(frame) = get_frame_ref(&self.lua, *widget_id)
            {
                match handler.call::<()>(MultiValue::from_vec(vec![frame, elapsed_val.clone()])) {
                    Ok(()) => {
                        // Success: reset consecutive error count.
                        error_counts.remove(widget_id);
                    }
                    Err(e) => {
                        let new_count = count + 1;
                        if new_count <= 3 || new_count == SUPPRESS_THRESHOLD {
                            let name = self.state.borrow().widgets.get(*widget_id)
                                .and_then(|f| f.name.clone())
                                .unwrap_or_else(|| format!("id={}", widget_id));
                            eprintln!(
                                "[OnUpdate] error #{} in frame '{}': {}",
                                new_count, name, e
                            );
                            call_error_handler(&self.lua, &e.to_string());
                            if new_count == SUPPRESS_THRESHOLD {
                                eprintln!(
                                    "[OnUpdate] suppressing '{}' after {} consecutive errors",
                                    name, SUPPRESS_THRESHOLD
                                );
                            }
                        }
                        error_counts.insert(*widget_id, new_count);
                    }
                }
            }
            if let Some(idx) = addon_idx {
                let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                let mut state = self.state.borrow_mut();
                if let Some(addon) = state.addons.get_mut(idx as usize) {
                    addon.runtime.current_frame_ms += elapsed_ms;
                }
            }
        }
    }


    /// Aggregate per-addon metrics at the end of each frame tick.
    fn finalize_frame_metrics(&self, frame_elapsed_ms: f64) {
        let mut state = self.state.borrow_mut();
        // Update app-level frame metrics (total frame time for percentage calculations).
        let app = &mut state.app_frame_metrics;
        app.recent_frame_ms.push_back(frame_elapsed_ms);
        if app.recent_frame_ms.len() > 60 {
            app.recent_frame_ms.pop_front();
        }
        if frame_elapsed_ms > app.peak_ms {
            app.peak_ms = frame_elapsed_ms;
        }
        app.session_total_ms += frame_elapsed_ms;
        app.session_frame_count += 1;

        for addon in &mut state.addons {
            let ms = addon.runtime.current_frame_ms;
            if ms > 0.0 {
                addon.runtime.recent_frames.push_back(ms);
                if addon.runtime.recent_frames.len() > 60 {
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

    /// Fire `EDIT_MODE_LAYOUTS_UPDATED` with layout info from `C_EditMode.GetLayouts()`.
    ///
    /// Triggers `EditModeManagerFrame:UpdateLayoutInfo()` to initialize `layoutInfo`
    /// and unblock action bar positioning. No-op if EditMode addon isn't loaded.
    pub fn fire_edit_mode_layouts_updated(&self) -> Result<()> {
        let Ok(true) = self.lua.load(
            "return C_EditMode ~= nil and C_EditMode.GetLayouts ~= nil"
        ).eval::<bool>() else { return Ok(()) };

        let Ok(info) = self.lua.load("return C_EditMode.GetLayouts()").eval::<mlua::Table>()
        else { return Ok(()) };

        self.fire_event_with_args(
            "EDIT_MODE_LAYOUTS_UPDATED",
            &[Value::Table(info), Value::Boolean(true)],
        )
    }

    /// Get the time until the next timer fires, if any.
    pub fn next_timer_delay(&self) -> Option<std::time::Duration> {
        let state = self.state.borrow();
        let now = Instant::now();
        state
            .timers
            .iter()
            .filter(|t| !t.cancelled)
            .map(|t| {
                if t.fire_at > now {
                    t.fire_at - now
                } else {
                    std::time::Duration::ZERO
                }
            })
            .min()
    }

    /// Dump all frame positions for debugging.
    pub fn dump_frames(&self) -> String {
        let state = self.state.borrow();
        super::diagnostics::dump_frames(&state)
    }
}

/// Increment threshold counters for a frame's addon time.
fn update_threshold_counters(rt: &mut AddonRuntimeMetrics, ms: f64) {
    if ms > 1.0 { rt.count_over_1ms += 1; }
    if ms > 5.0 { rt.count_over_5ms += 1; }
    if ms > 10.0 { rt.count_over_10ms += 1; }
    if ms > 50.0 { rt.count_over_50ms += 1; }
    if ms > 100.0 { rt.count_over_100ms += 1; }
    if ms > 500.0 { rt.count_over_500ms += 1; }
    if ms > 1000.0 { rt.count_over_1000ms += 1; }
}
