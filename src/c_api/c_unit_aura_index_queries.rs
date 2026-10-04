//! Retail 12.0.5 indexed aura argument boundaries over the existing aura model.

use super::unit_aura_access::require_unit_aura_access;
use crate::lua_api::globals::auras::{AuraFilter, filter_from_str, push_aura_at_filtered_index};
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAuraDataByIndex",
        get_aura_data_by_index,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetBuffDataByIndex",
        get_buff_data_by_index,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetDebuffDataByIndex",
        get_debuff_data_by_index,
    )
}

fn get_aura_data_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let (unit, index, filter) = read_index_arguments(state, "GetAuraDataByIndex")?;
    push_aura_at_filtered_index(state, &unit, filter_from_str(&filter), index)?;
    Ok(1)
}

fn get_buff_data_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let (unit, index, _) = read_index_arguments(state, "GetBuffDataByIndex")?;
    push_aura_at_filtered_index(state, &unit, AuraFilter::Helpful, index)?;
    Ok(1)
}

fn get_debuff_data_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let (unit, index, _) = read_index_arguments(state, "GetDebuffDataByIndex")?;
    push_aura_at_filtered_index(state, &unit, AuraFilter::Harmful, index)?;
    Ok(1)
}

fn read_index_arguments(state: &LuaState, name: &str) -> LuaResult<(String, i32, String)> {
    require_unit_aura_access(state, &format!("C_UnitAuras.{name}"))?;
    // AllowedWhenUntainted applies to all three documented positions, even the
    // wrapper filter. VM authentication retains caller taint and input secrecy.
    // UnitAuraAccess and restricted-output secrecy remain unmodeled.
    let unit = unwrap_secret(state, stack_val(state, 1))?;
    let index = unwrap_secret(state, stack_val(state, 2))?;
    let filter = unwrap_secret(state, stack_val(state, 3))?;
    let unit = read_required_string(state, name, unit, 1)?;
    let index = read_index(name, index)?;
    let filter = match filter {
        Val::Nil => String::new(),
        value => read_required_string(state, name, value, 3)?,
    };
    Ok((unit, index, filter))
}

fn read_required_string(
    state: &LuaState,
    name: &str,
    value: Val,
    position: i32,
) -> LuaResult<String> {
    // INFERRED representation policy: actual UTF-8 STRING, not numeric coercion.
    val_to_string(state, value).ok_or_else(|| {
        runtime_error(format!(
            "C_UnitAuras.{name}: argument {position} must be a UTF-8 string"
        ))
    })
}

fn read_index(name: &str, value: Val) -> LuaResult<i32> {
    // INFERRED representation policy: signed indices are valid, lossy casts are not.
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
            "C_UnitAuras.{name}: argument 2 must be a finite integral i32 number"
        ))),
    }
}
