//! Mists-only C API models from the R88ah native answer.

use super::helpers::{
    ensure_namespace, read_low_byte_enum_argument, read_untainted_signed32_argument,
};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
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
    read_low_byte_enum_argument(
        state,
        2,
        VISIBILITY_API,
        VISIBILITY_USAGE,
        MAX_VISIBILITY_TYPE,
    )
}
