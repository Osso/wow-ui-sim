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

// VERIFIED (Retail 12.1.5.70688, R88f.A.72): nil is required-argument
// failure; non-nil ordinary values use Lua truthiness after secret access checks.
pub(crate) fn set_legacy_reputations_shown(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: C_Reputation.SetLegacyReputationsShown(showLegacyReputations)";

    let value = stack_val(state, 1);
    if is_secret_value(state, value) && !can_access_secrets(state) {
        return Err(runtime_error(format!(
            "{USAGE}. Secret values are only allowed during untainted execution for this argument."
        )));
    }
    let value = unwrap_secret(state, value)?;
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

    let value = stack_val(state, 1);
    if is_secret_value(state, value) && !can_access_secrets(state) {
        return Err(runtime_error(format!(
            "{USAGE}. Secret values are only allowed during untainted execution for this argument."
        )));
    }
    let value = unwrap_secret(state, value)?;
    if matches!(value, Val::Nil) {
        return Err(runtime_error(USAGE));
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
    }
    .ok_or_else(|| runtime_error(USAGE))?;

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
