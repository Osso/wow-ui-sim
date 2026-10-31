//! Explicit display roles for existing player/active-party identities.
//! Unset, authority and alias policies are modeled; see unit-role-assignment spec.

use super::unit_secret_predicates::finish_identity_restricted;
use crate::lua_api::methods::{create_string, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
pub(crate) use super::unit_roles_assignment::read_role;

pub(super) fn require_string(state: &LuaState, value: Val) -> LuaResult<String> {
    if !matches!(value, Val::Str(_)) {
        return Err(runtime_error("unit role argument must be a string"));
    }
    val_to_string(state, value).ok_or_else(|| runtime_error("unit role argument must be UTF-8"))
}

/// Compare existing GUIDs only; do not infer native GUID components or raid IDs.
#[cfg(feature = "retail-12-0-0")]
pub(super) fn represented_identity(
    sim: &crate::lua_api::state::SimState,
    unit: &str,
) -> Option<(String, Option<usize>)> {
    use crate::lua_api::globals::unit_misc::{
        existing_guid_for_unit, guid_for_unit, party_guid_for_index,
    };

    let guid = existing_guid_for_unit(sim, unit)?;
    if guid == guid_for_unit(sim, "player") {
        return Some((guid, None));
    }
    if !sim.party_group_active {
        return None;
    }
    sim.party_members
        .iter()
        .enumerate()
        .find(|(index, _)| party_guid_for_index(*index) == guid)
        .map(|(index, _)| (guid, Some(index)))
}

#[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
pub(crate) fn read_role(state: &LuaState, _unit: &str) -> LuaResult<(&'static str, f64)> {
    // These profiles have no role assignment producer; preserve their unset reads.
    let _sim = crate::lua_api::methods::borrow_state(state)?;
    Ok(("NONE", -1.0))
}

fn read_nullable_unit(state: &LuaState) -> LuaResult<String> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Nil => Ok(String::new()),
        other => require_string(state, other),
    }
}

pub(crate) fn unit_group_roles_assigned(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_nullable_unit(state)?;
    // Snapshot before Lua allocation; never hold SimState through reentrant work.
    let role = read_role(state, &unit)?;
    let value = create_string(state, role.0);
    state.push(value);
    finish_identity_restricted(state, &[&unit], 1)
}

pub(crate) fn unit_group_roles_assigned_enum(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_nullable_unit(state)?;
    let role = read_role(state, &unit)?;
    state.push(Val::Num(role.1));
    finish_identity_restricted(state, &[&unit], 1)
}
