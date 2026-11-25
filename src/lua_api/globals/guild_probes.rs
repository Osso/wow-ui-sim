//! Guild-state probe globals.
//!
//! Migrates 4 entries off `GLOBAL_FALSE_STUBS`:
//!
//! - `IsInGuild()`                 -> `world.guild_name.is_some()`
//! - `CanReplaceGuildMaster()`     -> `SimState.can_replace_guild_master`
//! - `GetAutoDeclineGuildInvites()` -> `SimState.auto_decline_guild_invites`
//! - `GetGuildRosterShowOffline()` -> `SimState.guild_roster_show_offline`
//! - `CanGuildInvite()`             -> true when the player is in a guild
//! - `CanGuildRemove()`             -> true when the player is in a guild
//! - `CanEditPublicNote()`          -> true when the player is in a guild
//! - `CanEditMOTD()`                -> `world.guild_is_officer` when in a guild
//! - `CanEditGuildInfo()` -> Wrath number1/nil from player guild/rank rights;
//!   other profiles retain the membership/officer probe.
//! - `IsGuildLeader()`              -> `guild_members[0].rank_index == 1` (the
//!   "self is first guild member" hack mirrors `c_club_get_member_info_for_self`).
//! - `CanGuildPromote()` / `CanGuildDemote()` -> true when the player is the
//!   Guild Leader or an Officer (rank_index <= 2 on the simplified model).
//!   `GuildRoster.lua`'s `SetupRankDropdown` calls these unconditionally; if
//!   they're nil the rank radio menu errors out and stays empty.
//! - `QueryGuildRecipes()`          -> no-op, guild recipe state is unmodeled
//! - `CanViewGuildRecipes()`        -> false, guild recipe state is unmodeled
//! - `QueryGuildNews()`             -> fires `GUILD_NEWS_UPDATE` so the
//!   Communities panel calls `_Update` and seeds at least the MOTD row.
//! - `GuildNewsSort()`              -> fires `GUILD_NEWS_UPDATE` for the same
//!   reason. Real WoW sorts the news list; the sim has no records to sort.
//!
//! Migrates 4 roster-count entries off `GLOBAL_ZERO_STUBS` and supplies the
//! empty guild news count:
//!
//! - `GetNumGuildMembers()`   -> `world.guild_members.len()`
//! - `GetGuildRosterSize()`   -> `world.guild_members.len()`
//! - `GetGuildRosterInfo(i)`  -> synth row from `guild_members[i-1]` +
//!   `guild_ranks[rank_index-1]`
//! - `GetGuildRosterMOTD()`   -> `world.guild_motd`
//! - `GetNumGuildNews()`      -> 0, guild news state is unmodeled

use crate::lua_api::game_data::CLASS_LABELS;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, create_string, create_string_static};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const CLASS_FILES: &[&str] = &[
    "WARRIOR",
    "PALADIN",
    "HUNTER",
    "ROGUE",
    "PRIEST",
    "DEATHKNIGHT",
    "SHAMAN",
    "MAGE",
    "WARLOCK",
    "MONK",
    "DRUID",
    "DEMONHUNTER",
    "EVOKER",
];

fn player_class_info(class_index: i32) -> (&'static str, &'static str) {
    let idx = class_index
        .max(1)
        .min(CLASS_LABELS.len() as i32)
        .saturating_sub(1) as usize;
    (CLASS_LABELS[idx], CLASS_FILES[idx])
}

/// INFERRED (R88ce): Quick Join supplies GUIDs, so membership uses exact roster GUIDs.
fn is_guild_member(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let found = borrow_state(state)?
        .world
        .guild_members
        .iter()
        .any(|member| member.guid.as_deref() == Some(guid.as_str()));
    state.push(Val::Bool(found));
    Ok(1)
}

fn is_in_guild(state: &mut LuaState) -> LuaResult<u32> {
    let b = borrow_state(state)?.world.guild_name.is_some();
    state.push(Val::Bool(b));
    Ok(1)
}

fn can_replace_guild_master(state: &mut LuaState) -> LuaResult<u32> {
    let b = borrow_state(state)?.can_replace_guild_master;
    state.push(Val::Bool(b));
    Ok(1)
}

fn get_auto_decline_guild_invites(state: &mut LuaState) -> LuaResult<u32> {
    let b = borrow_state(state)?.auto_decline_guild_invites;
    state.push(Val::Bool(b));
    Ok(1)
}

fn get_guild_roster_show_offline(state: &mut LuaState) -> LuaResult<u32> {
    let b = borrow_state(state)?.guild_roster_show_offline;
    state.push(Val::Bool(b));
    Ok(1)
}

fn can_guild_invite(state: &mut LuaState) -> LuaResult<u32> {
    let b = borrow_state(state)?.world.guild_name.is_some();
    state.push(Val::Bool(b));
    Ok(1)
}

fn can_guild_remove(state: &mut LuaState) -> LuaResult<u32> {
    let b = borrow_state(state)?.world.guild_name.is_some();
    state.push(Val::Bool(b));
    Ok(1)
}

fn can_edit_public_note(state: &mut LuaState) -> LuaResult<u32> {
    let b = borrow_state(state)?.world.guild_name.is_some();
    state.push(Val::Bool(b));
    Ok(1)
}

fn can_edit_guild_details(state: &mut LuaState) -> LuaResult<bool> {
    let world = &borrow_state(state)?.world;
    Ok(world.guild_name.is_some() && world.guild_is_officer)
}

fn can_edit_motd(state: &mut LuaState) -> LuaResult<u32> {
    let b = can_edit_guild_details(state)?;
    state.push(Val::Bool(b));
    Ok(1)
}

fn can_edit_guild_info(state: &mut LuaState) -> LuaResult<u32> {
    if can_edit_guild_details(state)? {
        state.push(Val::Bool(true));
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}

fn self_rank_index(state: &mut LuaState) -> LuaResult<Option<i32>> {
    let world = &borrow_state(state)?.world;
    if world.guild_name.is_none() {
        return Ok(None);
    }
    Ok(world.guild_members.first().map(|m| m.rank_index))
}

fn is_guild_leader(state: &mut LuaState) -> LuaResult<u32> {
    let b = self_rank_index(state)?.is_some_and(|r| r == 1);
    state.push(Val::Bool(b));
    Ok(1)
}

fn can_guild_promote(state: &mut LuaState) -> LuaResult<u32> {
    let b = self_rank_index(state)?.is_some_and(|r| r <= 2);
    state.push(Val::Bool(b));
    Ok(1)
}

fn can_guild_demote(state: &mut LuaState) -> LuaResult<u32> {
    let b = self_rank_index(state)?.is_some_and(|r| r <= 2);
    state.push(Val::Bool(b));
    Ok(1)
}

fn query_guild_recipes(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

/// `QueryGuildEventLog()` — retail asks the server for the guild event log
/// and fires `GUILD_EVENT_LOG_UPDATE` when it arrives. The sim has the log
/// in-memory, so dispatch the event synchronously to drive
/// `CommunitiesGuildLogFrame_Update`.
fn query_guild_event_log(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "GUILD_EVENT_LOG_UPDATE", &[])?;
    Ok(0)
}

fn get_num_guild_events(state: &mut LuaState) -> LuaResult<u32> {
    let n = borrow_state(state)?.world.guild_events.len() as f64;
    state.push(Val::Num(n));
    Ok(1)
}

/// `GetGuildEventInfo(index)` — returns
/// `(type, player1, player2, rank, year, month, day, hour)`. Out-of-range
/// indices push 8 nils.
fn get_guild_event_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let zero_based = match usize::try_from(index.saturating_sub(1)) {
        Ok(i) => i,
        Err(_) => return push_nil_event_row(state),
    };
    let event = {
        let st = borrow_state(state)?;
        st.world.guild_events.get(zero_based).cloned()
    };
    let Some(event) = event else {
        return push_nil_event_row(state);
    };
    let event_type = create_string(state, &event.event_type);
    let player1 = create_string(state, &event.player1);
    state.push(event_type);
    state.push(player1);
    match event.player2 {
        Some(name) => {
            let v = create_string(state, &name);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
    match event.rank_name {
        Some(rank) => {
            let v = create_string(state, &rank);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
    state.push(Val::Num(event.year as f64));
    state.push(Val::Num(event.month as f64));
    state.push(Val::Num(event.day as f64));
    state.push(Val::Num(event.hour as f64));
    Ok(8)
}

fn push_nil_event_row(state: &mut LuaState) -> LuaResult<u32> {
    for _ in 0..8 {
        state.push(Val::Nil);
    }
    Ok(8)
}

fn query_guild_news(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    if !super::guild_info::guild_request_allowed(state, 900.0, true)? {
        return Ok(0);
    }
    dispatch_event_now(state, "GUILD_NEWS_UPDATE", &[])?;
    Ok(0)
}

/// `GuildNewsSort(sortMode)` — sorts the guild news list and fires
/// `GUILD_NEWS_UPDATE` so panels re-populate. The sim has no guild news
/// records so there's nothing to sort, but the event fire is what
/// `CommunitiesGuildNewsFrame_OnShow` relies on to call `_Update` and seed
/// at least the MOTD entry into the data provider.
fn guild_news_sort(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "GUILD_NEWS_UPDATE", &[])?;
    Ok(0)
}

fn can_view_guild_recipes(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// `GetNumGuildMembers()` — retail returns (total, online, onlineAndMobile).
fn get_num_guild_members(state: &mut LuaState) -> LuaResult<u32> {
    let (total, online) = {
        let sim = borrow_state(state)?;
        let total = sim.world.guild_members.len() as f64;
        let online = sim
            .world
            .guild_members
            .iter()
            .filter(|member| member.online)
            .count() as f64;
        (total, online)
    };
    state.push(Val::Num(total));
    state.push(Val::Num(online));
    state.push(Val::Num(0.0));
    Ok(3)
}

/// `GetGuildRosterSize()` — single-value variant. Retail returns the total
/// roster size (including offline members).
fn get_guild_roster_size(state: &mut LuaState) -> LuaResult<u32> {
    let n = borrow_state(state)?.world.guild_members.len() as f64;
    state.push(Val::Num(n));
    Ok(1)
}

fn get_num_guild_news(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_num_guild_perks(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.world.guild_perks.len() as f64;
    state.push(Val::Num(count));
    Ok(1)
}

/// INFERRED R88g3 (native evidence pending): returns perk name, spell ID and icon.
fn get_guild_perk_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let perk = borrow_state(state)?
        .world
        .guild_perks
        .get(index.saturating_sub(1).max(0) as usize)
        .copied();
    let Some(perk) = perk.filter(|_| index > 0) else {
        for _ in 0..3 {
            state.push(Val::Nil);
        }
        return Ok(3);
    };
    let Some(spell) = crate::spells::get_spell(perk.spell_id) else {
        for _ in 0..3 {
            state.push(Val::Nil);
        }
        return Ok(3);
    };
    let name = create_string(state, spell.name);
    state.push(name);
    state.push(Val::Num(f64::from(perk.spell_id)));
    state.push(Val::Num(f64::from(spell.icon_file_data_id)));
    Ok(3)
}

/// `GetGuildRosterMOTD()` — guild Message of the Day, empty string when unset.
fn get_guild_roster_motd(state: &mut LuaState) -> LuaResult<u32> {
    let motd = borrow_state(state)?.world.guild_motd.clone();
    let val = create_string(state, &motd);
    state.push(val);
    Ok(1)
}

/// `GetGuildRosterInfo(index)` — Retail/Forever include the roster GUID at return 17.
/// The sim synthesizes the remaining fields from the player's level/class and defaults.
fn get_guild_roster_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let Ok(zero_based) = usize::try_from(index.saturating_sub(1)) else {
        return push_nil_roster_row(state);
    };
    let maybe_row = {
        let st = borrow_state(state)?;
        build_roster_row(&st, zero_based)
    };
    let Some(row) = maybe_row else {
        return push_nil_roster_row(state);
    };

    push_roster_row_values(state, &row);
    Ok(guild_roster_info_field_count())
}

fn guild_roster_info_field_count() -> u32 {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail
        | crate::client_profile::ClientProfile::WowForever => 17,
        _ => 16,
    }
}

fn push_roster_row_values(state: &mut LuaState, row: &RosterRow) {
    let name = create_string(state, &row.name);
    let rank = create_string(state, &row.rank_name);
    let class_label = create_string_static(state, row.class_label);
    let class_file = create_string_static(state, row.class_file);
    let empty = create_string_static(state, "");

    state.push(name); // 1: name
    state.push(rank); // 2: rankName
    state.push(Val::Num(row.rank_index)); // 3: rankIndex
    state.push(Val::Num(row.level)); // 4: level
    state.push(class_label); // 5: class
    state.push(empty); // 6: zone
    state.push(empty); // 7: note
    state.push(empty); // 8: officernote
    state.push(Val::Bool(row.online)); // 9: online
    state.push(Val::Num(0.0)); // 10: status
    state.push(class_file); // 11: classFileName
    state.push(Val::Num(0.0)); // 12: achievementPoints
    state.push(Val::Num(0.0)); // 13: achievementRank
    state.push(Val::Bool(false)); // 14: isMobile
    state.push(Val::Bool(false)); // 15: isSoREligible
    state.push(Val::Num(0.0)); // 16: standingID
    push_roster_guid(state, row);
}

fn push_roster_guid(state: &mut LuaState, row: &RosterRow) {
    if guild_roster_info_field_count() != 17 {
        return;
    }
    let guid = row
        .guid
        .as_deref()
        .map(|guid| create_string(state, guid))
        .unwrap_or(Val::Nil);
    state.push(guid);
}

struct RosterRow {
    name: String,
    rank_name: String,
    rank_index: f64,
    level: f64,
    class_label: &'static str,
    class_file: &'static str,
    online: bool,
    guid: Option<String>,
}

fn build_roster_row(st: &crate::lua_api::state::SimState, zero_based: usize) -> Option<RosterRow> {
    let member = st.world.guild_members.get(zero_based)?;
    let rank_name = st
        .world
        .guild_ranks
        .get((member.rank_index.saturating_sub(1)).max(0) as usize)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| "Member".into());
    let (class_label, class_file) = player_class_info(st.player.class_index);
    Some(RosterRow {
        name: member.name.clone(),
        rank_name,
        rank_index: member.rank_index as f64 - 1.0,
        level: st.player.level as f64,
        class_label,
        class_file,
        online: member.online,
        guid: member.guid.clone(),
    })
}

fn push_nil_roster_row(state: &mut LuaState) -> LuaResult<u32> {
    for _ in 0..guild_roster_info_field_count() {
        state.push(Val::Nil);
    }
    Ok(guild_roster_info_field_count())
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_guild_membership_queries(lua)?;
    register_guild_permission_queries(lua)?;
    register_guild_event_queries(lua)?;
    register_guild_roster_queries(lua)?;
    Ok(())
}

fn register_guild_membership_queries(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "IsInGuild", is_in_guild)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "GetNumGuildPerks", get_num_guild_perks)?;
        LuaApiMut::register_function(lua, "GetGuildPerkInfo", get_guild_perk_info)?;
    }
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "IsGuildMember", is_guild_member)?;
    }
    LuaApiMut::register_function(lua, "CanReplaceGuildMaster", can_replace_guild_master)?;
    LuaApiMut::register_function(
        lua,
        "GetAutoDeclineGuildInvites",
        get_auto_decline_guild_invites,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetGuildRosterShowOffline",
        get_guild_roster_show_offline,
    )?;
    Ok(())
}

fn register_guild_permission_queries(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "CanGuildInvite", can_guild_invite)?;
    LuaApiMut::register_function(lua, "CanGuildRemove", can_guild_remove)?;
    LuaApiMut::register_function(lua, "CanEditPublicNote", can_edit_public_note)?;
    LuaApiMut::register_function(lua, "CanEditMOTD", can_edit_motd)?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        super::real::wrath_guild_info::register_all(lua)?;
        super::real::wrath_guild_bank::register_all(lua)?;
    } else {
        LuaApiMut::register_function(lua, "CanEditGuildInfo", can_edit_guild_info)?;
    }
    LuaApiMut::register_function(lua, "CanGuildPromote", can_guild_promote)?;
    LuaApiMut::register_function(lua, "CanGuildDemote", can_guild_demote)?;
    LuaApiMut::register_function(lua, "IsGuildLeader", is_guild_leader)?;
    LuaApiMut::register_function(lua, "QueryGuildRecipes", query_guild_recipes)?;
    LuaApiMut::register_function(lua, "CanViewGuildRecipes", can_view_guild_recipes)?;
    Ok(())
}

fn register_guild_event_queries(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "QueryGuildNews", query_guild_news)?;
    LuaApiMut::register_function(lua, "GuildNewsSort", guild_news_sort)?;
    LuaApiMut::register_function(lua, "QueryGuildEventLog", query_guild_event_log)?;
    LuaApiMut::register_function(lua, "GetNumGuildEvents", get_num_guild_events)?;
    LuaApiMut::register_function(lua, "GetGuildEventInfo", get_guild_event_info)?;
    LuaApiMut::register_function(lua, "GetNumGuildNews", get_num_guild_news)?;
    Ok(())
}

fn register_guild_roster_queries(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetNumGuildMembers", get_num_guild_members)?;
    LuaApiMut::register_function(lua, "GetGuildRosterSize", get_guild_roster_size)?;
    LuaApiMut::register_function(lua, "GetGuildRosterMOTD", get_guild_roster_motd)?;
    LuaApiMut::register_function(lua, "GetGuildRosterInfo", get_guild_roster_info)?;
    Ok(())
}
