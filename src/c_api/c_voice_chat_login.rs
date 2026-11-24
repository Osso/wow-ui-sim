//! Supplied voice-service login state for the loaded Blizzard voice-toggle consumer.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !publishes_login_state() {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_VoiceChat")?;
    table_set_rust_fn_static(state, namespace, "IsLoggedIn", query_logged_in)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "CanPlayerUseVoiceChat",
        query_can_use_voice_chat,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetJoinClubVoiceChannelError",
        get_join_club_voice_channel_error,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsChannelJoinPending",
        is_channel_join_pending,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !publishes_login_state() {
        return Ok(builder);
    }
    builder
        .set_function("SetVoiceChatLoggedIn", set_logged_in)?
        .set_function("SetCanPlayerUseVoiceChat", set_can_use_voice_chat)?
        .set_function(
            "SetJoinClubVoiceChannelError",
            set_join_club_voice_channel_error,
        )?
        .set_function("SetChannelJoinPending", set_channel_join_pending)
}

fn publishes_login_state() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

fn query_logged_in(state: &mut LuaState) -> LuaResult<u32> {
    let logged_in = borrow_state(state)?.voice_chat.logged_in;
    state.push(Val::Bool(logged_in));
    Ok(1)
}

fn query_can_use_voice_chat(state: &mut LuaState) -> LuaResult<u32> {
    let can_use = borrow_state(state)?.voice_chat.can_use_voice_chat;
    state.push(Val::Bool(can_use));
    Ok(1)
}

fn set_can_use_voice_chat(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(can_use) = unwrap_secret(state, stack_val(state, 1))? else {
        return Err(runtime_error(
            "A_Admin.SetCanPlayerUseVoiceChat: expected boolean",
        ));
    };
    borrow_state_mut(state)?.voice_chat.can_use_voice_chat = can_use;
    Ok(0)
}

fn is_channel_join_pending(state: &mut LuaState) -> LuaResult<u32> {
    let key = pending_channel_join_key(state)?;
    let pending = borrow_state(state)?
        .voice_chat
        .pending_channel_joins
        .contains(&key);
    state.push(Val::Bool(pending));
    Ok(1)
}

fn set_channel_join_pending(state: &mut LuaState) -> LuaResult<u32> {
    let key = pending_channel_join_key(state)?;
    let pending = bool::from_stack(state, 4)?;
    let mut sim = borrow_state_mut(state)?;
    if pending {
        sim.voice_chat.pending_channel_joins.insert(key);
    } else {
        sim.voice_chat.pending_channel_joins.remove(&key);
    }
    Ok(0)
}

fn pending_channel_join_key(
    state: &mut LuaState,
) -> LuaResult<(i32, Option<String>, Option<String>)> {
    let channel_type = i32::from_stack(state, 1)?;
    let club_id = optional_string_arg(state, 2)?;
    let stream_id = optional_stream_id_arg(state, 3)?;
    Ok((channel_type, club_id, stream_id))
}

fn optional_string_arg(state: &mut LuaState, index: usize) -> LuaResult<Option<String>> {
    if matches!(stack_val(state, index), Val::Nil) {
        Ok(None)
    } else {
        String::from_stack(state, index).map(Some)
    }
}

fn optional_stream_id_arg(state: &mut LuaState, index: usize) -> LuaResult<Option<String>> {
    if matches!(stack_val(state, index), Val::Nil) {
        Ok(None)
    } else {
        f64::from_stack(state, index).map(|stream_id| Some(stream_id.to_string()))
    }
}

fn get_join_club_voice_channel_error(state: &mut LuaState) -> LuaResult<u32> {
    let club_id = String::from_stack(state, 1)?;
    let reason = borrow_state(state)?
        .voice_chat
        .join_club_voice_channel_errors
        .get(&club_id)
        .copied();
    state.push(reason.map_or(Val::Nil, |reason| Val::Num(f64::from(reason))));
    Ok(1)
}

fn set_join_club_voice_channel_error(state: &mut LuaState) -> LuaResult<u32> {
    let club_id = String::from_stack(state, 1)?;
    let value = unwrap_secret(state, stack_val(state, 2))?;
    let reason = match value {
        Val::Nil => None,
        _ => Some(i32::from_stack(state, 2)?),
    };
    let mut sim = borrow_state_mut(state)?;
    match reason {
        Some(reason) => {
            sim.voice_chat
                .join_club_voice_channel_errors
                .insert(club_id, reason);
        }
        None => {
            sim.voice_chat
                .join_club_voice_channel_errors
                .remove(&club_id);
        }
    }
    Ok(0)
}

fn set_logged_in(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(logged_in) = unwrap_secret(state, stack_val(state, 1))? else {
        return Err(runtime_error(
            "A_Admin.SetVoiceChatLoggedIn: expected boolean",
        ));
    };
    borrow_state_mut(state)?.voice_chat.logged_in = logged_in;
    Ok(0)
}
