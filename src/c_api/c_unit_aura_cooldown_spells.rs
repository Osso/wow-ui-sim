//! Explicit cooldown-spell associations; chosen simulator inputs, not native acquisition.

use std::collections::HashMap;

use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::is_secret_value;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

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
    let cooldown_spell_id = match read_public_spell_identifier(state)? {
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

fn read_public_spell_identifier(state: &LuaState) -> LuaResult<Option<u32>> {
    let value = stack_val(state, 1);
    // INFERRED conservative rejection, not native AllowedWhenTainted permissions.
    // Never unwrap secrets, inspect private payloads, or change caller taint.
    if is_secret_value(state, value) {
        return Err(runtime_error(format!(
            "{API_NAME}: secret spell identifier access is not modeled"
        )));
    }
    // INFERRED strict public representations; validate before alias resolution.
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= 0.0
                && number <= u32::MAX as f64 => {}
        Val::Str(_) if val_to_string(state, value).is_some() => {}
        _ => {
            return Err(runtime_error(format!(
                "{API_NAME}: argument 1 must be a public UTF-8 string or finite integral u32 number"
            )));
        }
    }
    super::c_spell::read_spell_identifier_at(state, 1)
}
