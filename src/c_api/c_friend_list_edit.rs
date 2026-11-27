//! Retail 12.1.5 / Forever 1.60.1 FriendListDocumentation.lua friend and
//! ignore list edits over `SimState.social_friends` / `social_ignores`.
//!
//! INFERRED R88g9 (native evidence pending): the server accepts each request
//! at once; names match without ASCII case; a new friend is offline with empty
//! level/area/class/GUID until the host supplies presence; indices follow
//! `friends_in_display_order` and the ignore list order; each accepted change
//! queues `FRIENDLIST_UPDATE` or `IGNORELIST_UPDATE`, as `ShowFriends` does.

use crate::c_api::c_friend_list::friends_in_display_order;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
use crate::lua_api::state::{SimState, SocialFriend};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

type FriendFn = fn(&mut LuaState) -> LuaResult<u32>;

const METHODS: &[(&str, FriendFn)] = &[
    ("AddFriend", add_friend),
    ("RemoveFriend", remove_friend),
    ("AddOrRemoveFriend", add_or_remove_friend),
    ("RemoveFriendByIndex", remove_friend_by_index),
    ("SetFriendNotesByIndex", set_friend_notes_by_index),
    ("GetSelectedFriend", get_selected_friend),
    ("SetSelectedFriend", set_selected_friend),
    ("AddIgnore", add_ignore),
    ("DelIgnore", del_ignore),
    ("AddOrDelIgnore", add_or_del_ignore),
    ("DelIgnoreByIndex", del_ignore_by_index),
    ("IsIgnored", is_ignored),
    ("IsOnIgnoredList", is_ignored),
];

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_FriendList")?;
    for (name, function) in METHODS {
        table_set_rust_fn_static(state, namespace, name, *function)?;
    }
    Ok(())
}

fn string_argument(state: &mut LuaState, index: i32) -> LuaResult<Option<String>> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, index))?;
    match value {
        Val::Nil => Ok(None),
        Val::Str(_) | Val::Num(_) => Ok(val_to_string(state, value)),
        _ => Err(runtime_error("C_FriendList: expected a string")),
    }
}

fn name_argument(state: &mut LuaState) -> LuaResult<String> {
    string_argument(state, 1)?
        .filter(|name| !name.is_empty())
        .ok_or_else(|| runtime_error("C_FriendList: name is required"))
}

/// Zero-based slot for a `luaIndex` argument.
fn index_argument(state: &mut LuaState, index: i32) -> LuaResult<Option<usize>> {
    let number = i64::from_stack(state, index)?;
    Ok(usize::try_from(number - 1).ok())
}

fn friend_position(sim: &SimState, name: &str) -> Option<usize> {
    sim.social_friends
        .iter()
        .position(|friend| friend.name.eq_ignore_ascii_case(name))
}

fn friend_name_at(sim: &SimState, slot: Option<usize>) -> Option<String> {
    friends_in_display_order(&sim.social_friends)
        .get(slot?)
        .map(|friend| friend.name.clone())
}

fn offline_friend(name: String, note: String) -> SocialFriend {
    SocialFriend {
        name,
        level: 0,
        area: String::new(),
        class_name: String::new(),
        note,
        is_online: false,
        guid: String::new(),
    }
}

fn insert_friend(sim: &mut SimState, name: String, note: String) {
    if friend_position(sim, &name).is_none() {
        sim.social_friends.push(offline_friend(name, note));
        sim.events.push_simple("FRIENDLIST_UPDATE");
    }
}

fn delete_friend(sim: &mut SimState, name: &str) -> bool {
    let Some(position) = friend_position(sim, name) else {
        return false;
    };
    let removed = sim.social_friends.remove(position);
    if sim.selected_social_friend.as_deref() == Some(removed.name.as_str()) {
        sim.selected_social_friend = None;
    }
    sim.events.push_simple("FRIENDLIST_UPDATE");
    true
}

fn add_friend(state: &mut LuaState) -> LuaResult<u32> {
    let name = name_argument(state)?;
    let note = string_argument(state, 2)?.unwrap_or_default();
    insert_friend(&mut *borrow_state_mut(state)?, name, note);
    Ok(0)
}

fn remove_friend(state: &mut LuaState) -> LuaResult<u32> {
    let name = name_argument(state)?;
    let removed = delete_friend(&mut *borrow_state_mut(state)?, &name);
    state.push(Val::Bool(removed));
    Ok(1)
}

fn add_or_remove_friend(state: &mut LuaState) -> LuaResult<u32> {
    let name = name_argument(state)?;
    let note = string_argument(state, 2)?.unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    if !delete_friend(&mut sim, &name) {
        insert_friend(&mut sim, name, note);
    }
    Ok(0)
}

fn remove_friend_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let slot = index_argument(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(name) = friend_name_at(&sim, slot) {
        delete_friend(&mut sim, &name);
    }
    Ok(0)
}

fn set_friend_notes_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let slot = index_argument(state, 1)?;
    let note = string_argument(state, 2)?.unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    let position = friend_name_at(&sim, slot).and_then(|name| friend_position(&sim, &name));
    if let Some(position) = position {
        sim.social_friends[position].note = note;
        sim.events.push_simple("FRIENDLIST_UPDATE");
    }
    Ok(0)
}

fn get_selected_friend(state: &mut LuaState) -> LuaResult<u32> {
    let index = {
        let sim = borrow_state(state)?;
        sim.selected_social_friend.as_ref().and_then(|name| {
            friends_in_display_order(&sim.social_friends)
                .iter()
                .position(|friend| &friend.name == name)
        })
    };
    state.push(index.map_or(Val::Nil, |slot| Val::Num((slot + 1) as f64)));
    Ok(1)
}

fn set_selected_friend(state: &mut LuaState) -> LuaResult<u32> {
    let slot = index_argument(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.selected_social_friend = friend_name_at(&sim, slot);
    Ok(0)
}

fn ignore_position(sim: &SimState, name: &str) -> Option<usize> {
    sim.social_ignores
        .iter()
        .position(|ignored| ignored.eq_ignore_ascii_case(name))
}

fn insert_ignore(sim: &mut SimState, name: String) -> bool {
    if ignore_position(sim, &name).is_some() {
        return false;
    }
    sim.social_ignores.push(name);
    sim.events.push_simple("IGNORELIST_UPDATE");
    true
}

fn delete_ignore(sim: &mut SimState, slot: Option<usize>) -> bool {
    let Some(slot) = slot.filter(|slot| *slot < sim.social_ignores.len()) else {
        return false;
    };
    let removed = sim.social_ignores.remove(slot);
    if sim.selected_social_ignore.as_deref() == Some(removed.as_str()) {
        sim.selected_social_ignore = None;
    }
    sim.events.push_simple("IGNORELIST_UPDATE");
    true
}

fn add_ignore(state: &mut LuaState) -> LuaResult<u32> {
    let name = name_argument(state)?;
    let added = insert_ignore(&mut *borrow_state_mut(state)?, name);
    state.push(Val::Bool(added));
    Ok(1)
}

fn del_ignore(state: &mut LuaState) -> LuaResult<u32> {
    let name = name_argument(state)?;
    let mut sim = borrow_state_mut(state)?;
    let slot = ignore_position(&sim, &name);
    let removed = delete_ignore(&mut sim, slot);
    drop(sim);
    state.push(Val::Bool(removed));
    Ok(1)
}

fn add_or_del_ignore(state: &mut LuaState) -> LuaResult<u32> {
    let name = name_argument(state)?;
    let mut sim = borrow_state_mut(state)?;
    let slot = ignore_position(&sim, &name);
    if !delete_ignore(&mut sim, slot) {
        insert_ignore(&mut sim, name);
    }
    Ok(0)
}

fn del_ignore_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let slot = index_argument(state, 1)?;
    delete_ignore(&mut *borrow_state_mut(state)?, slot);
    Ok(0)
}

/// `IsIgnored(token)` / `IsOnIgnoredList(token)`. INFERRED R88g9: the token
/// is matched as a character name; unit-token resolution is pending evidence.
fn is_ignored(state: &mut LuaState) -> LuaResult<u32> {
    let name = name_argument(state)?;
    let ignored = ignore_position(&*borrow_state(state)?, &name).is_some();
    state.push(Val::Bool(ignored));
    Ok(1)
}
