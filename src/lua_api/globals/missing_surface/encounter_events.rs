//! `C_EncounterEvents` probe surface backed by a small mutable state table.
//!
//! Tests only need a stable encounter-event catalog plus color/sound override
//! round-tripping. The real client exposes a larger encounter-event system,
//! but this surface keeps the same shape for the functions currently covered
//! by the bucket tests.

use super::ensure_namespace;
use crate::lua_api::methods::{create_table, table_get, table_set, table_set_num, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const STATE_KEY: &str = "_state";
const COLORS_KEY: &str = "colors";
const SOUNDS_KEY: &str = "sounds";
const NEXT_HANDLE_KEY: &str = "nextSoundHandle";
const EVENT_IDS: &[u32] = &[1, 2, 3];

pub(super) fn register_encounter_events_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_EncounterEvents")?;
    register_encounter_timeline_surface(state)?;
    ensure_encounter_events_state(state);
    table_set_rust_fn_static(state, ns, "GetEventColor", get_event_color)?;
    table_set_rust_fn_static(state, ns, "GetEventInfo", get_event_info)?;
    table_set_rust_fn_static(state, ns, "GetEventList", get_event_list)?;
    table_set_rust_fn_static(state, ns, "GetEventSound", get_event_sound)?;
    table_set_rust_fn_static(state, ns, "HasEventInfo", has_event_info)?;
    table_set_rust_fn_static(state, ns, "PlayEventSound", play_event_sound)?;
    table_set_rust_fn_static(state, ns, "SetEventColor", set_event_color)?;
    table_set_rust_fn_static(state, ns, "SetEventSound", set_event_sound)?;
    Ok(())
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_encounter_timeline_surface(state: &mut LuaState) -> LuaResult<()> {
    if cfg!(feature = "retail-12-1-5") {
        // The modeled PTR namespace owns its ColorMixin return contract.
        return Ok(());
    }
    let timeline = ensure_namespace(state, "C_EncounterTimeline")?;
    table_set_rust_fn_static(state, timeline, "GetEventColor", get_timeline_event_color)
}

#[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
fn register_encounter_timeline_surface(_state: &mut LuaState) -> LuaResult<()> {
    Ok(())
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_timeline_event_color(state: &mut LuaState) -> LuaResult<u32> {
    let color = parse_event_id(stack_val(state, 1), state)
        .and_then(|event_id| copy_event_color(state, event_id));
    push_timeline_color_components(state, color);
    Ok(4)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn push_timeline_color_components(state: &mut LuaState, color: Option<Val>) {
    let color = color.unwrap_or(Val::Nil);
    for key in ["r", "g", "b", "a"] {
        let component = color_component(state, color, key);
        state.push(component);
    }
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn color_component(state: &mut LuaState, color: Val, key: &str) -> Val {
    match table_get(state, color, key) {
        Val::Num(value) => Val::Num(value),
        _ => Val::Num(1.0),
    }
}

fn get_event_list(state: &mut LuaState) -> LuaResult<u32> {
    let list = create_table(state);
    let Val::Table(list_ref) = list else {
        unreachable!("create_table must return a table");
    };
    for (index, event_id) in EVENT_IDS.iter().copied().enumerate() {
        table_set_num(
            state,
            list_ref,
            (index + 1) as f64,
            Val::Num(event_id as f64),
        );
    }
    state.push(list);
    Ok(1)
}

fn get_event_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
        return Ok(0);
    };
    if !is_known_event_id(event_id) {
        return Ok(0);
    }

    let info = create_table(state);
    table_set(state, info, "encounterEventID", Val::Num(event_id as f64));
    if let Some(color) = copy_event_color(state, event_id) {
        table_set(state, info, "color", color);
    }
    if let Some(sound) = copy_event_sound(state, event_id, 0) {
        table_set(state, info, "sound", sound);
    }
    state.push(info);
    Ok(1)
}

fn get_event_color(state: &mut LuaState) -> LuaResult<u32> {
    let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
        return Ok(0);
    };
    match copy_event_color(state, event_id) {
        Some(color) => {
            state.push(color);
            Ok(1)
        }
        None => Ok(0),
    }
}

fn set_event_color(state: &mut LuaState) -> LuaResult<u32> {
    let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
        return Ok(0);
    };
    if !is_known_event_id(event_id) {
        return Ok(0);
    }

    let color = match stack_val(state, 2) {
        Val::Nil => None,
        value @ Val::Table(_) => Some(copy_color_table(state, value)),
        _ => None,
    };
    let event_state = ensure_encounter_events_state(state);
    let colors = table_get(state, event_state, COLORS_KEY);
    match color {
        Some(color_table) => set_num_key(state, colors, event_id as f64, color_table),
        None => set_num_key(state, colors, event_id as f64, Val::Nil),
    }
    Ok(0)
}

fn get_event_sound(state: &mut LuaState) -> LuaResult<u32> {
    let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
        return Ok(0);
    };
    let Some(trigger) = parse_trigger(stack_val(state, 2), state) else {
        return Ok(0);
    };
    match copy_event_sound(state, event_id, trigger) {
        Some(sound) => {
            state.push(sound);
            Ok(1)
        }
        None => Ok(0),
    }
}

fn set_event_sound(state: &mut LuaState) -> LuaResult<u32> {
    let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
        return Ok(0);
    };
    let Some(trigger) = parse_trigger(stack_val(state, 2), state) else {
        return Ok(0);
    };
    if !is_known_event_id(event_id) {
        return Ok(0);
    }

    let sound = match stack_val(state, 3) {
        Val::Nil => None,
        value @ Val::Table(_) => Some(copy_sound_table(state, value)),
        _ => None,
    };
    let event_state = ensure_encounter_events_state(state);
    let sounds = table_get(state, event_state, SOUNDS_KEY);
    let trigger_table = match get_num_key(state, sounds, event_id as f64) {
        table @ Val::Table(_) => table,
        _ => {
            let trigger_table = create_table(state);
            set_num_key(state, sounds, event_id as f64, trigger_table);
            trigger_table
        }
    };
    match sound {
        Some(sound_table) => set_num_key(state, trigger_table, trigger as f64, sound_table),
        None => set_num_key(state, trigger_table, trigger as f64, Val::Nil),
    }
    Ok(0)
}

fn play_event_sound(state: &mut LuaState) -> LuaResult<u32> {
    let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
        return Ok(0);
    };
    let Some(trigger) = parse_trigger(stack_val(state, 2), state) else {
        return Ok(0);
    };
    if copy_event_sound(state, event_id, trigger).is_none() {
        return Ok(0);
    }

    let event_state = ensure_encounter_events_state(state);
    let next_handle = match table_get(state, event_state, NEXT_HANDLE_KEY) {
        Val::Num(number) if number >= 1.0 => number as i32,
        _ => 1,
    };
    table_set(
        state,
        event_state,
        NEXT_HANDLE_KEY,
        Val::Num((next_handle + 1) as f64),
    );
    state.push(Val::Num(next_handle as f64));
    Ok(1)
}

fn has_event_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(event_id) = parse_event_id(stack_val(state, 1), state) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    state.push(Val::Bool(is_known_event_id(event_id)));
    Ok(1)
}

fn ensure_encounter_events_state(state: &mut LuaState) -> Val {
    let namespace = Val::Table(
        ensure_namespace(state, "C_EncounterEvents")
            .expect("C_EncounterEvents namespace should exist"),
    );
    match table_get(state, namespace, STATE_KEY) {
        table @ Val::Table(_) => table,
        _ => {
            let event_state = create_table(state);
            let colors = create_table(state);
            let sounds = create_table(state);
            table_set(state, event_state, COLORS_KEY, colors);
            table_set(state, event_state, SOUNDS_KEY, sounds);
            table_set(state, event_state, NEXT_HANDLE_KEY, Val::Num(1.0));
            table_set(state, namespace, STATE_KEY, event_state);
            event_state
        }
    }
}

fn copy_event_color(state: &mut LuaState, event_id: u32) -> Option<Val> {
    let event_state = ensure_encounter_events_state(state);
    let colors = table_get(state, event_state, COLORS_KEY);
    let color = get_num_key(state, colors, event_id as f64);
    if matches!(color, Val::Nil) {
        None
    } else {
        Some(copy_color_table(state, color))
    }
}

fn copy_event_sound(state: &mut LuaState, event_id: u32, trigger: i32) -> Option<Val> {
    let event_state = ensure_encounter_events_state(state);
    let sounds = table_get(state, event_state, SOUNDS_KEY);
    let trigger_table = get_num_key(state, sounds, event_id as f64);
    let sound = get_num_key(state, trigger_table, trigger as f64);
    if matches!(sound, Val::Nil) {
        None
    } else {
        Some(copy_sound_table(state, sound))
    }
}

fn get_num_key(state: &LuaState, table: Val, key: f64) -> Val {
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.get(Val::Num(key), &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

fn set_num_key(state: &mut LuaState, table: Val, key: f64, value: Val) {
    let Val::Table(table_ref) = table else {
        return;
    };
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Num(key), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}

fn copy_color_table(state: &mut LuaState, color: Val) -> Val {
    let copy = create_table(state);
    for key in ["r", "g", "b", "a"] {
        let value = table_get(state, color, key);
        if !matches!(value, Val::Nil) {
            table_set(state, copy, key, value);
        }
    }
    copy
}

fn copy_sound_table(state: &mut LuaState, sound: Val) -> Val {
    let copy = create_table(state);
    for key in ["file", "channel", "volume"] {
        let value = table_get(state, sound, key);
        if !matches!(value, Val::Nil) {
            table_set(state, copy, key, value);
        }
    }
    copy
}

fn parse_event_id(value: Val, state: &mut LuaState) -> Option<u32> {
    parse_positive_u32(value, state)
}

fn parse_trigger(value: Val, state: &mut LuaState) -> Option<i32> {
    parse_nonnegative_i32(value, state)
}

fn parse_positive_u32(value: Val, state: &mut LuaState) -> Option<u32> {
    match value {
        Val::Num(number) if number > 0.0 => Some(number as u32),
        Val::Str(_) => val_to_string(state, value)
            .and_then(|text| text.parse::<u32>().ok())
            .filter(|number| *number > 0),
        _ => None,
    }
}

fn parse_nonnegative_i32(value: Val, state: &mut LuaState) -> Option<i32> {
    match value {
        Val::Num(number) if number >= 0.0 => Some(number as i32),
        Val::Str(_) => val_to_string(state, value)
            .and_then(|text| text.parse::<i32>().ok())
            .filter(|number| *number >= 0),
        _ => None,
    }
}

fn is_known_event_id(event_id: u32) -> bool {
    EVENT_IDS.contains(&event_id)
}
