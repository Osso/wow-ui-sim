//! Chat window presentation + subscription verbs.
//!
//! Migrates chat-window entries off `GLOBAL_NIL_STUBS`:
//!
//! - `SetChatWindowAlpha(index, a)`           — alpha ∈ [0,1]
//! - `SetChatWindowSize(index, fontSize)`     — store chat font size
//! - `SetChatWindowColor(index, r, g, b)`     — RGB ∈ [0,1]
//! - `SetChatWindowLocked(index, locked?)`    — default true when arg omitted
//! - `SetChatWindowUninteractable(i, u?)`     — default true when arg omitted
//! - `AddChatWindowChannel(index, channel)`   — subscribe window to channel
//! - `AddChatWindowMessages(index, group)`    — subscribe window to group
//! - `RemoveChatWindowMessages(index, group)` — unsubscribe window from group
//! - `ChangeChatColor(channel, r, g, b)`      — per-chat-type color override
//! - `GetChatWindowChannels(index)`           — returns a flat list
//! - `GetChatWindowMessages(index)`           — returns a flat list
//!
//! Chat windows are keyed by 1-based chat-frame index (`ChatFrame1` → 1).
//! Windows are lazily created on first touch.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::lua_api::methods::{borrow_state_mut, create_string, create_table};
use crate::lua_api::state::ChatWindow;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_f32(state: &mut LuaState, index: i32) -> Option<f32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as f32),
        _ => None,
    }
}

fn required_string(state: &mut LuaState, index: i32) -> Option<String> {
    Option::<String>::from_stack(state, index)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
}

fn stack_truthy(state: &mut LuaState, index: i32, default: bool) -> bool {
    match stack_val(state, index) {
        Val::Nil => default,
        Val::Bool(b) => b,
        Val::Num(n) => n != 0.0,
        _ => true,
    }
}

fn window_entry<'a>(
    state: &'a mut LuaState,
    index: i32,
) -> LuaResult<std::cell::RefMut<'a, crate::lua_api::SimState>> {
    let mut st = borrow_state_mut(state)?;
    st.chat_windows
        .entry(index)
        .or_insert_with(|| ChatWindow::for_index(index));
    Ok(st)
}

/// `SetChatWindowAlpha(index, alpha)` — alpha clamped to [0, 1].
fn set_chat_window_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let alpha = stack_f32(state, 2).unwrap_or(1.0).clamp(0.0, 1.0);
    let mut st = window_entry(state, index)?;
    if let Some(w) = st.chat_windows.get_mut(&index) {
        w.alpha = alpha;
    }
    Ok(0)
}

/// `SetChatWindowSize(index, fontSize)` — store the chat frame font size.
fn set_chat_window_size(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let font_size = stack_f32(state, 2).unwrap_or(12.0).max(1.0);
    let mut st = window_entry(state, index)?;
    if let Some(w) = st.chat_windows.get_mut(&index) {
        w.font_size = font_size;
    }
    Ok(0)
}

/// `SetChatWindowColor(index, r, g, b)` — channel values clamped to [0, 1].
fn set_chat_window_color(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let r = stack_f32(state, 2).unwrap_or(1.0).clamp(0.0, 1.0);
    let g = stack_f32(state, 3).unwrap_or(1.0).clamp(0.0, 1.0);
    let b = stack_f32(state, 4).unwrap_or(1.0).clamp(0.0, 1.0);
    let mut st = window_entry(state, index)?;
    if let Some(w) = st.chat_windows.get_mut(&index) {
        w.r = r;
        w.g = g;
        w.b = b;
    }
    Ok(0)
}

/// `SetChatWindowLocked(index, locked)` — defaults to true when omitted
/// (matches WoW: calling with no arg treats the window as locked).
fn set_chat_window_locked(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let locked = stack_truthy(state, 2, true);
    let mut st = window_entry(state, index)?;
    if let Some(w) = st.chat_windows.get_mut(&index) {
        w.locked = locked;
    }
    Ok(0)
}

/// `SetChatWindowUninteractable(index, uninteractable)` — defaults to true.
fn set_chat_window_uninteractable(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let uninteractable = stack_truthy(state, 2, true);
    let mut st = window_entry(state, index)?;
    if let Some(w) = st.chat_windows.get_mut(&index) {
        w.uninteractable = uninteractable;
    }
    Ok(0)
}

/// `AddChatWindowChannel(index, channel)` — subscribe window to channel.
/// Idempotent: re-adding a channel is a silent no-op.
fn add_chat_window_channel(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(index), Some(channel)) = (stack_i32(state, 1), required_string(state, 2)) else {
        return Ok(0);
    };
    let mut st = window_entry(state, index)?;
    if let Some(w) = st.chat_windows.get_mut(&index) {
        let is_new_channel = w.channel_names.insert(channel.clone());
        if is_new_channel {
            w.channels.push(channel);
        }
    }
    Ok(0)
}

/// `AddChatWindowMessages(index, group)` — subscribe window to message group.
/// Idempotent: re-adding a group is a silent no-op.
fn add_chat_window_messages(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(index), Some(group)) = (stack_i32(state, 1), required_string(state, 2)) else {
        return Ok(0);
    };
    let mut st = window_entry(state, index)?;
    if let Some(w) = st.chat_windows.get_mut(&index)
        && !w.messages.iter().any(|message| message == &group)
    {
        w.messages.push(group);
    }
    Ok(0)
}

/// `RemoveChatWindowMessages(index, group)` — unsubscribe window from group.
fn remove_chat_window_messages(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(index), Some(group)) = (stack_i32(state, 1), required_string(state, 2)) else {
        return Ok(0);
    };
    let mut st = window_entry(state, index)?;
    if let Some(w) = st.chat_windows.get_mut(&index) {
        w.messages.retain(|message| message != &group);
    }
    Ok(0)
}

/// `ChangeChatColor(channel, r, g, b)` — record per-chat-type color
/// override. Keyed by uppercase channel/type name.
fn change_chat_color(state: &mut LuaState) -> LuaResult<u32> {
    let Some(channel) = required_string(state, 1) else {
        return Ok(0);
    };
    let r = stack_f32(state, 2).unwrap_or(1.0).clamp(0.0, 1.0);
    let g = stack_f32(state, 3).unwrap_or(1.0).clamp(0.0, 1.0);
    let b = stack_f32(state, 4).unwrap_or(1.0).clamp(0.0, 1.0);
    let mut st = borrow_state_mut(state)?;
    st.chat_type_colors
        .insert(channel.to_uppercase(), (r, g, b));
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_chat_window_name(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let Some(name) = Option::<String>::from_stack(state, 2).ok().flatten() else {
        return Ok(0);
    };
    window_entry(state, index)?
        .chat_windows
        .get_mut(&index)
        .unwrap()
        .name = name;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_chat_window_shown(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let shown = matches!(stack_val(state, 2), Val::Bool(true));
    window_entry(state, index)?
        .chat_windows
        .get_mut(&index)
        .unwrap()
        .shown = shown;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_chat_window_docked(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let docked = !matches!(stack_val(state, 2), Val::Nil);
    window_entry(state, index)?
        .chat_windows
        .get_mut(&index)
        .unwrap()
        .docked = docked;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_chat_window_saved_dimensions(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let width = stack_f32(state, 2);
    let height = stack_f32(state, 3);
    let mut sim = window_entry(state, index)?;
    let window = sim.chat_windows.get_mut(&index).unwrap();
    window.saved_width = width;
    window.saved_height = height;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_chat_window_saved_dimensions(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        state.push(Val::Nil);
        state.push(Val::Nil);
        return Ok(2);
    };
    let (width, height) = {
        let sim = borrow_state_mut(state)?;
        let window = sim.chat_windows.get(&index);
        (
            window.and_then(|value| value.saved_width),
            window.and_then(|value| value.saved_height),
        )
    };
    state.push(width.map_or(Val::Nil, |n| Val::Num(n.into())));
    state.push(height.map_or(Val::Nil, |n| Val::Num(n.into())));
    Ok(2)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_chat_window_saved_position(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let point = Option::<String>::from_stack(state, 2).ok().flatten();
    let x_offset = stack_f32(state, 3);
    let y_offset = stack_f32(state, 4);
    let mut sim = window_entry(state, index)?;
    let window = sim.chat_windows.get_mut(&index).unwrap();
    window.saved_point = point;
    window.saved_x_offset = x_offset;
    window.saved_y_offset = y_offset;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_chat_window_saved_position(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        state.push(Val::Nil);
        state.push(Val::Nil);
        state.push(Val::Nil);
        return Ok(3);
    };
    let (point, x_offset, y_offset) = {
        let sim = borrow_state_mut(state)?;
        let window = sim.chat_windows.get(&index);
        (
            window.and_then(|value| value.saved_point.clone()),
            window.and_then(|value| value.saved_x_offset),
            window.and_then(|value| value.saved_y_offset),
        )
    };
    let point_value = point.map_or(Val::Nil, |value| create_string(state, &value));
    state.push(point_value);
    state.push(x_offset.map_or(Val::Nil, |n| Val::Num(n.into())));
    state.push(y_offset.map_or(Val::Nil, |n| Val::Num(n.into())));
    Ok(3)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_chat_window_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(1);
    let window = {
        let sim = borrow_state_mut(state)?;
        sim.chat_windows.get(&index).cloned()
    }
    .unwrap_or_else(|| ChatWindow::for_index(index));
    let name = create_string(state, &window.name);
    state.push(name);
    for value in [window.font_size, window.r, window.g, window.b, window.alpha] {
        state.push(Val::Num(value.into()));
    }
    for value in [
        window.shown,
        window.locked,
        window.docked,
        window.uninteractable,
    ] {
        state.push(Val::Bool(value));
    }
    Ok(10)
}

fn push_string_list(state: &mut LuaState, items: Vec<String>) -> LuaResult<u32> {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return Ok(0);
    };
    for (i, s) in items.into_iter().enumerate() {
        let value = create_string(state, &s);
        if let Some(t) = state.gc.tables.get_mut(table_ref) {
            let _ = t.raw_set(Val::Num((i + 1) as f64), value, &state.gc.string_arena);
        }
    }
    state.gc.barrier_back(table_ref);
    state.push(table);
    Ok(1)
}

/// `GetChatWindowChannels(index)` — return a flat list of channel names.
fn get_chat_window_channels(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        let empty = create_table(state);
        state.push(empty);
        return Ok(1);
    };
    let channels = {
        let st = borrow_state_mut(state)?;
        st.chat_windows
            .get(&index)
            .map(|w| w.channels.clone())
            .unwrap_or_default()
    };
    push_string_list(state, channels)
}

/// `GetChatWindowMessages(index)` — return a flat list of message types.
fn get_chat_window_messages(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        let empty = create_table(state);
        state.push(empty);
        return Ok(1);
    };
    let messages = {
        let st = borrow_state_mut(state)?;
        st.chat_windows
            .get(&index)
            .map(|w| w.messages.clone())
            .unwrap_or_default()
    };
    push_string_list(state, messages)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "SetChatWindowAlpha", set_chat_window_alpha)?;
    LuaApiMut::register_function(lua, "SetChatWindowSize", set_chat_window_size)?;
    LuaApiMut::register_function(lua, "SetChatWindowColor", set_chat_window_color)?;
    LuaApiMut::register_function(lua, "SetChatWindowLocked", set_chat_window_locked)?;
    LuaApiMut::register_function(
        lua,
        "SetChatWindowUninteractable",
        set_chat_window_uninteractable,
    )?;
    LuaApiMut::register_function(lua, "AddChatWindowChannel", add_chat_window_channel)?;
    LuaApiMut::register_function(lua, "AddChatWindowMessages", add_chat_window_messages)?;
    LuaApiMut::register_function(lua, "RemoveChatWindowMessages", remove_chat_window_messages)?;
    LuaApiMut::register_function(lua, "ChangeChatColor", change_chat_color)?;
    LuaApiMut::register_function(lua, "GetChatWindowChannels", get_chat_window_channels)?;
    LuaApiMut::register_function(lua, "GetChatWindowMessages", get_chat_window_messages)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    register_chat_window_layout_verbs(lua)?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn register_chat_window_layout_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetChatWindowInfo", get_chat_window_info)?;
    LuaApiMut::register_function(lua, "SetChatWindowName", set_chat_window_name)?;
    LuaApiMut::register_function(lua, "SetChatWindowShown", set_chat_window_shown)?;
    LuaApiMut::register_function(lua, "SetChatWindowDocked", set_chat_window_docked)?;
    LuaApiMut::register_function(
        lua,
        "GetChatWindowSavedDimensions",
        get_chat_window_saved_dimensions,
    )?;
    LuaApiMut::register_function(
        lua,
        "SetChatWindowSavedDimensions",
        set_chat_window_saved_dimensions,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetChatWindowSavedPosition",
        get_chat_window_saved_position,
    )?;
    LuaApiMut::register_function(
        lua,
        "SetChatWindowSavedPosition",
        set_chat_window_saved_position,
    )?;
    Ok(())
}
