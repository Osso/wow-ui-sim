//! `C_BattleNet` probe surface backed by `SimState.bnet_friends`.
//!
//! Migrates 5 entries off the namespace stub tables:
//!
//! - `C_BattleNet.GetNumFriends()` — returns the count of seeded bnet
//!   friends.
//! - `C_BattleNet.GetFriendAccountInfo(friendIndex, [wowAccountGUID])`
//!   — returns a `BNetAccountInfo` table for a 1-based friend index,
//!   or nil when out of range. The optional `wowAccountGUID` parameter
//!   is accepted but ignored (retail uses it to pick among multiple game
//!   accounts; we always return the first game-account entry).
//! - `C_BattleNet.GetAccountInfoByGUID(bnetAccountGUID)` — returns a
//!   `BNetAccountInfo` table for the friend matching the given bnet
//!   account GUID, or nil when unknown.
//! - `C_BattleNet.GetGameAccountInfoByGUID(wowAccountGUID)` — returns a
//!   `BNetGameAccountInfo` table for the game account matching the given
//!   WoW account GUID, or nil when unknown.
//! - `C_BattleNet.GetFriendNumAccounts(friendIndex)` — returns the
//!   number of game accounts for a 1-based friend index, or 0 when out
//!   of range.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::methods::{table_set_num, val_to_string};
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::state_types::BnetFriendInvite;
use crate::lua_api::state_types::{BnetFriend, BnetGameAccount};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

type BattleNetTable = GcRef<Table>;

pub(crate) fn register_c_battle_net_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_BattleNet")?;
    register_texture_methods(state, table_ref)?;
    #[cfg(feature = "client-wowforever")]
    table_set_rust_fn_static(
        state,
        table_ref,
        "SendGameData",
        super::addon_messages::send_game_data,
    )?;
    register_friend_query_methods(state, table_ref)
}

fn register_texture_methods(state: &mut LuaState, table_ref: BattleNetTable) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "AreHighResTexturesInstalled",
        c_bnet_are_high_res_textures_installed,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "InstallHighResTextures",
        c_bnet_install_high_res_textures,
    )
}

fn register_friend_query_methods(state: &mut LuaState, table_ref: BattleNetTable) -> LuaResult<()> {
    #[cfg(feature = "retail-12-1-0")]
    register_patch_12_1_friend_query_methods(state, table_ref)?;
    table_set_rust_fn_static(state, table_ref, "InviteFriend", c_bnet_invite_friend)?;
    table_set_rust_fn_static(state, table_ref, "GetNumFriends", c_bnet_get_num_friends)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFriendAccountInfo",
        c_bnet_get_friend_account_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAccountInfoByGUID",
        c_bnet_get_account_info_by_guid,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetGameAccountInfoByGUID",
        c_bnet_get_game_account_info_by_guid,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFriendNumAccounts",
        c_bnet_get_friend_num_accounts,
    )
}

#[cfg(feature = "retail-12-1-0")]
fn register_patch_12_1_friend_query_methods(
    state: &mut LuaState,
    table_ref: BattleNetTable,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "AreFriendTagsEnabled",
        c_bnet_feature_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "AreTitleFriendCustomNamesEnabled",
        c_bnet_feature_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "AreTitleFriendsEnabled",
        c_bnet_feature_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsBattleNetFriendsListEnabled",
        c_bnet_feature_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsBattleNetFriendsListSupported",
        c_bnet_feature_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetCustomTitleFriendName",
        c_bnet_get_custom_title_friend_name,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetCustomTitleFriendName",
        c_bnet_set_custom_title_friend_name,
    )?;
    table_set_rust_fn_static(state, table_ref, "SetFriendTags", c_bnet_set_friend_tags)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFriendInviteInfo",
        c_bnet_get_friend_invite_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SendVerifiedBattleNetFriendInvite",
        c_bnet_send_verified_battle_net_friend_invite,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "BNCheckTitleFriendInviteToUnit",
        c_bnet_check_title_friend_invite_to_unit,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetAppearOffline",
        c_bnet_set_appear_offline,
    )
}

#[cfg(feature = "retail-12-1-0")]
fn c_bnet_feature_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn c_bnet_check_title_friend_invite_to_unit(state: &mut LuaState) -> LuaResult<u32> {
    // The simulator has no title-friend unit-invite service. Keep the callable
    // probe deterministic and model-backed instead of a Lua inert default.
    state.push(Val::Bool(false));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn c_bnet_set_appear_offline(state: &mut LuaState) -> LuaResult<u32> {
    let appear_offline = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.bnet_appear_offline = appear_offline;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn c_bnet_get_custom_title_friend_name(state: &mut LuaState) -> LuaResult<u32> {
    let friend_index = i32::from_stack(state, 1)?;
    let name = {
        let sim = borrow_state(state)?;
        sim.bnet_friends
            .get(friend_index_to_offset(friend_index))
            .and_then(|friend| friend.custom_title_friend_name.clone())
    };
    match name {
        Some(name) => {
            let name = create_string(state, &name);
            state.push(name);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn c_bnet_set_custom_title_friend_name(state: &mut LuaState) -> LuaResult<u32> {
    let friend_index = i32::from_stack(state, 1)?;
    let name = Option::<String>::from_stack(state, 2)?;
    if let Some(friend) =
        friend_mut_by_index(&mut borrow_state_mut(state)?.bnet_friends, friend_index)
    {
        friend.custom_title_friend_name = name.filter(|name| !name.trim().is_empty());
    }
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn c_bnet_set_friend_tags(state: &mut LuaState) -> LuaResult<u32> {
    let friend_index = i32::from_stack(state, 1)?;
    let tags = string_array_from_lua_table(state, crate::lua_bridge::stack_val(state, 2));
    if let Some(friend) =
        friend_mut_by_index(&mut borrow_state_mut(state)?.bnet_friends, friend_index)
    {
        friend.friend_tags = tags;
    }
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn c_bnet_get_friend_invite_info(state: &mut LuaState) -> LuaResult<u32> {
    let invite_index = i32::from_stack(state, 1)?;
    let invite = {
        let sim = borrow_state(state)?;
        sim.bnet_friend_invites
            .get(friend_index_to_offset(invite_index))
            .cloned()
    };
    match invite {
        Some(invite) => {
            let table = push_friend_invite_info_table(state, &invite);
            state.push(table);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn c_bnet_send_verified_battle_net_friend_invite(state: &mut LuaState) -> LuaResult<u32> {
    let Some(raw_name) = Option::<String>::from_stack(state, 1)? else {
        return Ok(0);
    };
    let battle_tag = raw_name.trim();
    if battle_tag.is_empty() {
        return Ok(0);
    }

    let mut sim = borrow_state_mut(state)?;
    if sim
        .bnet_friend_invites
        .iter()
        .any(|invite| is_same_bnet_invite(invite, battle_tag))
    {
        return Ok(0);
    }

    let invite_id = next_bnet_invite_id(&sim.bnet_friend_invites);
    sim.bnet_friend_invites
        .push(pending_bnet_friend_invite(battle_tag, invite_id));
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn friend_index_to_offset(friend_index: i32) -> usize {
    usize::try_from(friend_index - 1).unwrap_or(usize::MAX)
}

#[cfg(feature = "retail-12-1-0")]
fn friend_mut_by_index(friends: &mut [BnetFriend], friend_index: i32) -> Option<&mut BnetFriend> {
    friends.get_mut(friend_index_to_offset(friend_index))
}

#[cfg(feature = "retail-12-1-0")]
fn string_array_from_lua_table(state: &LuaState, value: Val) -> Vec<String> {
    let Val::Table(table_ref) = value else {
        return Vec::new();
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| {
            table
                .array_slice()
                .iter()
                .copied()
                .take_while(|value| !matches!(value, Val::Nil))
                .filter_map(|value| val_to_string(state, value))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(feature = "retail-12-1-0")]
fn create_friend_tags_table(state: &mut LuaState, tags: &[String]) -> Val {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return table;
    };
    for (index, tag) in tags.iter().enumerate() {
        let tag = create_string(state, tag);
        table_set_num(state, table_ref, (index + 1) as f64, tag);
    }
    table
}

#[cfg(feature = "retail-12-1-0")]
fn is_same_bnet_invite(invite: &BnetFriendInvite, name: &str) -> bool {
    invite.battle_tag.eq_ignore_ascii_case(name) || invite.account_name.eq_ignore_ascii_case(name)
}

#[cfg(feature = "retail-12-1-0")]
fn next_bnet_invite_id(invites: &[BnetFriendInvite]) -> i32 {
    invites
        .iter()
        .map(|invite| invite.invite_id)
        .max()
        .unwrap_or(0)
        + 1
}

#[cfg(feature = "retail-12-1-0")]
fn pending_bnet_friend_invite(battle_tag: &str, invite_id: i32) -> BnetFriendInvite {
    BnetFriendInvite {
        invite_id,
        battle_tag: battle_tag.to_string(),
        account_name: account_name_from_invite(battle_tag),
        friend_level: 1,
        creation_timestamp: current_unix_timestamp(),
    }
}

#[cfg(feature = "retail-12-1-0")]
fn current_unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(feature = "retail-12-1-0")]
fn push_friend_invite_info_table(state: &mut LuaState, invite: &BnetFriendInvite) -> Val {
    let table = create_table(state);
    let account_name = create_string(state, &invite.account_name);
    let battle_tag = create_string(state, &invite.battle_tag);
    table_set(state, table, "inviteID", Val::Num(invite.invite_id as f64));
    table_set(state, table, "accountName", account_name);
    table_set(state, table, "battleTag", battle_tag);
    table_set(
        state,
        table,
        "friendLevel",
        Val::Num(invite.friend_level as f64),
    );
    table_set(
        state,
        table,
        "creationTimestamp",
        Val::Num(invite.creation_timestamp as f64),
    );
    table
}

fn c_bnet_are_high_res_textures_installed(state: &mut LuaState) -> LuaResult<u32> {
    let installed = borrow_state(state)?.cvars.get_bool("useHighResTextures");
    state.push(Val::Bool(installed));
    Ok(1)
}

fn c_bnet_get_num_friends(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.bnet_friends.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-0-7")]
fn c_bnet_invite_friend(state: &mut LuaState) -> LuaResult<u32> {
    crate::c_api::c_battle_net_invite::invite_friend(state)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn c_bnet_invite_friend(state: &mut LuaState) -> LuaResult<u32> {
    let Some(raw_name) = Option::<String>::from_stack(state, 1)? else {
        return Ok(0);
    };
    let friend_name = raw_name.trim();
    if friend_name.is_empty() {
        return Ok(0);
    }

    let mut sim = borrow_state_mut(state)?;
    if sim
        .bnet_friends
        .iter()
        .any(|friend| is_same_bnet_friend(friend, friend_name))
    {
        return Ok(0);
    }

    let next_index = sim.bnet_friends.len() as i32 + 1;
    let next_account_id = next_bnet_account_id(&sim.bnet_friends);
    sim.bnet_friends.push(invited_bnet_friend(
        friend_name,
        next_index,
        next_account_id,
    ));
    Ok(0)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn is_same_bnet_friend(friend: &BnetFriend, name: &str) -> bool {
    friend.battle_tag.eq_ignore_ascii_case(name) || friend.account_name.eq_ignore_ascii_case(name)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn next_bnet_account_id(friends: &[BnetFriend]) -> i32 {
    friends
        .iter()
        .map(|friend| friend.bnet_account_id)
        .max()
        .unwrap_or(100_000)
        + 1
}

#[cfg(not(feature = "retail-12-0-7"))]
fn invited_bnet_friend(name: &str, friend_index: i32, account_id: i32) -> BnetFriend {
    BnetFriend {
        friend_index,
        bnet_account_guid: format!("BNet-0-{account_id}"),
        bnet_account_id: account_id,
        battle_tag: name.to_string(),
        account_name: account_name_from_invite(name),
        note: String::new(),
        custom_title_friend_name: None,
        friend_tags: Vec::new(),
        custom_message: String::new(),
        custom_message_time: 0,
        appear_offline: false,
        is_battle_tag_friend: true,
        is_friend: true,
        is_favorite: false,
        is_afk: false,
        is_dnd: false,
        last_online_time: 0,
        raf_link_type: 0,
        game_accounts: Vec::new(),
    }
}

#[cfg(any(not(feature = "retail-12-0-7"), feature = "retail-12-1-0"))]
fn account_name_from_invite(name: &str) -> String {
    name.split('#').next().unwrap_or(name).to_string()
}

fn c_bnet_get_friend_account_info(state: &mut LuaState) -> LuaResult<u32> {
    let friend_index = i32::from_stack(state, 1)?;
    let entry = {
        let sim = borrow_state(state)?;
        let idx = usize::try_from(friend_index - 1).unwrap_or(usize::MAX);
        sim.bnet_friends.get(idx).cloned()
    };
    let Some(friend) = entry else {
        state.push(Val::Nil);
        return Ok(1);
    };
    // Pick the first game account (retail: caller can pass wowAccountGUID
    // as arg 2 to select a specific one; we return the first online one
    // or the first available).
    let game_account = friend.game_accounts.first().cloned();
    let t = push_account_info_table(state, &friend, game_account.as_ref());
    state.push(t);
    Ok(1)
}

fn c_bnet_get_account_info_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = match crate::lua_bridge::stack_val(state, 1) {
        Val::Str(_) => {
            crate::lua_api::methods::val_to_string(state, crate::lua_bridge::stack_val(state, 1))
                .unwrap_or_default()
        }
        _ => {
            state.push(Val::Nil);
            return Ok(1);
        }
    };
    let entry = {
        let sim = borrow_state(state)?;
        sim.bnet_friends
            .iter()
            .find(|f| f.bnet_account_guid == guid)
            .cloned()
    };
    let Some(friend) = entry else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let game_account = friend.game_accounts.first().cloned();
    let t = push_account_info_table(state, &friend, game_account.as_ref());
    state.push(t);
    Ok(1)
}

fn c_bnet_get_game_account_info_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = match crate::lua_bridge::stack_val(state, 1) {
        Val::Str(_) => {
            crate::lua_api::methods::val_to_string(state, crate::lua_bridge::stack_val(state, 1))
                .unwrap_or_default()
        }
        _ => {
            state.push(Val::Nil);
            return Ok(1);
        }
    };
    let entry = {
        let sim = borrow_state(state)?;
        sim.bnet_friends
            .iter()
            .flat_map(|f| f.game_accounts.iter())
            .find(|g| g.wow_account_guid == guid)
            .cloned()
    };
    let Some(ga) = entry else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let t = push_game_account_info_table(state, &ga);
    state.push(t);
    Ok(1)
}

fn c_bnet_get_friend_num_accounts(state: &mut LuaState) -> LuaResult<u32> {
    let friend_index = i32::from_stack(state, 1)?;
    let count = {
        let sim = borrow_state(state)?;
        usize::try_from(friend_index - 1)
            .ok()
            .and_then(|idx| sim.bnet_friends.get(idx))
            .map(|f| f.game_accounts.len())
            .unwrap_or(0)
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn c_bnet_install_high_res_textures(state: &mut LuaState) -> LuaResult<u32> {
    let _ = borrow_state_mut(state)?
        .cvars
        .set("useHighResTextures", "1");
    Ok(0)
}

/// Build a `BNetAccountInfo` Lua table from `friend` + optional
/// `game_account`. The `gameAccountInfo` nested table is set when a
/// game account is provided, or `nil` otherwise.
fn push_account_info_table(
    state: &mut LuaState,
    friend: &BnetFriend,
    game_account: Option<&BnetGameAccount>,
) -> Val {
    let t = create_table(state);
    write_account_identity_fields(state, t, friend);
    write_account_status_fields(state, t, friend);
    write_patch_12_1_account_fields(state, t, friend);
    attach_game_account_field(state, t, game_account);
    t
}

/// Build a `BNetGameAccountInfo` Lua table from a `BnetGameAccount`.
fn push_game_account_info_table(state: &mut LuaState, ga: &BnetGameAccount) -> Val {
    let t = create_table(state);
    write_game_account_character(state, t, ga);
    write_game_account_presence(state, t, ga);
    write_game_account_meta(state, t, ga);
    write_patch_12_1_game_account_fields(state, t, ga);
    t
}

fn write_game_account_character(state: &mut LuaState, t: Val, ga: &BnetGameAccount) {
    let character_name = create_string(state, &ga.character_name);
    let realm_name = create_string(state, &ga.realm_name);
    let realm_display_name = create_string(state, &ga.realm_display_name);
    let class_name = create_string(state, &ga.class_name);
    let faction_name = create_string(state, &ga.faction_name);
    let race_name = create_string(state, &ga.race_name);
    table_set(state, t, "characterName", character_name);
    table_set(state, t, "realmName", realm_name);
    table_set(state, t, "realmDisplayName", realm_display_name);
    table_set(state, t, "realmID", Val::Num(ga.realm_id as f64));
    table_set(state, t, "classID", Val::Num(ga.class_id as f64));
    table_set(state, t, "className", class_name);
    table_set(
        state,
        t,
        "characterLevel",
        Val::Num(ga.character_level as f64),
    );
    table_set(state, t, "factionName", faction_name);
    table_set(state, t, "raceName", race_name);
}

fn write_game_account_presence(state: &mut LuaState, t: Val, ga: &BnetGameAccount) {
    let area_name = create_string(state, &ga.area_name);
    let client_program = create_string(state, &ga.client_program);
    let rich_presence = create_string(state, &ga.rich_presence);
    table_set(state, t, "areaName", area_name);
    table_set(state, t, "isOnline", Val::Bool(ga.is_online));
    table_set(state, t, "isGameAFK", Val::Bool(ga.is_game_afk));
    table_set(state, t, "isGameBusy", Val::Bool(ga.is_game_busy));
    table_set(state, t, "clientProgram", client_program);
    table_set(state, t, "richPresence", rich_presence);
    table_set(state, t, "hasFocus", Val::Bool(ga.has_focus));
}

fn write_game_account_meta(state: &mut LuaState, t: Val, ga: &BnetGameAccount) {
    let player_guid = create_string(state, &ga.player_guid);
    table_set(state, t, "canSummon", Val::Bool(ga.can_summon));
    table_set(
        state,
        t,
        "isInCurrentRegion",
        Val::Bool(ga.is_in_current_region),
    );
    table_set(
        state,
        t,
        "gameAccountID",
        Val::Num(ga.game_account_id as f64),
    );
    table_set(state, t, "wowProjectID", Val::Num(ga.wow_project_id as f64));
    table_set(
        state,
        t,
        "timerunningSeasonID",
        Val::Num(ga.timerunning_season_id as f64),
    );
    table_set(state, t, "regionID", Val::Num(ga.region_id as f64));
    table_set(state, t, "playerGuid", player_guid);
}

fn write_account_identity_fields(state: &mut LuaState, t: Val, friend: &BnetFriend) {
    let battle_tag = create_string(state, &friend.battle_tag);
    let account_name = create_string(state, &friend.account_name);
    let note = create_string(state, &friend.note);
    let guid = create_string(state, &friend.bnet_account_guid);
    table_set(state, t, "battleTag", battle_tag);
    table_set(state, t, "accountName", account_name);
    table_set(state, t, "note", note);
    table_set(state, t, "bnetAccountGUID", guid);
    table_set(
        state,
        t,
        "bnetAccountID",
        Val::Num(friend.bnet_account_id as f64),
    );
}

fn write_account_status_fields(state: &mut LuaState, t: Val, friend: &BnetFriend) {
    write_custom_message_fields(state, t, friend);
    write_friend_relationship_flags(state, t, friend);
    write_presence_and_link_fields(state, t, friend);
}

#[cfg(feature = "retail-12-1-0")]
fn write_patch_12_1_account_fields(state: &mut LuaState, t: Val, friend: &BnetFriend) {
    let friend_tags = create_friend_tags_table(state, &friend.friend_tags);
    table_set(state, t, "friendLevel", Val::Num(0.0));
    table_set(state, t, "friendTags", friend_tags);
}

#[cfg(not(feature = "retail-12-1-0"))]
fn write_patch_12_1_account_fields(_state: &mut LuaState, _t: Val, _friend: &BnetFriend) {}

#[cfg(feature = "retail-12-1-0")]
fn write_patch_12_1_game_account_fields(state: &mut LuaState, t: Val, ga: &BnetGameAccount) {
    let class_filename = create_string(state, &ga.class_name.to_uppercase());
    table_set(state, t, "classFilename", class_filename);
}

#[cfg(not(feature = "retail-12-1-0"))]
fn write_patch_12_1_game_account_fields(_state: &mut LuaState, _t: Val, _ga: &BnetGameAccount) {}

/// `customMessage` + `customMessageTime` + `appearOffline` — the
/// user-set status block visible in the Battle.net friends panel.
fn write_custom_message_fields(state: &mut LuaState, t: Val, friend: &BnetFriend) {
    let custom_message = create_string(state, &friend.custom_message);
    table_set(state, t, "customMessage", custom_message);
    table_set(
        state,
        t,
        "customMessageTime",
        Val::Num(friend.custom_message_time as f64),
    );
    table_set(state, t, "appearOffline", Val::Bool(friend.appear_offline));
}

/// `isBattleTagFriend` / `isFriend` / `isFavorite` — the relationship
/// classification flags consumed by the friends list filter UI.
fn write_friend_relationship_flags(state: &mut LuaState, t: Val, friend: &BnetFriend) {
    table_set(
        state,
        t,
        "isBattleTagFriend",
        Val::Bool(friend.is_battle_tag_friend),
    );
    table_set(state, t, "isFriend", Val::Bool(friend.is_friend));
    table_set(state, t, "isFavorite", Val::Bool(friend.is_favorite));
}

/// `isAFK` / `isDND` / `lastOnlineTime` / `rafLinkType` — presence
/// state + Recruit-A-Friend link type returned by GetFriendAccountInfo.
fn write_presence_and_link_fields(state: &mut LuaState, t: Val, friend: &BnetFriend) {
    table_set(state, t, "isAFK", Val::Bool(friend.is_afk));
    table_set(state, t, "isDND", Val::Bool(friend.is_dnd));
    table_set(
        state,
        t,
        "lastOnlineTime",
        Val::Num(friend.last_online_time as f64),
    );
    table_set(
        state,
        t,
        "rafLinkType",
        Val::Num(friend.raf_link_type as f64),
    );
}

fn attach_game_account_field(state: &mut LuaState, t: Val, game_account: Option<&BnetGameAccount>) {
    let value = game_account
        .map(|ga| push_game_account_info_table(state, ga))
        .unwrap_or(Val::Nil);
    table_set(state, t, "gameAccountInfo", value);
}
