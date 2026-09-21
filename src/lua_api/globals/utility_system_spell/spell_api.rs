//! Spell-related globals: UnitHealth, UnitPower, UnitCastingInfo, CastSpellBy*.

use crate::lua_api::globals::unit_api::parse_party_index;
use crate::lua_api::globals::unit_stats::secondary_power_max;
use crate::lua_api::methods::{borrow_state, create_string, val_to_string};
use crate::lua_api::state_types::SecondaryPowerState;
use crate::lua_bridge::stack_val;
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table as RiluaTable;
use rilua::{LuaError, RuntimeError};
use rilua::{LuaResult, Val};

// ── Unit vitals ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub(super) struct UnitVitals {
    pub(super) health: i32,
    pub(super) health_max: i32,
    pub(super) power: i32,
    pub(super) power_max: i32,
    pub(super) power_type: i32,
    pub(super) power_type_name: String,
}

pub(super) fn lookup_unit_vitals(state: &LuaState, unit: &str) -> UnitVitals {
    let sim = borrow_state(state).expect("sim state should exist");
    if unit == "target"
        && let Some(target) = &sim.current_target
    {
        return UnitVitals {
            health: target.health,
            health_max: target.health_max,
            power: target.power,
            power_max: target.power_max,
            power_type: target.power_type,
            power_type_name: target.power_type_name.clone(),
        };
    }
    if let Some(index) = parse_party_index(unit)
        && let Some(member) = sim.party_members.get(index)
    {
        return UnitVitals {
            health: member.health,
            health_max: member.health_max,
            power: member.power,
            power_max: member.power_max,
            power_type: member.power_type,
            power_type_name: member.power_type_name.clone(),
        };
    }
    UnitVitals {
        health: sim.player.health,
        health_max: sim.player.health_max,
        power: sim.player.power,
        power_max: sim.player.power_max,
        power_type: sim.player.power_type,
        power_type_name: power_type_name(sim.player.power_type).to_string(),
    }
}

fn requested_power_type(state: &LuaState) -> Option<i64> {
    match stack_val(state, 2) {
        Val::Num(n) => Some(n as i64),
        _ => None,
    }
}

fn is_secondary_power_type(power_type: Option<i64>) -> bool {
    matches!(
        power_type,
        Some(4 | 5 | 6 | 7 | 8 | 9 | 11 | 12 | 13 | 16 | 17 | 18)
    )
}

pub(super) fn power_type_name(power_type: i32) -> &'static str {
    match power_type {
        0 => "MANA",
        1 => "RAGE",
        2 => "FOCUS",
        3 => "ENERGY",
        5 => "RUNES",
        6 => "RUNIC_POWER",
        7 => "SOUL_SHARDS",
        8 => "LUNAR_POWER",
        9 => "HOLY_POWER",
        11 => "MAELSTROM",
        13 => "INSANITY",
        17 => "FURY",
        18 => "PAIN",
        _ => "MANA",
    }
}

// ── Unit stat functions ──────────────────────────────────────────────────────

fn unit_health(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    state.push(Val::Num(vitals.health as f64));
    Ok(1)
}

fn unit_health_max(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    state.push(Val::Num(vitals.health_max as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn unit_health_missing(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    // Prediction state is unmodeled; usePredicted shares the current-health view.
    let missing = vitals.health_max as f64 - vitals.health as f64;
    state.push(Val::Num(missing));
    Ok(1)
}

fn unit_health_percent(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    let percent = if vitals.health_max > 0 {
        (vitals.health as f64 / vitals.health_max as f64) * 100.0
    } else {
        0.0
    };
    // Prediction is unmodeled. The existing 0..100 scale, including curve input,
    // is simulator policy; native input units remain unverified.
    let result = Val::Num(percent);
    #[cfg(feature = "retail-12-0-0")]
    let result = match stack_val(state, 3) {
        Val::Nil => result,
        curve => crate::c_api::c_curve_util::evaluate_curve_value(state, curve, percent)?,
    };
    state.push(result);
    Ok(1)
}

fn unit_power(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    let power = requested_power_values(state, &unit, &vitals).current;
    state.push(Val::Num(power as f64));
    Ok(1)
}

fn unit_power_max(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    let power_max = requested_power_values(state, &unit, &vitals).max;
    state.push(Val::Num(power_max as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn unit_power_missing(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    let power = requested_power_values(state, &unit, &vitals);
    // Unmodified resource scaling is unmodeled, as in UnitPower/UnitPowerMax.
    state.push(Val::Num(power.max as f64 - power.current as f64));
    Ok(1)
}

fn unit_power_percent(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    let power = requested_power_values(state, &unit, &vitals);
    let percent = if power.max > 0 {
        (power.current as f64 / power.max as f64) * 100.0
    } else {
        0.0
    };
    // Unmodified scaling is unmodeled. The existing 0..100 curve input is
    // simulator policy; native input units remain unverified.
    let result = Val::Num(percent);
    #[cfg(feature = "retail-12-0-0")]
    let result = match stack_val(state, 4) {
        Val::Nil => result,
        curve => crate::c_api::c_curve_util::evaluate_curve_value(state, curve, percent)?,
    };
    state.push(result);
    Ok(1)
}

fn requested_power_values(
    state: &LuaState,
    unit: &str,
    vitals: &UnitVitals,
) -> SecondaryPowerState {
    let Some(requested) = requested_power_type(state) else {
        return SecondaryPowerState {
            current: vitals.power,
            max: vitals.power_max,
        };
    };
    let requested = requested as i32;
    if requested == vitals.power_type {
        return SecondaryPowerState {
            current: vitals.power,
            max: vitals.power_max,
        };
    }
    if unit == "player"
        && is_secondary_power_type(Some(requested.into()))
        && let Some(power) = lookup_secondary_player_power(state, requested)
    {
        return power;
    }
    if is_secondary_power_type(Some(requested.into())) {
        return SecondaryPowerState {
            current: 0,
            max: secondary_power_max(requested),
        };
    }
    SecondaryPowerState {
        current: vitals.power,
        max: vitals.power_max,
    }
}

fn lookup_secondary_player_power(state: &LuaState, power_type: i32) -> Option<SecondaryPowerState> {
    let sim = borrow_state(state).expect("sim state should exist");
    sim.player.secondary_powers.get(&power_type).copied()
}

fn unit_power_type(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let vitals = lookup_unit_vitals(state, &unit);
    let power_type_name_val = create_string(state, &vitals.power_type_name);
    state.push(Val::Num(vitals.power_type as f64));
    state.push(power_type_name_val);
    Ok(2)
}

fn unit_power_bar_id(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    state.push(Val::Num(0.0));
    Ok(1)
}

fn unit_get_incoming_heals(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn unit_get_total_absorbs(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn unit_get_total_heal_absorbs(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn unit_get_detailed_heal_prediction(state: &mut LuaState) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_else(|| "player".to_string());
    let calculator = stack_val(state, 3);
    let vitals = lookup_unit_vitals(state, &unit);
    let predicted_values = heal_prediction_values_table(state, &vitals)?;
    set_calculator_field(state, calculator, "_predictedValues", predicted_values)?;
    set_calculator_field(state, calculator, "_hasSecretValues", Val::Bool(false))?;
    Ok(0)
}

fn heal_prediction_values_table(state: &mut LuaState, vitals: &UnitVitals) -> LuaResult<Val> {
    let mut table = RiluaTable::new();
    set_table_number(state, &mut table, "health", vitals.health as f64)?;
    set_table_number(state, &mut table, "healthMax", vitals.health_max as f64)?;
    set_table_number(state, &mut table, "totalDamageAbsorbs", 0.0)?;
    set_table_number(state, &mut table, "totalHealAbsorbs", 0.0)?;
    set_table_number(state, &mut table, "totalIncomingHeals", 0.0)?;
    set_table_number(state, &mut table, "totalIncomingHealsFromHealer", 0.0)?;
    Ok(Val::Table(state.gc.alloc_table(table)))
}

fn set_table_number(
    state: &mut LuaState,
    table: &mut RiluaTable,
    field: &str,
    value: f64,
) -> LuaResult<()> {
    let key = Val::Str(state.gc.intern_string(field.as_bytes()));
    table.raw_set(key, Val::Num(value), &state.gc.string_arena)
}

fn set_calculator_field(
    state: &mut LuaState,
    calculator: Val,
    field: &str,
    value: Val,
) -> LuaResult<()> {
    if !matches!(calculator, Val::Table(_)) {
        return Err(LuaError::Runtime(RuntimeError {
            message: "UnitHealPredictionCalculator expected".into(),
            level: 0,
            traceback: Vec::new(),
        }));
    }
    let key = Val::Str(state.gc.intern_string(field.as_bytes()));
    state.settable(calculator, key, value)
}

// ── Cast info readers ────────────────────────────────────────────────────────
//
// CastSpellByID / CastSpellByName are registered from
// `src/lua_api/globals/combat_verbs.rs` — they drive `SimState.casting`.

#[derive(Clone, Copy)]
enum CastSlot {
    Casting,
    Channeling,
}

struct CastInfoSnapshot {
    spell_name: String,
    icon_path: String,
    start_time: f64,
    end_time: f64,
    cast_id: u32,
    spell_id: u32,
    num_empower_stages: u32,
    #[cfg(feature = "player-cast-durations")]
    delay_time: f64,
}

fn unit_casting_info(state: &mut LuaState) -> LuaResult<u32> {
    push_unit_cast_info(state, CastSlot::Casting)
}

fn unit_channel_info(state: &mut LuaState) -> LuaResult<u32> {
    push_unit_cast_info(state, CastSlot::Channeling)
}

fn push_unit_cast_info(state: &mut LuaState, slot: CastSlot) -> LuaResult<u32> {
    let unit = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    if unit != "player" {
        return Ok(0);
    }
    let Some(cast_info) = extract_cast_info(state, slot)? else {
        return Ok(0);
    };
    if matches!(slot, CastSlot::Channeling) {
        return Ok(push_channel_info(state, &cast_info));
    }
    Ok(push_cast_info(state, &cast_info))
}

fn extract_cast_info(state: &mut LuaState, slot: CastSlot) -> LuaResult<Option<CastInfoSnapshot>> {
    let sim = borrow_state(state)?;
    let source = match slot {
        CastSlot::Casting => sim.casting.as_ref(),
        CastSlot::Channeling => sim.channeling.as_ref(),
    };
    Ok(source.map(|cast| CastInfoSnapshot {
        spell_name: cast.spell_name.clone(),
        icon_path: cast.icon_path.clone(),
        start_time: cast.start_time,
        end_time: cast.end_time,
        cast_id: cast.cast_id,
        spell_id: cast.spell_id,
        num_empower_stages: cast.empower_stage_count() as u32,
        #[cfg(feature = "player-cast-durations")]
        delay_time: cast.delay_time,
    }))
}

fn push_common_cast_fields(state: &mut LuaState, cast_info: &CastInfoSnapshot) {
    let spell_name_val = create_string(state, &cast_info.spell_name);
    let spell_name_display_val = create_string(state, &cast_info.spell_name);
    let icon_path_val = create_string(state, &cast_info.icon_path);
    state.push(spell_name_val);
    state.push(spell_name_display_val);
    state.push(icon_path_val);
    state.push(Val::Num(cast_info.start_time * 1000.0));
    state.push(Val::Num(cast_info.end_time * 1000.0));
    state.push(Val::Bool(false));
}

fn push_cast_info(state: &mut LuaState, cast_info: &CastInfoSnapshot) -> u32 {
    push_common_cast_fields(state, cast_info);
    #[cfg(feature = "player-cast-durations")]
    {
        let guid = crate::lua_api::spellcast_events::cast_guid(cast_info.cast_id);
        let value = create_string(state, &guid);
        state.push(value);
    }
    #[cfg(not(feature = "player-cast-durations"))]
    state.push(Val::Num(cast_info.cast_id as f64));
    state.push(Val::Bool(false));
    state.push(Val::Num(cast_info.spell_id as f64));
    #[cfg(feature = "player-cast-durations")]
    {
        state.push(Val::Num(cast_info.cast_id as f64));
        state.push(Val::Num(cast_info.delay_time * 1000.0));
        11
    }
    #[cfg(not(feature = "player-cast-durations"))]
    {
        9
    }
}

fn push_channel_info(state: &mut LuaState, cast_info: &CastInfoSnapshot) -> u32 {
    push_common_cast_fields(state, cast_info);
    state.push(Val::Bool(false));
    state.push(Val::Num(cast_info.spell_id as f64));
    state.push(Val::Bool(cast_info.num_empower_stages > 0));
    state.push(Val::Num(cast_info.num_empower_stages as f64));
    #[cfg(feature = "player-cast-durations")]
    {
        state.push(Val::Num(cast_info.cast_id as f64));
        11
    }
    #[cfg(not(feature = "player-cast-durations"))]
    {
        10
    }
}

// ── Registration ─────────────────────────────────────────────────────────────

pub(super) fn register_spell_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    #[cfg(feature = "player-cast-durations")]
    crate::lua_api::channeling::register_queries(lua)?;
    LuaApiMut::register_function(lua, "UnitHealth", unit_health)?;
    LuaApiMut::register_function(lua, "UnitHealthMax", unit_health_max)?;
    #[cfg(feature = "retail-12-0-0")]
    LuaApiMut::register_function(lua, "UnitHealthMissing", unit_health_missing)?;
    LuaApiMut::register_function(lua, "UnitHealthPercent", unit_health_percent)?;
    LuaApiMut::register_function(lua, "UnitPower", unit_power)?;
    LuaApiMut::register_function(lua, "UnitPowerMax", unit_power_max)?;
    #[cfg(feature = "retail-12-0-0")]
    LuaApiMut::register_function(lua, "UnitPowerMissing", unit_power_missing)?;
    LuaApiMut::register_function(lua, "UnitPowerPercent", unit_power_percent)?;
    LuaApiMut::register_function(lua, "UnitPowerBarID", unit_power_bar_id)?;
    LuaApiMut::register_function(lua, "UnitPowerType", unit_power_type)?;
    LuaApiMut::register_function(lua, "UnitGetIncomingHeals", unit_get_incoming_heals)?;
    LuaApiMut::register_function(lua, "UnitGetTotalAbsorbs", unit_get_total_absorbs)?;
    LuaApiMut::register_function(lua, "UnitGetTotalHealAbsorbs", unit_get_total_heal_absorbs)?;
    LuaApiMut::register_function(
        lua,
        "UnitGetDetailedHealPrediction",
        unit_get_detailed_heal_prediction,
    )?;
    LuaApiMut::register_function(lua, "UnitCastingInfo", unit_casting_info)?;
    LuaApiMut::register_function(lua, "UnitChannelInfo", unit_channel_info)?;
    LuaApiMut::register_function(
        lua,
        "PlayerGetTimerunningSeasonID",
        crate::c_api::c_spec::player_get_timerunning_season_id,
    )?;
    LuaApiMut::register_function(
        lua,
        "PlayerIsTimerunning",
        crate::c_api::c_spec::player_is_timerunning,
    )?;
    Ok(())
}
