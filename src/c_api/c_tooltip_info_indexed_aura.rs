//! Retail 12.0.5 indexed-aura tooltip input boundaries.
//! Lookup/default/miss policies are inferred; access/restricted output remain unmodeled.

use crate::lua_api::globals::auras::{
    AuraFilter, aura_matches_filter_string, collect_visible_unit_auras,
};
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_selected_indexed_aura;
use crate::lua_api::methods::val_to_string;
use crate::lua_api::state::AuraInfo;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// Publish only into the existing, globally rooted namespace.
pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetUnitBuff", get_unit_buff)?;
    table_set_rust_fn_static(state, namespace, "GetUnitDebuff", get_unit_debuff)
}

fn get_unit_buff(state: &mut LuaState) -> LuaResult<u32> {
    get_indexed_aura(state, "C_TooltipInfo.GetUnitBuff", AuraFilter::Helpful)
}

fn get_unit_debuff(state: &mut LuaState) -> LuaResult<u32> {
    get_indexed_aura(state, "C_TooltipInfo.GetUnitDebuff", AuraFilter::Harmful)
}

fn get_indexed_aura(state: &mut LuaState, api_name: &str, polarity: AuraFilter) -> LuaResult<u32> {
    let arguments = authenticate_arguments(state, api_name)?;
    let unit = parse_string(state, arguments[0], 1, api_name)?;
    let index = parse_index(arguments[1], api_name)?;
    let filter = parse_filter(state, arguments[2], polarity, api_name)?;
    let aura = select_indexed_aura(state, &unit, index, &filter, polarity);
    let tooltip = tooltip_for_selected_indexed_aura(state, aura);
    // Publish the builder result immediately, before another VM allocation.
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

fn parse_string(state: &LuaState, value: Val, position: i32, api_name: &str) -> LuaResult<String> {
    val_to_string(state, value).ok_or_else(|| {
        rilua::runtime_error(format!(
            "{api_name}: argument {position} requires a UTF-8 string"
        ))
    })
}

fn parse_index(value: Val, api_name: &str) -> LuaResult<i32> {
    if let Val::Num(number) = value {
        if number.is_finite() && f64::from(number as i32) == number {
            return Ok(number as i32);
        }
    }
    Err(rilua::runtime_error(format!(
        "{api_name}: argument 2 requires a finite integral i32 number"
    )))
}

fn parse_filter(
    state: &LuaState,
    value: Val,
    polarity: AuraFilter,
    api_name: &str,
) -> LuaResult<String> {
    if matches!(value, Val::Nil) {
        let default = if matches!(polarity, AuraFilter::Harmful) {
            "HARMFUL"
        } else {
            "HELPFUL"
        };
        return Ok(default.to_owned());
    }
    parse_string(state, value, 3, api_name)
}

fn select_indexed_aura(
    state: &mut LuaState,
    unit: &str,
    index: i32,
    filter: &str,
    polarity: AuraFilter,
) -> Option<AuraInfo> {
    if index <= 0 {
        return None;
    }
    // The helper fabricates target auras; they are not live Debuff backing state.
    if matches!(polarity, AuraFilter::Harmful) && unit == "target" {
        return None;
    }
    let helpful = matches!(polarity, AuraFilter::Helpful);
    let context = super::aura_filter::AuraFilterContext::for_unit(state, unit);
    collect_visible_unit_auras(state, unit, polarity)
        .into_iter()
        // Both party stores can contain malformed polarity; enforce the API's polarity.
        .filter(|aura| {
            aura.is_helpful == helpful && aura_matches_filter_string(aura, filter, &context)
        })
        .nth((index - 1) as usize)
}
