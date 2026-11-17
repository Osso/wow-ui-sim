//! Mists-only `C_CombatAudioAlert` spec settings from the R88ai native answer.

use super::helpers::{
    ensure_global_table, ensure_namespace, read_low_byte_enum_argument,
    read_untainted_signed32_argument,
};
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const GET_API: &str = "C_CombatAudioAlert.GetSpecSetting";
const GET_USAGE: &str = "Usage: local value = C_CombatAudioAlert.GetSpecSetting(setting)";
const SET_API: &str = "C_CombatAudioAlert.SetSpecSetting";
const SET_USAGE: &str = "Usage: local success = C_CombatAudioAlert.SetSpecSetting(setting, newVal)";
/// `Enum.CombatAudioAlertSpecSetting.MaxValue`.
const MAX_SETTING: u8 = 8;
const SETTING_COUNT: usize = MAX_SETTING as usize + 1;
/// Four specialization slots plus the fallback slot.
const SLOT_COUNT: usize = 5;
const FALLBACK_SLOT: usize = SLOT_COUNT - 1;
/// Loaded value for an absent list position, in `Enum.CombatAudioAlertSpecSetting` order.
const ABSENT_VALUES: [i32; SETTING_COUNT] = [0, 1, 0, 100, 0, 1, 0, 100, 1];
/// Native `Enum.CombatAudioAlertSpecSetting` members, in value order.
const SETTING_NAMES: [&str; SETTING_COUNT] = [
    "Resource1Percent",
    "Resource1Format",
    "Resource1Voice",
    "Resource1Volume",
    "Resource2Percent",
    "Resource2Format",
    "Resource2Voice",
    "Resource2Volume",
    "SayIfTargeted",
];

/// In-memory per-setting spec lists. Every registered CVar default is empty,
/// so each loaded position holds its absent value.
#[derive(Clone, Debug)]
pub(crate) struct CombatAudioSpecSettings([[i32; SLOT_COUNT]; SETTING_COUNT]);

impl Default for CombatAudioSpecSettings {
    fn default() -> Self {
        Self(ABSENT_VALUES.map(|value| [value; SLOT_COUNT]))
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    publish_spec_setting_enum(state);
    let namespace = ensure_namespace(state, "C_CombatAudioAlert")?;
    table_set_rust_fn_static(state, namespace, "GetSpecSetting", get_spec_setting)?;
    table_set_rust_fn_static(state, namespace, "SetSpecSetting", set_spec_setting)
}

/// Replace the legacy five-member table with the native nine-member publication.
fn publish_spec_setting_enum(state: &mut LuaState) {
    let enums = ensure_global_table(state, "Enum");
    let values = create_table(state);
    table_set(state, enums, "CombatAudioAlertSpecSetting", values);
    for (value, name) in SETTING_NAMES.into_iter().enumerate() {
        table_set(state, values, name, Val::Num(value as f64));
    }
    let meta = create_table(state);
    table_set(state, enums, "CombatAudioAlertSpecSettingMeta", meta);
    table_set(state, meta, "NumValues", Val::Num(SETTING_COUNT as f64));
    table_set(state, meta, "MinValue", Val::Num(0.0));
    table_set(state, meta, "MaxValue", Val::Num(f64::from(MAX_SETTING)));
}

fn get_spec_setting(state: &mut LuaState) -> LuaResult<u32> {
    let setting = read_low_byte_enum_argument(state, 1, GET_API, GET_USAGE, MAX_SETTING)?;
    let value = loaded_value(&*borrow_state(state)?, setting);
    state.push(Val::Num(f64::from(value)));
    Ok(1)
}

fn set_spec_setting(state: &mut LuaState) -> LuaResult<u32> {
    let setting = read_low_byte_enum_argument(state, 1, SET_API, SET_USAGE, MAX_SETTING)?;
    let value = read_untainted_signed32_argument(state, 2, SET_API, SET_USAGE)?;
    let stored = store_value(&mut *borrow_state_mut(state)?, setting, value);
    state.push(Val::Bool(stored));
    Ok(1)
}

/// Settings that are not loaded read as 0 for every setting.
fn loaded_value(sim: &SimState, setting: u8) -> i32 {
    if !sim.mists_combat_audio_alert_state_gate {
        return 0;
    }
    sim.mists_combat_audio_spec_settings.0[usize::from(setting)][spec_slot(sim)]
}

/// Store the full value; the CVar text encoding is UNKNOWN, so no CVar write.
fn store_value(sim: &mut SimState, setting: u8, value: i32) -> bool {
    let voice_count = sim.mists_combat_audio_voice_count;
    if !sim.mists_combat_audio_alert_state_gate || !accepts(setting, value, voice_count) {
        return false;
    }
    let slot = spec_slot(sim);
    sim.mists_combat_audio_spec_settings.0[usize::from(setting)][slot] = value;
    true
}

fn accepts(setting: u8, value: i32, voice_count: usize) -> bool {
    let low_byte = value as u8;
    match setting {
        // Resource1/2 Percent and Format.
        0 | 1 | 4 | 5 => low_byte <= 5,
        // Resource1/2 Voice.
        2 | 6 => usize::try_from(value).is_ok_and(|index| index < voice_count),
        // Resource1/2 Volume.
        3 | 7 => (0..=100).contains(&value),
        // SayIfTargeted.
        _ => low_byte <= 3,
    }
}

/// INFERRED (R88ai): the slot field is ChrSpecialization OrderIndex, which is
/// the 1-based active specialization index minus one.
fn spec_slot(sim: &SimState) -> usize {
    match sim.player.active_spec_index {
        index @ 1..=4 => (index - 1) as usize,
        _ => FALLBACK_SLOT,
    }
}
