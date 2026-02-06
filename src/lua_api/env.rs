//! WoW Lua environment.

use super::builtin_frames::create_builtin_frames;
use super::layout::{compute_frame_rect, get_parent_depth};
use super::state::{AddonInfo, PendingTimer, SimState};
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
    lua: Lua,
    state: Rc<RefCell<SimState>>,
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
            create_builtin_frames(&mut s.widgets);
        }

        // Register global functions
        super::globals::register_globals(&lua, Rc::clone(&state))?;

        Ok(Self { lua, state })
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

    /// Fire an event to all registered frames.
    pub fn fire_event(&self, event: &str) -> Result<()> {
        self.fire_event_with_args(event, &[])
    }

    /// Fire an event with arguments to all registered frames.
    pub fn fire_event_with_args(&self, event: &str, args: &[Value]) -> Result<()> {
        let listeners = {
            let state = self.state.borrow();
            state.widgets.get_event_listeners(event)
        };

        for widget_id in listeners {
            // Get the handler function from our scripts table
            let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();

            if let Some(table) = scripts_table {
                let frame_key = format!("{}_OnEvent", widget_id);
                let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

                if let Some(handler) = handler {
                    // Get the frame userdata
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                    // Build arguments: (self, event, ...args)
                    let mut call_args = vec![frame, Value::String(self.lua.create_string(event)?)];
                    call_args.extend(args.iter().cloned());

                    handler.call::<()>(MultiValue::from_vec(call_args))?;
                }
            }
        }

        Ok(())
    }

    /// Fire a script handler for a specific widget.
    /// handler_name is like "OnClick", "OnEnter", etc.
    /// extra_args are passed after the frame (self) argument.
    pub fn fire_script_handler(
        &self,
        widget_id: u64,
        handler_name: &str,
        extra_args: Vec<Value>,
    ) -> Result<()> {
        let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();

        if let Some(table) = scripts_table {
            let frame_key = format!("{}_{}", widget_id, handler_name);
            let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

            if let Some(handler) = handler {
                // Get the frame userdata
                let frame_ref_key = format!("__frame_{}", widget_id);
                let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                // Build arguments: (self, ...extra_args)
                let mut call_args = vec![frame];
                call_args.extend(extra_args);

                handler.call::<()>(MultiValue::from_vec(call_args))?;
            }
        }

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
            if let Value::String(slash_str) = value {
                if slash_str.to_str()?.to_lowercase() == cmd_lower {
                    // Found a match! Look up the handler in SlashCmdList
                    let handler: Option<mlua::Function> = slash_cmd_list.get(name).ok();
                    if let Some(handler) = handler {
                        let msg_value = self.lua.create_string(msg)?;
                        handler.call::<()>(msg_value)?;
                        return Ok(true);
                    }
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

    /// Set the font system for text measurement from Lua API methods.
    ///
    /// This stores the font system as Lua app_data so that methods like
    /// `GetStringWidth()` can measure text accurately via cosmic-text.
    pub fn set_font_system(&self, font_system: Rc<RefCell<WowFontSystem>>) {
        self.lua.set_app_data(font_system);
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
            if name.starts_with('.') || name == "wow-ui-source" {
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
                folder_name: name,
                title,
                notes,
                enabled: true,
                loaded: false,
                load_on_demand,
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

        let timer = PendingTimer {
            id,
            fire_at,
            callback_key,
            interval,
            remaining: iterations,
            cancelled: false,
            handle_key: None,
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

    /// Process any timers that are ready to fire.
    /// Returns the number of callbacks invoked.
    pub fn process_timers(&self) -> Result<usize> {
        let now = Instant::now();
        let mut fired = 0;
        let mut to_reschedule = Vec::new();

        // Collect timers that need to fire
        let mut state = self.state.borrow_mut();
        let mut i = 0;
        while i < state.timers.len() {
            if state.timers[i].cancelled {
                // Remove cancelled timers and clean up registry
                let timer = state.timers.remove(i).unwrap();
                self.lua.remove_registry_value(timer.callback_key).ok();
                if let Some(hk) = timer.handle_key {
                    self.lua.remove_registry_value(hk).ok();
                }
                continue;
            }

            if state.timers[i].fire_at <= now {
                let mut timer = state.timers.remove(i).unwrap();

                // Get the callback from registry
                if let Ok(callback) = self.lua.registry_value::<mlua::Function>(&timer.callback_key)
                {
                    // Get the handle table if present (for NewTimer/NewTicker)
                    let handle: Option<mlua::Table> = timer
                        .handle_key
                        .as_ref()
                        .and_then(|k| self.lua.registry_value(k).ok());

                    // Drop state borrow before calling Lua
                    drop(state);

                    // Call the callback with the handle as argument (if present)
                    let result = if let Some(h) = handle {
                        callback.call::<()>(h)
                    } else {
                        callback.call::<()>(())
                    };
                    if let Err(e) = result {
                        eprintln!("Timer callback error: {}", e);
                    }
                    fired += 1;

                    // Re-borrow state
                    state = self.state.borrow_mut();

                    // Check if this is a ticker that should repeat
                    if let Some(interval) = timer.interval {
                        let should_repeat = match &mut timer.remaining {
                            Some(n) if *n > 1 => {
                                *n -= 1;
                                true
                            }
                            Some(_) => false, // Last iteration
                            None => true,     // Infinite ticker
                        };

                        if should_repeat {
                            timer.fire_at = now + interval;
                            to_reschedule.push(timer);
                        } else {
                            // Clean up registry keys for finished timer
                            self.lua.remove_registry_value(timer.callback_key).ok();
                            if let Some(hk) = timer.handle_key {
                                self.lua.remove_registry_value(hk).ok();
                            }
                        }
                    } else {
                        // One-shot timer, clean up registry keys
                        self.lua.remove_registry_value(timer.callback_key).ok();
                        if let Some(hk) = timer.handle_key {
                            self.lua.remove_registry_value(hk).ok();
                        }
                    }
                } else {
                    // Callback not found, clean up
                    self.lua.remove_registry_value(timer.callback_key).ok();
                    if let Some(hk) = timer.handle_key {
                        self.lua.remove_registry_value(hk).ok();
                    }
                }
                continue;
            }
            i += 1;
        }

        // Re-add tickers that should repeat
        for timer in to_reschedule {
            state.timers.push_back(timer);
        }

        Ok(fired)
    }

    /// Check if there are any pending timers.
    pub fn has_pending_timers(&self) -> bool {
        !self.state.borrow().timers.is_empty()
    }

    /// Fire OnUpdate handlers for all frames that have them registered.
    /// `elapsed` is the time in seconds since the last frame.
    pub fn fire_on_update(&self, elapsed: f64) -> Result<()> {
        let frame_ids: Vec<u64> = {
            let state = self.state.borrow();
            state.on_update_frames.iter().copied().collect()
        };

        if frame_ids.is_empty() {
            return Ok(());
        }

        let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();
        let Some(table) = scripts_table else {
            return Ok(());
        };

        let elapsed_val = Value::Number(elapsed);

        for widget_id in frame_ids {
            let frame_key = format!("{}_OnUpdate", widget_id);
            let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

            if let Some(handler) = handler {
                let frame_ref_key = format!("__frame_{}", widget_id);
                let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                if let Err(e) =
                    handler.call::<()>(MultiValue::from_vec(vec![frame, elapsed_val.clone()]))
                {
                    eprintln!("OnUpdate error (frame {}): {}", widget_id, e);
                }
            }
        }

        Ok(())
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
    /// Returns a formatted string similar to iced-debug output.
    pub fn dump_frames(&self) -> String {
        let state = self.state.borrow();
        let screen_width = 500.0_f32;
        let screen_height = 375.0_f32;

        let mut output = String::new();
        output.push_str(&format!(
            "[WoW Frames: {}x{}]\n\n",
            screen_width, screen_height
        ));

        // Collect and sort frames by strata/level
        let mut frames: Vec<_> = state.widgets.all_ids().into_iter().collect();
        frames.sort_by(|&a, &b| {
            let fa = state.widgets.get(a);
            let fb = state.widgets.get(b);
            match (fa, fb) {
                (Some(fa), Some(fb)) => fa
                    .frame_strata
                    .cmp(&fb.frame_strata)
                    .then_with(|| fa.frame_level.cmp(&fb.frame_level)),
                _ => std::cmp::Ordering::Equal,
            }
        });

        for id in frames {
            let frame = match state.widgets.get(id) {
                Some(f) => f,
                None => continue,
            };

            // Compute position
            let rect = compute_frame_rect(&state.widgets, id, screen_width, screen_height);

            // Format: Name [Type] (x,y w×h) visible/hidden
            let name = frame.name.as_deref().unwrap_or("(anon)");
            let vis = if frame.visible { "" } else { " HIDDEN" };
            let mouse = if frame.mouse_enabled { " mouse" } else { "" };

            // Indentation based on parent depth
            let depth = get_parent_depth(&state.widgets, id);
            let indent = "  ".repeat(depth);

            // Get parent name for context
            let parent_name = frame
                .parent_id
                .and_then(|pid| state.widgets.get(pid))
                .and_then(|p| p.name.as_deref())
                .unwrap_or("(root)");

            output.push_str(&format!(
                "{}{} [{}] ({:.0},{:.0} {:.0}x{:.0}){}{} parent={}\n",
                indent,
                name,
                frame.widget_type.as_str(),
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                vis,
                mouse,
                parent_name,
            ));

            // Show anchor info
            if !frame.anchors.is_empty() {
                let anchor = &frame.anchors[0];
                output.push_str(&format!(
                    "{}  └─ {:?} -> {:?} offset ({:.0},{:.0})\n",
                    indent, anchor.point, anchor.relative_point, anchor.x_offset, anchor.y_offset
                ));
            } else {
                output.push_str(&format!("{}  └─ (no anchors - centered)\n", indent));
            }
        }

        output
    }
}
