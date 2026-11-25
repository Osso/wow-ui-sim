use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::ChatWindow;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(super) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetChatWindowState", set_chat_window_state)
}

pub(super) fn set_chat_window_state(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let shown = bool::from_stack(state, 3)?;
    let docked_index = Option::<i32>::from_stack(state, 4)?;
    if index <= 0 {
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    let window = sim
        .chat_windows
        .entry(index)
        .or_insert_with(|| ChatWindow::for_index(index));
    window.name = name;
    window.shown = shown;
    window.docked_index = docked_index;
    Ok(0)
}
