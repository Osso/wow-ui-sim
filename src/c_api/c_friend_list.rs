//! Exact current-profile friend-name queries and note mutation over owned social state.
//! Indexed and earlier name queries share the existing simulator projection.

use crate::lua_api::methods::{borrow_state_mut, create_string, create_table, table_set_static};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::state::BnetBlockedEntry;
use crate::lua_api::state::SocialFriend;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::FromStack;
use rilua::Val;
use rilua::vm::state::LuaState;

/// Existing online-first simulator projection, shared by namespace and legacy reads.
pub(crate) fn friends_in_display_order(friends: &[SocialFriend]) -> Vec<SocialFriend> {
    let mut ordered = friends.to_vec();
    ordered.sort_by_key(|friend| !friend.is_online);
    ordered
}

/// Simulator response policy: enqueue refresh notification over the supplied list.
pub(crate) fn show_friends(state: &mut LuaState) -> rilua::LuaResult<u32> {
    borrow_state_mut(state)?
        .events
        .push_simple("FRIENDLIST_UPDATE");
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn set_social_ignore_name(state: &mut LuaState) -> rilua::LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    if index < 1 || index as usize > sim.social_ignores.len() + 1 {
        return Err(rilua::runtime_error(
            "A_Admin.SetSocialIgnoreName: index must append or replace an existing entry",
        ));
    }
    let index = index as usize;
    if index == sim.social_ignores.len() + 1 {
        sim.social_ignores.push(name);
    } else {
        sim.social_ignores[index - 1] = name;
    }
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn set_social_blocked_entry(state: &mut LuaState) -> rilua::LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let blocked_id = i32::from_stack(state, 2)?;
    let name = String::from_stack(state, 3)?;
    let mut sim = borrow_state_mut(state)?;
    if index < 1 || index as usize > sim.bnet_blocked_entries.len() + 1 {
        return Err(rilua::runtime_error(
            "A_Admin.SetSocialBlockedEntry: index must append or replace an existing entry",
        ));
    }
    let entry = BnetBlockedEntry { blocked_id, name };
    let index = index as usize;
    if index == sim.bnet_blocked_entries.len() + 1 {
        sim.bnet_blocked_entries.push(entry);
    } else {
        sim.bnet_blocked_entries[index - 1] = entry;
    }
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod selected_ignore {
    use crate::c_api::helpers::ensure_namespace;
    use crate::lua_api::methods::{borrow_state, borrow_state_mut};
    use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
    use rilua::vm::state::LuaState;
    use rilua::{LuaResult, Val};

    pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
        let namespace = ensure_namespace(state, "C_FriendList")?;
        table_set_rust_fn_static(state, namespace, "GetSelectedIgnore", get_selected)?;
        table_set_rust_fn_static(state, namespace, "SetSelectedIgnore", set_selected)
    }

    fn get_selected(state: &mut LuaState) -> LuaResult<u32> {
        let selected = {
            let sim = borrow_state(state)?;
            sim.selected_social_ignore.as_ref().and_then(|identity| {
                sim.social_ignores
                    .iter()
                    .position(|name| name == identity)
                    .map(|index| index + 1)
            })
        };
        state.push(selected.map_or(Val::Nil, |index| Val::Num(index as f64)));
        Ok(1)
    }

    fn set_selected(state: &mut LuaState) -> LuaResult<u32> {
        let index = i32::from_stack(state, 1)?;
        let mut sim = borrow_state_mut(state)?;
        sim.selected_social_ignore = index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| sim.social_ignores.get(index).cloned());
        Ok(0)
    }
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
pub(crate) mod current {
    use super::build_friend_info_table;
    use crate::c_api::helpers::{ensure_namespace, mark_namespace_keys_removed};
    use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
    use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
    use rilua::vm::state::LuaState;
    use rilua::{LuaResult, Val};

    pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
        let namespace = ensure_namespace(state, "C_FriendList")?;
        mark_namespace_keys_removed(state, namespace, &["GetFriendInfoByName"]);
        table_set_rust_fn_static(state, namespace, "GetFriendInfo", get_friend_info)?;
        table_set_rust_fn_static(state, namespace, "SetFriendNotes", set_friend_notes)
    }

    fn set_friend_notes(state: &mut LuaState) -> LuaResult<u32> {
        let name_argument = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
        let notes_argument = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
        let name = val_to_string(state, name_argument).ok_or_else(|| {
            rilua::runtime_error("C_FriendList.SetFriendNotes: argument 1 must be a UTF-8 string")
        })?;
        let notes = val_to_string(state, notes_argument).ok_or_else(|| {
            rilua::runtime_error("C_FriendList.SetFriendNotes: argument 2 must be a UTF-8 string")
        })?;
        let found = {
            let mut sim = borrow_state_mut(state)?;
            match sim
                .social_friends
                .iter_mut()
                .find(|friend| friend.name == name)
            {
                Some(friend) => {
                    friend.note = notes;
                    true
                }
                None => false,
            }
        };
        state.push(Val::Bool(found));
        Ok(1)
    }

    fn get_friend_info(state: &mut LuaState) -> LuaResult<u32> {
        let argument = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
        let name = val_to_string(state, argument).ok_or_else(|| {
            rilua::runtime_error("C_FriendList.GetFriendInfo: argument 1 must be a UTF-8 string")
        })?;
        let friend = {
            let sim = borrow_state(state)?;
            sim.social_friends
                .iter()
                .find(|friend| friend.name == name)
                .cloned()
        };
        let info = match friend {
            Some(friend) => build_friend_info_table(state, &friend),
            None => Val::Nil,
        };
        state.push(info);
        Ok(1)
    }
}

// Existing simulator compatibility projection, not a native DTO claim.
// Synthetic race, gender and guild constants retain their previous semantics.
pub(crate) fn build_friend_info_table(state: &mut LuaState, friend: &SocialFriend) -> Val {
    let table = create_table(state);
    let name = create_string(state, &friend.name);
    let full_name = create_string(state, &friend.name);
    let area = create_string(state, &friend.area);
    let class_name = create_string(state, &friend.class_name);
    let class_str = create_string(state, &friend.class_name);
    let filename = create_string(state, &friend.class_name.to_uppercase());
    let notes = create_string(state, &friend.note);
    let guid = create_string(state, &friend.guid);
    let race_str = create_string(state, "Human");
    let full_guild_name = create_string(state, "Heroes of Azeroth");
    table_set_static(state, table, "name", name);
    table_set_static(state, table, "fullName", full_name);
    table_set_static(state, table, "level", Val::Num(friend.level as f64));
    table_set_static(state, table, "area", area);
    table_set_static(state, table, "className", class_name);
    table_set_static(state, table, "classStr", class_str);
    table_set_static(state, table, "filename", filename);
    table_set_static(state, table, "notes", notes);
    table_set_static(state, table, "connected", Val::Bool(friend.is_online));
    table_set_static(state, table, "guid", guid);
    table_set_static(state, table, "raceStr", race_str);
    table_set_static(state, table, "gender", Val::Num(2.0));
    table_set_static(state, table, "fullGuildName", full_guild_name);
    table
}
