//! `C_Club` probe surface backed by `WorldState.guild_*` fields.
//!
//! Synthesises one club entry representing the player's guild.
//! No new SimState fields are added — all data is derived from
//! the existing `world.guild_name`, `world.guild_members`, and
//! `world.guild_num_members`.
//!
//! Migrates 4 entries off the namespace stub tables:
//!
//! - `C_Club.GetSubscribedClubs()` — returns a single-element array
//!   containing a `ClubInfo`-like table for the guild when
//!   `world.guild_name` is set, else an empty array.
//! - `C_Club.GetClubMembers(clubId)` — returns an array of member IDs.
//! - `C_Club.GetMemberInfo(clubId, memberId)` — returns a
//!   `ClubMemberInfo`-like table derived from `world.guild_members`.
//! - `C_Club.GetStreams(clubId)` / message history probes — returns synthetic
//!   guild and officer streams, with deterministic generated guild messages.
//! - `C_Club.GetClubCapacity(clubId)` — returns 1000 (hard-coded guild
//!   capacity; retail is unbounded in practice).
//! - `C_Club.IsEnabled()` — returns true unconditionally.
//! - `C_Club.IsRestricted()` — returns `ClubRestrictionReason.None`.

use super::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
    val_to_string,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state_types::character_world::GuildChatMessage;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const GUILD_CLUB_ID: &str = "guild-0";
const GUILD_CLUB_TYPE: f64 = 2.0;
const GUILD_CLUB_CAPACITY: f64 = 1000.0;
const GUILD_STREAM_ID: f64 = 1.0;
const GUILD_STREAM_TYPE: f64 = 1.0;
const OFFICER_STREAM_ID: f64 = 2.0;
const OFFICER_STREAM_TYPE: f64 = 2.0;
const FIRST_MESSAGE_EPOCH: i64 = 1_700_000_000_000_000;
const MESSAGE_EPOCH_STEP: i64 = 120_000_000;

pub(super) fn register_club_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Club")?;
    register_club_lookup_methods(state, table_ref)?;
    register_club_status_methods(state, table_ref)?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(
        state,
        table_ref,
        "SendTitleFriendRequest",
        c_club_send_title_friend_request,
    )?;
    Ok(())
}

/// Sends an in-game friend request to a guild club member, by their roster name.
#[cfg(feature = "retail-12-1-0")]
fn c_club_send_title_friend_request(state: &mut LuaState) -> LuaResult<u32> {
    if !is_guild_club_arg(state) {
        return Ok(0);
    }
    let member_id = i64::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    // Member 1 is the player; befriending yourself is not a request.
    let name = index_from_member_id(member_id)
        .filter(|index| *index != 0)
        .and_then(|index| sim.world.guild_members.get(index))
        .map(|member| member.name.clone());
    if let Some(name) = name {
        crate::c_api::c_battle_net_friend_search::record_title_friend_request(&mut *sim, &name);
    }
    Ok(0)
}

fn register_club_lookup_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetSubscribedClubs",
        c_club_get_subscribed_clubs,
    )?;
    table_set_rust_fn_static(state, table_ref, "GetClubInfo", c_club_get_club_info)?;
    table_set_rust_fn_static(state, table_ref, "GetClubMembers", c_club_get_club_members)?;
    table_set_rust_fn_static(state, table_ref, "GetMemberInfo", c_club_get_member_info)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetMemberInfoForSelf",
        c_club_get_member_info_for_self,
    )?;
    Ok(())
}

fn register_club_status_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_club_privilege_methods(state, table_ref)?;
    register_club_stream_methods(state, table_ref)?;
    register_club_message_methods(state, table_ref)?;
    register_club_readiness_methods(state, table_ref)?;
    Ok(())
}

fn register_club_privilege_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetClubPrivileges",
        c_club_get_club_privileges,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetClubCapacity",
        c_club_get_club_capacity,
    )?;
    table_set_rust_fn_static(state, table_ref, "GetClubLimits", c_club_get_club_limits)?;
    table_set_rust_fn_static(state, table_ref, "IsEnabled", c_club_is_enabled)?;
    table_set_rust_fn_static(state, table_ref, "IsRestricted", c_club_is_restricted)?;
    Ok(())
}

fn register_club_stream_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table_ref, "GetStreams", c_club_get_streams)?;
    table_set_rust_fn_static(state, table_ref, "GetStreamInfo", c_club_get_stream_info)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsSubscribedToStream",
        c_club_is_subscribed_to_stream,
    )?;
    Ok(())
}

fn register_club_message_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table_ref, "GetMessageInfo", c_club_get_message_info)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetMessageRanges",
        c_club_get_message_ranges,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetMessagesBefore",
        c_club_get_messages_before,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "RequestMoreMessagesBefore",
        c_club_request_more_messages_before,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsBeginningOfStream",
        c_club_is_beginning_of_stream,
    )?;
    table_set_rust_fn_static(state, table_ref, "SendMessage", c_club_send_message)?;
    Ok(())
}

fn register_club_readiness_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table_ref, "GetGuildClubId", c_club_get_guild_club_id)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "AreMembersReady",
        c_club_are_members_ready,
    )?;
    table_set_rust_fn_static(state, table_ref, "FocusMembers", c_club_focus_members)?;
    Ok(())
}

fn c_club_get_subscribed_clubs(state: &mut LuaState) -> LuaResult<u32> {
    let guild_name = borrow_state(state)?.world.guild_name.clone();
    let array = create_table(state);
    if let Some(name) = guild_name {
        let entry = build_club_info_table(state, &name);
        set_table_array(state, array, 1, entry);
    }
    state.push(array);
    Ok(1)
}

fn c_club_get_club_info(state: &mut LuaState) -> LuaResult<u32> {
    if !is_guild_club_arg(state) {
        state.push(Val::Nil);
        return Ok(1);
    }
    let guild_name = borrow_state(state)?.world.guild_name.clone();
    match guild_name {
        Some(name) => {
            let info = build_club_info_table(state, &name);
            state.push(info);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_club_get_club_members(state: &mut LuaState) -> LuaResult<u32> {
    if !is_guild_club_arg(state) {
        let empty_members = create_table(state);
        state.push(empty_members);
        return Ok(1);
    }

    let member_count = borrow_state(state)?.world.guild_members.len();
    let array = create_table(state);
    for zero_based_index in 0..member_count {
        let member_id = member_id_from_index(zero_based_index);
        set_table_array(state, array, member_id, Val::Num(member_id as f64));
    }
    state.push(array);
    Ok(1)
}

fn c_club_get_member_info(state: &mut LuaState) -> LuaResult<u32> {
    if !is_guild_club_arg(state) {
        state.push(Val::Nil);
        return Ok(1);
    }

    let member_id = i64::from_stack(state, 2)?;
    let Some(zero_based_index) = index_from_member_id(member_id) else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let member = borrow_state(state)?
        .world
        .guild_members
        .get(zero_based_index)
        .cloned();
    match member {
        Some(member) => {
            let member_info = build_member_info_table(
                state,
                member_id,
                member.rank_index,
                &member.name,
                zero_based_index == 0,
                member.online,
            );
            state.push(member_info);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_club_get_member_info_for_self(state: &mut LuaState) -> LuaResult<u32> {
    if !is_guild_club_arg(state) {
        state.push(Val::Nil);
        return Ok(1);
    }
    let member = {
        let sim = borrow_state(state)?;
        sim.world.guild_name.as_ref().map(|_| {
            sim.world
                .guild_members
                .first()
                .map(|member| (member.rank_index, member.name.clone(), member.online))
                .unwrap_or_else(|| (1, sim.player.name.clone(), true))
        })
    };
    let Some((rank_index, name, online)) = member else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let member_info = build_member_info_table(state, 1, rank_index, &name, true, online);
    state.push(member_info);
    Ok(1)
}

fn c_club_get_club_privileges(state: &mut LuaState) -> LuaResult<u32> {
    let privileges = create_table(state);
    for field in CLUB_PRIVILEGE_FIELDS {
        table_set(state, privileges, field, Val::Bool(false));
    }
    state.push(privileges);
    Ok(1)
}

fn c_club_get_streams(state: &mut LuaState) -> LuaResult<u32> {
    let array = create_table(state);
    if is_guild_club_arg(state) {
        let guild_stream = build_stream_table(
            state,
            GUILD_STREAM_ID,
            GUILD_STREAM_TYPE,
            "Guild",
            "General guild chat",
            false,
        );
        let officer_stream = build_stream_table(
            state,
            OFFICER_STREAM_ID,
            OFFICER_STREAM_TYPE,
            "Officer",
            "Officer chat",
            true,
        );
        set_table_array(state, array, 1, guild_stream);
        set_table_array(state, array, 2, officer_stream);
    }
    state.push(array);
    Ok(1)
}

fn c_club_get_stream_info(state: &mut LuaState) -> LuaResult<u32> {
    let stream = stream_table_from_stack(state).unwrap_or(Val::Nil);
    state.push(stream);
    Ok(1)
}

fn c_club_is_subscribed_to_stream(state: &mut LuaState) -> LuaResult<u32> {
    let subscribed = is_known_stream_arg(state);
    state.push(Val::Bool(subscribed));
    Ok(1)
}

fn c_club_get_message_info(state: &mut LuaState) -> LuaResult<u32> {
    if !is_guild_stream_arg(state) {
        state.push(Val::Nil);
        return Ok(1);
    }

    let Some(message_id) = message_id_from_stack(state, 3) else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let messages = resolved_guild_messages(state)?;
    match messages.iter().find(|message| message.id == message_id) {
        Some(message) => {
            let message_info = build_message_info_table(state, message);
            state.push(message_info);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_club_get_message_ranges(state: &mut LuaState) -> LuaResult<u32> {
    let ranges = create_table(state);
    if is_guild_stream_arg(state) {
        let messages = resolved_guild_messages(state)?;
        let range = build_message_range_table(
            state,
            messages.first().expect("seeded guild messages"),
            messages.last().expect("seeded guild messages"),
        );
        set_table_array(state, ranges, 1, range);
    }
    state.push(ranges);
    Ok(1)
}

fn c_club_get_messages_before(state: &mut LuaState) -> LuaResult<u32> {
    if !is_guild_stream_arg(state) {
        let empty_messages = create_table(state);
        state.push(empty_messages);
        return Ok(1);
    }

    let all_messages = resolved_guild_messages(state)?;
    let newest = message_id_from_stack(state, 3)
        .unwrap_or_else(|| all_messages.last().map(|m| m.id).unwrap_or_default());
    let count = i64::from_stack(state, 4)?.max(0) as usize;
    let messages = messages_before(&all_messages, newest, count);
    let array = create_table(state);
    for (index, message) in messages.iter().enumerate() {
        let message_info = build_message_info_table(state, message);
        set_table_array(state, array, index as i64 + 1, message_info);
    }
    state.push(array);
    Ok(1)
}

fn c_club_request_more_messages_before(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn c_club_is_beginning_of_stream(state: &mut LuaState) -> LuaResult<u32> {
    let message_id = message_id_from_stack(state, 3);
    let messages = resolved_guild_messages(state)?;
    let first_message_id = messages.first().map(|m| m.id).unwrap_or_default();
    state.push(Val::Bool(message_id == Some(first_message_id)));
    Ok(1)
}

fn c_club_send_message(state: &mut LuaState) -> LuaResult<u32> {
    if !is_guild_stream_arg(state) {
        return Ok(0);
    }
    let Ok(text) = String::from_stack(state, 3) else {
        return Ok(0);
    };
    if text.is_empty() {
        return Ok(0);
    }

    let new_index = {
        let mut sim = borrow_state_mut(state)?;
        sim.world.guild_chat_messages.push(GuildChatMessage {
            author_member_id: 1,
            content: text,
        });
        sim.world.guild_chat_messages.len() - 1
    };

    let message_id = dynamic_message_id(new_index);
    let club_id_val = create_string(state, GUILD_CLUB_ID);
    let stream_id_val = Val::Num(GUILD_STREAM_ID);
    let message_id_val = build_message_id_table(state, message_id);

    fire_named_event_state(
        state,
        "CLUB_MESSAGE_ADDED",
        &[club_id_val, stream_id_val, message_id_val],
    );
    Ok(0)
}

fn c_club_get_club_capacity(_state: &mut LuaState) -> LuaResult<u32> {
    _state.push(Val::Num(GUILD_CLUB_CAPACITY));
    Ok(1)
}

fn c_club_get_club_limits(state: &mut LuaState) -> LuaResult<u32> {
    let limits = create_table(state);
    table_set(state, limits, "maximumNumberOfStreams", Val::Num(2.0));
    state.push(limits);
    Ok(1)
}

fn c_club_get_guild_club_id(state: &mut LuaState) -> LuaResult<u32> {
    if borrow_state(state)?.world.guild_name.is_some() {
        let club_id = create_string(state, GUILD_CLUB_ID);
        state.push(club_id);
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}

fn c_club_are_members_ready(state: &mut LuaState) -> LuaResult<u32> {
    let members_ready = is_guild_club_arg(state);
    state.push(Val::Bool(members_ready));
    Ok(1)
}

fn c_club_focus_members(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn c_club_is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_club_is_restricted(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn is_guild_club_arg(state: &mut LuaState) -> bool {
    let value = stack_val(state, 1);
    matches!(value, Val::Str(_))
        && val_to_string(state, value).is_some_and(|club_id| club_id == GUILD_CLUB_ID)
}

fn is_guild_stream_arg(state: &mut LuaState) -> bool {
    is_stream_arg(state, GUILD_STREAM_ID)
}

fn is_officer_stream_arg(state: &mut LuaState) -> bool {
    is_stream_arg(state, OFFICER_STREAM_ID)
}

fn is_known_stream_arg(state: &mut LuaState) -> bool {
    is_guild_stream_arg(state) || is_officer_stream_arg(state)
}

fn is_stream_arg(state: &mut LuaState, stream_id: f64) -> bool {
    is_guild_club_arg(state) && matches!(f64::from_stack(state, 2), Ok(id) if id == stream_id)
}

fn stream_table_from_stack(state: &mut LuaState) -> Option<Val> {
    if is_guild_stream_arg(state) {
        Some(build_stream_table(
            state,
            GUILD_STREAM_ID,
            GUILD_STREAM_TYPE,
            "Guild",
            "General guild chat",
            false,
        ))
    } else if is_officer_stream_arg(state) {
        Some(build_stream_table(
            state,
            OFFICER_STREAM_ID,
            OFFICER_STREAM_TYPE,
            "Officer",
            "Officer chat",
            true,
        ))
    } else {
        None
    }
}

fn member_id_from_index(zero_based_index: usize) -> i64 {
    zero_based_index as i64 + 1
}

fn index_from_member_id(member_id: i64) -> Option<usize> {
    member_id
        .checked_sub(1)
        .and_then(|zero_based| usize::try_from(zero_based).ok())
}

#[derive(Clone)]
struct ResolvedMessage {
    id: MessageId,
    author_member_id: i64,
    content: String,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct MessageId {
    epoch: i64,
    position: i64,
}

const STATIC_GUILD_MESSAGES: &[(i64, &str)] = &[
    (
        1,
        "Welcome to Heroes of Azeroth. Repairs are open for raid night.",
    ),
    (
        1,
        "Mythic plus keys start after reset. Bring flasks if you have them.",
    ),
    (2, "I put extra feasts and vantus runes in the guild bank."),
    (
        1,
        "Transmog run on Sunday. Invites go out ten minutes early.",
    ),
];

fn resolved_guild_messages(state: &LuaState) -> LuaResult<Vec<ResolvedMessage>> {
    let mut messages: Vec<ResolvedMessage> = STATIC_GUILD_MESSAGES
        .iter()
        .enumerate()
        .map(|(index, (author, content))| ResolvedMessage {
            id: message_id_at(index),
            author_member_id: *author,
            content: (*content).to_string(),
        })
        .collect();

    let sim = borrow_state(state)?;
    for (index, msg) in sim.world.guild_chat_messages.iter().enumerate() {
        messages.push(ResolvedMessage {
            id: dynamic_message_id(index),
            author_member_id: msg.author_member_id,
            content: msg.content.clone(),
        });
    }
    Ok(messages)
}

fn message_id_at(absolute_index: usize) -> MessageId {
    let index = absolute_index as i64;
    MessageId {
        epoch: FIRST_MESSAGE_EPOCH + index * MESSAGE_EPOCH_STEP,
        position: index + 1,
    }
}

fn dynamic_message_id(dynamic_index: usize) -> MessageId {
    message_id_at(STATIC_GUILD_MESSAGES.len() + dynamic_index)
}

fn messages_before(
    all_messages: &[ResolvedMessage],
    newest: MessageId,
    count: usize,
) -> Vec<ResolvedMessage> {
    let mut filtered: Vec<ResolvedMessage> = all_messages
        .iter()
        .filter(|message| message.id.epoch <= newest.epoch)
        .cloned()
        .collect();
    let keep_from = filtered.len().saturating_sub(count);
    filtered.drain(..keep_from);
    filtered
}

fn message_id_from_stack(state: &mut LuaState, index: i32) -> Option<MessageId> {
    let table = stack_val(state, index);
    Some(MessageId {
        epoch: table_get_i64(state, table, "epoch")?,
        position: table_get_i64(state, table, "position")?,
    })
}

fn table_get_i64(state: &mut LuaState, table: Val, key: &str) -> Option<i64> {
    match table_get(state, table, key) {
        Val::Num(value) => Some(value as i64),
        _ => None,
    }
}

fn build_club_info_table(state: &mut LuaState, name: &str) -> Val {
    let t = create_table(state);
    let club_id = create_string(state, GUILD_CLUB_ID);
    let name_str = create_string(state, name);
    table_set(state, t, "clubId", club_id);
    table_set(state, t, "clubType", Val::Num(GUILD_CLUB_TYPE));
    table_set(state, t, "name", name_str);
    let desc = create_string(state, "");
    let broadcast = create_string(state, "");
    table_set(state, t, "description", desc);
    table_set(state, t, "broadcast", broadcast);
    table_set(state, t, "avatarId", Val::Num(0.0));
    table_set(state, t, "memberCount", Val::Num(GUILD_CLUB_CAPACITY));
    t
}

fn build_stream_table(
    state: &mut LuaState,
    stream_id: f64,
    stream_type: f64,
    stream_name: &str,
    stream_subject: &str,
    leaders_and_moderators_only: bool,
) -> Val {
    let stream = create_table(state);
    let name = create_string(state, stream_name);
    let subject = create_string(state, stream_subject);
    table_set(state, stream, "streamId", Val::Num(stream_id));
    table_set(state, stream, "name", name);
    table_set(state, stream, "subject", subject);
    table_set(
        state,
        stream,
        "leadersAndModeratorsOnly",
        Val::Bool(leaders_and_moderators_only),
    );
    table_set(state, stream, "streamType", Val::Num(stream_type));
    table_set(
        state,
        stream,
        "creationTime",
        Val::Num(FIRST_MESSAGE_EPOCH as f64),
    );
    stream
}

fn build_member_info_table(
    state: &mut LuaState,
    member_id: i64,
    rank_order: i32,
    name: &str,
    is_self: bool,
    online: bool,
) -> Val {
    let t = create_table(state);
    let name_str = create_string(state, name);
    let guid = create_string(state, &format!("member-{member_id}"));
    table_set(state, t, "memberId", Val::Num(member_id as f64));
    table_set(state, t, "guid", guid);
    table_set(state, t, "name", name_str);
    table_set(state, t, "isSelf", Val::Bool(is_self));
    table_set(state, t, "guildRankOrder", Val::Num(rank_order as f64));
    table_set(state, t, "role", Val::Num(4.0));
    // ClubMemberPresence: 1 = Online, 3 = Offline.
    table_set(
        state,
        t,
        "presence",
        Val::Num(if online { 1.0 } else { 3.0 }),
    );
    t
}

fn build_message_range_table(
    state: &mut LuaState,
    oldest: &ResolvedMessage,
    newest: &ResolvedMessage,
) -> Val {
    let range = create_table(state);
    let oldest_id = build_message_id_table(state, oldest.id);
    let newest_id = build_message_id_table(state, newest.id);
    table_set(state, range, "oldestMessageId", oldest_id);
    table_set(state, range, "newestMessageId", newest_id);
    range
}

fn build_message_info_table(state: &mut LuaState, message: &ResolvedMessage) -> Val {
    let info = create_table(state);
    let message_id = build_message_id_table(state, message.id);
    let content = create_string(state, &message.content);
    let author = build_message_author_table(state, message.author_member_id);
    table_set(state, info, "messageId", message_id);
    table_set(state, info, "content", content);
    table_set(state, info, "author", author);
    table_set(state, info, "destroyer", Val::Nil);
    table_set(state, info, "destroyed", Val::Bool(false));
    table_set(state, info, "edited", Val::Bool(false));
    info
}

fn build_message_id_table(state: &mut LuaState, message_id: MessageId) -> Val {
    let table = create_table(state);
    table_set(state, table, "epoch", Val::Num(message_id.epoch as f64));
    table_set(
        state,
        table,
        "position",
        Val::Num(message_id.position as f64),
    );
    table
}

fn build_message_author_table(state: &mut LuaState, member_id: i64) -> Val {
    let zero_based_index = index_from_member_id(member_id).unwrap_or(0);
    let member = borrow_state(state)
        .ok()
        .and_then(|sim| sim.world.guild_members.get(zero_based_index).cloned());

    match member {
        Some(member) => build_member_info_table(
            state,
            member_id,
            member.rank_index,
            &member.name,
            zero_based_index == 0,
            member.online,
        ),
        None => build_member_info_table(state, member_id, 1, "Unknown", false, false),
    }
}

const CLUB_PRIVILEGE_FIELDS: &[&str] = &[
    "canDestroy",
    "canSetName",
    "canSetDescription",
    "canSetAvatar",
    "canSetBroadcast",
    "canSetPrivacyLevel",
    "canSetOwnMemberAttribute",
    "canSetOtherMemberAttribute",
    "canSetOwnMemberNote",
    "canSetOtherMemberNote",
    "canSetOwnVoiceState",
    "canSetOwnPresenceLevel",
    "canUseVoice",
    "canVoiceMuteMemberForAll",
    "canGetInvitation",
    "canSendInvitation",
    "canCreateStream",
    "canDestroyStream",
    "canSetStreamName",
    "canSetStreamSubject",
    "canSetStreamAccess",
    "canSetStreamVoiceLevel",
    "canCreateTicket",
];
