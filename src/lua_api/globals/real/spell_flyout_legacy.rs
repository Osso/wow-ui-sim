//! Legacy spell flyout globals.
//!
//! `GetFlyoutInfo` and `GetFlyoutSlotInfo` wrap the state-backed `C_SpellBook`
//! flyout queries. Retail `GetCallPetSpellInfo` reads explicit host-supplied
//! spell/index candidates and current-player pet names.

use crate::c_api::parse_numeric_string;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_legacy_spell_flyout_globals(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        state.global,
        "GetFlyoutInfo",
        crate::c_api::c_spell_book::get_flyout_info,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetFlyoutSlotInfo",
        crate::c_api::c_spell_book::get_flyout_slot_info,
    )?;
    #[cfg(feature = "client-retail")]
    table_set_rust_fn_static(
        state,
        state.global,
        "GetCallPetSpellInfo",
        get_call_pet_spell_info,
    )?;
    Ok(())
}

pub(crate) fn numeric(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|value| parse_numeric_string(value.data())),
        _ => None,
    }
}

pub(crate) fn signed_integer(number: f64) -> LuaResult<i32> {
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number) {
        return Err(rilua::runtime_error(format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    Ok(number.trunc() as i32)
}

#[cfg(feature = "client-retail")]
fn get_call_pet_spell_info(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: GetCallPetSpellInfo(spellID)";
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))
        .map_err(|_| rilua::runtime_error(USAGE))?;
    let number = numeric(state, value).ok_or_else(|| rilua::runtime_error(USAGE))?;
    let spell_id = signed_integer(number)? as u32;
    let (index, pet_name) = {
        let model = borrow_state(state)?;
        let Some(index) = model.call_pet_spell_indices.get(&spell_id).copied() else {
            return Ok(0);
        };
        if !(0..=4).contains(&index) {
            return Ok(0);
        }
        let pet_name = model
            .player_call_pet_names
            .get(index as usize)
            .cloned()
            .flatten();
        (index, pet_name)
    };
    state.push(Val::Num(index as f64));
    match pet_name {
        Some(name) => {
            let value = create_string(state, &name);
            state.push(value);
        }
        None => state.push(Val::Nil),
    }
    Ok(2)
}
