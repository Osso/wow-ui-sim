//! Bounded numeric game-account invitation request capture; no Battle.net service.
//! All storage/range/extra-argument policies are INFERRED from the current cache.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn invite_friend(state: &mut LuaState) -> LuaResult<u32> {
    let account = unwrap_secret(state, stack_val(state, 1))?;
    for value in state.stack.iter().take(state.top).skip(state.base + 1) {
        unwrap_secret(state, *value)?;
    }
    let account_id = validate_game_account_id(account)?;
    // Captures a request only. Never fabricate accepted/offline friendships.
    borrow_state_mut(state)?.last_bnet_invite_game_account_id = Some(account_id);
    Ok(0)
}

fn validate_game_account_id(account: Val) -> LuaResult<i32> {
    if let Val::Num(number) = account {
        let is_integer = number.is_finite() && number.fract() == 0.0;
        let in_domain = (1.0..=i32::MAX as f64).contains(&number);
        if is_integer && in_domain {
            return Ok(number as i32);
        }
    }
    Err(runtime_error(
        "C_BattleNet.InviteFriend: gameAccountID must be a positive integral i32 number",
    ))
}
