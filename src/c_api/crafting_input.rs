//! Typed input validation. No legacy itemID/reagentItems alternate path.
use super::crafting_reagents::*;
use super::crafting_tables::read_field;
use crate::lua_api::methods::val_to_string;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn read_array(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let value = rilua::table_security::unwrap_secret(state, value)?;
    let Val::Table(reference) = value else {
        return Err(runtime_error("crafting array required"));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("crafting array collected"))?;
    let len = table.len(&state.gc.string_arena);
    Ok((1..=len).map(|index| table.get_int(index as i64)).collect())
}
pub(crate) fn integer(value: Val, label: &str, min: i32) -> LuaResult<i32> {
    match value {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && value >= f64::from(min)
                && value <= f64::from(i32::MAX) =>
        {
            Ok(value as i32)
        }
        _ => Err(runtime_error(format!(
            "{label} must be an integer >= {min}"
        ))),
    }
}
pub(crate) fn read_identity(state: &mut LuaState, value: Val) -> LuaResult<CraftingReagent> {
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error("nested crafting reagent required"));
    }
    let item = read_field(state, value, "itemID")?;
    let currency = read_field(state, value, "currencyID")?;
    // INFERRED: consumers branch on one identity; ambiguous/neither identities are rejected.
    match (item, currency) {
        (Val::Nil, Val::Num(_)) => Ok(CraftingReagent::Currency(
            integer(currency, "currencyID", 1)? as u32,
        )),
        (Val::Num(_), Val::Nil) => Ok(CraftingReagent::Item(integer(item, "itemID", 1)? as u32)),
        _ => Err(runtime_error(
            "crafting reagent requires exactly one itemID or currencyID",
        )),
    }
}
pub(crate) fn read_regular(state: &mut LuaState, value: Val) -> LuaResult<RegularReagentInfo> {
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error("reagent info table required"));
    }
    let nested = read_field(state, value, "reagent")?;
    let reagent = read_identity(state, nested)?;
    let quantity = integer(read_field(state, value, "quantity")?, "quantity", 1)?;
    Ok(RegularReagentInfo { reagent, quantity })
}
pub(crate) fn read_allocation(state: &mut LuaState, value: Val) -> LuaResult<CraftingReagentInfo> {
    let regular = read_regular(state, value)?;
    let data_slot_index = integer(
        read_field(state, value, "dataSlotIndex")?,
        "dataSlotIndex",
        1,
    )?;
    Ok(CraftingReagentInfo {
        reagent: regular.reagent,
        quantity: regular.quantity,
        data_slot_index,
    })
}
pub(crate) fn read_allocations(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<Vec<CraftingReagentInfo>> {
    read_array(state, value)?
        .into_iter()
        .map(|value| read_allocation(state, value))
        .collect()
}
fn optional_integer(state: &mut LuaState, table: Val, key: &str) -> LuaResult<Option<i32>> {
    let value = read_field(state, table, key)?;
    if matches!(value, Val::Nil) {
        Ok(None)
    } else {
        integer(value, key, 1).map(Some)
    }
}
fn optional_text(state: &mut LuaState, table: Val, key: &str) -> LuaResult<Option<String>> {
    let value = read_field(state, table, key)?;
    if matches!(value, Val::Nil) {
        return Ok(None);
    }
    if !matches!(value, Val::Str(_)) {
        return Err(runtime_error(format!("{key} must be a string")));
    }
    Ok(val_to_string(state, value))
}
pub(crate) fn read_order(state: &mut LuaState, table: Val) -> LuaResult<NewCraftingOrderInfo> {
    if !matches!(table, Val::Table(_)) {
        return Err(runtime_error("NewCraftingOrderInfo required"));
    }
    if !matches!(read_field(state, table, "reagentItems")?, Val::Nil) {
        return Err(runtime_error("reagentItems retired; use reagentInfos"));
    }
    let skill_line_ability_id = integer(
        read_field(state, table, "skillLineAbilityID")?,
        "skillLineAbilityID",
        1,
    )?;
    let order_type = integer(read_field(state, table, "orderType")?, "orderType", 0)?;
    let order_duration = integer(
        read_field(state, table, "orderDuration")?,
        "orderDuration",
        0,
    )?;
    let Val::Num(tip_amount) = read_field(state, table, "tipAmount")? else {
        return Err(runtime_error("tipAmount must be money"));
    };
    if !tip_amount.is_finite() || tip_amount < 0.0 || tip_amount.fract() != 0.0 {
        return Err(runtime_error("invalid tipAmount"));
    }
    let customer_notes = optional_text(state, table, "customerNotes")?
        .ok_or_else(|| runtime_error("customerNotes required"))?;
    let regular = read_field(state, table, "reagentInfos")?;
    let reagent_infos = read_array(state, regular)?
        .into_iter()
        .map(|value| read_regular(state, value))
        .collect::<LuaResult<_>>()?;
    let crafting = read_field(state, table, "craftingReagentItems")?;
    let crafting_reagent_items = read_allocations(state, crafting)?;
    Ok(NewCraftingOrderInfo {
        skill_line_ability_id,
        order_type,
        order_duration,
        tip_amount,
        customer_notes,
        reagent_infos,
        crafting_reagent_items,
        min_crafting_quality_id: optional_integer(state, table, "minCraftingQualityID")?,
        order_target: optional_text(state, table, "orderTarget")?,
        recraft_item: optional_text(state, table, "recraftItem")?,
    })
}
