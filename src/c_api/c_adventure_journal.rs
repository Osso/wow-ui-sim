//! INFERRED: R88dn (native evidence pending). Retail and Forever select
//! `AdventureGuideUtil.lua:10-11`, which consumes this supplied availability.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetAdventureJournalCanBeShown", set_can_be_shown)
}

fn set_can_be_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = bool::from_stack(state, 1)
        .map_err(|_| runtime_error("A_Admin.SetAdventureJournalCanBeShown: expected a boolean"))?;
    borrow_state_mut(state)?
        .encounter_journal
        .adventure_can_be_shown = shown;
    Ok(0)
}
