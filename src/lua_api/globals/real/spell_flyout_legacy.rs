//! Legacy spell flyout globals.
//!
//! `GetFlyoutInfo` and `GetFlyoutSlotInfo` wrap the state-backed `C_SpellBook`
//! flyout queries. Retail and Forever `GetCallPetSpellInfo` read host-supplied
//! spell/index candidates and current-player pet names.

use crate::c_api::parse_numeric_string;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
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
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
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

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetCallPetSpellInfo", set_call_pet_spell_info)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_call_pet_spell_info(state: &mut LuaState) -> LuaResult<u32> {
    let (Val::Num(spell), Val::Num(slot)) = (stack_val(state, 1), stack_val(state, 2)) else {
        return Err(rilua::runtime_error(
            "A_Admin.SetCallPetSpellInfo: expected spell ID and slot",
        ));
    };
    if spell.fract() != 0.0 || !(1.0..=f64::from(i32::MAX)).contains(&spell) {
        return Err(rilua::runtime_error(
            "A_Admin.SetCallPetSpellInfo: expected a positive integer spell ID",
        ));
    }
    if slot.fract() != 0.0 || !(0.0..=4.0).contains(&slot) {
        return Err(rilua::runtime_error(
            "A_Admin.SetCallPetSpellInfo: slot must be 0 through 4",
        ));
    }
    let name = match stack_val(state, 3) {
        Val::Nil => None,
        Val::Str(_) => Some(String::from_stack(state, 3)?),
        _ => {
            return Err(rilua::runtime_error(
                "A_Admin.SetCallPetSpellInfo: pet name must be string or nil",
            ));
        }
    };
    let mut model = borrow_state_mut(state)?;
    model
        .call_pet_spell_indices
        .insert(spell as u32, slot as i32);
    if model.player_call_pet_names.len() <= slot as usize {
        model.player_call_pet_names.resize(slot as usize + 1, None);
    }
    model.player_call_pet_names[slot as usize] = name;
    Ok(0)
}

#[cfg(feature = "client-retail")]
fn read_call_pet_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: GetCallPetSpellInfo(spellID)";
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))
        .map_err(|_| rilua::runtime_error(USAGE))?;
    let number = numeric(state, value).ok_or_else(|| rilua::runtime_error(USAGE))?;
    Ok(signed_integer(number)? as u32)
}

// Forever's selected SpellFlyout uses numeric IDs. Native boundary is unknown.
#[cfg(feature = "client-wowforever")]
fn read_call_pet_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    match stack_val(state, 1) {
        Val::Num(value) if value.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&value) => {
            Ok(value as u32)
        }
        _ => Err(rilua::runtime_error(
            "GetCallPetSpellInfo expects a numeric spell ID",
        )),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_call_pet_spell_info(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_call_pet_spell_id(state)?;
    let Some((index, pet_name)) = read_call_pet_observation(state, spell_id)? else {
        // Preserve Forever's prior nil/nil miss; native arity remains unknown.
        if cfg!(feature = "client-wowforever") {
            state.push(Val::Nil);
            state.push(Val::Nil);
            return Ok(2);
        }
        return Ok(0);
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

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_call_pet_observation(
    state: &LuaState,
    spell_id: u32,
) -> LuaResult<Option<(i32, Option<String>)>> {
    let model = borrow_state(state)?;
    let Some(index) = model
        .call_pet_spell_indices
        .get(&spell_id)
        .copied()
        .filter(|index| (0..=4).contains(index))
    else {
        return Ok(None);
    };
    let pet_name = model
        .player_call_pet_names
        .get(index as usize)
        .cloned()
        .flatten();
    Ok(Some((index, pet_name)))
}
