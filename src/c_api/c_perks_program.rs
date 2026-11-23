//! Supplied Perks Program catalog, currency, and chest reward state.
//!
//! INFERRED R88ep (native evidence pending): requests deliver the current
//! supplied snapshot through the events consumed by the Blizzard UI.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Default)]
pub(crate) struct PerksProgramState {
    vendor_items: Vec<serde_json::Value>,
    currency_amount: f64,
    pending_chest_rewards: Vec<serde_json::Value>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_PerksProgram")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAvailableVendorItemIDs",
        get_vendor_item_ids,
    )?;
    table_set_rust_fn_static(state, namespace, "GetVendorItemInfo", get_vendor_item_info)?;
    table_set_rust_fn_static(state, namespace, "GetCurrencyAmount", get_currency_amount)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetPendingChestRewards",
        get_pending_chest_rewards,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "RequestPendingChestRewards",
        request_pending_chest_rewards,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetPerksProgramVendorItems", set_vendor_items)?
        .set_function("SetPerksProgramCurrencyAmount", set_currency_amount)?
        .set_function(
            "SetPerksProgramPendingChestRewards",
            set_pending_chest_rewards,
        )
}

fn get_vendor_item_ids(state: &mut LuaState) -> LuaResult<u32> {
    let vendor_items = borrow_state(state)?.perks_program.vendor_items.clone();
    let values = vendor_items
        .iter()
        .map(|item| {
            item.get("perksVendorItemID")
                .and_then(serde_json::Value::as_f64)
        })
        .map(|id| Val::Num(id.unwrap_or_default()));
    push_sequence(state, values.collect())
}

fn get_vendor_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let vendor_item_id = f64::from_stack(state, 1)?;
    let vendor_items = borrow_state(state)?.perks_program.vendor_items.clone();
    let info = vendor_items.iter().find(|item| {
        item.get("perksVendorItemID")
            .and_then(serde_json::Value::as_f64)
            == Some(vendor_item_id)
    });
    if let Some(info) = info {
        let value = json_to_lua_value(state, info);
        state.push(value);
        Ok(1)
    } else {
        Ok(0)
    }
}

fn get_currency_amount(state: &mut LuaState) -> LuaResult<u32> {
    let amount = borrow_state(state)?.perks_program.currency_amount;
    state.push(Val::Num(amount));
    Ok(1)
}

fn get_pending_chest_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let rewards = borrow_state(state)?
        .perks_program
        .pending_chest_rewards
        .clone();
    let values = rewards
        .iter()
        .map(|reward| json_to_lua_value(state, reward));
    push_sequence(state, values.collect())
}

fn request_pending_chest_rewards(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event(state, "CHEST_REWARDS_UPDATED_FROM_SERVER")?;
    dispatch_event(state, "PERKS_PROGRAM_DATA_REFRESH")?;
    Ok(0)
}

fn set_vendor_items(state: &mut LuaState) -> LuaResult<u32> {
    let items = read_json_array(state, 1, "Perks Program vendor items")?;
    borrow_state_mut(state)?.perks_program.vendor_items = items;
    dispatch_event(state, "PERKS_PROGRAM_DATA_REFRESH")?;
    Ok(0)
}

fn set_currency_amount(state: &mut LuaState) -> LuaResult<u32> {
    let amount = f64::from_stack(state, 1)?;
    let previous = {
        let mut sim = borrow_state_mut(state)?;
        let previous = sim.perks_program.currency_amount;
        sim.perks_program.currency_amount = amount;
        previous
    };
    crate::lua_api::globals::lfg_list::fire_event_with_args(
        state,
        "PERKS_PROGRAM_CURRENCY_REFRESH",
        vec![Val::Num(previous), Val::Num(amount)],
    )?;
    Ok(0)
}

fn set_pending_chest_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let rewards = read_json_array(state, 1, "Perks Program pending chest rewards")?;
    borrow_state_mut(state)?.perks_program.pending_chest_rewards = rewards;
    dispatch_event(state, "CHEST_REWARDS_UPDATED_FROM_SERVER")?;
    Ok(0)
}

fn read_json_array(
    state: &LuaState,
    index: i32,
    description: &str,
) -> LuaResult<Vec<serde_json::Value>> {
    let input = String::from_stack(state, index)?;
    let value: serde_json::Value = serde_json::from_str(&input)
        .map_err(|error| runtime_error(format!("invalid {description} JSON: {error}")))?;
    value
        .as_array()
        .cloned()
        .ok_or_else(|| runtime_error(format!("{description} must be a JSON array")))
}

fn dispatch_event(state: &mut LuaState, event: &'static str) -> LuaResult<()> {
    crate::lua_api::globals::lfg_list::fire_event_with_args(state, event, Vec::new())
}

fn push_sequence(state: &mut LuaState, values: Vec<Val>) -> LuaResult<u32> {
    let sequence = create_table(state);
    let Val::Table(table) = sequence else {
        unreachable!("new Lua table must be a table")
    };
    for (index, value) in values.into_iter().enumerate() {
        crate::lua_api::methods::table_set_num(state, table, (index + 1) as f64, value);
    }
    state.push(sequence);
    Ok(1)
}

fn json_to_lua_value(state: &mut LuaState, value: &serde_json::Value) -> Val {
    match value {
        serde_json::Value::Null => Val::Nil,
        serde_json::Value::Bool(value) => Val::Bool(*value),
        serde_json::Value::Number(value) => Val::Num(value.as_f64().unwrap_or_default()),
        serde_json::Value::String(value) => create_string(state, value),
        serde_json::Value::Array(values) => json_array_to_lua_table(state, values),
        serde_json::Value::Object(values) => json_object_to_lua_table(state, values),
    }
}

fn json_array_to_lua_table(state: &mut LuaState, values: &[serde_json::Value]) -> Val {
    let table = create_table(state);
    let Val::Table(reference) = table else {
        unreachable!("new Lua table must be a table")
    };
    for (index, value) in values.iter().enumerate() {
        let value = json_to_lua_value(state, value);
        crate::lua_api::methods::table_set_num(state, reference, (index + 1) as f64, value);
    }
    table
}

fn json_object_to_lua_table(
    state: &mut LuaState,
    values: &serde_json::Map<String, serde_json::Value>,
) -> Val {
    let table = create_table(state);
    for (key, value) in values {
        let value = json_to_lua_value(state, value);
        table_set(state, table, key.as_str(), value);
    }
    table
}
