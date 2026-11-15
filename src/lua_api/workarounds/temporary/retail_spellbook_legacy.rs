//! R88m.4 Retail legacy input boundaries over explicitly unmodeled data.
//!
//! Call-pet SpellEffect/pet-info records and selection/name-variant caches are
//! not supplied by the simulator. Do not infer them from unrelated spell or
//! pet-action fields. These remain temporary no-data seams, not native defaults.
use crate::c_api::{global_val, parse_numeric_string};
use crate::lua_api::globals::spellbook_data;
use crate::lua_bridge::stack_val;
use rilua::{LuaApiMut, LuaResult, Val, vm::state::LuaState};

pub(crate) fn register_call_pet(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(global_val(lua.state_mut(), "GetCallPetSpellInfo"), Val::Nil) {
        LuaApiMut::register_function(lua, "GetCallPetSpellInfo", call_pet)?;
    }
    Ok(())
}

pub(crate) fn register_selected(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(
        global_val(lua.state_mut(), "IsSelectedSpellBookItem"),
        Val::Nil
    ) {
        LuaApiMut::register_function(lua, "IsSelectedSpellBookItem", selected)?;
    }
    Ok(())
}

fn numeric(state: &LuaState, value: Val) -> Option<f64> {
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

fn signed_integer(number: f64) -> LuaResult<i32> {
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number) {
        return Err(rilua::runtime_error(format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    Ok(number.trunc() as i32)
}

fn call_pet(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: GetCallPetSpellInfo(spellID)";
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))
        .map_err(|_| rilua::runtime_error(USAGE))?;
    let number = numeric(state, value).ok_or_else(|| rilua::runtime_error(USAGE))?;
    let _spell_id = signed_integer(number)? as u32;
    // No supplied first-effect-56/call-pet-index and runtime pet-info records.
    // The verified no-result path publishes zero values, not two nils.
    Ok(0)
}

fn selected(state: &mut LuaState) -> LuaResult<u32> {
    const INVALID: &str = "IsSelectedSpellBookItem(): Invalid spell slot";
    let first = rilua::table_security::unwrap_secret(state, stack_val(state, 1))
        .map_err(|_| rilua::runtime_error(INVALID))?;
    if !matches!(first, Val::Num(_) | Val::Str(_)) {
        return Err(rilua::runtime_error(INVALID));
    }
    let second = stack_val(state, 2);
    if let Some(number) = numeric(state, first)
        && matches!(second, Val::Num(_) | Val::Str(_))
    {
        return selected_numeric_slot(state, number, second);
    }
    // The native locale/secondary-text variant store is not supplied. A missing
    // name lookup publishes zero results. Do not use numeric input as a spell ID.
    Ok(0)
}

fn selected_numeric_slot(state: &mut LuaState, number: f64, bank: Val) -> LuaResult<u32> {
    let slot = signed_integer(number)?;
    let pet_bank = match bank {
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .is_some_and(|value| value.data() == b"pet"),
        _ => false,
    };
    // The existing adapter has only a contiguous player spell bank, no
    // ordered pet bank. Do not reinterpret the unordered pet spell set.
    let valid_player_slot = slot >= 1 && spellbook_data::get_spell_at_slot(slot).is_some();
    if pet_bank || !valid_player_slot {
        return Ok(0);
    }
    // Selection/effect caches are absent. Preserve the existing unselected
    // seam for represented rows; no guessed SkillLineAbility predicate.
    state.push(Val::Bool(false));
    Ok(1)
}
