//! Bounded 12.0.7 party operations over explicit environment-local host state.
//! INFERRED: name matching, permission input, event timing, category domain,
//! updated semantics and public outputs are simulator policies, not native proof.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
use crate::lua_api::state::{SEEDED_LOCAL_CHARACTER_GUID, SimState};
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const HOME_CATEGORY: i32 = 1;
const INSTANCE_CATEGORY: i32 = 2;

fn authenticate_arguments(state: &LuaState) -> LuaResult<[Val; 3]> {
    // Authenticate ALL arguments, including extras, before any validation.
    // Original wrappers stay rooted on the call stack; no VM allocation here.
    for value in state.stack.iter().take(state.top).skip(state.base) {
        unwrap_secret(state, *value)?;
    }
    Ok([
        unwrap_secret(state, stack_val(state, 1))?,
        unwrap_secret(state, stack_val(state, 2))?,
        unwrap_secret(state, stack_val(state, 3))?,
    ])
}

fn require_string(state: &LuaState, value: Val, argument: &str) -> LuaResult<String> {
    if !matches!(value, Val::Str(_)) {
        return Err(runtime_error(format!(
            "C_PartyInfo: {argument} must be a string"
        )));
    }
    val_to_string(state, value)
        .ok_or_else(|| runtime_error(format!("C_PartyInfo: {argument} must be UTF-8")))
}

fn optional_bool(value: Val) -> LuaResult<bool> {
    match value {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(
            "C_PartyInfo: exactNameMatch must be boolean or nil",
        )),
    }
}

fn read_name_arguments(state: &LuaState, exact_position: usize) -> LuaResult<(String, bool)> {
    let arguments = authenticate_arguments(state)?;
    let name = require_string(state, arguments[0], "name")?;
    let exact = optional_bool(arguments[exact_position])?;
    if exact_position == 2 && !matches!(arguments[1], Val::Nil) {
        require_string(state, arguments[1], "reason")?;
    }
    Ok((name, exact))
}

fn reject_restricted_operation(state: &LuaState) -> LuaResult<()> {
    // INFERRED: host input models restrictions; no combat/leadership derivation.
    if borrow_state(state)?.party_operations_restricted {
        return Err(runtime_error("C_PartyInfo: operation restricted by host"));
    }
    Ok(())
}

// INFERRED: case-sensitive exact/realm-qualified matching; short names omit realm.
fn names_match(candidate: &str, name: &str, exact: bool) -> bool {
    if exact || name.contains('-') {
        return candidate == name;
    }
    candidate.split('-').next() == Some(name)
}

fn resolve_member_name(sim: &SimState, name: &str, exact: bool) -> Option<String> {
    if !sim.party_group_active {
        return None;
    }
    if matches!(name, "player" | "pet" | "vehicle") || names_match(&sim.player.name, name, exact) {
        return Some(sim.player.name.clone());
    }
    if let Some(index) = crate::lua_api::globals::unit_api::parse_party_index(name) {
        return sim
            .party_members
            .get(index)
            .map(|member| member.name.clone());
    }
    if let Some(index) = name
        .strip_prefix("raid")
        .and_then(|number| number.parse::<usize>().ok())
        .and_then(|number| number.checked_sub(1))
    {
        return sim
            .party_members
            .get(index)
            .map(|member| member.name.clone());
    }
    // INFERRED: ambiguous short names do not select an arbitrary member.
    let mut matches = sim
        .party_members
        .iter()
        .filter(|member| names_match(&member.name, name, exact));
    let member = matches.next()?;
    matches.next().is_none().then(|| member.name.clone())
}

pub(crate) fn is_assistant(sim: &SimState, unit: &str) -> bool {
    let Some(name) = resolve_member_name(sim, unit, true) else {
        return false;
    };
    if sim.party_assistant_exclusions.contains(&name) {
        return false;
    }
    sim.everyone_assistant || sim.party_assistants.contains(&name)
}

fn mutate_assistant(state: &mut LuaState, promote: bool) -> LuaResult<u32> {
    let (name, exact) = read_name_arguments(state, 1)?;
    reject_restricted_operation(state)?;
    let mut sim = borrow_state_mut(state)?;
    let Some(name) = resolve_member_name(&sim, &name, exact) else {
        return Ok(0);
    };
    if promote {
        sim.party_assistant_exclusions.remove(&name);
        sim.party_assistants.insert(name);
    } else {
        sim.party_assistants.remove(&name);
        // INFERRED: an individual demotion overrides the everyone toggle.
        if sim.everyone_assistant {
            sim.party_assistant_exclusions.insert(name);
        }
    }
    Ok(0)
}

pub(super) fn demote_assistant(state: &mut LuaState) -> LuaResult<u32> {
    mutate_assistant(state, false)
}

pub(super) fn promote_to_assistant(state: &mut LuaState) -> LuaResult<u32> {
    mutate_assistant(state, true)
}

pub(super) fn promote_to_leader(state: &mut LuaState) -> LuaResult<u32> {
    let (name, exact) = read_name_arguments(state, 1)?;
    reject_restricted_operation(state)?;
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let Some(name) = resolve_member_name(&sim, &name, exact) else {
            return Ok(0);
        };
        let index = sim
            .party_members
            .iter()
            .position(|member| member.name == name);
        let changed = sim.party_leader_index != index;
        sim.party_leader_index = index;
        changed
    };
    // INFERRED: publish synchronously after mutation, only on a changed leader.
    if changed {
        dispatch_event_now(state, "PARTY_LEADER_CHANGED", &[])?;
    }
    Ok(0)
}

pub(super) fn set_everyone_is_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let arguments = authenticate_arguments(state)?;
    let Val::Bool(enabled) = arguments[0] else {
        return Err(runtime_error("C_PartyInfo: isAssistant must be boolean"));
    };
    reject_restricted_operation(state)?;
    let updated = {
        let mut sim = borrow_state_mut(state)?;
        // INFERRED: updated means a live toggle changed in an active group.
        let updated = sim.party_group_active && sim.everyone_assistant != enabled;
        if updated {
            sim.everyone_assistant = enabled;
            sim.party_assistant_exclusions.clear();
        }
        updated
    };
    state.push(Val::Bool(updated));
    Ok(1)
}

// INFERRED: nil selects home and the accepted category domain is only 1/2.
fn read_category(value: Val) -> LuaResult<i32> {
    match value {
        Val::Nil => Ok(HOME_CATEGORY),
        Val::Num(value) if value == f64::from(HOME_CATEGORY) => Ok(HOME_CATEGORY),
        Val::Num(value) if value == f64::from(INSTANCE_CATEGORY) => Ok(INSTANCE_CATEGORY),
        _ => Err(runtime_error("C_PartyInfo: category must be 1, 2 or nil")),
    }
}

pub(super) fn is_guid_in_group(state: &mut LuaState) -> LuaResult<u32> {
    let arguments = authenticate_arguments(state)?;
    let guid = require_string(state, arguments[0], "guid")?;
    let category = read_category(arguments[1])?;
    let member = {
        let sim = borrow_state(state)?;
        if category == HOME_CATEGORY {
            sim.party_group_active
                && (guid == SEEDED_LOCAL_CHARACTER_GUID
                    || sim
                        .party_members
                        .iter()
                        .enumerate()
                        .any(|(index, _)| super::party_member_guid(index) == guid))
        } else {
            sim.party_category_guids
                .get(&category)
                .is_some_and(|members| members.contains(&guid))
        }
    };
    // INFERRED: public boolean output; roster GUID synthesis is pre-existing.
    state.push(Val::Bool(member));
    Ok(1)
}

fn remove_member(sim: &mut SimState, name: &str) -> bool {
    let Some(index) = sim
        .party_members
        .iter()
        .position(|member| member.name == name)
    else {
        return false;
    };
    sim.party_members.remove(index);
    sim.party_assistants.remove(name);
    sim.party_assistant_exclusions.remove(name);
    // INFERRED: removal of the leader returns leadership to the local player.
    sim.party_leader_index = match sim.party_leader_index {
        Some(leader) if leader == index => None,
        Some(leader) if leader > index => Some(leader - 1),
        leader => leader,
    };
    true
}

pub(super) fn uninvite_unit(state: &mut LuaState) -> LuaResult<u32> {
    let (name, exact) = read_name_arguments(state, 2)?;
    reject_restricted_operation(state)?;
    let removed = {
        let mut sim = borrow_state_mut(state)?;
        let Some(name) = resolve_member_name(&sim, &name, exact) else {
            return Ok(0);
        };
        // INFERRED: self-uninvite is a no-op; use LeaveParty explicitly.
        if name == sim.player.name {
            return Ok(0);
        }
        remove_member(&mut sim, &name)
    };
    if removed {
        dispatch_event_now(state, "GROUP_ROSTER_UPDATE", &[])?;
    }
    Ok(0)
}
