//! `C_GuildInfo.GetClubId` / `IsGuildOfficer` / `CanSpeakInGuildChat` and the
//! Communities Guild Info panel text getters/setters.
//!
//! Backed by `SimState::world`:
//!
//! - `GetClubId()` — returns `world.guild_club_id` string, or nil.
//! - `IsDiscordStreamSeparate()` — false; Discord guild stream separation is not modeled.
//! - `IsGuildOfficer()` — `world.guild_is_officer` (default false).
//! - `CanSpeakInGuildChat()` — `world.guild_can_speak_in_chat` (default true;
//!   retail's "no explicit mute" baseline keeps addons' chat input enabled).
//! - `CanViewOfficerNote()` / `CanEditOfficerNote()` — `world.guild_is_officer`.
//! - `GetMOTD()` / `SetMOTD(text)` — read/write `world.guild_motd`. `SetMOTD`
//!   fires `GUILD_MOTD` so `CommunitiesGuildInfoFrame_OnEvent` repaints.
//! - `GetInfoText()` / `SetInfoText(text)` — read/write `world.guild_info_text`.
//!   `SetInfoText` fires `GUILD_ROSTER_UPDATE` so the panel re-runs
//!   `CommunitiesGuildInfoFrame_UpdateText`.
//! - `GetGuildNewsInfo(index)` — nil while guild news state is unmodeled.
//! - `IsGuildRankAssignmentAllowed(guid, rankOrder)` — true. The sim has no
//!   per-rank authenticator gate, so every rank is freely assignable.
//! - `SetGuildRankOrder(guid, rankOrder)` — no-op. Wired up so the rank radio
//!   menu in `CommunitiesGuildMemberDetail` can fire `:SetSelected()` without
//!   error; per-member rank state isn't yet round-tripped.
//!
//! Admin:
//! - `A_Admin.SetGuildClubId(id?)` — nil / empty clears.
//! - `A_Admin.SetGuildIsOfficer(b?)` / `A_Admin.SetGuildCanSpeakInChat(b?)` —
//!   no-arg defaults to true.
//! - `A_Admin.SetGuildMOTD(text?)` / `A_Admin.SetGuildInfoText(text?)` —
//!   nil clears.
//! - `A_Admin.SetGuildChallenges(rows?)` — replace the challenge list. Each
//!   row is a table with `challengeType`, `current`, `max`, `gold`, `maxGold`.
//!   Pass nil to clear.

use super::guild_verbs::{guild_invite, guild_leave, guild_promote, guild_uninvite};
use crate::event::{Event, EventArg};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, create_table};
use crate::lua_api::state_types::GuildChallenge;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

pub fn get_club_id(state: &mut LuaState) -> LuaResult<u32> {
    let club_id = borrow_state(state)?.world.guild_club_id.clone();
    match club_id {
        Some(id) => {
            let val = create_string(state, &id);
            state.push(val);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub fn is_discord_stream_separate(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

pub fn is_guild_officer(state: &mut LuaState) -> LuaResult<u32> {
    let v = borrow_state(state)?.world.guild_is_officer;
    state.push(Val::Bool(v));
    Ok(1)
}

pub fn can_speak_in_guild_chat(state: &mut LuaState) -> LuaResult<u32> {
    let v = borrow_state(state)?.world.guild_can_speak_in_chat;
    state.push(Val::Bool(v));
    Ok(1)
}

pub fn can_view_officer_note(state: &mut LuaState) -> LuaResult<u32> {
    let v = borrow_state(state)?.world.guild_is_officer;
    state.push(Val::Bool(v));
    Ok(1)
}

pub fn can_edit_officer_note(state: &mut LuaState) -> LuaResult<u32> {
    let v = borrow_state(state)?.world.guild_is_officer;
    state.push(Val::Bool(v));
    Ok(1)
}

pub fn guild_roster(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    if !guild_request_allowed(state, 10.0, false)? {
        return Ok(0);
    }
    borrow_state_mut(state)?.events.push(Event {
        name: "GUILD_ROSTER_UPDATE".to_string(),
        args: Vec::new(),
    });
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn guild_request_allowed(
    state: &mut LuaState,
    interval_seconds: f64,
    is_news_request: bool,
) -> LuaResult<bool> {
    let now = borrow_state(state)?.clock_elapsed().as_secs_f64();
    let mut sim = borrow_state_mut(state)?;
    let last_request = if is_news_request {
        &mut sim.guild_news_last_request
    } else {
        &mut sim.guild_roster_last_request
    };
    let is_allowed = last_request.is_none_or(|last| now - last >= interval_seconds);
    if is_allowed {
        *last_request = Some(now);
    }
    Ok(is_allowed)
}

pub fn get_guild_news_info(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

pub fn is_guild_rank_assignment_allowed(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

pub fn set_guild_rank_order(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub fn get_motd(state: &mut LuaState) -> LuaResult<u32> {
    let motd = borrow_state(state)?.world.guild_motd.clone();
    let val = create_string(state, &motd);
    state.push(val);
    Ok(1)
}

pub fn set_motd(state: &mut LuaState) -> LuaResult<u32> {
    let text = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    {
        let mut st = borrow_state_mut(state)?;
        st.world.guild_motd = text.clone();
        st.events.push(Event {
            name: "GUILD_MOTD".to_string(),
            args: vec![EventArg::String(text)],
        });
    }
    Ok(0)
}

pub fn get_info_text(state: &mut LuaState) -> LuaResult<u32> {
    let text = borrow_state(state)?.world.guild_info_text.clone();
    let val = create_string(state, &text);
    state.push(val);
    Ok(1)
}

pub fn set_info_text(state: &mut LuaState) -> LuaResult<u32> {
    let text = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    {
        let mut st = borrow_state_mut(state)?;
        st.world.guild_info_text = text;
        st.events.push(Event {
            name: "GUILD_ROSTER_UPDATE".to_string(),
            args: Vec::new(),
        });
    }
    Ok(0)
}

pub fn get_num_guild_challenges(state: &mut LuaState) -> LuaResult<u32> {
    let n = borrow_state(state)?.world.guild_challenges.len() as f64;
    state.push(Val::Num(n));
    Ok(1)
}

/// `GetGuildChallengeInfo(orderIndex)` — `orderIndex` is the challenge type id
/// (Blizzard's `GUILD_CHALLENGE_ORDER` maps panel rows → type ids). Returns
/// `(challengeType, current, max, gold, maxGold)` or nil if no challenge of
/// that type is active.
pub fn get_guild_challenge_info(state: &mut LuaState) -> LuaResult<u32> {
    let order_index = i32::from_stack(state, 1)?;
    let challenge = borrow_state(state)?
        .world
        .guild_challenges
        .iter()
        .find(|c| c.challenge_type == order_index)
        .cloned();
    let Some(c) = challenge else {
        state.push(Val::Nil);
        return Ok(1);
    };
    state.push(Val::Num(c.challenge_type as f64));
    state.push(Val::Num(c.current as f64));
    state.push(Val::Num(c.max as f64));
    state.push(Val::Num(c.gold as f64));
    state.push(Val::Num(c.max_gold as f64));
    Ok(5)
}

fn ensure_c_guild_info_table(state: &mut LuaState) -> GcRef<Table> {
    let key = state.gc.intern_string_static(b"C_GuildInfo");
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(r)) = existing {
        return r;
    }
    let new_val = create_table(state);
    let Val::Table(new_ref) = new_val else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), new_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    new_ref
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    let table_ref = ensure_c_guild_info_table(state);
    register_c_guild_info_methods(state, table_ref)?;
    register_guild_challenge_globals(lua)?;
    Ok(())
}

fn register_c_guild_info_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_guild_identity_methods(state, table_ref)?;
    register_guild_permission_methods(state, table_ref)?;
    register_guild_text_methods(state, table_ref)?;
    table_set_rust_fn_static(state, table_ref, "Invite", guild_invite)?;
    table_set_rust_fn_static(state, table_ref, "Uninvite", guild_uninvite)?;
    table_set_rust_fn_static(state, table_ref, "Promote", guild_promote)?;
    table_set_rust_fn_static(state, table_ref, "Leave", guild_leave)?;
    Ok(())
}

fn register_guild_identity_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table_ref, "GetClubId", get_club_id)?;
    table_set_rust_fn_static(state, table_ref, "GuildRoster", guild_roster)?;
    table_set_rust_fn_static(state, table_ref, "GetGuildNewsInfo", get_guild_news_info)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsDiscordStreamSeparate",
        is_discord_stream_separate,
    )?;
    table_set_rust_fn_static(state, table_ref, "IsGuildOfficer", is_guild_officer)?;
    Ok(())
}

fn register_guild_permission_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "CanSpeakInGuildChat",
        can_speak_in_guild_chat,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "CanViewOfficerNote",
        can_view_officer_note,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "CanEditOfficerNote",
        can_edit_officer_note,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsGuildRankAssignmentAllowed",
        is_guild_rank_assignment_allowed,
    )?;
    table_set_rust_fn_static(state, table_ref, "SetGuildRankOrder", set_guild_rank_order)?;
    Ok(())
}

fn register_guild_text_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, table_ref, "GetMOTD", get_motd)?;
    table_set_rust_fn_static(state, table_ref, "SetMOTD", set_motd)?;
    table_set_rust_fn_static(state, table_ref, "GetInfoText", get_info_text)?;
    table_set_rust_fn_static(state, table_ref, "SetInfoText", set_info_text)?;
    Ok(())
}

fn register_guild_challenge_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetNumGuildChallenges", get_num_guild_challenges)?;
    LuaApiMut::register_function(lua, "GetGuildChallengeInfo", get_guild_challenge_info)?;
    Ok(())
}

/// `A_Admin.SetGuildClubId(id?)` — pass nil or empty string to clear.
pub fn admin_set_guild_club_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = Option::<String>::from_stack(state, 1)?.filter(|s| !s.is_empty());
    borrow_state_mut(state)?.world.guild_club_id = id;
    Ok(0)
}

/// `A_Admin.SetGuildIsOfficer(b?)` — no-arg defaults to true.
pub fn admin_set_guild_is_officer(state: &mut LuaState) -> LuaResult<u32> {
    let v = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.world.guild_is_officer = v;
    Ok(0)
}

/// `A_Admin.SetGuildCanSpeakInChat(b?)` — no-arg defaults to true.
pub fn admin_set_guild_can_speak_in_chat(state: &mut LuaState) -> LuaResult<u32> {
    let v = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.world.guild_can_speak_in_chat = v;
    Ok(0)
}

/// `A_Admin.SetGuildMOTD(text?)` — pass nil to clear.
pub fn admin_set_guild_motd(state: &mut LuaState) -> LuaResult<u32> {
    let text = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    borrow_state_mut(state)?.world.guild_motd = text;
    Ok(0)
}

/// `A_Admin.SetGuildInfoText(text?)` — pass nil to clear.
pub fn admin_set_guild_info_text(state: &mut LuaState) -> LuaResult<u32> {
    let text = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    borrow_state_mut(state)?.world.guild_info_text = text;
    Ok(0)
}

/// `A_Admin.SetGuildChallenges(rows?)` — replace `world.guild_challenges`.
/// Each row is a table `{ challengeType, current, max, gold, maxGold }`.
/// Pass nil or an empty table to clear.
pub fn admin_set_guild_challenges(state: &mut LuaState) -> LuaResult<u32> {
    let rows: Vec<GuildChallenge> = match Val::from_stack(state, 1)? {
        Val::Table(tref) => {
            let len = state
                .gc
                .tables
                .get(tref)
                .map(|t| t.array_len())
                .unwrap_or(0);
            let row_refs: Vec<GcRef<Table>> = (1..=len)
                .filter_map(|i| {
                    let val = state.gc.tables.get(tref)?.get_int(i as i64);
                    if let Val::Table(r) = val {
                        Some(r)
                    } else {
                        None
                    }
                })
                .collect();
            row_refs
                .into_iter()
                .map(|row_ref| GuildChallenge {
                    challenge_type: read_field_int(state, row_ref, "challengeType"),
                    current: read_field_int(state, row_ref, "current"),
                    max: read_field_int(state, row_ref, "max"),
                    gold: read_field_int(state, row_ref, "gold"),
                    max_gold: read_field_int(state, row_ref, "maxGold"),
                })
                .collect()
        }
        _ => Vec::new(),
    };
    borrow_state_mut(state)?.world.guild_challenges = rows;
    Ok(0)
}

fn read_field_int(state: &mut LuaState, row_ref: GcRef<Table>, key: &str) -> i32 {
    let key_str = state.gc.intern_string(key.as_bytes());
    let val = state
        .gc
        .tables
        .get(row_ref)
        .map(|t| t.get_str(key_str, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    match val {
        Val::Num(n) => n as i32,
        _ => 0,
    }
}
