//! Legacy template queries backed by the registered virtual-frame model.

use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn does_template_exist(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    state.push(Val::Bool(crate::xml::get_template(&name).is_some()));
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "DoesTemplateExist", does_template_exist)
}
