//! Retail 12.0.5 indexed-buff tooltip input boundary.
//! Helpful lookup/miss policies are inferred; restricted output remains unmodeled.

use crate::lua_api::globals::auras::{
    AuraFilter, aura_matches_filter_string, collect_visible_unit_auras,
};
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_selected_unit_buff;
use crate::lua_api::methods::val_to_string;
use crate::lua_api::state::AuraInfo;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const API_NAME: &str = "C_TooltipInfo.GetUnitBuff";

/// Publish only into the existing, globally rooted namespace.
pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetUnitBuff", get_unit_buff)
}

fn get_unit_buff(state: &mut LuaState) -> LuaResult<u32> {
    let arguments = authenticate_arguments(state)?;
    let unit = parse_string(state, arguments[0], 1)?;
    let index = parse_index(arguments[1])?;
    let filter = parse_filter(state, arguments[2])?;
    let aura = select_unit_buff(state, &unit, index, &filter);
    let tooltip = tooltip_for_selected_unit_buff(state, aura);
    // Publish the builder result immediately, before another VM allocation.
    state.push(tooltip);
    Ok(1)
}

fn authenticate_arguments(state: &LuaState) -> LuaResult<[Val; 3]> {
    let mut arguments = [Val::Nil; 3];
    for (offset, argument) in arguments.iter_mut().enumerate() {
        let position = offset as i32 + 1;
        *argument = unwrap_secret(state, stack_val(state, position)).map_err(|error| {
            rilua::runtime_error(format!("{API_NAME}: argument {position}: {error}"))
        })?;
    }
    Ok(arguments)
}

fn parse_string(state: &LuaState, value: Val, position: i32) -> LuaResult<String> {
    val_to_string(state, value).ok_or_else(|| {
        rilua::runtime_error(format!(
            "{API_NAME}: argument {position} requires a UTF-8 string"
        ))
    })
}

fn parse_index(value: Val) -> LuaResult<i32> {
    if let Val::Num(number) = value {
        if number.is_finite() && f64::from(number as i32) == number {
            return Ok(number as i32);
        }
    }
    Err(rilua::runtime_error(format!(
        "{API_NAME}: argument 2 requires a finite integral i32 number"
    )))
}

fn parse_filter(state: &LuaState, value: Val) -> LuaResult<String> {
    if matches!(value, Val::Nil) {
        return Ok("HELPFUL".to_owned());
    }
    parse_string(state, value, 3)
}

fn select_unit_buff(
    state: &mut LuaState,
    unit: &str,
    index: i32,
    filter: &str,
) -> Option<AuraInfo> {
    if index <= 0 {
        return None;
    }
    collect_visible_unit_auras(state, unit, AuraFilter::Helpful)
        .into_iter()
        // Party buffs can contain malformed polarity; Buff remains helpful-only.
        .filter(|aura| aura.is_helpful && aura_matches_filter_string(aura, filter))
        .nth((index - 1) as usize)
}
