//! MessageFrame widget methods: AddMessage, scrolling, fading, message history.

use super::FrameHandle;
use mlua::{UserDataMethods, Value};

pub fn add_message_frame_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_message_frame_add_methods(methods);
    add_message_frame_count_methods(methods);
    add_message_frame_fade_methods(methods);
    add_message_frame_insert_methods(methods);
    add_message_frame_scroll_methods(methods);
    add_message_frame_misc_methods(methods);
    add_message_frame_callback_stubs(methods);
}

fn add_message_frame_add_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // AddMessage(text, r, g, b, messageID, holdTime) - Add message to a MessageFrame
    methods.add_method("AddMessage", |_, this, args: mlua::MultiValue| {
        add_message_impl(this, args);
        Ok(())
    });

    // AddMsg(text, ...) - Alias for AddMessage (used by some addons like DBM)
    methods.add_method("AddMsg", |_, this, args: mlua::MultiValue| {
        add_message_impl(this, args);
        Ok(())
    });

    // BackFillMessage(text, r, g, b, ...) - Add message to back of history
    methods.add_method("BackFillMessage", |_, this, args: mlua::MultiValue| {
        let args_vec: Vec<Value> = args.into_iter().collect();
        let text = match args_vec.first() {
            Some(Value::String(s)) => s.to_string_lossy().to_string(),
            _ => return Ok(()),
        };
        let r = val_to_f32_ref(args_vec.get(1), 1.0);
        let g = val_to_f32_ref(args_vec.get(2), 1.0);
        let b = val_to_f32_ref(args_vec.get(3), 1.0);
        let a = val_to_f32_ref(args_vec.get(4), 1.0);
        log_message(this, &text);
        let mut state = this.state.borrow_mut();
        let timestamp = state.start_time.elapsed().as_secs_f64();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.messages.insert(0, crate::lua_api::message_frame::Message {
            text, r, g, b, a, message_id: None, timestamp,
        });
        if data.messages.len() > data.max_lines {
            data.messages.pop();
        }
        Ok(())
    });

    // Clear() - Clear all messages (overrides tooltip Clear for MessageFrame)
    methods.add_method("Clear", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(data) = state.message_frames.get_mut(&this.id) {
            data.messages.clear();
            data.scroll_offset = 0;
        }
        Ok(())
    });
}

fn add_message_frame_count_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // GetNumMessages()
    methods.add_method("GetNumMessages", |_, this, ()| {
        let state = this.state.borrow();
        let count = state.message_frames.get(&this.id)
            .map(|d| d.messages.len())
            .unwrap_or(0);
        Ok(count as i32)
    });

    // SetMaxLines(maxLines)
    methods.add_method_mut("SetMaxLines", |_, this, max_lines: i32| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.max_lines = max_lines.max(1) as usize;
        data.messages.truncate(data.max_lines);
        Ok(())
    });

    // GetMaxLines()
    methods.add_method("GetMaxLines", |_, this, ()| {
        let state = this.state.borrow();
        let max = state.message_frames.get(&this.id)
            .map(|d| d.max_lines)
            .unwrap_or(120);
        Ok(max as i32)
    });
}

fn add_message_frame_fade_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetFading(fading) - override the stub in methods_core
    methods.add_method("SetFading", |_, this, fading: bool| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fading = fading;
        Ok(())
    });

    // GetFading()
    methods.add_method("GetFading", |_, this, ()| {
        let state = this.state.borrow();
        let fading = state.message_frames.get(&this.id)
            .map(|d| d.fading)
            .unwrap_or(true);
        Ok(fading)
    });

    // SetTimeVisible(secs) - override the stub in methods_core
    methods.add_method("SetTimeVisible", |_, this, secs: f64| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.time_visible = secs;
        Ok(())
    });

    // GetTimeVisible()
    methods.add_method("GetTimeVisible", |_, this, ()| {
        let state = this.state.borrow();
        let secs = state.message_frames.get(&this.id)
            .map(|d| d.time_visible)
            .unwrap_or(10.0);
        Ok(secs)
    });

    add_message_frame_fade_duration_methods(methods);
}

fn add_message_frame_fade_duration_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetFadeDuration(secs) - override the stub in methods_core
    methods.add_method("SetFadeDuration", |_, this, secs: f64| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fade_duration = secs;
        Ok(())
    });

    // GetFadeDuration()
    methods.add_method("GetFadeDuration", |_, this, ()| {
        let state = this.state.borrow();
        let secs = state.message_frames.get(&this.id)
            .map(|d| d.fade_duration)
            .unwrap_or(3.0);
        Ok(secs)
    });

    // SetFadePower(power)
    methods.add_method("SetFadePower", |_, this, power: f64| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.fade_power = power;
        Ok(())
    });

    // GetFadePower()
    methods.add_method("GetFadePower", |_, this, ()| {
        let state = this.state.borrow();
        let power = state.message_frames.get(&this.id)
            .map(|d| d.fade_power)
            .unwrap_or(1.0);
        Ok(power)
    });
}

fn add_message_frame_insert_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetInsertMode(mode) - override the stub in methods_core
    methods.add_method("SetInsertMode", |_, this, mode: String| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.insert_mode = mode;
        Ok(())
    });

    // GetInsertMode()
    methods.add_method("GetInsertMode", |_, this, ()| {
        let state = this.state.borrow();
        let mode = state.message_frames.get(&this.id)
            .map(|d| d.insert_mode.clone())
            .unwrap_or_else(|| "BOTTOM".to_string());
        Ok(mode)
    });
}

fn add_message_frame_scroll_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // Scroll methods (no-ops for now, no visual scrolling)
    methods.add_method("ScrollUp", |_, _this, ()| Ok(()));
    methods.add_method("ScrollDown", |_, _this, ()| Ok(()));
    methods.add_method("PageUp", |_, _this, ()| Ok(()));
    methods.add_method("PageDown", |_, _this, ()| Ok(()));
    methods.add_method("ScrollToTop", |_, _this, ()| Ok(()));
    methods.add_method("ScrollToBottom", |_, _this, ()| Ok(()));

    // AtTop() / AtBottom()
    methods.add_method("AtTop", |_, _this, ()| Ok(true));
    methods.add_method("AtBottom", |_, _this, ()| Ok(true));

    // SetScrollOffset(offset) / GetScrollOffset()
    methods.add_method("SetScrollOffset", |_, this, offset: i32| {
        let mut state = this.state.borrow_mut();
        if let Some(data) = state.message_frames.get_mut(&this.id) {
            data.scroll_offset = offset;
        }
        Ok(())
    });
    methods.add_method("GetScrollOffset", |_, this, ()| {
        let state = this.state.borrow();
        let offset = state.message_frames.get(&this.id)
            .map(|d| d.scroll_offset)
            .unwrap_or(0);
        Ok(offset)
    });

    // GetMaxScrollRange()
    methods.add_method("GetMaxScrollRange", |_, _this, ()| Ok(0_i32));

    // SetScrollAllowed(allowed) / IsScrollAllowed()
    methods.add_method("SetScrollAllowed", |_, this, allowed: bool| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.scroll_allowed = allowed;
        Ok(())
    });
    methods.add_method("IsScrollAllowed", |_, this, ()| {
        let state = this.state.borrow();
        let allowed = state.message_frames.get(&this.id)
            .map(|d| d.scroll_allowed)
            .unwrap_or(true);
        Ok(allowed)
    });
}

fn add_message_frame_misc_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // SetTextCopyable(copyable) / IsTextCopyable()
    methods.add_method("SetTextCopyable", |_, this, copyable: bool| {
        let mut state = this.state.borrow_mut();
        let data = state.message_frames.entry(this.id)
            .or_insert_with(crate::lua_api::message_frame::MessageFrameData::default);
        data.text_copyable = copyable;
        Ok(())
    });
    methods.add_method("IsTextCopyable", |_, this, ()| {
        let state = this.state.borrow();
        let copyable = state.message_frames.get(&this.id)
            .map(|d| d.text_copyable)
            .unwrap_or(false);
        Ok(copyable)
    });

    // HasMessageByID(messageID)
    methods.add_method("HasMessageByID", |_, this, id: i64| {
        let state = this.state.borrow();
        let has = state.message_frames.get(&this.id)
            .map(|d| d.messages.iter().any(|m| m.message_id == Some(id)))
            .unwrap_or(false);
        Ok(has)
    });

    // GetMessageInfo(index) - 1-based, returns (text, r, g, b, a, timestamp)
    methods.add_method("GetMessageInfo", |_, this, index: i32| {
        let state = this.state.borrow();
        if let Some(data) = state.message_frames.get(&this.id) {
            let idx = (index - 1) as usize;
            if let Some(msg) = data.messages.get(idx) {
                return Ok((msg.text.clone(), msg.r as f64, msg.g as f64, msg.b as f64, msg.a as f64, msg.timestamp));
            }
        }
        Ok((String::new(), 1.0_f64, 1.0_f64, 1.0_f64, 1.0_f64, 0.0_f64))
    });
}

fn add_message_frame_callback_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // Callback stubs
    methods.add_method("SetOnScrollChangedCallback", |_, _this, _func: Value| Ok(()));
    methods.add_method("SetOnTextCopiedCallback", |_, _this, _func: Value| Ok(()));
    methods.add_method("SetOnLineRightClickedCallback", |_, _this, _func: Value| Ok(()));
    methods.add_method("AddOnDisplayRefreshedCallback", |_, _this, _func: Value| Ok(()));
    methods.add_method("RemoveMessagesByPredicate", |_, _this, _func: Value| Ok(()));
    methods.add_method("TransformMessages", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("AdjustMessageColors", |_, _this, _func: Value| Ok(()));
    methods.add_method("GetFontStringByID", |_, _this, _id: i64| Ok(Value::Nil));
    methods.add_method("ResetMessageFadeByID", |_, _this, _id: i64| Ok(()));

    // ResetAllFadeTimes() - resets fade timestamps so all messages appear fully visible
    methods.add_method("ResetAllFadeTimes", |_, _this, ()| Ok(()));

    // MarkDisplayDirty() - marks the display as needing a refresh
    methods.add_method("MarkDisplayDirty", |_, _this, ()| Ok(()));
}

// --- Helper functions ---

/// Log a MessageFrame message to stderr so chat text is visible in terminal.
fn log_message(handle: &FrameHandle, text: &str) {
    let state = handle.state.borrow();
    let name = state.widgets.get(handle.id)
        .and_then(|w| w.name.as_deref())
        .unwrap_or("?");
    let clean = crate::dump::strip_wow_escapes(text);
    eprintln!("[{name}] {clean}");
}

/// Shared AddMessage implementation for AddMessage/AddMsg.
fn add_message_impl(this: &FrameHandle, args: mlua::MultiValue) {
    let args_vec: Vec<Value> = args.into_iter().collect();
    let text = match args_vec.first() {
        Some(Value::String(s)) => s.to_string_lossy().to_string(),
        _ => return,
    };
    let r = val_to_f32_ref(args_vec.get(1), 1.0);
    let g = val_to_f32_ref(args_vec.get(2), 1.0);
    let b = val_to_f32_ref(args_vec.get(3), 1.0);
    let a = val_to_f32_ref(args_vec.get(4), 1.0);
    let message_id = match args_vec.get(5) {
        Some(Value::Integer(n)) => Some(*n),
        Some(Value::Number(n)) => Some(*n as i64),
        _ => None,
    };
    log_message(this, &text);
    let mut state = this.state.borrow_mut();
    let timestamp = state.start_time.elapsed().as_secs_f64();
    let data = state.message_frames.entry(this.id)
        .or_default();
    if data.insert_mode == "TOP" {
        data.messages.insert(0, crate::lua_api::message_frame::Message {
            text, r, g, b, a, message_id, timestamp,
        });
    } else {
        data.messages.push(crate::lua_api::message_frame::Message {
            text, r, g, b, a, message_id, timestamp,
        });
    }
    if data.messages.len() > data.max_lines {
        if data.insert_mode == "TOP" {
            data.messages.pop();
        } else {
            data.messages.remove(0);
        }
    }
}

/// Extract f32 from a reference to a Lua Value.
fn val_to_f32_ref(val: Option<&Value>, default: f32) -> f32 {
    match val {
        Some(Value::Number(n)) => *n as f32,
        Some(Value::Integer(n)) => *n as f32,
        _ => default,
    }
}
