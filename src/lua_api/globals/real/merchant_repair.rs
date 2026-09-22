//! Forever merchant service capability query; policy is inferred from cached UI use.

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn can_merchant_repair(state: &mut LuaState) -> LuaResult<u32> {
    let can_repair = {
        let model = borrow_state(state)?;
        model.merchant_frame_open && model.merchant_repair_capable
    };
    state.push(Val::Bool(can_repair));
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "CanMerchantRepair", can_merchant_repair)
}
