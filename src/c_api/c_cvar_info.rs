//! CVar information combines live simulator values with profile-pinned native flags.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::table_security::is_secret_value;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data_profile::cvar_info::CVAR_FLAGS;

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_CVar")?;
    table_set_rust_fn_static(state, namespace, "GetCVarInfo", get_cvar_info)
}

fn get_cvar_info(state: &mut LuaState) -> LuaResult<u32> {
    if is_secret_value(state, stack_val(state, 1)) {
        return Err(runtime_error("C_CVar.GetCVarInfo: name must not be secret"));
    }
    let name = String::from_stack(state, 1)?;
    let (value, default, flags) = {
        let sim = borrow_state(state)?;
        let Some(value) = sim.cvars.get(&name) else {
            return Ok(0);
        };
        let flags = if let Some(flags) = CVAR_FLAGS.get(name.to_lowercase().as_str()) {
            *flags
        } else if sim.cvars.is_runtime_registered(&name) {
            // RegisterCVar creates a local variable with no native server scope.
            0
        } else {
            // A stale/default-only name is not evidence of native CVar metadata.
            return Ok(0);
        };
        (value, sim.cvars.get_default(&name), flags)
    };
    let value = create_string(state, &value);
    state.push(value);
    let default = default.map_or(Val::Nil, |value| create_string(state, &value));
    state.push(default);
    for bit in [1, 2, 4, 8, 16] {
        state.push(Val::Bool(flags & bit != 0));
    }
    Ok(7)
}
