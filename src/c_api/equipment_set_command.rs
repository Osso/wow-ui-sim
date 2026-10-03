//! Bounded /equipset name dispatch into the existing equipment manager.

use crate::lua_api::methods::borrow_state;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn run_equipment_set_command(state: &mut LuaState, name: &str) -> LuaResult<()> {
    // INFERRED: exact case-sensitive lookup after the existing slash parser's
    // whitespace trimming. Conditional and quoted macro syntax is not modeled.
    let set_id = {
        let sim = borrow_state(state)?;
        sim.equipment_manager
            .sets
            .iter()
            .find(|set| set.name == name && !name.is_empty())
            .map(|set| set.id)
    };
    let Some(set_id) = set_id else {
        return Ok(());
    };
    crate::c_api::item_spell::use_equipment_set_by_id(state, set_id)?;
    Ok(())
}
