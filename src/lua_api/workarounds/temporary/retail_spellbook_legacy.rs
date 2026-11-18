//! R88m.4 Retail selected-spell input boundaries over unmodeled selection data.
//!
//! Call-pet result projection is modeled in `real::spell_flyout_legacy` with
//! empty-default host inputs. Selection/name-variant caches remain unmodeled.
use crate::c_api::global_val;
use crate::lua_api::globals::real::spell_flyout_legacy::{numeric, signed_integer};
use crate::lua_api::globals::spellbook_data;
use crate::lua_bridge::stack_val;
use rilua::{LuaApiMut, LuaResult, Val, vm::state::LuaState};

pub(crate) fn register_selected(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(
        global_val(lua.state_mut(), "IsSelectedSpellBookItem"),
        Val::Nil
    ) {
        LuaApiMut::register_function(lua, "IsSelectedSpellBookItem", selected)?;
    }
    Ok(())
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
