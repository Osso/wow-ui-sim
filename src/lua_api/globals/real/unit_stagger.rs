use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "UnitStagger", unit_stagger)?;
    Ok(())
}

fn unit_stagger(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let amount = if matches!(unit.as_str(), "player" | "self") {
        borrow_state(state)?.player.stagger_amount
    } else {
        0.0
    };
    state.push(Val::Num(amount));
    Ok(1)
}

pub(crate) fn set_player_stagger(state: &mut LuaState) -> LuaResult<u32> {
    let amount = f64::from_stack(state, 1)?;
    borrow_state_mut(state)?.player.stagger_amount = amount;
    Ok(0)
}
