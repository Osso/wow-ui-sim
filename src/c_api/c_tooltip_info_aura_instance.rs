//! Retail 12.0.5 aura-instance tooltip input boundary.
//! Polarity/miss policies are inferred; restricted aura output remains unmodeled.

use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_aura_instance;
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// Publish only into the existing, globally rooted namespace.
pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetUnitAuraByAuraInstanceID", get_aura)?;
    table_set_rust_fn_static(state, namespace, "GetUnitBuffByAuraInstanceID", get_buff)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetUnitDebuffByAuraInstanceID",
        get_debuff,
    )?;
    Ok(())
}

fn get_aura(state: &mut LuaState) -> LuaResult<u32> {
    push_tooltip(state, "C_TooltipInfo.GetUnitAuraByAuraInstanceID", None)
}

fn get_buff(state: &mut LuaState) -> LuaResult<u32> {
    push_tooltip(
        state,
        "C_TooltipInfo.GetUnitBuffByAuraInstanceID",
        Some(true),
    )
}

fn get_debuff(state: &mut LuaState) -> LuaResult<u32> {
    push_tooltip(
        state,
        "C_TooltipInfo.GetUnitDebuffByAuraInstanceID",
        Some(false),
    )
}

fn push_tooltip(state: &mut LuaState, api_name: &str, helpful: Option<bool>) -> LuaResult<u32> {
    let arguments = authenticate_arguments(state, api_name)?;
    let unit = val_to_string(state, arguments[0]).ok_or_else(|| {
        rilua::runtime_error(format!("{api_name}: argument 1 requires a UTF-8 string"))
    })?;
    let instance_id = parse_instance_id(arguments[1], api_name)?;
    // Arg3 is authenticated above, but retains the provider's ignored-option behavior.
    let tooltip = tooltip_for_aura_instance(state, &unit, instance_id, helpful);
    // The bridge returns Val; publish immediately without another VM allocation.
    state.push(tooltip);
    Ok(1)
}

fn authenticate_arguments(state: &LuaState, api_name: &str) -> LuaResult<[Val; 3]> {
    let mut arguments = [Val::Nil; 3];
    for (offset, argument) in arguments.iter_mut().enumerate() {
        let position = offset as i32 + 1;
        *argument = unwrap_secret(state, stack_val(state, position)).map_err(|error| {
            rilua::runtime_error(format!("{api_name}: argument {position}: {error}"))
        })?;
    }
    Ok(arguments)
}

fn parse_instance_id(value: Val, api_name: &str) -> LuaResult<i32> {
    if let Val::Num(number) = value {
        if is_exact_i32(number) {
            return Ok(number as i32);
        }
    }
    Err(rilua::runtime_error(format!(
        "{api_name}: argument 2 requires a finite integral i32 number"
    )))
}

fn is_exact_i32(number: f64) -> bool {
    number.is_finite() && f64::from(number as i32) == number
}
