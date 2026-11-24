//! A_Admin input for the account type and its restricted-account limits.

use crate::lua_api::methods::{borrow_state_mut, table_get};
use crate::lua_api::state_types::AccountTypeState;
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetAccountType", set_account_type)
}

fn set_account_type(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    if !matches!(value, Val::Table(_)) {
        return Err(runtime_error("SetAccountType: expected an account table"));
    }
    let account_type = read_account_type(state, value)?;
    borrow_state_mut(state)?.account_type = account_type;
    Ok(0)
}

fn read_account_type(state: &mut LuaState, value: Val) -> LuaResult<AccountTypeState> {
    let account_type = AccountTypeState {
        account_type: read_field(state, value, "accountType")?,
        restricted: read_field(state, value, "restricted")?,
        expansion_trial: read_field(state, value, "expansionTrial")?,
        expansion_trial_expiry: read_field(state, value, "expansionTrialExpiry")?,
        restricted_level: read_field(state, value, "restrictedLevel")?,
        restricted_money: read_field(state, value, "restrictedMoney")?,
    };
    validate_account_type(&account_type)?;
    Ok(account_type)
}

fn validate_account_type(account_type: &AccountTypeState) -> LuaResult<()> {
    if !account_type.expansion_trial_expiry.is_finite() || account_type.expansion_trial_expiry < 0.0
    {
        return Err(runtime_error(
            "SetAccountType: expansionTrialExpiry must be finite and nonnegative",
        ));
    }
    if account_type.restricted_level < 0 {
        return Err(runtime_error(
            "SetAccountType: restrictedLevel must be nonnegative",
        ));
    }
    if account_type.restricted_money < 0 {
        return Err(runtime_error(
            "SetAccountType: restrictedMoney must be nonnegative",
        ));
    }
    Ok(())
}

fn read_field<T: FromStack>(state: &mut LuaState, table: Val, key: &str) -> LuaResult<T> {
    let value = table_get(state, table, key);
    let top = state.top;
    state.push(value);
    let parsed = T::from_stack(state, -1);
    state.top = top;
    parsed
}
