//! `C_LFGListRoles`: the solo roles a player advertises in the Vanilla-style
//! Group Finder (Forever `LFGListRolesDocumentation.lua:10-45`).
//!
//! `current` is the live selection; `saved` is the persisted copy that
//! `Blizzard_LFGVanilla_Listing.lua:553` reads once on startup.
//! INFERRED (R-request pending): `SetRoles` persists the roles it accepts, and
//! a character with no saved roles starts with all three unset.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table, table_get, table_set};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// `LFGRoles` (Forever `LFGConstantsDocumentation.lua:156-163`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LfgRoles {
    pub tank: bool,
    pub healer: bool,
    pub dps: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LfgListRoles {
    pub current: LfgRoles,
    pub saved: LfgRoles,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_LFGListRoles")?;
    table_set_rust_fn_static(state, namespace, "GetRoles", get_roles)?;
    table_set_rust_fn_static(state, namespace, "GetSavedRoles", get_saved_roles)?;
    table_set_rust_fn_static(state, namespace, "SetRoles", set_roles)
}

/// `A_Admin.SetSavedLFGListRoles(roles)`: the roles persisted by an earlier session.
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return Ok(builder);
    }
    builder.set_function("SetSavedLFGListRoles", admin_set_saved_roles)
}

fn get_roles(state: &mut LuaState) -> LuaResult<u32> {
    let roles = borrow_state(state)?.lfg_list_roles.current;
    push_roles(state, roles);
    Ok(1)
}

fn get_saved_roles(state: &mut LuaState) -> LuaResult<u32> {
    let roles = borrow_state(state)?.lfg_list_roles.saved;
    push_roles(state, roles);
    Ok(1)
}

/// `SetRoles(roles, limitToClassRoles)` -> success; fires the synchronous
/// `LFG_LIST_ROLE_UPDATE` (`LFGListRolesDocumentation.lua:50-55`).
/// INFERRED: `limitToClassRoles` intersects with the player's available
/// roles, which the simulator models as all three (`UnitGetAvailableRoles`).
fn set_roles(state: &mut LuaState) -> LuaResult<u32> {
    let roles = read_roles(state, stack_val(state, 1), "C_LFGListRoles.SetRoles")?;
    if !matches!(stack_val(state, 2), Val::Nil | Val::Bool(_)) {
        return Err(runtime_error(
            "C_LFGListRoles.SetRoles: limitToClassRoles must be a boolean",
        ));
    }
    {
        let mut sim = borrow_state_mut(state)?;
        sim.lfg_list_roles.current = roles;
        sim.lfg_list_roles.saved = roles;
    }
    dispatch_event_now(state, "LFG_LIST_ROLE_UPDATE", &[])?;
    state.push(Val::Bool(true));
    Ok(1)
}

fn admin_set_saved_roles(state: &mut LuaState) -> LuaResult<u32> {
    let roles = read_roles(state, stack_val(state, 1), "A_Admin.SetSavedLFGListRoles")?;
    borrow_state_mut(state)?.lfg_list_roles.saved = roles;
    Ok(0)
}

fn read_roles(state: &mut LuaState, value: Val, api: &str) -> LuaResult<LfgRoles> {
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error(format!(
            "{api}: roles must be an LFGRoles table"
        )));
    }
    let mut read = |key: &str| match table_get(state, value, key) {
        Val::Bool(flag) => Ok(flag),
        _ => Err(runtime_error(format!(
            "{api}: roles.{key} must be a boolean"
        ))),
    };
    Ok(LfgRoles {
        tank: read("tank")?,
        healer: read("healer")?,
        dps: read("dps")?,
    })
}

fn push_roles(state: &mut LuaState, roles: LfgRoles) {
    let table = create_table(state);
    table_set(state, table, "tank", Val::Bool(roles.tank));
    table_set(state, table, "healer", Val::Bool(roles.healer));
    table_set(state, table, "dps", Val::Bool(roles.dps));
    state.push(table);
}
