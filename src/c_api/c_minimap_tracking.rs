//! Retail minimap tracking-default query.

use super::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::val_to_string;
use crate::lua_api::workarounds::RETAIL_TRACKING_DEFAULT_MASK;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE == ClientProfile::Retail {
        let namespace = ensure_namespace(state, "C_Minimap")?;
        // Retail MinimapDocumentation.lua:28.
        table_set_rust_fn_static(state, namespace, "GetDefaultTrackingValue", query_default)?;
    }
    Ok(())
}

fn query_default(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Nil => return Err(runtime_error("GetDefaultTrackingValue requires filterType")),
        Val::Num(number) => number,
        Val::Str(_) => val_to_string(state, value)
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.0),
        _ => 0.0,
    };
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(
            "GetDefaultTrackingValue filterType integer overflow",
        ));
    }
    let filter = number as i32;
    if !(0..=0x7fffff).contains(&filter) {
        return Err(runtime_error(
            "GetDefaultTrackingValue filterType out of range",
        ));
    }
    let filter = filter as u32;
    state.push(Val::Bool(RETAIL_TRACKING_DEFAULT_MASK & filter == filter));
    Ok(1)
}
