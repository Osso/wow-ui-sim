//! State-backed paragon reputation queries.

use crate::lua_api::methods::{
    borrow_state, call_function_state, table_get_static, table_set_static,
};
use crate::lua_bridge::stack_val;
use rilua::table_security::{can_access_secrets, is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// Mists fixture policy; native account-wide reputation behavior is unknown.
pub(crate) fn fixture_account_wide_default() -> bool {
    crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Mists
}

/// Seeded paragon payload; presence in `SimState.faction_paragon` marks a faction
/// as paragon. Storage level is explicit state, not derived from reputation.
#[derive(Clone, Debug)]
pub struct FactionParagonInfo {
    pub current_value: i32,
    pub threshold: i32,
    pub reward_quest_id: i32,
    pub has_reward_pending: bool,
    pub too_low_level_for_paragon: bool,
    pub paragon_storage_level: i32,
}

fn read_allowed_argument(state: &LuaState, value: Val, usage: &str) -> LuaResult<Val> {
    if is_secret_value(state, value) && !can_access_secrets(state) {
        return Err(runtime_error(format!(
            "{usage}. Secret values are only allowed during untainted execution for this argument."
        )));
    }
    unwrap_secret(state, value)
}

fn read_required_number(state: &mut LuaState, usage: &str) -> LuaResult<f64> {
    let value = read_allowed_argument(state, stack_val(state, 1), usage)?;
    if matches!(value, Val::Nil) {
        return Err(runtime_error(usage));
    }

    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => {
            let tonumber = crate::c_api::global_val(state, "tonumber");
            match call_function_state(state, tonumber, &[value])? {
                Val::Num(number) => Some(number),
                _ => None,
            }
        }
        _ => None,
    };
    number.ok_or_else(|| runtime_error(usage))
}

fn integer_table_value(state: &LuaState, table: Val, index: i32) -> Val {
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.get_int(i64::from(index)))
        .unwrap_or(Val::Nil)
}

fn is_account_wide_reputation_in_rows(state: &mut LuaState, rows: Val, faction_id: i32) -> bool {
    let mut index = 1;
    loop {
        let row = integer_table_value(state, rows, index);
        if row == Val::Nil {
            return false;
        }
        let is_header = matches!(table_get_static(state, row, "isHeader"), Val::Bool(true));
        let row_id = table_get_static(state, row, "factionID");
        if !is_header && row_id == Val::Num(f64::from(faction_id)) {
            return matches!(
                table_get_static(state, row, "isAccountWide"),
                Val::Bool(true)
            );
        }
        index += 1;
    }
}

// VERIFIED (Retail 12.1.5.70688, R88f.A.74): factionID is a required signed-i32
// input; classification reads the Faction row directly and missing rows return false.
pub(crate) fn is_account_wide_reputation(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str =
        "Usage: local isAccountWide = C_Reputation.IsAccountWideReputation(factionID)";
    let number = read_required_number(state, USAGE)?;
    if number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(format!(
            "outside of expected range {} to {}",
            i32::MIN,
            i32::MAX
        )));
    }
    if !number.is_finite() {
        return Err(runtime_error(USAGE));
    }

    let global = Val::Table(state.global);
    let reputation_state = table_get_static(state, global, "__wow_reputation_state");
    let rows = table_get_static(state, reputation_state, "factions");
    let is_account_wide = is_account_wide_reputation_in_rows(state, rows, number as i32);
    state.push(Val::Bool(is_account_wide));
    Ok(1)
}

// VERIFIED (R88f.A.80-A.81): the API index is one-based; native subtracts one
// before signed-integer truncation. The seeded Lua list remains one-based.
fn read_faction_sort_index(state: &mut LuaState, usage: &str) -> LuaResult<Option<i32>> {
    let index = read_required_number(state, usage)?;
    if index < 0.0 || index > f64::from(u32::MAX) {
        return Err(runtime_error(format!(
            "outside of expected range 0 to {}",
            u32::MAX
        )));
    }
    if !index.is_finite() {
        return Err(runtime_error(usage));
    }

    let internal_index = (index - 1.0).trunc();
    if internal_index < 0.0 || internal_index > f64::from(i32::MAX) {
        return Ok(None);
    }
    Ok(Some(internal_index as i32))
}

fn reputation_faction_at_index(state: &mut LuaState, index: i32) -> Val {
    let Some(row_index) = index.checked_add(1) else {
        return Val::Nil;
    };
    let global = Val::Table(state.global);
    let reputation_state = table_get_static(state, global, "__wow_reputation_state");
    let rows = table_get_static(state, reputation_state, "factions");
    integer_table_value(state, rows, row_index)
}

// INFERRED (R88f.A.80-A.81): reputationIndex is the scenario row's
// representation of the adjacent-build ReputationIndex field.
fn has_valid_activity_reputation_index(state: &mut LuaState, row: Val) -> bool {
    let Val::Num(index) = table_get_static(state, row, "reputationIndex") else {
        return false;
    };
    index.is_finite() && index.fract() == 0.0 && (0.0..=999.0).contains(&index)
}

// INFERRED fixture input: isInactive records the status marker. Native refresh
// and packet submission are not modeled in this seeded-state surface.
pub(crate) fn is_faction_active(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: local isActive = C_Reputation.IsFactionActive(factionSortIndex)";
    let index = read_faction_sort_index(state, USAGE)?;
    let row = index
        .map(|index| reputation_faction_at_index(state, index))
        .unwrap_or(Val::Nil);
    let active = if row == Val::Nil || !has_valid_activity_reputation_index(state, row) {
        true
    } else {
        !matches!(table_get_static(state, row, "isInactive"), Val::Bool(true))
    };
    state.push(Val::Bool(active));
    Ok(1)
}

// INFERRED fixture input: canSetInactive and isInactive select the modeled
// local status row; native network submission and refresh effects remain unknown.
pub(crate) fn set_faction_active(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: C_Reputation.SetFactionActive(factionSortIndex, setActive)";
    let index = read_faction_sort_index(state, USAGE)?;
    let active_value = read_allowed_argument(state, stack_val(state, 2), USAGE)?;
    if matches!(active_value, Val::Nil) {
        return Err(runtime_error(USAGE));
    }
    let set_active = !matches!(active_value, Val::Bool(false));
    let Some(index) = index else {
        return Ok(0);
    };
    let row = reputation_faction_at_index(state, index);
    let faction_id = table_get_static(state, row, "factionID");
    let can_set_inactive = matches!(
        table_get_static(state, row, "canSetInactive"),
        Val::Bool(true)
    );
    if row == Val::Nil
        || !matches!(faction_id, Val::Num(id) if id != 0.0)
        || !can_set_inactive
        || !has_valid_activity_reputation_index(state, row)
    {
        return Ok(0);
    }
    table_set_static(state, row, "isInactive", Val::Bool(!set_active));
    Ok(0)
}

// VERIFIED (Retail 12.1.5.70688, R88f.A.72): nil is required-argument
// failure; non-nil ordinary values use Lua truthiness after secret access checks.
pub(crate) fn set_legacy_reputations_shown(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: C_Reputation.SetLegacyReputationsShown(showLegacyReputations)";
    let value = read_allowed_argument(state, stack_val(state, 1), USAGE)?;
    if matches!(value, Val::Nil) {
        return Err(runtime_error(USAGE));
    }

    let show_legacy = !matches!(value, Val::Bool(false));
    let global = Val::Table(state.global);
    let reputation_state = table_get_static(state, global, "__wow_reputation_state");
    table_set_static(
        state,
        reputation_state,
        "showLegacy",
        Val::Bool(show_legacy),
    );
    Ok(0)
}

// VERIFIED (Retail 12.1.5.70688, R88f.A.70): the setter accepts signed-i32
// conversions but stores only ReputationSortType values 0, 1, and 2.
pub(crate) fn set_reputation_sort_type(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: C_Reputation.SetReputationSortType(sortType)";
    let number = read_required_number(state, USAGE)?;
    if number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    if !number.is_finite() {
        return Err(runtime_error(USAGE));
    }

    let sort_type = number as i32;
    if !matches!(sort_type, 0..=2) {
        return Err(runtime_error(USAGE));
    }

    let global = Val::Table(state.global);
    let reputation_state = table_get_static(state, global, "__wow_reputation_state");
    table_set_static(
        state,
        reputation_state,
        "sortType",
        Val::Num(f64::from(sort_type)),
    );
    Ok(0)
}

pub(crate) fn get_faction_paragon_info(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(faction_id) = stack_val(state, 1) else {
        return Ok(0);
    };
    let Some(info) = borrow_state(state)?
        .faction_paragon
        .get(&(faction_id as i32 as i64))
        .cloned()
    else {
        return Ok(0);
    };
    state.push(Val::Num(info.current_value as f64));
    state.push(Val::Num(info.threshold as f64));
    state.push(Val::Num(info.reward_quest_id as f64));
    state.push(Val::Bool(info.has_reward_pending));
    state.push(Val::Bool(info.too_low_level_for_paragon));
    if cfg!(feature = "retail-12-0-0") {
        state.push(Val::Num(info.paragon_storage_level as f64));
        Ok(6)
    } else {
        Ok(5)
    }
}
