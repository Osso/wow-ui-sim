//! Base spell aura secrecy from native SpellMisc attributes, not combat policy.

use super::ensure_namespace;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[path = "../../data/spell_aura_secrecy.rs"]
mod attributes;

const NEVER_SECRET: i32 = 0;
const ALWAYS_SECRET: i32 = 1;
const CONTEXTUALLY_SECRET: i32 = 2;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Secrets")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetSpellAuraSecrecy",
        get_spell_aura_secrecy,
    )
}

fn get_spell_aura_secrecy(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = super::c_spell::numeric_spell_id(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let secrecy = classify_aura_secrecy(spell_id)?;
    state.push(Val::Num(f64::from(secrecy)));
    Ok(1)
}

fn classify_aura_secrecy(spell_id: u32) -> LuaResult<i32> {
    match attributes::aura_flags(spell_id) {
        0 => Ok(CONTEXTUALLY_SECRET),
        attributes::AURA_ALWAYS_SECRET => Ok(ALWAYS_SECRET),
        attributes::AURA_NEVER_SECRET => Ok(NEVER_SECRET),
        flags => Err(runtime_error(format!(
            "C_Secrets.GetSpellAuraSecrecy: ambiguous aura secrecy flags {flags:#x} for spell {spell_id}; native precedence is unverified"
        ))),
    }
}
