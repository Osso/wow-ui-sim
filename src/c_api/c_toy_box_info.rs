//! Newly acquired toys remain wrapped until the player clears their fanfare.

use std::collections::HashSet;

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, IntoStack, TableBuilder};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

#[derive(Debug, Default, Clone)]
pub struct ToyFanfare {
    pending: HashSet<u32>,
}

impl ToyFanfare {
    pub fn needs_fanfare(&self, item_id: u32) -> bool {
        self.pending.contains(&item_id)
    }

    pub fn mark_acquired(&mut self, item_id: u32) {
        self.pending.insert(item_id);
    }

    pub fn clear(&mut self, item_id: u32) {
        self.pending.remove(&item_id);
    }
}

fn needs_fanfare(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let needs = borrow_state(state)?
        .world
        .toy_fanfare
        .needs_fanfare(item_id);
    needs.into_stack(state)
}

fn clear_fanfare(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    borrow_state_mut(state)?.world.toy_fanfare.clear(item_id);
    Ok(0)
}

pub(crate) fn register_fanfare(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("NeedsFanfare", needs_fanfare)?
        .set_function("ClearFanfare", clear_fanfare)
}
