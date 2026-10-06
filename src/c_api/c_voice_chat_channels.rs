//! Host channel snapshots; selectors preserve opaque community identifiers.
//! No voice transport, membership mutation or chat-lockdown secrecy model.

use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Clone)]
pub struct VoiceChannel {
    pub name: String,
    pub channel_id: u32,
    pub channel_type: u32,
    pub club_id: String,
    pub stream_id: String,
    pub volume: f64,
    pub is_active: bool,
    pub is_muted: bool,
    pub is_transmitting: bool,
    pub is_transcribing: bool,
    pub members: Vec<VoiceMember>,
}

#[derive(Debug, Clone)]
pub struct VoiceMember {
    pub energy: f64,
    pub member_id: u32,
    pub is_active: bool,
    pub is_speaking: bool,
    pub is_muted_for_all: bool,
    pub is_silenced: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_VoiceChat")?;
    table_set_rust_fn_static(state, namespace, "GetChannelForChannelType", by_type)?;
    table_set_rust_fn_static(state, namespace, "GetChannelForCommunityStream", by_stream)
}

fn by_type(state: &mut LuaState) -> LuaResult<u32> {
    let kind = u32::from_stack(state, 1)?;
    let channel = borrow_state(state)?
        .voice_channels
        .iter()
        .find(|channel| channel.channel_type == kind)
        .cloned();
    push_channel(state, channel)
}

fn by_stream(state: &mut LuaState) -> LuaResult<u32> {
    let club = String::from_stack(state, 1)?;
    let stream = String::from_stack(state, 2)?;
    let channel = borrow_state(state)?
        .voice_channels
        .iter()
        .find(|channel| channel.club_id == club && channel.stream_id == stream)
        .cloned();
    push_channel(state, channel)
}

fn push_channel(state: &mut LuaState, channel: Option<VoiceChannel>) -> LuaResult<u32> {
    let Some(channel) = channel else { return Ok(0) };
    let row = create_table(state);
    state.push(row);
    write_channel_fields(state, row, &channel);
    write_channel_members(state, row, &channel.members);
    Ok(1)
}

fn write_channel_fields(state: &mut LuaState, row: Val, channel: &VoiceChannel) {
    for (field, value) in [
        ("name", &channel.name),
        ("clubId", &channel.club_id),
        ("streamId", &channel.stream_id),
    ] {
        let value = create_string(state, value);
        table_set(state, row, field, value);
    }
    for (field, value) in [
        ("channelID", f64::from(channel.channel_id)),
        ("channelType", f64::from(channel.channel_type)),
        ("volume", channel.volume),
    ] {
        table_set(state, row, field, Val::Num(value));
    }
    for (field, value) in [
        ("isActive", channel.is_active),
        ("isMuted", channel.is_muted),
        ("isTransmitting", channel.is_transmitting),
        ("isTranscribing", channel.is_transcribing),
    ] {
        table_set(state, row, field, Val::Bool(value));
    }
}

fn write_channel_members(state: &mut LuaState, row: Val, records: &[VoiceMember]) {
    let members = create_table(state);
    table_set(state, row, "members", members);
    let Val::Table(array) = members else {
        unreachable!()
    };
    for (index, member) in records.iter().enumerate() {
        let value = create_table(state);
        table_set_num(state, array, (index + 1) as f64, value);
        write_member(state, value, member);
    }
}

fn write_member(state: &mut LuaState, row: Val, member: &VoiceMember) {
    table_set(state, row, "energy", Val::Num(member.energy));
    table_set(
        state,
        row,
        "memberID",
        Val::Num(f64::from(member.member_id)),
    );
    for (field, value) in [
        ("isActive", member.is_active),
        ("isSpeaking", member.is_speaking),
        ("isMutedForAll", member.is_muted_for_all),
        ("isSilenced", member.is_silenced),
    ] {
        table_set(state, row, field, Val::Bool(value));
    }
}
