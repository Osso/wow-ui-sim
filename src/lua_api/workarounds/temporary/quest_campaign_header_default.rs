//! Retire when quest campaign-header membership and ordering are modeled.

use rilua::{LuaResult, vm::state::LuaState};

pub(crate) fn update_campaign_headers(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
