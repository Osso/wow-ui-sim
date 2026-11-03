//! SimState-backed chat channel globals and channel metadata probes.
//!
//! Round-trips the player's joined chat channels through the global channel
//! verb family and `C_ChatInfo` (TTS preferences live in `c_api`):
//!
//! - `GetChannelList()` — returns `(id, name, disabled)` triples.
//! - `GetChannelName(channel)` — resolves local channel IDs / names.
//! - `EnumerateServerChannels()` — returns server-managed channel names.
//! - `JoinChannelByName(name)`   — append new channel if missing.
//! - `JoinTemporaryChannel(name)` — alias of `JoinChannelByName`; retail
//!                                   auto-removes these on logout. Not
//!                                   modelled here.
//! - `JoinPermanentChannel(name, password?, frameID?, hasVoice?)` — Forever
//!   local-session membership, chat-window routing, and join notifications.
//! - `ChannelLeave(name)`        — remove channel by name.
//! - `ChannelBan(name, player)`  — add player to channel's banned set,
//!                                   remove from members.
//! - `ChannelInvite(name, p)`    — add player to members (idempotent).
//! - `ChannelKick(name, p)`      — remove from members (no ban).
//! - `ChannelModerator(n, p)`    — add to moderators (player must be in
//!                                   channel first; silent no-op otherwise).
//! - `ChannelUnmoderator(n, p)`  — remove from moderators.
//! - `SwapChatChannelLinks(a, b)` — swap channels at positions a and b.
//! - `C_ChatInfo.GetChannelInfoFromIdentifier(idOrName)` — returns metadata.
//!
//! Channel-number semantics mirror retail: slot 1 = channel #1.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
#[cfg(feature = "client-wowforever")]
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state::ChatChannel;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

const CHANNEL_TYPE_ZONE: f64 = 1.0;
const CHANNEL_TYPE_CUSTOM: f64 = 3.0;

fn required_string(state: &mut LuaState, index: i32) -> Option<String> {
    Option::<String>::from_stack(state, index)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
}

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn channel_identifier(state: &mut LuaState, index: i32) -> Option<String> {
    match stack_val(state, index) {
        Val::Num(n) => Some((n as i32).to_string()),
        _ => required_string(state, index),
    }
}

fn ensure_channel<'a>(
    channels: &'a mut Vec<ChatChannel>,
    name: &str,
) -> Option<&'a mut ChatChannel> {
    channels.iter_mut().find(|c| c.name == name)
}

fn channel_index(channels: &[ChatChannel], name: &str) -> Option<usize> {
    channels.iter().position(|c| c.name == name)
}

fn channel_type(channel_name: &str) -> f64 {
    match channel_name {
        "General" | "Trade" | "LocalDefense" | "LookingForGroup" => CHANNEL_TYPE_ZONE,
        _ => CHANNEL_TYPE_CUSTOM,
    }
}

fn ensure_namespace(state: &mut LuaState, name: &'static str) -> GcRef<Table> {
    let key_ref = state.gc.intern_string_static(name.as_bytes());
    let current = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if let Val::Table(table_ref) = current {
        return table_ref;
    }

    let table_ref = state.gc.alloc_table(Table::new());
    if let Some(globals) = state.gc.tables.get_mut(state.global) {
        let _ = globals.raw_set(
            Val::Str(key_ref),
            Val::Table(table_ref),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(state.global);
    table_ref
}

fn find_channel(channels: &[ChatChannel], identifier: &str) -> Option<(usize, ChatChannel)> {
    if let Ok(local_id) = identifier.parse::<usize>()
        && local_id > 0
    {
        return channels.get(local_id - 1).cloned().map(|ch| (local_id, ch));
    }

    channels
        .iter()
        .position(|ch| ch.name == identifier)
        .map(|index| (index + 1, channels[index].clone()))
}

fn build_channel_info(state: &mut LuaState, local_id: usize, channel: &ChatChannel) -> Val {
    let info = create_table(state);
    let name = create_string(state, &channel.name);
    let shortcut = create_string(state, &local_id.to_string());
    table_set(state, info, "name", name);
    table_set(state, info, "shortcut", shortcut);
    table_set(state, info, "localID", Val::Num(local_id as f64));
    table_set(state, info, "instanceID", Val::Num(0.0));
    table_set(state, info, "zoneChannelID", Val::Num(local_id as f64));
    table_set(
        state,
        info,
        "channelType",
        Val::Num(channel_type(&channel.name)),
    );
    info
}

/// `JoinChannelByName(name)` — append a new channel when missing.
fn join_channel_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    let mut st = borrow_state_mut(state)?;
    if channel_index(&st.chat_channels, &name).is_none() {
        st.chat_channels.push(ChatChannel {
            name,
            ..ChatChannel::default()
        });
    }
    Ok(0)
}

/// The selected Forever SlashCommands.lua and ChannelFrame.lua consume a
/// truthy channel kind plus optional canonical name. The kind agrees with the
/// existing PermanentChatChannelType-backed channel metadata query.
///
/// This local server model accepts the join immediately. Password authentication,
/// voice transport, and rejoining across simulator sessions are not modeled.
#[cfg(feature = "client-wowforever")]
fn join_permanent_channel(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    let frame_id = stack_i32(state, 3).filter(|id| (1..=10).contains(id));
    let (local_id, joined) = {
        let mut sim = borrow_state_mut(state)?;
        let (index, joined) = match channel_index(&sim.chat_channels, &name) {
            Some(index) => (index, false),
            None => {
                let index = sim.chat_channels.len();
                sim.chat_channels.push(ChatChannel {
                    name: name.clone(),
                    ..ChatChannel::default()
                });
                (index, true)
            }
        };
        if let Some(frame_id) = frame_id {
            let window = sim.chat_windows.entry(frame_id).or_default();
            if window.channel_names.insert(name.clone()) {
                window.channels.push(name.clone());
            }
        }
        (index + 1, joined)
    };
    // Commit membership/routing before callbacks query GetChannelName. Event
    // timing/order is an inferred local policy, not a native server recording.
    if joined {
        notify_channel_joined(state, local_id, &name);
        fire_named_event_state(state, "CHANNEL_UI_UPDATE", &[]);
    }
    let canonical_name = create_string(state, &name);
    state.push(Val::Num(channel_type(&name)));
    state.push(canonical_name);
    Ok(2)
}

#[cfg(feature = "client-wowforever")]
fn notify_channel_joined(state: &mut LuaState, local_id: usize, name: &str) {
    // Forever 70707 ChatInfoDocumentation.lua: ChatMsgChannelNotice has 18
    // positional fields. No speaker/Discord message accompanies this notice.
    let reason = create_string(state, "YOU_JOINED");
    let empty = create_string(state, "");
    let display = create_string(state, &format!("{local_id}. {name}"));
    let base_name = create_string(state, name);
    let discord = create_table(state);
    for field in [
        "userID",
        "globalName",
        "lastOnlineGUID",
        "lastOnlineName",
        "forwardedMessage",
    ] {
        table_set(state, discord, field, empty);
    }
    table_set(state, discord, "type", Val::Num(0.0));
    for field in [
        "hasAttachment",
        "hasPoll",
        "hasEmbed",
        "hasSticker",
        "hasEmoji",
        "hasError",
        "hasForwardedMessage",
        "fromDiscord",
    ] {
        table_set(state, discord, field, Val::Bool(false));
    }
    let zone_id = if channel_type(name) == CHANNEL_TYPE_ZONE {
        local_id as f64
    } else {
        0.0
    };
    fire_named_event_state(
        state,
        "CHAT_MSG_CHANNEL_NOTICE",
        &[
            reason,
            empty,
            empty,
            display,
            empty,
            empty,
            Val::Num(zone_id),
            Val::Num(local_id as f64),
            base_name,
            Val::Num(0.0),
            Val::Num(0.0),
            empty,
            Val::Num(0.0),
            Val::Bool(false),
            Val::Bool(false),
            Val::Bool(false),
            Val::Bool(false),
            discord,
        ],
    );
}

fn get_channel_list(state: &mut LuaState) -> LuaResult<u32> {
    let channels = borrow_state(state)?.chat_channels.clone();
    let value_count = channels.len() * 3;
    state.ensure_stack(state.top + value_count);
    for (index, channel) in channels.iter().enumerate() {
        let channel_id = index + 1;
        let name = create_string(state, &channel.name);
        state.push(Val::Num(channel_id as f64));
        state.push(name);
        state.push(Val::Bool(false));
    }
    Ok(value_count as u32)
}

fn get_channel_name(state: &mut LuaState) -> LuaResult<u32> {
    let Some(identifier) = channel_identifier(state, 1) else {
        state.push(Val::Num(0.0));
        state.push(Val::Nil);
        state.push(Val::Num(0.0));
        state.push(Val::Bool(false));
        return Ok(4);
    };
    let channels = borrow_state(state)?.chat_channels.clone();
    match find_channel(&channels, &identifier) {
        Some((local_id, channel)) => {
            let name = create_string(state, &channel.name);
            state.push(Val::Num(local_id as f64));
            state.push(name);
        }
        None => {
            state.push(Val::Num(0.0));
            state.push(Val::Nil);
        }
    }
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
    Ok(4)
}

fn enumerate_server_channels(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_channel_info_from_identifier(state: &mut LuaState) -> LuaResult<u32> {
    let Some(identifier) = channel_identifier(state, 1) else {
        return Ok(0);
    };
    let channels = borrow_state(state)?.chat_channels.clone();
    if let Some((local_id, channel)) = find_channel(&channels, &identifier) {
        let info = build_channel_info(state, local_id, &channel);
        state.push(info);
        return Ok(1);
    }
    Ok(0)
}

fn get_num_active_channels(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.chat_channels.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_channel_shortcut(state: &mut LuaState) -> LuaResult<u32> {
    let channel_id = stack_i32(state, 1).unwrap_or(0);
    let shortcut = create_string(state, &channel_id.to_string());
    state.push(shortcut);
    Ok(1)
}

fn get_general_channel_local_id(state: &mut LuaState) -> LuaResult<u32> {
    let channels = borrow_state(state)?.chat_channels.clone();
    let local_id = channel_index(&channels, "General")
        .map(|index| Val::Num(index as f64 + 1.0))
        .unwrap_or(Val::Nil);
    state.push(local_id);
    Ok(1)
}

fn is_channel_regional(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_regional_service_available(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

/// `JoinTemporaryChannel(name)` — alias. Retail expires these on logout;
/// sim does not model session boundaries so the entry is permanent.
fn join_temporary_channel(state: &mut LuaState) -> LuaResult<u32> {
    join_channel_by_name(state)
}

/// `ChannelLeave(name)` — drop channel by name. Silent no-op when absent.
fn channel_leave(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = required_string(state, 1) else {
        return Ok(0);
    };
    borrow_state_mut(state)?
        .chat_channels
        .retain(|c| c.name != name);
    Ok(0)
}

fn mutate_channel_player(
    state: &mut LuaState,
    mutate: impl FnOnce(&mut ChatChannel, &str),
) -> LuaResult<u32> {
    let (Some(channel), Some(player)) = (required_string(state, 1), required_string(state, 2))
    else {
        return Ok(0);
    };
    let mut st = borrow_state_mut(state)?;
    if let Some(ch) = ensure_channel(&mut st.chat_channels, &channel) {
        mutate(ch, &player);
    }
    Ok(0)
}

/// `ChannelBan(name, player)` — ban a player from a channel, also
/// evicting them if currently a member.
fn channel_ban(state: &mut LuaState) -> LuaResult<u32> {
    mutate_channel_player(state, |ch, player| {
        ch.banned.insert(player.to_string());
        ch.members.remove(player);
        ch.moderators.remove(player);
    })
}

/// `ChannelInvite(name, player)` — add member; idempotent via BTreeSet.
fn channel_invite(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(channel), Some(player)) = (required_string(state, 1), required_string(state, 2))
    else {
        return Ok(0);
    };
    let mut st = borrow_state_mut(state)?;
    if let Some(ch) = ensure_channel(&mut st.chat_channels, &channel)
        && !ch.banned.contains(&player)
    {
        ch.members.insert(player);
    }
    Ok(0)
}

/// `ChannelKick(name, player)` — remove from members (no ban record).
fn channel_kick(state: &mut LuaState) -> LuaResult<u32> {
    mutate_channel_player(state, |ch, player| {
        ch.members.remove(player);
        ch.moderators.remove(player);
    })
}

/// `ChannelModerator(name, player)` — grant moderator; player must be a
/// current member.
fn channel_moderator(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(channel), Some(player)) = (required_string(state, 1), required_string(state, 2))
    else {
        return Ok(0);
    };
    let mut st = borrow_state_mut(state)?;
    if let Some(ch) = ensure_channel(&mut st.chat_channels, &channel)
        && ch.members.contains(&player)
    {
        ch.moderators.insert(player);
    }
    Ok(0)
}

/// `ChannelUnmoderator(name, player)` — revoke moderator.
fn channel_unmoderator(state: &mut LuaState) -> LuaResult<u32> {
    mutate_channel_player(state, |ch, player| {
        ch.moderators.remove(player);
    })
}

/// `SwapChatChannelLinks(a, b)` — swap channels at positions a and b
/// (1-based). Silent no-op for out-of-range indices.
fn swap_chat_channel_links(state: &mut LuaState) -> LuaResult<u32> {
    let (Some(a), Some(b)) = (stack_i32(state, 1), stack_i32(state, 2)) else {
        return Ok(0);
    };
    let Some(a0) = a.checked_sub(1).and_then(|n| usize::try_from(n).ok()) else {
        return Ok(0);
    };
    let Some(b0) = b.checked_sub(1).and_then(|n| usize::try_from(n).ok()) else {
        return Ok(0);
    };
    let mut st = borrow_state_mut(state)?;
    let len = st.chat_channels.len();
    if a0 < len && b0 < len {
        st.chat_channels.swap(a0, b0);
    }
    Ok(0)
}

fn register_channel_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetChannelList", get_channel_list)?;
    LuaApiMut::register_function(lua, "GetChannelName", get_channel_name)?;
    LuaApiMut::register_function(lua, "EnumerateServerChannels", enumerate_server_channels)?;
    LuaApiMut::register_function(lua, "JoinChannelByName", join_channel_by_name)?;
    LuaApiMut::register_function(lua, "JoinTemporaryChannel", join_temporary_channel)?;
    #[cfg(feature = "client-wowforever")]
    LuaApiMut::register_function(lua, "JoinPermanentChannel", join_permanent_channel)?;
    LuaApiMut::register_function(lua, "ChannelLeave", channel_leave)?;
    LuaApiMut::register_function(lua, "ChannelBan", channel_ban)?;
    LuaApiMut::register_function(lua, "ChannelInvite", channel_invite)?;
    LuaApiMut::register_function(lua, "ChannelKick", channel_kick)?;
    LuaApiMut::register_function(lua, "ChannelModerator", channel_moderator)?;
    LuaApiMut::register_function(lua, "ChannelUnmoderator", channel_unmoderator)?;
    LuaApiMut::register_function(lua, "SwapChatChannelLinks", swap_chat_channel_links)?;
    Ok(())
}

fn register_chat_info_namespace(state: &mut LuaState) -> LuaResult<()> {
    let chat_info = ensure_namespace(state, "C_ChatInfo");
    register_chat_info_lookup_functions(state, chat_info)?;
    register_chat_info_regional_functions(state, chat_info)
}

fn register_chat_info_lookup_functions(
    state: &mut LuaState,
    chat_info: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        chat_info,
        "GetChannelInfoFromIdentifier",
        get_channel_info_from_identifier,
    )?;
    table_set_rust_fn_static(
        state,
        chat_info,
        "GetNumActiveChannels",
        get_num_active_channels,
    )?;
    table_set_rust_fn_static(state, chat_info, "GetChannelShortcut", get_channel_shortcut)?;
    table_set_rust_fn_static(
        state,
        chat_info,
        "GetGeneralChannelLocalID",
        get_general_channel_local_id,
    )?;
    table_set_rust_fn_static(
        state,
        chat_info,
        "GetGeneralChannelID",
        get_general_channel_local_id,
    )
}

fn register_chat_info_regional_functions(
    state: &mut LuaState,
    chat_info: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, chat_info, "IsChannelRegional", is_channel_regional)?;
    table_set_rust_fn_static(
        state,
        chat_info,
        "IsChannelRegionalForChannelID",
        is_channel_regional,
    )?;
    table_set_rust_fn_static(
        state,
        chat_info,
        "IsRegionalServiceAvailable",
        is_regional_service_available,
    )
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_channel_globals(lua)?;
    let state = lua.state_mut();
    register_chat_info_namespace(state)?;
    Ok(())
}
