//! Explicit War Mode inputs, independent of generic PvP and active phase state.

use crate::{lua_api::methods::borrow_state, lua_bridge::table_set_rust_fn_static};
use rilua::{LuaResult, Val, vm::gc::arena::GcRef, vm::state::LuaState, vm::table::Table};

#[derive(Debug, Clone)]
pub struct WarModeState {
    pub desired: bool,
    pub reward_bonus: f64,
}

pub(crate) fn register(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "IsWarModeDesired", is_war_mode_desired)?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetWarModeRewardBonus",
        get_war_mode_reward_bonus,
    )
}

pub(super) fn is_war_mode_desired(state: &mut LuaState) -> LuaResult<u32> {
    let desired = borrow_state(state)?.war_mode.desired;
    state.push(Val::Bool(desired));
    Ok(1)
}

fn get_war_mode_reward_bonus(state: &mut LuaState) -> LuaResult<u32> {
    let reward_bonus = borrow_state(state)?.war_mode.reward_bonus;
    state.push(Val::Num(reward_bonus));
    Ok(1)
}

impl Default for WarModeState {
    fn default() -> Self {
        Self {
            desired: false,
            // Preserve the existing simulator reward getter's 10-percent baseline.
            // This is compatibility policy, not a native reward/default claim.
            reward_bonus: 10.0,
        }
    }
}
