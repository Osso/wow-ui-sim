//! Guild club streams/messages and opaque community membership.
pub use super::club_inputs::{receive_member, receive_member_removal};
use super::helpers::{ensure_namespace, set_table_array};
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
use rilua::{LuaResult, Val, runtime_error};

const GUILD_CLUB_ID: &str = "guild-0";
const GUILD_CLUB_CAPACITY: f64 = 1000.0;
const GUILD_STREAM_ID: f64 = 1.0;
const GUILD_STREAM_TYPE: f64 = 1.0;
const OFFICER_STREAM_ID: f64 = 2.0;
const OFFICER_STREAM_TYPE: f64 = 2.0;
const FIRST_MESSAGE_EPOCH: i64 = 1_700_000_000_000_000;
const MESSAGE_EPOCH_STEP: i64 = 120_000_000;

pub(crate) fn register_club_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Club")?;
    super::club_members::register(state)?;
    register_club_status_methods(state, table_ref)?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(
        state,
        table_ref,
        "SendTitleFriendRequest",
        c_club_send_title_friend_request,
    )?;
    #[cfg(feature = "retail-12-0-7")]
    table_set_rust_fn_static(
        state,
        table_ref,
        "SendBattleTagFriendRequest",
        c_club_send_battle_tag_friend_request,
    )?;
    Ok(())
}

/// Records a Battle.net friend request to a guild club member's account, once per
/// member; acceptance is a server decision the simulator does not fabricate.
#[cfg(feature = "retail-12-0-7")]
fn c_club_send_battle_tag_friend_request(state: &mut LuaState) -> LuaResult<u32> {
    let member = super::club_members::guild_member_arg(state)?;
    let name = member
        .filter(|member| !member.is_self)
        .map(|member| member.name);
    let mut sim = borrow_state_mut(state)?;
    if let Some(name) = name
        && !sim.club_battle_tag_friend_requests.contains(&name)
    {
        sim.club_battle_tag_friend_requests.push(name);
    }
    Ok(0)
}

/// Sends an in-game friend request to a guild club member, by their roster name.
#[cfg(feature = "retail-12-1-0")]
fn c_club_send_title_friend_request(state: &mut LuaState) -> LuaResult<u32> {
    let member = super::club_members::guild_member_arg(state)?;
    let name = member
        .filter(|member| !member.is_self)
        .map(|member| member.name);
    let mut sim = borrow_state_mut(state)?;
    if let Some(name) = name {
        crate::c_api::c_battle_net_friend_search::record_title_friend_request(&mut *sim, &name);
    }
    Ok(0)
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
        "GetClubCapacity",
        c_club_get_club_capacity,
    )?;
    table_set_rust_fn_static(state, table_ref, "GetClubLimits", c_club_get_club_limits)?;
    table_set_rust_fn_static(state, table_ref, "IsEnabled", c_club_is_enabled)?;
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
    table_set_rust_fn_static(state, table_ref, "FocusMembers", c_club_focus_members)?;
    Ok(())
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
            let message_info = build_message_info_table(state, message)?;
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
        if let (Some(first), Some(last)) = (messages.first(), messages.last()) {
            let range = build_message_range_table(state, first, last);
            set_table_array(state, ranges, 1, range);
        }
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
        let message_info = build_message_info_table(state, message)?;
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

    super::club_members::sync_guild(state)?;
    let author_member_id = borrow_state(state)?
        .clubs
        .clubs
        .get(GUILD_CLUB_ID)
        .and_then(|club| club.self_member())
        .map(|member| member.id.clone());
    let Some(author_member_id) = author_member_id else {
        return Ok(0);
    };
    let new_index = {
        let mut sim = borrow_state_mut(state)?;
        sim.world.guild_chat_messages.push(GuildChatMessage {
            author_member_id,
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

fn c_club_focus_members(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn c_club_is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
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

#[derive(Clone)]
struct ResolvedMessage {
    id: MessageId,
    author_member_id: String,
    content: String,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct MessageId {
    epoch: i64,
    position: i64,
}

const STATIC_GUILD_MESSAGES: &[(usize, &str)] = &[
    (
        0,
        "Welcome to Heroes of Azeroth. Repairs are open for raid night.",
    ),
    (
        0,
        "Mythic plus keys start after reset. Bring flasks if you have them.",
    ),
    (1, "I put extra feasts and vantus runes in the guild bank."),
    (
        0,
        "Transmog run on Sunday. Invites go out ten minutes early.",
    ),
];

fn resolved_guild_messages(state: &LuaState) -> LuaResult<Vec<ResolvedMessage>> {
    super::club_members::sync_guild(state)?;
    let sim = borrow_state(state)?;
    // INFERRED static conversation fixture binds authors at initial projection,
    // not at each read. Reordering/removal never changes a message's author ID.
    let mut messages: Vec<ResolvedMessage> = STATIC_GUILD_MESSAGES
        .iter()
        .enumerate()
        .filter_map(|(index, (author_index, content))| {
            let id = sim.clubs.guild_seed_author_ids.get(*author_index)?;
            Some(ResolvedMessage {
                id: message_id_at(index),
                author_member_id: id.clone(),
                content: (*content).into(),
            })
        })
        .collect();
    for (index, msg) in sim.world.guild_chat_messages.iter().enumerate() {
        messages.push(ResolvedMessage {
            id: dynamic_message_id(index),
            author_member_id: msg.author_member_id.clone(),
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

fn build_message_info_table(state: &mut LuaState, message: &ResolvedMessage) -> LuaResult<Val> {
    let info = create_table(state);
    let message_id = build_message_id_table(state, message.id);
    let content = create_string(state, &message.content);
    let author = build_message_author_table(state, &message.author_member_id)?;
    table_set(state, info, "messageId", message_id);
    table_set(state, info, "content", content);
    table_set(state, info, "author", author);
    table_set(state, info, "destroyer", Val::Nil);
    table_set(state, info, "destroyed", Val::Bool(false));
    table_set(state, info, "edited", Val::Bool(false));
    Ok(info)
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

fn build_message_author_table(state: &mut LuaState, member_id: &str) -> LuaResult<Val> {
    let member = borrow_state(state)?
        .clubs
        .guild_message_authors
        .get(member_id)
        .cloned()
        .ok_or_else(|| {
            runtime_error(format!(
                "C_Club message: missing author snapshot {member_id}"
            ))
        })?;
    Ok(super::club_members::build_member_info(state, &member, 2))
}
