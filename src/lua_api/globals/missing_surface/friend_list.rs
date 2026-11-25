//! `C_FriendList` probe surface backed by social friend state.
//!
//! Blizzard's Friends frame only needs a tiny read surface in the sim:
//! friend counts, friend-by-index/name lookups, and the who-list row
//! probe used by the search UI.

use super::ensure_namespace;
use crate::c_api::c_friend_list::{
    build_friend_info_table, friends_in_display_order, show_friends,
};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set_static};
use crate::lua_api::state::SocialFriend;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Copy)]
struct FriendListRow {
    name: &'static str,
    level: i32,
    area: &'static str,
    class_name: &'static str,
    race_str: &'static str,
    gender: i32,
    full_guild_name: &'static str,
}

const FRIEND_LIST_ROWS: &[FriendListRow] = &[
    FriendListRow {
        name: "Alyth",
        level: 80,
        area: "Stormwind City",
        class_name: "Paladin",
        race_str: "Human",
        gender: 2,
        full_guild_name: "Heroes of Azeroth",
    },
    FriendListRow {
        name: "Brennor",
        level: 80,
        area: "Duskwood",
        class_name: "Warrior",
        race_str: "Human",
        gender: 2,
        full_guild_name: "Heroes of Azeroth",
    },
];

pub(super) fn register_friend_list_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_FriendList")?;
    table_set_rust_fn_static(state, table_ref, "GetNumFriends", get_num_friends)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetNumOnlineFriends",
        get_num_online_friends,
    )?;
    table_set_rust_fn_static(state, table_ref, "GetNumIgnores", get_num_ignores)?;
    table_set_rust_fn_static(state, table_ref, "GetIgnoreName", get_ignore_name)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFriendInfoByIndex",
        get_friend_info_by_index,
    )?;
    #[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFriendInfoByName",
        get_friend_info_by_name,
    )?;
    table_set_rust_fn_static(state, table_ref, "IsFriend", is_friend)?;
    table_set_rust_fn_static(state, table_ref, "GetWhoInfo", get_who_info)?;
    table_set_rust_fn_static(state, table_ref, "GetNumWhoResults", get_num_who_results)?;
    table_set_rust_fn_static(state, table_ref, "SetWhoToUi", set_who_to_ui)?;
    table_set_rust_fn_static(state, table_ref, "ShowFriends", show_friends)?;
    Ok(())
}

fn get_num_friends(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.social_friends.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_num_online_friends(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .social_friends
        .iter()
        .filter(|friend| friend.is_online)
        .count();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_num_ignores(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    let count = borrow_state(state)?.social_ignores.len().min(50);
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    let count = 0;
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_ignore_name(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    let _ = index;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    let name = {
        let ignores = &borrow_state(state)?.social_ignores;
        index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| ignores.get(index).cloned())
    };
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    let name: Option<String> = None;
    let value = name.map_or_else(|| Val::Nil, |name| create_string(state, &name));
    state.push(value);
    Ok(1)
}

fn get_friend_info_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let Some(friend) = social_friend_by_display_index(state, index)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = build_friend_info_table(state, &friend);
    state.push(table);
    Ok(1)
}

#[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
fn get_friend_info_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let friend = {
        let sim = borrow_state(state)?;
        sim.social_friends
            .iter()
            .find(|friend| friend.name == name)
            .cloned()
    };
    let Some(friend) = friend else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = build_friend_info_table(state, &friend);
    state.push(table);
    Ok(1)
}

fn is_friend(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let is_friend = borrow_state(state)?
        .social_friends
        .iter()
        .any(|friend| friend.name == name);
    state.push(Val::Bool(is_friend));
    Ok(1)
}

fn get_who_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let Some(row) = friend_row_by_index(index) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let who_info = build_who_info_table(state, row);
    state.push(who_info);
    Ok(1)
}

fn get_num_who_results(state: &mut LuaState) -> LuaResult<u32> {
    let who_count = FRIEND_LIST_ROWS.len() as f64;
    state.push(Val::Num(who_count));
    state.push(Val::Num(who_count));
    Ok(2)
}

fn set_who_to_ui(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = bool::from_stack(state, 1)?;
    #[cfg(feature = "client-wrath")]
    {
        crate::lua_api::methods::borrow_state_mut(state)?
            .who_routing
            .to_ui = enabled;
    }
    #[cfg(not(feature = "client-wrath"))]
    let _ = enabled;
    Ok(0)
}

fn friend_row_by_index(index: i32) -> Option<&'static FriendListRow> {
    let zero_based = usize::try_from(index - 1).ok()?;
    FRIEND_LIST_ROWS.get(zero_based)
}

fn social_friend_by_display_index(
    state: &mut LuaState,
    index: i32,
) -> LuaResult<Option<SocialFriend>> {
    let Some(zero_based) = usize::try_from(index - 1).ok() else {
        return Ok(None);
    };
    let friends = friends_in_display_order(&borrow_state(state)?.social_friends);
    Ok(friends.get(zero_based).cloned())
}

fn build_who_info_table(state: &mut LuaState, row: &FriendListRow) -> Val {
    let table = create_table(state);
    let full_name = create_string(state, row.name);
    let name = create_string(state, row.name);
    let class_str = create_string(state, row.class_name);
    let filename = create_string(state, &row.class_name.to_uppercase());
    let area = create_string(state, row.area);
    let race_str = create_string(state, row.race_str);
    let full_guild_name = create_string(state, row.full_guild_name);
    table_set_static(state, table, "fullName", full_name);
    table_set_static(state, table, "name", name);
    table_set_static(state, table, "level", Val::Num(row.level as f64));
    table_set_static(state, table, "classStr", class_str);
    table_set_static(state, table, "filename", filename);
    table_set_static(state, table, "area", area);
    table_set_static(state, table, "raceStr", race_str);
    table_set_static(state, table, "gender", Val::Num(row.gender as f64));
    table_set_static(state, table, "fullGuildName", full_guild_name);
    table
}
