//! Profile-specific `C_PlayerInfo.GUIDIsPlayer` behavior.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::globals::unit_misc::{existing_guid_for_unit, party_guid_for_index};
use crate::lua_api::globals::unit_probes::resolve_unit_is_player;
use crate::lua_api::methods::{borrow_state, table_set_static, val_to_string};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const FOREVER_USAGE: &str = "Usage: local isPlayer = C_PlayerInfo.GUIDIsPlayer(guid)";
const FOREVER_SECRET_ACCESS_ERROR: &str =
    "Secret values are only allowed during untainted execution for this argument.";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }

    let namespace = ensure_namespace(state, "C_PlayerInfo")?;
    table_set_rust_fn_static(state, namespace, "GUIDIsPlayer", guid_is_player)
}

/// Apply Forever's Game API installer surface to the selected screen.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) {
    if ACTIVE != ClientProfile::WowForever {
        return;
    }

    let mut lua = env.lua.borrow_mut();
    update_registration(lua.state_mut(), !is_glue)
        .expect("register C_PlayerInfo.GUIDIsPlayer for the selected screen");
}

fn update_registration(state: &mut LuaState, should_register: bool) -> LuaResult<()> {
    let namespace = match LuaApiMut::get_global_val(state, "C_PlayerInfo") {
        Val::Table(namespace) => namespace,
        _ if should_register => ensure_namespace(state, "C_PlayerInfo")?,
        _ => return Ok(()),
    };

    if should_register {
        table_set_rust_fn_static(state, namespace, "GUIDIsPlayer", guid_is_player)
    } else {
        table_set_static(state, Val::Table(namespace), "GUIDIsPlayer", Val::Nil);
        Ok(())
    }
}

fn guid_is_player(state: &mut LuaState) -> LuaResult<u32> {
    let argument = if ACTIVE == ClientProfile::WowForever {
        unwrap_forever_guid_argument(state)?
    } else {
        rilua::table_security::unwrap_secret(state, stack_val(state, 1))?
    };

    let is_player = if ACTIVE == ClientProfile::WowForever {
        parse_forever_player_guid(&forever_guid_argument_bytes(state, argument)?)
    } else {
        let guid = val_to_string(state, argument).ok_or_else(|| {
            runtime_error("C_PlayerInfo.GUIDIsPlayer: argument 1 must be a UTF-8 string")
        })?;
        let sim = borrow_state(state)?;
        represented_guid_is_player(&sim, &guid)
    };

    state.push(Val::Bool(is_player));
    Ok(1)
}

fn unwrap_forever_guid_argument(state: &LuaState) -> LuaResult<Val> {
    rilua::table_security::unwrap_secret(state, stack_val(state, 1))
        .map_err(|_| runtime_error(format!("{FOREVER_USAGE}. {FOREVER_SECRET_ACCESS_ERROR}")))
}

fn forever_guid_argument_bytes(state: &LuaState, argument: Val) -> LuaResult<Vec<u8>> {
    match argument {
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .map(|value| value.data().to_vec())
            .ok_or_else(|| runtime_error(FOREVER_USAGE)),
        Val::Num(number) => Ok(number.to_string().into_bytes()),
        _ => Err(runtime_error(FOREVER_USAGE)),
    }
}

fn parse_forever_player_guid(guid: &[u8]) -> bool {
    const PLAYER_PREFIX: &[u8] = b"Player-";
    let Some(player_fields) = guid.strip_prefix(PLAYER_PREFIX) else {
        return false;
    };
    player_fields
        .iter()
        .take_while(|byte| byte.is_ascii_hexdigit())
        .next()
        .is_some()
}

fn represented_guid_is_player(sim: &SimState, guid: &str) -> bool {
    ["player", "target", "focus"].into_iter().any(|unit| {
        resolve_unit_is_player(sim, unit)
            && existing_guid_for_unit(sim, unit).as_deref() == Some(guid)
    }) || (sim.party_group_active
        && sim
            .party_members
            .iter()
            .enumerate()
            .any(|(index, _)| party_guid_for_index(index) == guid))
}
