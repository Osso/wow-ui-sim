//! INFERRED bounded registration/context model; no native playback claim.

use super::{AuraSoundRegistration, PrivateAuraSoundRegistrations};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

// Both entry points identify the original bounded contract, including cached aliases.
const API: &str = "C_UnitAuras.AddPrivateAuraAppliedSound";
const ADDED: u32 = 0;

fn input_error(message: &str) -> rilua::LuaError {
    runtime_error(format!("{API}: {message}"))
}

pub(super) fn add_private(state: &mut LuaState) -> LuaResult<u32> {
    let sound = unwrap_secret(state, stack_val(state, 1))
        .map_err(|error| input_error(&format!("secret sound argument access denied: {error}")))?;
    authenticate_extra_arguments(state, 1)?;
    let fields = authenticate_fields(state, sound)?;
    let registration = read_sound(state, fields, ADDED)?;
    register_sound(state, registration)
}

#[cfg(feature = "retail-12-1-0")]
pub(super) fn add_modern(state: &mut LuaState) -> LuaResult<u32> {
    // Authenticate BOTH originals before any type/domain/host-context validation.
    let trigger = unwrap_secret(state, stack_val(state, 1))
        .map_err(|error| input_error(&format!("secret trigger argument access denied: {error}")))?;
    let sound = unwrap_secret(state, stack_val(state, 2))
        .map_err(|error| input_error(&format!("secret sound argument access denied: {error}")))?;
    authenticate_extra_arguments(state, 2)?;
    let fields = authenticate_fields(state, sound)?;
    let trigger = read_number(trigger, "trigger must be an integral u32")?;
    if trigger > 2 {
        return Err(input_error(
            "trigger must be Added0, ApplicationsIncreased1 or Removed2",
        ));
    }
    let registration = read_sound(state, fields, trigger)?;
    register_sound(state, registration)
}

fn authenticate_extra_arguments(state: &LuaState, declared_count: usize) -> LuaResult<()> {
    for value in state
        .stack
        .iter()
        .take(state.top)
        .skip(state.base + declared_count)
    {
        unwrap_secret(state, *value).map_err(|error| {
            input_error(&format!("secret extra argument access denied: {error}"))
        })?;
    }
    Ok(())
}

fn authenticate_table_entries(state: &LuaState, sound: Val) -> LuaResult<()> {
    let Val::Table(reference) = sound else {
        return Ok(());
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| input_error("sound table is unavailable"))?;
    let mut previous_key = Val::Nil;
    while let Some((key, value)) = table.next(previous_key, &state.gc.string_arena)? {
        unwrap_secret(state, key)?;
        unwrap_secret(state, value)?;
        previous_key = key;
    }
    Ok(())
}

fn authenticate_fields(state: &mut LuaState, sound: Val) -> LuaResult<Option<[Val; 5]>> {
    authenticate_table_entries(state, sound)
        .map_err(|error| input_error(&format!("secret sound field access denied: {error}")))?;
    if !matches!(sound, Val::Table(_)) {
        return Ok(None);
    }
    let keys = [
        "unitToken",
        "spellID",
        "soundFileName",
        "soundFileID",
        "outputChannel",
    ];
    let originals = keys.map(|key| table_get(state, sound, key));
    // Authenticate every field before malformed public fields can mask denial.
    let unit = unwrap_secret(state, originals[0])?;
    let spell = unwrap_secret(state, originals[1])?;
    let file_name = unwrap_secret(state, originals[2])?;
    let file_id = unwrap_secret(state, originals[3])?;
    let channel = unwrap_secret(state, originals[4])?;
    Ok(Some([unit, spell, file_name, file_id, channel]))
}

fn read_sound(
    state: &LuaState,
    fields: Option<[Val; 5]>,
    trigger: u32,
) -> LuaResult<AuraSoundRegistration> {
    let [unit, spell, file_name, file_id, channel] =
        fields.ok_or_else(|| input_error("sound must be a table"))?;
    let unit_token = read_string(state, unit, "unitToken must be a nonempty string")?;
    let spell_id = read_number(spell, "spellID must be an integral u32")?;
    let sound_file_name = read_optional_string(state, file_name)?;
    let sound_file_id = read_optional_number(file_id)?;
    let output_channel = read_optional_string(state, channel)?;
    if sound_file_name.is_some() == sound_file_id.is_some() {
        return Err(input_error(
            "exactly one soundFileName or soundFileID is required",
        ));
    }
    Ok(AuraSoundRegistration {
        trigger,
        unit_token,
        spell_id,
        sound_file_name,
        sound_file_id,
        output_channel,
    })
}

fn read_string(state: &LuaState, value: Val, message: &str) -> LuaResult<String> {
    let text = val_to_string(state, value).ok_or_else(|| input_error(message))?;
    if text.is_empty() {
        return Err(input_error(message));
    }
    Ok(text)
}

fn read_number(value: Val, message: &str) -> LuaResult<u32> {
    let Val::Num(number) = value else {
        return Err(input_error(message));
    };
    let integral = number.is_finite() && number.fract() == 0.0;
    let in_range = number >= 0.0 && number <= f64::from(u32::MAX);
    if integral && in_range {
        return Ok(number as u32);
    }
    Err(input_error(message))
}

fn read_optional_string(state: &LuaState, value: Val) -> LuaResult<Option<String>> {
    if value.is_nil() {
        return Ok(None);
    }
    read_string(
        state,
        value,
        "optional sound text must be a nonempty string",
    )
    .map(Some)
}

fn read_optional_number(value: Val) -> LuaResult<Option<u32>> {
    if value.is_nil() {
        return Ok(None);
    }
    read_number(value, "soundFileID must be an integral u32").map(Some)
}

fn check_context(state: &LuaState) -> LuaResult<()> {
    let sim = borrow_state(state)?;
    #[cfg(feature = "retail-12-0-7")]
    let mythic_restricted = sim.mythic_plus.is_active && sim.player.in_combat;
    #[cfg(not(feature = "retail-12-0-7"))]
    let mythic_restricted = sim.mythic_plus.is_active;
    let restricted = sim.world.encounter_in_progress
        || mythic_restricted
        || sim.private_aura_sound_registrations.pvp_match_active;
    // INFERRED: encounter/PvP restrictions still win over out-of-combat M+ permission.
    if restricted && !rilua::api::state_is_secure(state) {
        return Err(input_error(
            "insecure registration denied during encounter/M+/PvP match",
        ));
    }
    Ok(())
}

fn allocate_id(sounds: &mut PrivateAuraSoundRegistrations) -> Option<u32> {
    let mut candidate = sounds.next_id?;
    // Each collision corresponds to a stored live ID, not a scan of the u32 domain.
    while sounds.live_ids.contains(&candidate) {
        sounds.next_id = candidate.checked_add(1);
        candidate = sounds.next_id?;
    }
    sounds.next_id = candidate.checked_add(1);
    Some(candidate)
}

fn register_sound(state: &mut LuaState, registration: AuraSoundRegistration) -> LuaResult<u32> {
    check_context(state)?;
    let id = {
        let mut sim = borrow_state_mut(state)?;
        let sounds = &mut sim.private_aura_sound_registrations;
        let id = allocate_id(sounds);
        if let Some(id) = id {
            sounds.live_ids.insert(id);
            sounds.registrations.insert(id, registration);
        }
        id
    };
    state.push(match id {
        Some(id) => Val::Num(f64::from(id)),
        None => Val::Nil,
    });
    Ok(1)
}
