//! Forever input probe backed by the existing C_InputInterfaceStyle model.

use crate::c_api::c_input_interface_style::InputInterfaceStyle;
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn is_using_gamepad(state: &mut LuaState) -> LuaResult<u32> {
    let style = borrow_state(state)?.input_interface_style;
    state.push(Val::Bool(style == InputInterfaceStyle::Gamepad));
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "IsUsingGamepad", is_using_gamepad)
}
