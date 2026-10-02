//! INFERRED explicit spell classifications; no native catalog or acquisition claim.

use std::collections::HashMap;

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Copy, Default)]
pub struct AuraSpellClassification {
    pub is_big_defensive: bool,
    pub is_private: bool,
}

#[derive(Default)]
pub struct AuraSpellClassifications {
    /// Host-declared resolved spell IDs only; empty by default.
    /// Generic buffs, private instances and cooldown associations supply no classifications.
    pub spells: HashMap<u32, AuraSpellClassification>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "AuraIsBigDefensive",
        aura_is_big_defensive,
    )?;
    table_set_rust_fn_static(state, namespace, "AuraIsPrivate", aura_is_private)
}

fn read_spell_classification(
    state: &LuaState,
    api_name: &str,
) -> LuaResult<AuraSpellClassification> {
    let Some(spell_id) = super::c_spell::read_public_spell_identifier_at(state, 1, api_name)?
    else {
        return Ok(AuraSpellClassification::default());
    };
    Ok(borrow_state(state)?
        .aura_spell_classifications
        .spells
        .get(&spell_id)
        .copied()
        .unwrap_or_default())
}

fn aura_is_big_defensive(state: &mut LuaState) -> LuaResult<u32> {
    let classification = read_spell_classification(state, "C_UnitAuras.AuraIsBigDefensive")?;
    state.push(Val::Bool(classification.is_big_defensive));
    Ok(1)
}

fn aura_is_private(state: &mut LuaState) -> LuaResult<u32> {
    let classification = read_spell_classification(state, "C_UnitAuras.AuraIsPrivate")?;
    state.push(Val::Bool(classification.is_private));
    Ok(1)
}
