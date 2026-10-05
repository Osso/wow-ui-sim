//! `C_AutoComplete` name-completion surface backed by seeded social state.
//!
//! Retail returns an array of `{ name, priority }` tables. The simulator does
//! not model the full "recently interacted" or guild-roster caches yet, but it
//! can provide real entries from the state it already owns: friends, party
//! members, Battle.net accounts, and the current account character.

use std::collections::HashSet;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_api::state_types::{BnetFriend, SEEDED_LOCAL_CHARACTER_NAME, SocialFriend};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const FLAG_IN_GROUP: u32 = 1;
const FLAG_FRIEND: u32 = 4;
const FLAG_BNET: u32 = 8;
const FLAG_ONLINE: u32 = 32;
const FLAG_ACCOUNT_CHARACTER: u32 = 128;
const FLAG_ALL: u32 = u32::MAX;

const PRIORITY_IN_GROUP: i32 = 3;
const PRIORITY_FRIEND: i32 = 5;
const PRIORITY_ACCOUNT_CHARACTER: i32 = 6;

#[derive(Clone, Debug, Eq, PartialEq)]
struct AutoCompleteCandidate {
    name: String,
    flags: u32,
    priority: i32,
    online: bool,
}

pub(crate) fn register_c_auto_complete_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_AutoComplete")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAutoCompleteResults",
        c_auto_complete_get_results,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsRecognizedName",
        c_auto_complete_is_recognized_name,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAutoCompletePresenceID",
        c_auto_complete_get_presence_id,
    )
}

/// INFERRED: a name is recognized when a flag-filtered candidate matches it
/// exactly, ignoring ASCII case like the result search does.
fn c_auto_complete_is_recognized_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let include_flags = flags_from_stack(state, 2, FLAG_ALL)?;
    let exclude_flags = flags_from_stack(state, 3, 0)?;
    let recognized = collect_candidates(state)?.iter().any(|candidate| {
        matches_flags(candidate, include_flags, exclude_flags)
            && candidate.name.eq_ignore_ascii_case(&name)
    });
    state.push(Val::Bool(recognized));
    Ok(1)
}

/// Resolves a Battle.net friend by account name or by BattleTag without its
/// `#1234` suffix (Blizzard `BNet_GetBNetIDAccount`), returning the account id.
fn c_auto_complete_get_presence_id(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let presence_id = borrow_state(state)?
        .bnet_friends
        .iter()
        .find(|friend| bnet_friend_has_name(friend, &name))
        .map(|friend| friend.bnet_account_id);
    state.push(presence_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

fn bnet_friend_has_name(friend: &BnetFriend, name: &str) -> bool {
    let tag_name = friend
        .battle_tag
        .split_once('#')
        .map_or(friend.battle_tag.as_str(), |(tag_name, _)| tag_name);
    friend.account_name.eq_ignore_ascii_case(name) || tag_name.eq_ignore_ascii_case(name)
}

fn c_auto_complete_get_results(state: &mut LuaState) -> LuaResult<u32> {
    let query = String::from_stack(state, 1)?;
    let max_results = Option::<i32>::from_stack(state, 2)?.unwrap_or(0);
    let cursor_position = Option::<i32>::from_stack(state, 3)?.unwrap_or(query.len() as i32);
    let allow_full_match = Option::<bool>::from_stack(state, 4)?.unwrap_or(true);
    let include_flags = flags_from_stack(state, 5, FLAG_ALL)?;
    let exclude_flags = flags_from_stack(state, 6, 0)?;

    let search_text = search_text_at_cursor(&query, cursor_position);
    let candidates = collect_candidates(state)?;
    let results = filter_candidates(
        candidates,
        &search_text,
        max_results,
        allow_full_match,
        include_flags,
        exclude_flags,
    );

    let table = create_table(state);
    for (index, candidate) in results.iter().enumerate() {
        let entry = create_table(state);
        let name = create_string(state, &candidate.name);
        table_set(state, entry, "name", name);
        table_set(
            state,
            entry,
            "priority",
            Val::Num(candidate.priority as f64),
        );
        set_table_array(state, table, index as i64 + 1, entry);
    }
    state.push(table);
    Ok(1)
}

fn collect_candidates(state: &mut LuaState) -> LuaResult<Vec<AutoCompleteCandidate>> {
    let sim = borrow_state(state)?;
    let mut candidates = Vec::new();

    candidates.push(account_character_candidate());
    candidates.extend(sim.social_friends.iter().map(friend_candidate));
    candidates.extend(
        sim.party_members
            .iter()
            .map(|member| AutoCompleteCandidate {
                name: member.name.clone(),
                flags: FLAG_IN_GROUP | FLAG_ONLINE,
                priority: PRIORITY_IN_GROUP,
                online: true,
            }),
    );
    candidates.extend(sim.bnet_friends.iter().map(bnet_candidate));

    Ok(candidates)
}

fn account_character_candidate() -> AutoCompleteCandidate {
    AutoCompleteCandidate {
        name: SEEDED_LOCAL_CHARACTER_NAME.to_string(),
        flags: FLAG_ACCOUNT_CHARACTER | FLAG_ONLINE,
        priority: PRIORITY_ACCOUNT_CHARACTER,
        online: true,
    }
}

fn friend_candidate(friend: &SocialFriend) -> AutoCompleteCandidate {
    let mut flags = FLAG_FRIEND;
    if friend.is_online {
        flags |= FLAG_ONLINE;
    }
    AutoCompleteCandidate {
        name: friend.name.clone(),
        flags,
        priority: PRIORITY_FRIEND,
        online: friend.is_online,
    }
}

fn bnet_candidate(friend: &BnetFriend) -> AutoCompleteCandidate {
    let online = friend.game_accounts.iter().any(|account| account.is_online);
    let mut flags = FLAG_BNET;
    if online {
        flags |= FLAG_ONLINE;
    }
    AutoCompleteCandidate {
        name: friend.account_name.clone(),
        flags,
        priority: PRIORITY_FRIEND,
        online,
    }
}

fn filter_candidates(
    candidates: Vec<AutoCompleteCandidate>,
    search_text: &str,
    max_results: i32,
    allow_full_match: bool,
    include_flags: u32,
    exclude_flags: u32,
) -> Vec<AutoCompleteCandidate> {
    let mut seen = HashSet::new();
    let limit = usize::try_from(max_results).ok().filter(|limit| *limit > 0);
    let mut matches = Vec::new();

    for candidate in candidates {
        if !matches_flags(&candidate, include_flags, exclude_flags) {
            continue;
        }
        if !matches_search_text(&candidate.name, search_text, allow_full_match) {
            continue;
        }
        if !seen.insert(candidate.name.to_ascii_lowercase()) {
            continue;
        }
        matches.push(candidate);
        if limit.is_some_and(|limit| matches.len() >= limit) {
            break;
        }
    }

    matches
}

fn flags_from_stack(state: &LuaState, index: i32, default: u32) -> LuaResult<u32> {
    match crate::lua_bridge::stack_val(state, index) {
        Val::Nil => Ok(default),
        Val::Num(value) if value < 0.0 => Ok(value as i64 as u32),
        Val::Num(value) => Ok(value as u64 as u32),
        other => Err(runtime_error(format!(
            "bad argument #{index} to C_AutoComplete.GetAutoCompleteResults (number expected, got {})",
            other.type_name()
        ))),
    }
}

fn matches_flags(
    candidate: &AutoCompleteCandidate,
    include_flags: u32,
    exclude_flags: u32,
) -> bool {
    if exclude_flags != 0 && candidate.flags & exclude_flags != 0 {
        return false;
    }
    if include_flags == FLAG_ALL {
        return true;
    }
    if include_flags & FLAG_ONLINE != 0 && !candidate.online {
        return false;
    }
    candidate.flags & include_flags != 0
}

fn matches_search_text(name: &str, search_text: &str, allow_full_match: bool) -> bool {
    if search_text.is_empty() {
        return true;
    }
    let name = name.to_ascii_lowercase();
    let search_text = search_text.to_ascii_lowercase();
    if !allow_full_match && name == search_text {
        return false;
    }
    name.starts_with(&search_text)
}

fn search_text_at_cursor(query: &str, cursor_position: i32) -> String {
    let cursor = cursor_position_to_byte_index(query, cursor_position);
    query[..cursor]
        .rsplit_once(|character: char| character.is_whitespace() || character == '/')
        .map(|(_, suffix)| suffix)
        .unwrap_or(&query[..cursor])
        .to_string()
}

fn cursor_position_to_byte_index(query: &str, cursor_position: i32) -> usize {
    if cursor_position <= 0 {
        return query.len();
    }
    query
        .char_indices()
        .nth(cursor_position as usize)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(query.len())
}
