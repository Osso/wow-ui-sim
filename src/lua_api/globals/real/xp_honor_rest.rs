//! XP / honor / rest globals consumed by `Blizzard_ActionBar/Shared/ExpBar.lua`,
//! `Mainline/HonorBar.lua`, and `Mainline/ExhaustionTickMixin:UpdateTickPosition`.
//!
//! All read from `state.player_xp` (rest/exhaustion/honor/trial/limited
//! mode cluster). Honor and trial unit probes treat unit tokens other than
//! `"player"` as nil/0 — the simulator only models the local player here.
//!
//! Globals registered:
//!
//! - `GetXPExhaustion()` → rest XP (nil when none).
//! - `GetRestState()` → `(state, name, multiplier)`.
//! - `IsPlayerAtEffectiveMaxLevel()` → `state.player_xp.is_max_level` (pre-12.1.0 profiles).
//! - `GameLimitedMode_IsBankedXPActive()` → `banked_xp_active`.
//! - `GameLimitedMode_GetLevelLimit()` → `level_limit`.
//! - `UnitHonor(unit)` → current honor (was a 0-stub).
//! - `UnitHonorMax(unit)` → honor required for next level (was a bootstrap stub).
//! - `UnitTrialXP(unit)` → trial-capped XP.
//! - `UnitTrialBankedLevels(unit)` → banked levels (was a bootstrap stub).
//! - `GetRestrictedAccountData()` → `(level, money, profession)` (replaces the
//!   custom stub that always returned `(20, 0, 0)`).
//!
//! `IsResting`, `IsXPUserDisabled`, `UnitHonorLevel`, and `UnitXP` /
//! `UnitXPMax` already live elsewhere (`combat_probes`, `player_probes`,
//! `state_backed_queries`, `unit_stats`) and continue to read PlayerState
//! directly.

use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

use crate::lua_api::methods::val_to_string;

fn unit_arg_is_player(state: &mut LuaState, index: i32) -> bool {
    val_to_string(state, stack_val(state, index)).as_deref() == Some("player")
}

fn get_xp_exhaustion(state: &mut LuaState) -> LuaResult<u32> {
    let exhaustion = borrow_state(state)?.player_xp.exhaustion;
    match exhaustion {
        Some(value) => state.push(Val::Num(value as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_rest_state(state: &mut LuaState) -> LuaResult<u32> {
    let (rest_state, name, multiplier) = {
        let xp = &borrow_state(state)?.player_xp;
        (
            xp.rest_state,
            xp.rest_state_name.clone(),
            xp.rest_multiplier,
        )
    };
    let name_val = create_string(state, &name);
    state.push(Val::Num(rest_state as f64));
    state.push(name_val);
    state.push(Val::Num(multiplier));
    Ok(3)
}

#[cfg(not(feature = "retail-12-1-0"))]
fn is_player_at_effective_max_level(state: &mut LuaState) -> LuaResult<u32> {
    let at_max = borrow_state(state)?.player_xp.is_max_level;
    state.push(Val::Bool(at_max));
    Ok(1)
}

fn game_limited_mode_is_banked_xp_active(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.player_xp.banked_xp_active;
    state.push(Val::Bool(active));
    Ok(1)
}

fn game_limited_mode_get_level_limit(state: &mut LuaState) -> LuaResult<u32> {
    let limit = borrow_state(state)?.player_xp.level_limit;
    state.push(Val::Num(limit as f64));
    Ok(1)
}

fn push_player_xp_field(
    state: &mut LuaState,
    read: impl FnOnce(&crate::lua_api::state_types::character_world::PlayerXpState) -> i64,
) -> LuaResult<u32> {
    let value = if unit_arg_is_player(state, 1) {
        read(&borrow_state(state)?.player_xp)
    } else {
        0
    };
    state.push(Val::Num(value as f64));
    Ok(1)
}

fn unit_honor(state: &mut LuaState) -> LuaResult<u32> {
    push_player_xp_field(state, |xp| xp.honor as i64)
}

fn unit_honor_max(state: &mut LuaState) -> LuaResult<u32> {
    push_player_xp_field(state, |xp| xp.honor_max as i64)
}

fn unit_trial_xp(state: &mut LuaState) -> LuaResult<u32> {
    push_player_xp_field(state, |xp| xp.trial_xp as i64)
}

fn unit_trial_banked_levels(state: &mut LuaState) -> LuaResult<u32> {
    push_player_xp_field(state, |xp| xp.trial_banked_levels as i64)
}

fn get_restricted_account_data(state: &mut LuaState) -> LuaResult<u32> {
    let (level, money, profession) = {
        let xp = &borrow_state(state)?.player_xp;
        (
            xp.restricted_level,
            xp.restricted_money,
            xp.restricted_profession,
        )
    };
    state.push(Val::Num(level as f64));
    state.push(Val::Num(money as f64));
    state.push(Val::Num(profession as f64));
    Ok(3)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetXPExhaustion", get_xp_exhaustion)?;
    LuaApiMut::register_function(lua, "GetRestState", get_rest_state)?;
    // Retail 12.1.0 removed the global; Blizzard uses GameRulesUtil instead.
    #[cfg(not(feature = "retail-12-1-0"))]
    LuaApiMut::register_function(
        lua,
        "IsPlayerAtEffectiveMaxLevel",
        is_player_at_effective_max_level,
    )?;
    LuaApiMut::register_function(
        lua,
        "GameLimitedMode_IsBankedXPActive",
        game_limited_mode_is_banked_xp_active,
    )?;
    LuaApiMut::register_function(
        lua,
        "GameLimitedMode_GetLevelLimit",
        game_limited_mode_get_level_limit,
    )?;
    LuaApiMut::register_function(lua, "UnitHonor", unit_honor)?;
    LuaApiMut::register_function(lua, "UnitHonorMax", unit_honor_max)?;
    LuaApiMut::register_function(lua, "UnitTrialXP", unit_trial_xp)?;
    LuaApiMut::register_function(lua, "UnitTrialBankedLevels", unit_trial_banked_levels)?;
    LuaApiMut::register_function(lua, "GetRestrictedAccountData", get_restricted_account_data)?;
    Ok(())
}
