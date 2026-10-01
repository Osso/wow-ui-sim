//! Retail 12.0.5 spell-identifier queries over existing public aura records.

use crate::lua_api::globals::auras::push_aura_by_spell_id;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::table_security::is_secret_value;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetPlayerAuraBySpellID",
        get_player_aura_by_spell_id,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetUnitAuraBySpellID",
        get_unit_aura_by_spell_id,
    )
}

fn get_player_aura_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    match read_public_spell_identifier(state, 1)? {
        Some(id) => push_aura_by_spell_id(state, "player", id, false),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_unit_aura_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    // Validate the identifier even when the unit has no modeled aura store.
    let spell_id = read_public_spell_identifier(state, 2)?;
    let unit = read_public_unit(state)?;
    match (unit, spell_id) {
        // INFERRED first-match order: helpful then harmful; no visibility model.
        (Some(unit), Some(id)) => push_aura_by_spell_id(state, &unit, id, true),
        _ => state.push(Val::Nil),
    }
    Ok(1)
}

fn reject_secret(state: &LuaState, value: Val, argument: &str) -> LuaResult<()> {
    // Conservative rejection, not native AllowedWhenTainted enforcement.
    if is_secret_value(state, value) {
        return Err(runtime_error(format!(
            "C_UnitAuras: secret {argument} access is not modeled"
        )));
    }
    Ok(())
}

fn read_public_spell_identifier(state: &LuaState, index: i32) -> LuaResult<Option<u32>> {
    let value = stack_val(state, index);
    reject_secret(state, value, "spell identifier")?;
    match value {
        Val::Num(number) if number.is_finite() => {}
        Val::Str(_) => {}
        _ => {
            return Err(runtime_error(format!(
                "C_UnitAuras: argument {index} must be a public finite number or string spell identifier"
            )));
        }
    }
    super::c_spell::read_spell_identifier_at(state, index)
}

fn read_public_unit(state: &LuaState) -> LuaResult<Option<String>> {
    let value = stack_val(state, 1);
    reject_secret(state, value, "unit")?;
    match value {
        Val::Nil => Ok(None),
        Val::Str(_) => String::from_stack(state, 1).map(Some),
        _ => Err(runtime_error(
            "C_UnitAuras: unit must be a public string or nil",
        )),
    }
}
