use super::env::WowLuaEnv;
use super::env_init::{addon_taint_name, is_blizzard_addon, record_addon_time};
use super::handler_timing;
use super::state::SimState;
use crate::Result;
use crate::lua_api::methods::{
    call_function as call_rilua_function, create_string, frame_ref, val_to_string,
};
use crate::lua_api::script_helpers::{
    call_error_handler, event_matches_unit_filter, get_event_listeners, get_script,
    protected_lua_pcall_state,
};
use rilua::{LuaApi, LuaApiMut, Val};
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::time::{Duration, Instant};

type EventTraceLabel = (String, Instant);

fn event_dispatch_trace_enabled(event: &str) -> bool {
    let Some(filter) = env::var_os("WOW_SIM_TRACE_EVENT_DISPATCH") else {
        return false;
    };
    let filter = filter.to_string_lossy();
    filter == "*" || filter.split(',').any(|name| name.trim() == event)
}

fn trace_event_phase(event: &str, phase: &str, started: Instant) {
    if !event_dispatch_trace_enabled(event) {
        return;
    }
    eprintln!(
        "[event-dispatch] event={event} phase={phase} duration_ms={:.3}",
        started.elapsed().as_secs_f64() * 1000.0
    );
}

fn should_skip_startup_actionbar_dispatch(state: &SimState, widget_id: u64, event: &str) -> bool {
    if event != "PLAYER_ENTERING_WORLD" {
        return false;
    }

    matches!(
        state
            .widgets
            .get(widget_id)
            .and_then(|frame| frame.name.as_deref()),
        Some("ActionBarButtonEventsFrame" | "ActionBarController")
    )
}

fn should_skip_mists_addon_player_login(
    state: &SimState,
    addon_idx: Option<u16>,
    event: &str,
) -> bool {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Mists {
        return false;
    }
    if event != "PLAYER_LOGIN" {
        return false;
    }

    let Some(addon_idx) = addon_idx else {
        return false;
    };

    state
        .addons
        .get(addon_idx as usize)
        .map(|addon| addon.folder_name == "ElvUI_Libraries")
        .unwrap_or(false)
}

fn should_skip_mists_raid_frame_group_roster(
    state: &SimState,
    widget_id: u64,
    event: &str,
) -> bool {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Mists {
        return false;
    }
    if event != "GROUP_ROSTER_UPDATE" {
        return false;
    }

    state
        .widgets
        .get(widget_id)
        .and_then(|frame| frame.name.as_deref())
        == Some("RaidFrame")
}

fn log_widget_handler_timing(
    state: &Rc<RefCell<SimState>>,
    widget_id: u64,
    addon_idx: Option<u16>,
    handler_name: &str,
    duration: Duration,
    source: Option<&str>,
) {
    if !handler_timing::should_log(duration) {
        return;
    }

    let (addon_name, frame_name) = {
        let sim = state.borrow();
        let addon_name = addon_idx
            .and_then(|idx| sim.addons.get(idx as usize))
            .map(|addon| addon.folder_name.clone());
        let frame_name = sim
            .widgets
            .get(widget_id)
            .and_then(|frame| frame.name.clone());
        (addon_name, frame_name)
    };

    handler_timing::log_with_source(
        addon_name.as_deref(),
        handler_name,
        frame_name.as_deref(),
        widget_id,
        duration,
        source,
    );
}

fn widget_handler_source_label(lua: &rilua::Lua, handler: Val) -> Option<String> {
    let Val::Function(func_ref) = handler else {
        return None;
    };
    handler_timing::lua_closure_source_label(lua.state(), func_ref)
}

fn widget_handler_error_message(
    state: &Rc<RefCell<SimState>>,
    widget_id: u64,
    addon_idx: Option<u16>,
    handler_name: &str,
    source: Option<&str>,
    error: &str,
) -> String {
    let (addon_name, frame_name) = {
        let sim = state.borrow();
        let addon_name = addon_idx
            .and_then(|idx| sim.addons.get(idx as usize))
            .map(|addon| addon.folder_name.clone())
            .unwrap_or_else(|| "__BuiltIn".to_string());
        let frame_name = sim
            .widgets
            .get(widget_id)
            .and_then(|frame| frame.name.clone())
            .unwrap_or_else(|| format!("#{widget_id}"));
        (addon_name, frame_name)
    };

    let mut message = format!("[{handler_name}] frame={frame_name} addon={addon_name}");
    if let Some(source) = source.filter(|source| !source.is_empty()) {
        message.push_str(" source=");
        message.push_str(source);
    }
    message.push_str(": ");
    message.push_str(error);
    message
}

fn global_hash_entries(
    state: &rilua::vm::state::LuaState,
    globals: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> Vec<(Val, Val)> {
    state
        .gc
        .tables
        .get(globals)
        .map(|table| table.hash_entries())
        .unwrap_or_default()
}

fn slash_command_name(key: &str) -> Option<&str> {
    if !key.starts_with("SLASH_") {
        return None;
    }
    let suffix = &key[6..];
    let name = suffix.trim_end_matches(|c: char| c.is_ascii_digit());
    (!name.is_empty()).then_some(name)
}

fn matching_slash_handlers(
    state: &mut rilua::vm::state::LuaState,
    globals: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    slash_table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    command: &str,
) -> Vec<(String, Val)> {
    let entries = global_hash_entries(state, globals);
    let mut matches = Vec::new();

    for (key, value) in entries {
        let Some(key_string) = val_to_string(state, key) else {
            continue;
        };
        let Some(name) = slash_command_name(&key_string) else {
            continue;
        };
        let Some(slash_command) = val_to_string(state, value) else {
            continue;
        };
        if slash_command.to_lowercase() != command {
            continue;
        }

        let handler_key = state.gc.intern_string(name.as_bytes());
        let handler = state
            .gc
            .tables
            .get(slash_table_ref)
            .map(|table| table.get_str(handler_key, &state.gc.string_arena))
            .unwrap_or(Val::Nil);
        matches.push((name.to_string(), handler));
    }

    matches
}

impl WowLuaEnv {
    /// Fire an event to all registered frames.
    pub fn fire_event(&self, event: &str) -> Result<()> {
        self.fire_event_with_args(event, &[])
    }

    /// Fire an event with arguments to all registered frames.
    pub fn fire_event_with_args(&self, event: &str, args: &[Val]) -> Result<()> {
        #[cfg(feature = "retail-12-0-0")]
        super::globals::real::event_callbacks::dispatch_event_callbacks(
            self.lua.borrow_mut().state_mut(),
            event,
            args,
        )?;
        let collect_started = Instant::now();
        let listeners = {
            let mut lua = self.lua.borrow_mut();
            get_event_listeners(lua.state_mut(), event)
        };
        trace_event_phase(event, "collect-listeners", collect_started);
        let dispatch_started = Instant::now();
        for widget_id in listeners {
            self.dispatch_event_to_frame(widget_id, event, args)?;
        }
        trace_event_phase(event, "dispatch-listeners", dispatch_started);
        if event == "PLAYER_ENTERING_WORLD" {
            self.apply_post_event_workarounds();
        }
        Ok(())
    }

    fn handler_owner_addon(&self, widget_id: u64) -> Option<u16> {
        self.state
            .borrow()
            .widgets
            .get(widget_id)
            .and_then(|frame| frame.owner_addon)
    }

    fn build_event_call_args(
        &self,
        lua: &mut rilua::Lua,
        widget_id: u64,
        event: &str,
        args: &[Val],
    ) -> Result<Vec<Val>> {
        let frame = {
            let state = lua.state_mut();
            frame_ref(state, widget_id)?
        };
        let event_name = {
            let state = lua.state_mut();
            create_string(state, event)
        };
        let mut call_args = Vec::with_capacity(args.len() + 2);
        call_args.push(frame);
        call_args.push(event_name);
        call_args.extend_from_slice(args);
        Ok(call_args)
    }

    fn build_script_call_args(
        &self,
        lua: &mut rilua::Lua,
        widget_id: u64,
        extra_args: Vec<Val>,
    ) -> Result<Vec<Val>> {
        let frame = {
            let state = lua.state_mut();
            frame_ref(state, widget_id)?
        };
        let mut call_args = Vec::with_capacity(extra_args.len() + 1);
        call_args.push(frame);
        call_args.extend(extra_args);
        Ok(call_args)
    }

    fn call_widget_handler(
        &self,
        lua: &mut rilua::Lua,
        widget_id: u64,
        addon_idx: Option<u16>,
        handler_name: &str,
        handler: Val,
        call_args: &[Val],
    ) {
        let taint = addon_taint_name(&self.state, addon_idx);
        let blizzard = is_blizzard_addon(&self.state, addon_idx);
        let _ = (taint, blizzard);

        let start = Instant::now();
        self.state.borrow_mut().executing_addon_index = addon_idx;
        let call_result = protected_lua_pcall_state(lua.state_mut(), handler, call_args);
        if let Err(error) = call_result {
            let source = widget_handler_source_label(lua, handler);
            let error = widget_handler_error_message(
                &self.state,
                widget_id,
                addon_idx,
                handler_name,
                source.as_deref(),
                &error,
            );
            call_error_handler(lua, &error);
        }
        self.state.borrow_mut().executing_addon_index = None;
        let elapsed = start.elapsed();
        record_addon_time(&self.state, addon_idx, &start);
        let source = widget_handler_source_label(lua, handler);
        log_widget_handler_timing(
            &self.state,
            widget_id,
            addon_idx,
            handler_name,
            elapsed,
            source.as_deref(),
        );
    }

    fn event_trace_label(&self, widget_id: u64, event: &str) -> Option<EventTraceLabel> {
        if !event_dispatch_trace_enabled(event) {
            return None;
        }

        let state = self.state.borrow();
        let frame_name = state
            .widgets
            .get(widget_id)
            .map(|frame| {
                let name = frame
                    .name
                    .clone()
                    .or_else(|| frame.parent_key.clone())
                    .unwrap_or_else(|| format!("#{widget_id}"));
                let object_type = frame
                    .object_type_name
                    .clone()
                    .unwrap_or_else(|| format!("{:?}", frame.widget_type));
                let owner = frame
                    .owner_addon
                    .and_then(|index| state.addons.get(index as usize))
                    .map(|addon| addon.folder_name.clone())
                    .unwrap_or_else(|| "?".to_string());
                format!("{name} [{object_type}] owner={owner}")
            })
            .unwrap_or_else(|| format!("#{widget_id}"));
        Some((frame_name, state.start_time))
    }

    fn on_event_handler(&self, lua: &mut rilua::Lua, widget_id: u64) -> Option<Val> {
        let state = lua.state_mut();
        get_script(state, widget_id, "OnEvent")
    }

    fn log_event_dispatch(&self, trace_label: &Option<EventTraceLabel>, event: &str, phase: &str) {
        let Some((frame_name, start_time)) = trace_label else {
            return;
        };
        eprintln!(
            "{} [EventTrace] {event} -> {frame_name} {phase}",
            crate::logging::elapsed_prefix(*start_time)
        );
    }

    fn dispatch_event_to_frame(&self, widget_id: u64, event: &str, args: &[Val]) -> Result<()> {
        {
            let state = self.state.borrow();
            if should_skip_startup_actionbar_dispatch(&state, widget_id, event)
                || should_skip_mists_raid_frame_group_roster(&state, widget_id, event)
            {
                return Ok(());
            }
        }
        let addon_idx = self.handler_owner_addon(widget_id);
        if should_skip_mists_addon_player_login(&self.state.borrow(), addon_idx, event) {
            return Ok(());
        }
        let trace_label = self.event_trace_label(widget_id, event);
        let mut lua = self.lua.borrow_mut();
        if !event_matches_unit_filter(lua.state(), widget_id, event, args)? {
            return Ok(());
        }
        let handler = self.on_event_handler(&mut lua, widget_id);
        let Some(handler) = handler else {
            return Ok(());
        };
        self.log_event_dispatch(&trace_label, event, "begin");
        let call_args = self.build_event_call_args(&mut lua, widget_id, event, args)?;
        self.call_widget_handler(
            &mut lua, widget_id, addon_idx, "OnEvent", handler, &call_args,
        );
        self.log_event_dispatch(&trace_label, event, "end");
        Ok(())
    }

    /// Fire a script handler for a specific widget with per-addon taint restoration.
    pub fn fire_script_handler(
        &self,
        widget_id: u64,
        handler_name: &str,
        extra_args: Vec<Val>,
    ) -> Result<()> {
        let addon_idx = self.handler_owner_addon(widget_id);
        let mut lua = self.lua.borrow_mut();
        let handlers = {
            let state = lua.state_mut();
            crate::lua_api::script_helpers::get_scripts_for_dispatch(state, widget_id, handler_name)
        };
        if handlers.is_empty() {
            return Ok(());
        };

        let call_args = self.build_script_call_args(&mut lua, widget_id, extra_args)?;
        for handler in handlers {
            self.call_widget_handler(
                &mut lua,
                widget_id,
                addon_idx,
                handler_name,
                handler,
                &call_args,
            );
        }
        Ok(())
    }

    /// Check if a script handler is registered for a widget.
    pub fn has_script_handler(&self, widget_id: u64, handler_name: &str) -> bool {
        let mut lua = self.lua.borrow_mut();
        get_script(lua.state_mut(), widget_id, handler_name).is_some()
    }

    /// Resolve a clicked frame to the nearest EditBox in its parent chain.
    pub(crate) fn resolve_editbox_focus_target(&self, clicked_frame: Option<u64>) -> Option<u64> {
        use crate::widget::WidgetType;

        let state = self.state.borrow();
        let mut current = clicked_frame;

        while let Some(frame_id) = current {
            let Some(frame) = state.widgets.get(frame_id) else {
                break;
            };
            if frame.widget_type == WidgetType::EditBox {
                return Some(frame_id);
            }
            current = frame.parent_id;
        }

        None
    }

    /// Simulate a left-click on a frame by ID.
    pub fn send_click(&self, frame_id: u64) -> Result<()> {
        let editbox_target = self.resolve_editbox_focus_target(Some(frame_id));
        let old_focus = self.state.borrow().focused_frame_id;

        if let Some(editbox_id) = editbox_target {
            if old_focus != Some(editbox_id) {
                self.state.borrow_mut().focused_frame_id = Some(editbox_id);
                if let Some(old_id) = old_focus {
                    self.fire_script_handler(old_id, "OnEditFocusLost", vec![])?;
                }
                self.fire_script_handler(editbox_id, "OnEditFocusGained", vec![])?;
            }
        } else if let Some(old_id) = old_focus {
            self.state.borrow_mut().focused_frame_id = None;
            self.fire_script_handler(old_id, "OnEditFocusLost", vec![])?;
        }

        let button_val = self.lua_string("LeftButton");
        self.fire_script_handler(frame_id, "OnMouseDown", vec![button_val])?;
        self.fire_script_handler(frame_id, "OnClick", vec![button_val, Val::Bool(false)])?;
        self.fire_script_handler(frame_id, "OnMouseUp", vec![button_val, Val::Bool(true)])?;
        Ok(())
    }

    /// Dispatch a slash command (e.g., "/wa options").
    /// Returns Ok(true) if a handler was found and called, Ok(false) if no handler matched.
    pub fn dispatch_slash_command(&self, input: &str) -> Result<bool> {
        let input = input.trim();
        if !input.starts_with('/') {
            return Ok(false);
        }

        let (cmd, msg) = match input.find(' ') {
            Some(pos) => (&input[..pos], input[pos + 1..].trim()),
            None => (input, ""),
        };
        let cmd_lower = cmd.to_lowercase();

        let mut lua = self.lua.borrow_mut();
        let slash_cmd_list = LuaApiMut::get_global_val(&mut *lua, "SlashCmdList");
        let Val::Table(slash_table_ref) = slash_cmd_list else {
            return Ok(false);
        };
        let state = lua.state_mut();
        let globals = state.global;

        for (name, handler) in matching_slash_handlers(state, globals, slash_table_ref, &cmd_lower)
        {
            if !matches!(handler, Val::Function(_)) {
                continue;
            }
            let msg_val = create_string(state, msg);
            let _ = call_rilua_function(&mut lua, handler, &[msg_val])?;
            let _ = name;
            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::WowLuaEnv;

    #[test]
    fn dispatch_slash_command_uses_registered_handler() {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.exec(
            r#"
            SlashCmdList = {}
            local calls = {}
            function SlashCmdList.TEST(msg)
                calls[#calls + 1] = msg
            end
            SLASH_TEST1 = "/test"
            __dispatch_calls = calls
            "#,
        )
        .expect("Failed to register slash command");

        assert!(
            env.dispatch_slash_command("/test payload")
                .expect("slash dispatch should succeed")
        );
        assert_eq!(
            env.eval::<String>("return __dispatch_calls[1]")
                .expect("dispatch should record payload"),
            "payload"
        );
    }
}
