//! Retail/PTR role writes with changed-only synchronous notification.

use super::unit_roles::{represented_identity, require_string};
use crate::lua_api::globals::group_queries::relationships::{is_group_assistant, is_group_leader};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnitRole {
    Tank,
    Healer,
    Damage,
}

impl UnitRole {
    fn number(self) -> f64 {
        match self {
            Self::Tank => 0.0,
            Self::Healer => 1.0,
            Self::Damage => 2.0,
        }
    }
}

fn role_name(role: Option<UnitRole>) -> &'static str {
    match role {
        Some(UnitRole::Tank) => "TANK",
        Some(UnitRole::Healer) => "HEALER",
        Some(UnitRole::Damage) => "DAMAGER",
        None => "NONE",
    }
}

pub(crate) fn read_role(state: &LuaState, unit: &str) -> LuaResult<(&'static str, f64)> {
    let sim = borrow_state(state)?;
    let role = represented_identity(&sim, unit)
        .and_then(|(guid, _)| sim.unit_roles.get(&guid).copied().flatten());
    Ok((role_name(role), role.map_or(-1.0, UnitRole::number)))
}

fn parse_enum_role(value: Val) -> LuaResult<Option<Option<UnitRole>>> {
    match value {
        Val::Nil | Val::Num(-1.0) => Ok(Some(None)),
        Val::Num(0.0) => Ok(Some(Some(UnitRole::Tank))),
        Val::Num(1.0) => Ok(Some(Some(UnitRole::Healer))),
        Val::Num(2.0) => Ok(Some(Some(UnitRole::Damage))),
        Val::Num(_) => Ok(None),
        _ => Err(runtime_error(
            "UnitSetRoleEnum role must be a number or nil",
        )),
    }
}

fn parse_string_role(state: &LuaState, value: Val) -> LuaResult<Option<Option<UnitRole>>> {
    if matches!(value, Val::Nil) {
        return Ok(Some(None));
    }
    Ok(match require_string(state, value)?.as_str() {
        "NONE" => Some(None),
        "TANK" => Some(Some(UnitRole::Tank)),
        "HEALER" => Some(Some(UnitRole::Healer)),
        "DAMAGER" => Some(Some(UnitRole::Damage)),
        _ => None,
    })
}

fn authenticate_arguments(state: &LuaState) -> LuaResult<(Val, Val)> {
    // Authenticate both original values BEFORE parsing either one. In particular,
    // a malformed public unit cannot bypass denial of a secret role argument.
    let unit = unwrap_secret(state, stack_val(state, 1))?;
    let role = unwrap_secret(state, stack_val(state, 2))?;
    Ok((unit, role))
}

fn unit_set_role_enum(state: &mut LuaState) -> LuaResult<u32> {
    let (unit, role) = authenticate_arguments(state)?;
    let unit = require_string(state, unit)?;
    let role = parse_enum_role(role)?;
    assign_role(state, &unit, role)
}

fn unit_set_role(state: &mut LuaState) -> LuaResult<u32> {
    let (unit, role) = authenticate_arguments(state)?;
    let unit = require_string(state, unit)?;
    let role = parse_string_role(state, role)?;
    assign_role(state, &unit, role)
}

fn assign_role(
    state: &mut LuaState,
    unit: &str,
    parsed_role: Option<Option<UnitRole>>,
) -> LuaResult<u32> {
    let Some(role) = parsed_role else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let (accepted, event) = commit_role(state, unit, role)?;
    if let Some(payload) = event {
        dispatch_role_changed(state, &payload);
    }
    state.push(Val::Bool(accepted));
    Ok(1)
}

fn commit_role(
    state: &LuaState,
    unit: &str,
    role: Option<UnitRole>,
) -> LuaResult<(bool, Option<[String; 4]>)> {
    let mut sim = borrow_state_mut(state)?;
    let Some((guid, party_index)) = represented_identity(&sim, unit) else {
        return Ok((false, None));
    };
    if party_index.is_some()
        && !is_group_leader(&sim, "player")
        && !is_group_assistant(&sim, "player")
    {
        return Ok((false, None));
    }
    let old_role = sim.unit_roles.get(&guid).copied().flatten();
    if old_role == role {
        return Ok((true, None));
    }
    let changed_name = match party_index {
        Some(index) => sim.party_members[index].name.clone(),
        None => sim.player.name.clone(),
    };
    let payload = [
        changed_name,
        sim.player.name.clone(),
        role_name(old_role).to_owned(),
        role_name(role).to_owned(),
    ];
    sim.unit_roles.insert(guid, role);
    // The borrow ends on return, before event handlers can reenter these APIs.
    Ok((true, Some(payload)))
}

fn dispatch_role_changed(state: &mut LuaState, payload: &[String; 4]) {
    let saved_top = state.top;
    let mut args = [Val::Nil; 4];
    for (argument, text) in args.iter_mut().zip(payload) {
        *argument = create_string(state, text);
        // Root each string before allocating the next one.
        state.push(*argument);
    }
    fire_named_event_state(state, "ROLE_CHANGED_INFORM", &args);
    state.top = saved_top;
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "UnitSetRoleEnum", unit_set_role_enum)?;
    LuaApiMut::register_function(lua, "UnitSetRole", unit_set_role)
}
