//! Values published by `Enum.GameMode` and used by game-rule state.
pub(crate) const STANDARD: i32 = 1;
pub(crate) const PLUNDERSTORM: i32 = 2;
pub(crate) const WOWHACK: i32 = 3;

pub(crate) fn is_standard(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let active = crate::lua_api::methods::borrow_state(state)?
        .game_rules
        .active_game_mode
        == STANDARD;
    state.push(rilua::Val::Bool(active));
    Ok(1)
}
