//! Supplied Perks Program catalog, currency, and chest reward state.
//!
//! INFERRED R88ep: response event names and order are unknown; eligible
//! simulator requests emit the consumer refresh events.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// INFERRED R88ep: the unsupplied account state is unknown. The simulator uses
/// an empty catalog, zero currency, and disabled request conditions by default.
#[derive(Debug, Default)]
pub(crate) struct PerksProgramState {
    vendor_items: Vec<serde_json::Value>,
    currency_amount: i64,
    pending_chest_rewards: Vec<serde_json::Value>,
    feature_enabled: bool,
    client_available: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    // INFERRED R88ep: API documentation declares table results, but native result
    // shapes remain unknown. Forever declares vendor IDs; no handler was found.
    let namespace = super::ensure_namespace(state, "C_PerksProgram")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAvailableVendorItemIDs",
        read_vendor_item_ids,
    )?;
    table_set_rust_fn_static(state, namespace, "GetVendorItemInfo", find_vendor_item_info)?;
    table_set_rust_fn_static(state, namespace, "GetCurrencyAmount", read_currency_amount)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetPendingChestRewards",
        read_pending_chest_rewards,
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
        .set_function("SetPerksProgramFeatureEnabled", set_feature_enabled)?
        .set_function("SetPerksProgramClientAvailable", set_client_available)?
        .set_function(
            "SetPerksProgramPendingChestRewards",
            set_pending_chest_rewards,
        )
}

fn read_vendor_item_ids(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED R88ep (Retail): no arguments; IDs come from runtime Perks state.
    // INFERRED R88ep: expose them with the documented table result shape.
    let vendor_items = borrow_state(state)?.perks_program.vendor_items.clone();
    let values = vendor_items
        .iter()
        .filter_map(|item| {
            item.get("perksVendorItemID")
                .and_then(serde_json::Value::as_f64)
        })
        .map(Val::Num)
        .collect();
    push_sequence(state, values)
}

fn find_vendor_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let vendor_item_id = f64::from_stack(state, 1)?;
    let vendor_items = borrow_state(state)?.perks_program.vendor_items.clone();
    let info = vendor_items.iter().find(|item| {
        item.get("perksVendorItemID")
            .and_then(serde_json::Value::as_f64)
            == Some(vendor_item_id)
    });
    if let Some(info) = info {
        let value = allocate_lua_value_from_json(state, info);
        state.push(value);
        Ok(1)
    } else {
        Ok(0)
    }
}

fn read_currency_amount(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED R88ep: no arguments; publish one integer, or zero without an owner/player.
    let amount = borrow_state(state)?.perks_program.currency_amount;
    state.push(Val::Num(amount as f64));
    Ok(1)
}

fn read_pending_chest_rewards(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED R88ep: the getter reads runtime state; the exact Lua payload is unknown.
    // INFERRED R88ep: use the documented table shape for supplied records.
    let rewards = borrow_state(state)?
        .perks_program
        .pending_chest_rewards
        .clone();
    push_json_sequence(state, &rewards)
}

fn request_pending_chest_rewards(state: &mut LuaState) -> LuaResult<u32> {
    // VERIFIED R88ep: no arguments and zero returns. Retail requires a connection;
    // Forever also requires the Perks feature to be enabled.
    let (feature_enabled, client_available) = {
        let state = borrow_state(state)?;
        (
            state.perks_program.feature_enabled,
            state.perks_program.client_available,
        )
    };
    if !client_available {
        return Ok(0);
    }
    if ACTIVE == ClientProfile::WowForever && !feature_enabled {
        return Ok(0);
    }

    // INFERRED R88ep: native response event and order are unknown.
    dispatch_event(state, "CHEST_REWARDS_UPDATED_FROM_SERVER")?;
    dispatch_event(state, "PERKS_PROGRAM_DATA_REFRESH")?;
    Ok(0)
}

fn set_vendor_items(state: &mut LuaState) -> LuaResult<u32> {
    let items = read_json_array(state, 1, "Perks Program vendor items")?;
    borrow_state_mut(state)?.perks_program.vendor_items = items;
    // INFERRED R88ep: notify the vendor consumer after supplied catalog changes.
    dispatch_event(state, "PERKS_PROGRAM_DATA_REFRESH")?;
    Ok(0)
}

fn set_currency_amount(state: &mut LuaState) -> LuaResult<u32> {
    let amount = i64::from_stack(state, 1)?;
    let previous = {
        let mut sim = borrow_state_mut(state)?;
        let previous = sim.perks_program.currency_amount;
        sim.perks_program.currency_amount = amount;
        previous
    };
    // INFERRED R88ep: update the currency consumer with old and new amounts.
    crate::lua_api::globals::lfg_list::fire_event_with_args(
        state,
        "PERKS_PROGRAM_CURRENCY_REFRESH",
        vec![Val::Num(previous as f64), Val::Num(amount as f64)],
    )?;
    Ok(0)
}

fn set_feature_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.perks_program.feature_enabled = enabled;
    Ok(0)
}

fn set_client_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.perks_program.client_available = available;
    Ok(0)
}

fn set_pending_chest_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let rewards = read_json_array(state, 1, "Perks Program pending chest rewards")?;
    borrow_state_mut(state)?.perks_program.pending_chest_rewards = rewards;
    // INFERRED R88ep: notify the pending reward consumer after supplied data changes.
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

fn push_json_sequence(state: &mut LuaState, values: &[serde_json::Value]) -> LuaResult<u32> {
    let sequence = create_table(state);
    let Val::Table(table) = sequence else {
        unreachable!("new Lua table must be a table")
    };
    for (index, value) in values.iter().enumerate() {
        let value = allocate_lua_value_from_json(state, value);
        crate::lua_api::methods::table_set_num(state, table, (index + 1) as f64, value);
    }
    state.push(sequence);
    Ok(1)
}

fn allocate_lua_value_from_json(state: &mut LuaState, value: &serde_json::Value) -> Val {
    match value {
        serde_json::Value::Null => Val::Nil,
        serde_json::Value::Bool(value) => Val::Bool(*value),
        serde_json::Value::Number(value) => Val::Num(
            value
                .as_f64()
                .expect("serde_json numbers must convert to Lua numbers"),
        ),
        serde_json::Value::String(value) => create_string(state, value),
        serde_json::Value::Array(values) => allocate_lua_table_from_json_array(state, values),
        serde_json::Value::Object(values) => allocate_lua_table_from_json_object(state, values),
    }
}

fn allocate_lua_table_from_json_array(state: &mut LuaState, values: &[serde_json::Value]) -> Val {
    let table = create_table(state);
    let Val::Table(reference) = table else {
        unreachable!("new Lua table must be a table")
    };
    for (index, value) in values.iter().enumerate() {
        let value = allocate_lua_value_from_json(state, value);
        crate::lua_api::methods::table_set_num(state, reference, (index + 1) as f64, value);
    }
    table
}

fn allocate_lua_table_from_json_object(
    state: &mut LuaState,
    values: &serde_json::Map<String, serde_json::Value>,
) -> Val {
    let table = create_table(state);
    for (key, value) in values {
        let value = allocate_lua_value_from_json(state, value);
        table_set(state, table, key.as_str(), value);
    }
    table
}
