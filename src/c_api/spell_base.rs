//! Explicit specialization-specific base spell relationships.
//!
//! No live replacement dataset is seeded. An absent relationship means the
//! supplied spell is its own base, as documented by GetBaseSpell.

use std::collections::HashMap;

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Default)]
pub struct BaseSpellRelationships {
    by_spec_and_spell: HashMap<(u32, u32), u32>,
}

impl BaseSpellRelationships {
    /// Install an explicit base relationship; callers supply actual base IDs,
    /// not another override to traverse or an identifier alias to reverse.
    pub fn set(&mut self, spec_id: u32, spell_id: u32, base_spell_id: u32) {
        self.by_spec_and_spell
            .insert((spec_id, spell_id), base_spell_id);
    }

    pub fn resolve(&self, spec_id: u32, spell_id: u32) -> u32 {
        self.by_spec_and_spell
            .get(&(spec_id, spell_id))
            .copied()
            .unwrap_or(spell_id)
    }
}

fn public_argument(state: &LuaState, index: i32) -> LuaResult<Val> {
    let value = stack_val(state, index);
    if rilua::table_security::is_secret_value(state, value) {
        return Err(runtime_error(
            "C_Spell.GetBaseSpell secret arguments are not modeled",
        ));
    }
    Ok(value)
}

fn valid_id(number: f64) -> bool {
    number.is_finite() && number.fract() == 0.0 && number > 0.0 && number <= f64::from(u32::MAX)
}

fn read_spell_id(state: &LuaState) -> LuaResult<u32> {
    let value = public_argument(state, 1)?;
    let valid_kind = match value {
        Val::Num(number) => valid_id(number),
        Val::Str(_) => true,
        _ => false,
    };
    if valid_kind {
        if let Some(id) = super::c_spell::numeric_spell_id(state, 1).filter(|id| *id != 0) {
            return Ok(id);
        }
    }
    Err(runtime_error(
        "C_Spell.GetBaseSpell expects a positive spell ID or known spell name",
    ))
}

fn read_requested_spec(state: &LuaState) -> LuaResult<u32> {
    match public_argument(state, 2)? {
        Val::Nil | Val::Num(0.0) => Ok(0),
        Val::Num(number) if valid_id(number) => Ok(number as u32),
        _ => Err(runtime_error(
            "C_Spell.GetBaseSpell spec must be a non-negative integer",
        )),
    }
}

pub(super) fn get_base_spell(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_spell_id(state)?;
    let requested_spec = read_requested_spec(state)?;
    let base = {
        let sim = borrow_state(state)?;
        let spec_id = if requested_spec == 0 {
            let index = usize::try_from(sim.player.active_spec_index - 1)
                .map_err(|_| runtime_error("player has no active specialization"))?;
            crate::specializations::specs_for_class(sim.player.class_index as u32)
                .nth(index)
                .ok_or_else(|| runtime_error("player has no active specialization"))?
                .id
        } else {
            requested_spec
        };
        sim.base_spell_relationships.resolve(spec_id, spell_id)
    };
    state.push(Val::Num(f64::from(base)));
    Ok(1)
}
