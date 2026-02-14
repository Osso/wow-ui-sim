//! MessageFrame widget methods: AddMessage, scrolling, fading, message history.

use crate::lua_api::frame::handle::{get_sim_state, lud_to_id};
use crate::lua_api::SimState;
use mlua::{LightUserData, Lua, Value};

pub fn add_message_frame_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_message_frame_add_methods(lua, methods)?;
    add_message_frame_count_methods(lua, methods)?;
    add_message_frame_fade_methods(lua, methods)?;
    add_message_frame_fade_duration_methods(lua, methods)?;
    add_message_frame_insert_methods(lua, methods)?;
    add_message_frame_scroll_methods(lua, methods)?;
    add_message_frame_misc_methods(lua, methods)?;
    add_message_frame_callback_stubs(lua, methods)?;
    Ok(())
}

fn add_message_frame_add_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // AddMessage(text, r, g, b, messageID, holdTime)
    methods.set("AddMessage", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        add_message_core(&mut state_rc.borrow_mut(), id, args, true);
        Ok(())
    })?)?;

    // AddMsg(text, ...) - Alias for AddMessage
    methods.set("AddMsg", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        add_message_core(&mut state_rc.borrow_mut(), id, args, true);
        Ok(())
    })?)?;

    // _AddMessageSilent(text, r, g, b, ...) - same as AddMessage but no terminal log
    methods.set("_AddMessageSilent", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        add_message_core(&mut state_rc.borrow_mut(), id, args, false);
        Ok(())
    })?)?;

    // BackFillMessage(text, r, g, b, ...)
    methods.set("BackFillMessage", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        backfill_message(&mut state_rc.borrow_mut(), id, args);
        Ok(())
    })?)?;

    // Clear() - Clear all messages (overrides tooltip Clear for MessageFrame)
    methods.set("Clear", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(data) = state.message_frames.get_mut(&id) {
            data.messages.clear();
            data.scroll_offset = 0;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn add_message_frame_count_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetNumMessages", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let count = state.message_frames.get(&id)
            .map(|d| d.messages.len())
            .unwrap_or(0);
        Ok(count as i32)
    })?)?;

    methods.set("SetMaxLines", lua.create_function(|lua, (ud, max_lines): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let data = state.message_frames.entry(id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.max_lines = max_lines.max(1) as usize;
        data.messages.truncate(data.max_lines);
        Ok(())
    })?)?;

    methods.set("GetMaxLines", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let max = state.message_frames.get(&id)
            .map(|d| d.max_lines)
            .unwrap_or(120);
        Ok(max as i32)
    })?)?;

    Ok(())
}

fn add_message_frame_fade_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetFading", lua.create_function(|lua, (ud, fading): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let data = state.message_frames.entry(id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fading = fading;
        Ok(())
    })?)?;

    methods.set("GetFading", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let fading = state.message_frames.get(&id)
            .map(|d| d.fading)
            .unwrap_or(true);
        Ok(fading)
    })?)?;

    methods.set("SetTimeVisible", lua.create_function(|lua, (ud, secs): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let data = state.message_frames.entry(id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.time_visible = secs;
        Ok(())
    })?)?;

    methods.set("GetTimeVisible", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let secs = state.message_frames.get(&id)
            .map(|d| d.time_visible)
            .unwrap_or(10.0);
        Ok(secs)
    })?)?;

    Ok(())
}

fn add_message_frame_fade_duration_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetFadeDuration", lua.create_function(|lua, (ud, secs): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let data = state.message_frames.entry(id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fade_duration = secs;
        Ok(())
    })?)?;

    methods.set("GetFadeDuration", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let secs = state.message_frames.get(&id)
            .map(|d| d.fade_duration)
            .unwrap_or(3.0);
        Ok(secs)
    })?)?;

    methods.set("SetFadePower", lua.create_function(|lua, (ud, power): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let data = state.message_frames.entry(id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fade_power = power;
        Ok(())
    })?)?;

    methods.set("GetFadePower", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let power = state.message_frames.get(&id)
            .map(|d| d.fade_power)
            .unwrap_or(1.0);
        Ok(power)
    })?)?;

    Ok(())
}

fn add_message_frame_insert_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetInsertMode", lua.create_function(|lua, (ud, mode): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let data = state.message_frames.entry(id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.insert_mode = mode;
        Ok(())
    })?)?;

    methods.set("GetInsertMode", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let mode = state.message_frames.get(&id)
            .map(|d| d.insert_mode.clone())
            .unwrap_or_else(|| "BOTTOM".to_string());
        Ok(mode)
    })?)?;

    Ok(())
}

fn add_message_frame_scroll_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    // Scroll methods (no-ops for now, no visual scrolling)
    methods.set("ScrollUp", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("ScrollDown", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("PageUp", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("PageDown", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("ScrollToTop", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("ScrollToBottom", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("AtTop", lua.create_function(|_, _ud: LightUserData| Ok(true))?)?;
    methods.set("AtBottom", lua.create_function(|_, _ud: LightUserData| Ok(true))?)?;

    methods.set("SetScrollOffset", lua.create_function(|lua, (ud, offset): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(data) = state.message_frames.get_mut(&id) {
            data.scroll_offset = offset;
        }
        Ok(())
    })?)?;

    methods.set("GetScrollOffset", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let offset = state.message_frames.get(&id)
            .map(|d| d.scroll_offset)
            .unwrap_or(0);
        Ok(offset)
    })?)?;

    methods.set("GetMaxScrollRange", lua.create_function(|_, _ud: LightUserData| Ok(0_i32))?)?;

    methods.set("SetScrollAllowed", lua.create_function(|lua, (ud, allowed): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let data = state.message_frames.entry(id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.scroll_allowed = allowed;
        Ok(())
    })?)?;

    methods.set("IsScrollAllowed", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let allowed = state.message_frames.get(&id)
            .map(|d| d.scroll_allowed)
            .unwrap_or(true);
        Ok(allowed)
    })?)?;

    Ok(())
}

fn add_message_frame_misc_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetTextCopyable", lua.create_function(|lua, (ud, copyable): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        let data = state.message_frames.entry(id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.text_copyable = copyable;
        Ok(())
    })?)?;

    methods.set("IsTextCopyable", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let copyable = state.message_frames.get(&id)
            .map(|d| d.text_copyable)
            .unwrap_or(false);
        Ok(copyable)
    })?)?;

    methods.set("HasMessageByID", lua.create_function(|lua, (ud, msg_id): (LightUserData, i64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let has = state.message_frames.get(&id)
            .map(|d| d.messages.iter().any(|m| m.message_id == Some(msg_id)))
            .unwrap_or(false);
        Ok(has)
    })?)?;

    // GetMessageInfo(index) - 1-based, returns (text, r, g, b, a, timestamp)
    methods.set("GetMessageInfo", lua.create_function(|lua, (ud, index): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(data) = state.message_frames.get(&id) {
            let idx = (index - 1) as usize;
            if let Some(msg) = data.messages.get(idx) {
                return Ok((msg.text.clone(), msg.r as f64, msg.g as f64, msg.b as f64, msg.a as f64, msg.timestamp));
            }
        }
        Ok((String::new(), 1.0_f64, 1.0_f64, 1.0_f64, 1.0_f64, 0.0_f64))
    })?)?;

    Ok(())
}

fn add_message_frame_callback_stubs(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetOnScrollChangedCallback", lua.create_function(|_, (_ud, _func): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetOnTextCopiedCallback", lua.create_function(|_, (_ud, _func): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetOnLineRightClickedCallback", lua.create_function(|_, (_ud, _func): (LightUserData, Value)| Ok(()))?)?;
    methods.set("AddOnDisplayRefreshedCallback", lua.create_function(|_, (_ud, _func): (LightUserData, Value)| Ok(()))?)?;
    methods.set("RemoveMessagesByPredicate", lua.create_function(|_, (_ud, _func): (LightUserData, Value)| Ok(()))?)?;
    methods.set("TransformMessages", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("AdjustMessageColors", lua.create_function(|_, (_ud, _func): (LightUserData, Value)| Ok(()))?)?;
    methods.set("GetFontStringByID", lua.create_function(|_, (_ud, _id): (LightUserData, i64)| Ok(Value::Nil))?)?;
    methods.set("ResetMessageFadeByID", lua.create_function(|_, (_ud, _id): (LightUserData, i64)| Ok(()))?)?;
    methods.set("ResetAllFadeTimes", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("MarkDisplayDirty", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    Ok(())
}

// --- Helper functions ---

/// Log a MessageFrame message to stderr so chat text is visible in terminal.
fn log_message(state: &SimState, id: u64, text: &str) {
    let name = state.widgets.get(id)
        .and_then(|w| w.name.as_deref())
        .unwrap_or("?");
    let clean = crate::dump::strip_wow_escapes(text);
    eprintln!("[{name}] {clean}");
}

fn add_message_core(state: &mut SimState, id: u64, args: mlua::MultiValue, log: bool) {
    let args_vec: Vec<Value> = args.into_iter().collect();
    let text = match args_vec.first() {
        Some(Value::String(s)) => s.to_string_lossy().to_string(),
        _ => return,
    };
    let (r, g, b, a) = extract_rgba(&args_vec, 1);
    let message_id = match args_vec.get(5) {
        Some(Value::Integer(n)) => Some(*n),
        Some(Value::Number(n)) => Some(*n as i64),
        _ => None,
    };
    if log {
        log_message(state, id, &text);
    }
    let timestamp = state.start_time.elapsed().as_secs_f64();
    let data = state.message_frames.entry(id).or_default();
    insert_message(data, text, r, g, b, a, message_id, timestamp);
    truncate_messages(data);
}

fn backfill_message(state: &mut SimState, id: u64, args: mlua::MultiValue) {
    let args_vec: Vec<Value> = args.into_iter().collect();
    let text = match args_vec.first() {
        Some(Value::String(s)) => s.to_string_lossy().to_string(),
        _ => return,
    };
    let (r, g, b, a) = extract_rgba(&args_vec, 1);
    log_message(state, id, &text);
    let timestamp = state.start_time.elapsed().as_secs_f64();
    let data = state.message_frames.entry(id)
        .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
    data.messages.insert(0, crate::lua_api::message_frame::Message {
        text, r, g, b, a, message_id: None, timestamp,
    });
    if data.messages.len() > data.max_lines {
        data.messages.pop();
    }
}

fn insert_message(
    data: &mut crate::lua_api::message_frame::MessageFrameData,
    text: String, r: f32, g: f32, b: f32, a: f32,
    message_id: Option<i64>, timestamp: f64,
) {
    let msg = crate::lua_api::message_frame::Message {
        text, r, g, b, a, message_id, timestamp,
    };
    if data.insert_mode == "TOP" {
        data.messages.insert(0, msg);
    } else {
        data.messages.push(msg);
    }
}

fn truncate_messages(data: &mut crate::lua_api::message_frame::MessageFrameData) {
    if data.messages.len() > data.max_lines {
        if data.insert_mode == "TOP" {
            data.messages.pop();
        } else {
            data.messages.remove(0);
        }
    }
}

/// Extract RGBA color values from args starting at `offset`.
fn extract_rgba(args: &[Value], offset: usize) -> (f32, f32, f32, f32) {
    let r = val_to_f32_ref(args.get(offset), 1.0);
    let g = val_to_f32_ref(args.get(offset + 1), 1.0);
    let b = val_to_f32_ref(args.get(offset + 2), 1.0);
    let a = val_to_f32_ref(args.get(offset + 3), 1.0);
    (r, g, b, a)
}

/// Extract f32 from a reference to a Lua Value.
fn val_to_f32_ref(val: Option<&Value>, default: f32) -> f32 {
    match val {
        Some(Value::Number(n)) => *n as f32,
        Some(Value::Integer(n)) => *n as f32,
        _ => default,
    }
}
