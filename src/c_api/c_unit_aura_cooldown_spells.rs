//! Explicit cooldown-spell associations; chosen simulator inputs, not native acquisition.

use std::collections::HashMap;

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const API_NAME: &str = "C_UnitAuras.GetCooldownAuraBySpellID";

#[derive(Default)]
pub struct CooldownAuraAssociations {
    /// Keys are resolved query spell IDs; values are declared returned cooldown spell IDs.
    /// Empty by default. No generic aura lookup, implicit association, or native direction claim.
    pub cooldown_spell_ids: HashMap<u32, u32>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCooldownAuraBySpellID",
        get_cooldown_aura_by_spell_id,
    )
}

fn get_cooldown_aura_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    let cooldown_spell_id =
        match super::c_spell::read_public_spell_identifier_at(state, 1, API_NAME)? {
            Some(query_spell_id) => borrow_state(state)?
                .cooldown_aura_associations
                .cooldown_spell_ids
                .get(&query_spell_id)
                .copied(),
            None => None,
        };
    state.push(cooldown_spell_id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}
