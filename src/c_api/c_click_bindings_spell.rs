//! INFERRED host-declared click-bindable spell IDs; no native eligibility catalog.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_ClickBindings")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "CanSpellBeClickBound",
        can_spell_be_click_bound,
    )
}

fn can_spell_be_click_bound(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED strict public/secret boundary shared with spell classifications.
    let spell_id = super::c_spell::read_public_spell_identifier_at(
        state,
        1,
        "C_ClickBindings.CanSpellBeClickBound",
    )?;
    // INFERRED unknown/nonmember policy: only explicit host membership grants eligibility.
    let can_be_bound = match spell_id {
        Some(spell_id) => borrow_state(state)?
            .click_bindable_spells
            .contains(&spell_id),
        None => false,
    };
    state.push(Val::Bool(can_be_bound));
    Ok(1)
}
