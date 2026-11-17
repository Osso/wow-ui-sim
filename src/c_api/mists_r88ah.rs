//! Mists-only C API models from the R88ah native answer.

use super::helpers::{
    bad_argument, ensure_namespace, parse_numeric_string, read_untainted_signed32_argument,
};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const VISIBILITY_API: &str = "C_Spell.GetVisibilityInfo";
const VISIBILITY_USAGE: &str = "Usage: local hasCustom, alwaysShowMine, showForMySpec = C_Spell.GetVisibilityInfo(spellID, visibilityType)";
const MAX_VISIBILITY_TYPE: u8 = 2;
const FALSE_VISIBILITY: (bool, bool, bool) = (false, false, false);

pub(crate) fn register_combat_audio_alert(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CombatAudioAlert")?;
    table_set_rust_fn_static(state, namespace, "IsEnabled", is_enabled)
}

pub(crate) fn register_spell_visibility(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Spell")?;
    table_set_rust_fn_static(state, namespace, "GetVisibilityInfo", get_visibility_info)
}

fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = {
        let sim = borrow_state(state)?;
        sim.mists_combat_audio_alert_state_gate
            && sim
                .cvars
                .cached_integer_nonzero("CAAEnabled")
                .unwrap_or(false)
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn get_visibility_info(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_untainted_signed32_argument(state, 1, VISIBILITY_API, VISIBILITY_USAGE)?;
    let visibility_type = read_visibility_type(state)?;
    let result = {
        let sim = borrow_state(state)?;
        visibility_result(&sim, spell_id, visibility_type)
    };
    let Some((has_custom, always_show_mine, show_for_my_spec)) = result else {
        return Ok(0);
    };
    state.push(Val::Bool(has_custom));
    state.push(Val::Bool(always_show_mine));
    state.push(Val::Bool(show_for_my_spec));
    Ok(3)
}

fn visibility_result(
    sim: &crate::lua_api::SimState,
    spell_id: i32,
    visibility_type: u8,
) -> Option<(bool, bool, bool)> {
    if !sim.player.present {
        return None;
    }
    let id = u32::try_from(spell_id).ok();
    let result = id
        .and_then(|id| {
            sim.spell_visibility_results
                .as_ref()
                .and_then(|rows| rows.get(&(id, visibility_type)).copied())
        })
        .unwrap_or(FALSE_VISIBILITY);
    Some(result)
}

fn read_visibility_type(state: &LuaState) -> LuaResult<u8> {
    let number = read_visibility_number(state)?;
    visibility_type_from_number(number)
}

fn read_visibility_number(state: &LuaState) -> LuaResult<f64> {
    let original = stack_val(state, 2);
    if original == Val::Nil {
        return Err(usage_error(2));
    }
    let secret = is_secret_value(state, original);
    let value = unwrap_secret(state, original).map_err(|_| secret_argument_error())?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) if !secret => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| parse_numeric_string(string.data())),
        _ => None,
    };
    number.ok_or_else(|| usage_error(2))
}

fn visibility_type_from_number(number: f64) -> LuaResult<u8> {
    let within_i32_range = (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number);
    if !number.is_finite() || !within_i32_range {
        let error = format!("integer overflow attempting to store {number:.6}");
        return Err(bad_argument(2, VISIBILITY_API, &error));
    }
    let visibility_type = (number.trunc() as i32) as u8;
    if visibility_type > MAX_VISIBILITY_TYPE {
        return Err(usage_error(2));
    }
    Ok(visibility_type)
}

fn usage_error(index: i32) -> rilua::LuaError {
    bad_argument(index, VISIBILITY_API, VISIBILITY_USAGE)
}

fn secret_argument_error() -> rilua::LuaError {
    let error = format!(
        "{VISIBILITY_USAGE}. Secret values are only allowed during untainted execution for this argument."
    );
    bad_argument(2, VISIBILITY_API, &error)
}
