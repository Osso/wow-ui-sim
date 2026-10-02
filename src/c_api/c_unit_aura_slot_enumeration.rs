//! Retail 12.0.5 slot enumeration argument boundary over the existing aura store.

use crate::lua_api::globals::auras::{collect_visible_unit_auras, filter_from_str};
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const API_NAME: &str = "C_UnitAuras.GetAuraSlots";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(state, namespace, "GetAuraSlots", get_aura_slots)
}

fn get_aura_slots(state: &mut LuaState) -> LuaResult<u32> {
    // Authenticate every position before validation, lookup or termination,
    // including unused maxSlots. Inputs remain rooted on the VM argument stack;
    // unwrap_secret reads payloads without changing wrappers or caller taint.
    let unit = unwrap_secret(state, stack_val(state, 1))?;
    let filter = unwrap_secret(state, stack_val(state, 2))?;
    let max_slots = unwrap_secret(state, stack_val(state, 3))?;
    let token = unwrap_secret(state, stack_val(state, 4))?;
    let unit = read_required_string(state, unit, 1)?;
    let filter = match filter {
        Val::Nil => String::new(),
        value => read_required_string(state, value, 2)?,
    };
    read_optional_number(max_slots, 3)?;
    let token = read_optional_number(token, 4)?;

    // Retained simulator behavior, not native pagination: maxSlots is unused
    // and any supplied finite token terminates after all argument validation.
    if token.is_some() {
        return Ok(0);
    }
    let auras = collect_visible_unit_auras(state, &unit, filter_from_str(&filter));
    state.push(Val::Nil);
    for aura in &auras {
        state.push(Val::Num(aura.aura_instance_id as f64));
    }
    Ok(auras.len() as u32 + 1)
}

fn read_required_string(state: &LuaState, value: Val, position: i32) -> LuaResult<String> {
    // INFERRED: actual UTF-8 STRING, without defaults or numeric coercion.
    val_to_string(state, value).ok_or_else(|| {
        runtime_error(format!(
            "{API_NAME}: argument {position} must be a UTF-8 string"
        ))
    })
}

fn read_optional_number(value: Val, position: i32) -> LuaResult<Option<f64>> {
    // INFERRED: nil or finite actual f64; no integral, positive or i32 cap.
    match value {
        Val::Nil => Ok(None),
        Val::Num(number) if number.is_finite() => Ok(Some(number)),
        _ => Err(runtime_error(format!(
            "{API_NAME}: argument {position} must be nil or a finite number"
        ))),
    }
}
