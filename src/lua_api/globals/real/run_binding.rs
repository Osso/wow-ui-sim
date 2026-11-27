//! `RunBinding(command [, keyState])` runs the body a Bindings.xml file
//! declared for a binding command (retail-12-1-5-70688.csv `_G,RunBinding`;
//! Forever consumers: CoinPickupFrame.lua:183, StackSplitFrame.lua:259).
//!
//! INFERRED: R88g9 (native evidence pending): the second argument selects the
//! `keystate` global the body sees ("up" when it is "up", otherwise "down"),
//! and a command without a declared body runs nothing.

use crate::lua_api::globals::keybindings::execute_binding_body;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

fn run_binding(state: &mut LuaState) -> LuaResult<u32> {
    let command = String::from_stack(state, 1)?;
    let key_state = match Option::<String>::from_stack(state, 2)?.as_deref() {
        Some("up") => "up",
        _ => "down",
    };
    let body = borrow_state(state)?
        .keybindings
        .bodies
        .get(&command)
        .cloned();
    if let Some(body) = body {
        execute_binding_body(state, &body, key_state);
    }
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "RunBinding", run_binding)
}
