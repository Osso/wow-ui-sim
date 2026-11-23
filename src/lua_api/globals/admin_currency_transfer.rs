//! A_Admin inputs for the account-currency transfer model.

use crate::c_api::currency_list::{CurrencyAccountSource, CurrencyTransferRules};
use crate::lua_api::methods::{borrow_state_mut, table_get};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function(
            "SetAccountCharacterCurrencyData",
            set_account_character_currency_data,
        )?
        .set_function("SetCurrencyTransferEnabled", set_currency_transfer_enabled)?
        .set_function("SetCurrencyTransferRules", set_currency_transfer_rules)
}

fn set_account_character_currency_data(state: &mut LuaState) -> LuaResult<u32> {
    let currency_id = i32::from_stack(state, 1)?;
    if currency_id <= 0 {
        return Err(runtime_error(
            "SetAccountCharacterCurrencyData: currencyID must be positive",
        ));
    }
    let sources = read_account_character_sources(state, stack_val(state, 2))?;
    borrow_state_mut(state)?
        .currency_list
        .account_currency_sources
        .insert(currency_id, sources);
    Ok(0)
}

fn read_account_character_sources(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<Vec<CurrencyAccountSource>> {
    let Val::Table(table_ref) = value else {
        return Err(runtime_error(
            "SetAccountCharacterCurrencyData: sources must be a table",
        ));
    };
    let entries = state
        .gc
        .tables
        .get(table_ref)
        .ok_or_else(|| runtime_error("SetAccountCharacterCurrencyData: sources table unavailable"))?
        .array_slice()
        .to_vec();
    entries
        .into_iter()
        .map(|entry| read_account_character_source(state, entry))
        .collect()
}

fn read_account_character_source(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<CurrencyAccountSource> {
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error(
            "SetAccountCharacterCurrencyData: each source must be a table",
        ));
    }
    let character_guid: String = read_field(state, value, "characterGUID")?;
    if character_guid.is_empty() {
        return Err(runtime_error(
            "SetAccountCharacterCurrencyData: characterGUID must not be empty",
        ));
    }
    Ok(CurrencyAccountSource {
        character_guid,
        character_name: read_field(state, value, "characterName")?,
        full_character_name: read_field(state, value, "fullCharacterName")?,
        quantity: read_account_character_quantity(state, value)?,
    })
}

fn read_account_character_quantity(state: &mut LuaState, value: Val) -> LuaResult<u64> {
    let quantity = read_field::<f64>(state, value, "quantity")?;
    if !quantity.is_finite()
        || quantity < 0.0
        || quantity.fract() != 0.0
        || quantity >= u64::MAX as f64
    {
        return Err(runtime_error(
            "SetAccountCharacterCurrencyData: quantity must be a nonnegative integer",
        ));
    }
    Ok(quantity as u64)
}

fn set_currency_transfer_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.currency_list.server_enabled = enabled;
    Ok(0)
}

fn set_currency_transfer_rules(state: &mut LuaState) -> LuaResult<u32> {
    let currency_id = i32::from_stack(state, 1)?;
    if currency_id <= 0 {
        return Err(runtime_error(
            "SetCurrencyTransferRules: currencyID must be positive",
        ));
    }
    let rules = read_currency_transfer_rules(state, stack_val(state, 2))?;
    borrow_state_mut(state)?
        .currency_list
        .transfer_rules
        .insert(currency_id, rules);
    Ok(0)
}

fn read_currency_transfer_rules(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<CurrencyTransferRules> {
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error(
            "SetCurrencyTransferRules: rules must be a table",
        ));
    }
    Ok(CurrencyTransferRules {
        faction_key_0: read_optional_bool(state, value, "faction_key_0")?,
        faction_key_1: read_optional_bool(state, value, "faction_key_1")?,
        award_condition_met: read_optional_bool(state, value, "award_condition_met")?,
        quantity_cap_reached: read_optional_bool(state, value, "quantity_cap_reached")?,
        weekly_cap_reached: read_optional_bool(state, value, "weekly_cap_reached")?,
    })
}

fn read_optional_bool(state: &mut LuaState, table: Val, key: &str) -> LuaResult<Option<bool>> {
    match table_get(state, table, key) {
        Val::Nil => Ok(None),
        Val::Bool(value) => Ok(Some(value)),
        _ => Err(runtime_error(format!(
            "SetCurrencyTransferRules: {key} must be a boolean or nil",
        ))),
    }
}

fn read_field<T: FromStack>(state: &mut LuaState, table: Val, key: &str) -> LuaResult<T> {
    let value = table_get(state, table, key);
    let top = state.top;
    state.push(value);
    let result = T::from_stack(state, -1);
    state.top = top;
    result
}
