//! Retail 12.0.5 slot argument boundary over the existing aura lookup/DTO.

use super::unit_aura_access::require_unit_aura_access;
use crate::lua_api::globals::auras::push_aura_by_instance_id;
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const API_NAME: &str = "C_UnitAuras.GetAuraDataBySlot";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(state, namespace, "GetAuraDataBySlot", get_aura_data_by_slot)
}

fn get_aura_data_by_slot(state: &mut LuaState) -> LuaResult<u32> {
    require_unit_aura_access(state, API_NAME)?;
    let unit = read_public_unit(state)?;
    let slot = read_authenticated_slot(state)?;
    // Validate before the existing blocked-inclusive lookup. Native aura access
    // and restricted-output secrecy remain unmodeled.
    push_aura_by_instance_id(state, &unit, slot)?;
    Ok(1)
}

fn read_public_unit(state: &LuaState) -> LuaResult<String> {
    let value = stack_val(state, 1);
    // NeverSecret remains independent of caller security; only actual VM
    // wrappers are secrets. Do not unwrap this argument, even for secure callers.
    if is_secret_value(state, value) {
        return Err(runtime_error(format!(
            "{API_NAME}: argument 1 must not be secret"
        )));
    }
    // INFERRED: required actual UTF-8 STRING, no default or numeric coercion.
    val_to_string(state, value)
        .ok_or_else(|| runtime_error(format!("{API_NAME}: argument 1 must be a UTF-8 string")))
}

fn read_authenticated_slot(state: &LuaState) -> LuaResult<i32> {
    // VM authentication preserves caller taint and the original secret input.
    let value = unwrap_secret(state, stack_val(state, 2))?;
    // INFERRED: finite integral signed-i32 representation, without a positive cap.
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= i32::MIN as f64
                && number <= i32::MAX as f64 =>
        {
            Ok(number as i32)
        }
        _ => Err(runtime_error(format!(
            "{API_NAME}: argument 2 must be a finite integral i32 number"
        ))),
    }
}
