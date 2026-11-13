//! State-backed action cooldown and texture queries.

use super::cooldown_duration::{read_untainted_ignore_gcd, select_cooldown_duration_times};
use crate::lua_api::globals::lua_duration_object::push_timed_duration_object;
use crate::lua_api::methods::{borrow_state, create_table_with_capacity, table_set};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// These pinned profiles return SpellCooldownInfo; earlier profiles retain the
// four-field ActionBarCooldownInfo payload. This does not date its introduction.
const HAS_ACTIVE_FIELD: bool = cfg!(any(
    feature = "retail-12-1-0",
    feature = "client-wowforever"
));
const ACTION_COOLDOWN_HASH_FIELDS: usize = 4 + HAS_ACTIVE_FIELD as usize;

const SPELL_RESTRICTION_FUNCTION: &str = "SpellMatchesActionSpellTypeRestriction";
const SPELL_RESTRICTION_USAGE: &str = "Usage: local actionMatchesSpellTypeRestriction = C_ActionBar.SpellMatchesActionSpellTypeRestriction(spellID, actionID)";

pub(crate) fn register_ptr_methods(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Ptr {
        return Ok(());
    }
    let namespace = crate::c_api::helpers::ensure_namespace(state, "C_ActionBar")?;
    crate::lua_bridge::table_set_rust_fn_static(
        state,
        namespace,
        "ShouldShowKeyring",
        should_show_keyring,
    )?;
    crate::lua_bridge::table_set_rust_fn_static(
        state,
        namespace,
        SPELL_RESTRICTION_FUNCTION,
        spell_matches_action_spell_type_restriction,
    )?;
    Ok(())
}

fn should_show_keyring(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.action_bar_state.keyring_ui_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn unsigned_argument_error(index: i32, reason: &str) -> rilua::LuaError {
    rilua::runtime_error(format!(
        "bad argument #{index} to '{SPELL_RESTRICTION_FUNCTION}' ({reason})"
    ))
}

fn read_untainted_u32_argument(state: &LuaState, index: i32) -> LuaResult<u32> {
    let original = stack_val(state, index);
    if matches!(original, Val::Nil) {
        return Err(unsigned_argument_error(index, SPELL_RESTRICTION_USAGE));
    }
    let marked = rilua::table_security::is_secret_value(state, original);
    let value = rilua::table_security::unwrap_secret(state, original).map_err(|_| {
        unsigned_argument_error(
            index,
            &format!(
                "{SPELL_RESTRICTION_USAGE}. Secret values are only allowed during untainted execution for this argument."
            ),
        )
    })?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) if !marked => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| super::helpers::parse_numeric_string(string.data())),
        _ => None,
    };
    let number = number
        .filter(|number| (0.0..=f64::from(u32::MAX)).contains(number))
        .ok_or_else(|| {
            unsigned_argument_error(
                index,
                &format!("outside of expected range 0 to 4294967295 - {SPELL_RESTRICTION_USAGE}"),
            )
        })?;
    Ok(number.trunc() as u32)
}

/// R33: PTR WoW Hack slots that require a SpellLabel; other slots are unrestricted.
fn required_spell_label(action_id: u32) -> Option<i32> {
    match action_id {
        1..=12 | 157..=168 => Some(6590),
        61..=72 => Some(6595),
        145..=156 => Some(6609),
        _ => None,
    }
}

fn spell_matches_action_spell_type_restriction(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = super::helpers::read_untainted_signed32_argument(
        state,
        1,
        SPELL_RESTRICTION_FUNCTION,
        SPELL_RESTRICTION_USAGE,
    )?;
    let action_id = read_untainted_u32_argument(state, 2)?;
    if !(1..=408).contains(&action_id) {
        return Ok(0);
    }

    let matches = {
        let sim = borrow_state(state)?;
        if sim.game_rules.active_game_mode != crate::c_api::game_mode::WOWHACK {
            true
        } else if spell_id == 0 {
            false
        } else {
            match required_spell_label(action_id) {
                Some(label) => sim
                    .spell_labels
                    .get(&spell_id)
                    .is_some_and(|labels| labels.contains(&label)),
                None => true,
            }
        }
    };
    state.push(Val::Bool(matches));
    Ok(1)
}

pub(crate) fn unregister_action_ui_button(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = crate::lua_api::methods::extract_frame_id(state, stack_val(state, 1)) else {
        return Err(rilua::runtime_error(
            "UnregisterActionUIButton requires a frame",
        ));
    };
    crate::lua_api::methods::borrow_state_mut(state)?
        .action_ui_buttons
        .retain(|(button, _)| *button != id);
    Ok(0)
}

pub(crate) fn action_texture_path(sim: &crate::lua_api::SimState, slot: u32) -> Option<String> {
    if let Some(macro_id) = sim.action_macros.get(&slot) {
        let index = macro_id.checked_sub(1)? as usize;
        return sim
            .macros
            .get(index)
            .map(|entry| &entry.icon)
            .filter(|icon| !icon.is_empty())
            .cloned();
    }
    let spell_id = sim.action_bars.get(&slot)?;
    let spell = crate::spells::get_spell(*spell_id)?;
    crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id).map(str::to_string)
}

pub(crate) fn get_action_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let (start, duration) = read_action_cooldown(state)?;
    let restricted = {
        let sim = borrow_state(state)?;
        super::charge_state::cooldowns_are_restricted(&sim)
    };
    let is_enabled = true;
    let info = create_table_with_capacity(state, ACTION_COOLDOWN_HASH_FIELDS);
    // Root the public result before wrappers or field keys allocate.
    state.push(info);
    for (name, number) in [
        ("startTime", start),
        ("duration", duration),
        ("modRate", 1.0),
    ] {
        let value = if restricted {
            rilua::table_security::wrap_host_secret_number(state, number)
        } else {
            Val::Num(number)
        };
        table_set(state, info, name, value);
    }
    table_set(state, info, "isEnabled", Val::Bool(is_enabled));
    if HAS_ACTIVE_FIELD {
        let is_active = is_enabled && start != 0.0 && duration != 0.0;
        table_set(state, info, "isActive", Val::Bool(is_active));
    }
    Ok(1)
}

pub(crate) fn read_action_cooldown(state: &LuaState) -> LuaResult<(f64, f64)> {
    read_action_cooldown_with_gcd(state, false)
}

fn read_action_cooldown_with_gcd(state: &LuaState, ignore_gcd: bool) -> LuaResult<(f64, f64)> {
    let slot = match stack_val(state, 1) {
        Val::Num(number) if number >= 0.0 => Some(number as u32),
        _ => None,
    };
    let sim = borrow_state(state)?;
    let now = sim.clock_elapsed().as_secs_f64();
    Ok(slot
        .and_then(|slot| sim.action_bars.get(&slot).copied())
        .map(|spell_id| select_cooldown_duration_times(&sim, spell_id, now, ignore_gcd))
        .unwrap_or((0.0, 0.0)))
}

pub(crate) fn get_action_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let ignore_gcd = read_untainted_ignore_gcd(state, 2)?;
    let (start, seconds) = read_action_cooldown_with_gcd(state, ignore_gcd)?;
    push_timed_duration_object(state, start, seconds)
}
