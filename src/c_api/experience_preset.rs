//! R47/R50 CVar-backed experience-preset selection, Forever/PTR.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::cvars::CVarStorage;
use crate::lua_api::globals::set_cvar_verb::set_native_cvar;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state_mut, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) const CVAR: &str = "hasSelectedExperiencePreset";
const FUNCTION: &str = "SetForeverExperiencePresetPreLogin";
const USAGE: &str = "Usage: C_GameRules.SetForeverExperiencePresetPreLogin(preset)";

pub(crate) fn has_selected(cvars: &CVarStorage) -> LuaResult<bool> {
    Ok(cvars
        .cached_integer_nonzero(CVAR)
        .unwrap_or(ACTIVE == ClientProfile::Ptr))
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        return Ok(());
    }
    let namespace = super::helpers::ensure_namespace(state, "C_GameRules")?;
    table_set_rust_fn(state, namespace, FUNCTION, set_preset)?;
    let enums = super::helpers::ensure_global_table(state, "Enum");
    let presets = create_table(state);
    table_set(state, presets, "Classic", Val::Num(0.0));
    table_set(state, presets, "Enhanced", Val::Num(1.0));
    table_set(state, enums, "ForeverExperiencePreset", presets);
    Ok(())
}

fn argument_error(reason: &str) -> rilua::LuaError {
    rilua::runtime_error(format!("bad argument #1 to '{FUNCTION}' ({reason})"))
}

fn read_preset(state: &LuaState) -> LuaResult<u8> {
    let number = match stack_val(state, 1) {
        Val::Nil => return Err(argument_error(USAGE)),
        Val::Num(number) => number,
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|text| super::helpers::parse_numeric_string(text.data()))
            .unwrap_or(0.0),
        _ => 0.0,
    };
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(argument_error(&format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    let preset = number.trunc() as i32 as u8;
    if preset > 1 {
        return Err(argument_error(USAGE));
    }
    Ok(preset)
}

fn set_preset(state: &mut LuaState) -> LuaResult<u32> {
    let preset = read_preset(state)?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.game_rules.experience_preset_changed = true;
        sim.game_rules.classic_experience_chosen = Some(preset == 0);
    }
    // Unknown native refresh and optional server requests are intentionally omitted.
    // No synthetic validation callback or server response is introduced.
    let _accepted = set_native_cvar(state, CVAR, "1")?;
    dispatch_event_now(state, "LOGIN_STATE_CHANGED", &[])?;
    Ok(0)
}
