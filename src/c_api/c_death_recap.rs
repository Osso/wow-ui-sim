//! `C_DeathRecap` probe surface backed by `SimState.death_recaps`.
//!
//! - `C_DeathRecap.GetKillingBlows()` returns an array of `KillingBlowInfo`
//!   tables from the most recent death recap entry, or an empty array when no
//!   deaths have been recorded.
//! - `C_DeathRecap.GetMostRecentDeathRecap()` returns the most recent
//!   `DeathRecapEntry` as a table (`recapID`, `zoneName`, `killingBlows`), or
//!   nil when the list is empty.

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, create_table, table_get, table_set,
    val_to_string,
};
use crate::lua_api::state::{DeathRecapEntry, KillingBlowInfo};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_death_recap_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_DeathRecap")?;
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        table_set_rust_fn_static(state, table_ref, "GetRecapEvents", get_recap_events)?;
        table_set_rust_fn_static(state, table_ref, "HasRecapEvents", has_recap_events)?;
        table_set_rust_fn_static(state, table_ref, "GetRecapLink", get_recap_link)?;
    }
    #[cfg(feature = "retail-12-0-5")]
    table_set_rust_fn_static(state, table_ref, "GetRecapMaxHealth", get_recap_max_health)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetKillingBlows",
        c_death_recap_get_killing_blows,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetMostRecentDeathRecap",
        c_death_recap_get_most_recent_death_recap,
    )?;
    Ok(())
}

fn read_recap_id(state: &LuaState) -> LuaResult<Option<i32>> {
    let value = stack_val(state, 1);
    if value == Val::Nil {
        return Ok(None);
    }
    let number = match value {
        Val::Num(number) => number,
        Val::Str(_) => val_to_string(state, value)
            .and_then(|text| text.trim().parse().ok())
            .unwrap_or(0.0),
        _ => 0.0,
    };
    if !number.is_finite() || number < i32::MIN as f64 || number > i32::MAX as f64 {
        return Err(rilua::runtime_error(format!(
            "integer overflow attempting to store {number}"
        )));
    }
    Ok(Some(number as i32))
}

fn read_strict_recap_id(state: &LuaState, usage: &str) -> LuaResult<Option<i32>> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if value == Val::Nil {
        return Ok(None);
    }
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value).and_then(|text| text.trim().parse().ok()),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error(usage))?;
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(rilua::runtime_error(format!(
            "integer overflow attempting to store {number}"
        )));
    }
    Ok(Some(number as i32))
}

fn find_recap(recaps: &[DeathRecapEntry], id: Option<i32>) -> Option<&DeathRecapEntry> {
    let mut retained = recaps.iter().rev().take(60);
    match id {
        Some(id) => retained.find(|entry| i64::from(entry.recap_id) == i64::from(id)),
        None => retained.find(|entry| entry.default_eligible),
    }
}

fn has_recap_events(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_strict_recap_id(
        state,
        "Usage: local hasEvents = C_DeathRecap.HasRecapEvents([recapID])",
    )?;
    let found = find_recap(&borrow_state(state)?.death_recaps, id).is_some();
    state.push(Val::Bool(found));
    Ok(1)
}

fn get_recap_link(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_strict_recap_id(
        state,
        "Usage: local link = C_DeathRecap.GetRecapLink([recapID])",
    )?;
    let recap_id = find_recap(&borrow_state(state)?.death_recaps, id).map(|entry| entry.recap_id);
    let globals = Val::Table(state.global);
    let result = match recap_id {
        Some(recap_id) => {
            let template = table_get(state, globals, "DEATH_RECAP_LINK");
            let strings = table_get(state, globals, "string");
            let format = table_get(state, strings, "format");
            call_function_state(state, format, &[template, Val::Num(f64::from(recap_id))])?
        }
        None => table_get(state, globals, "DEATH_RECAP_TEXT"),
    };
    state.push(result);
    Ok(1)
}

fn get_recap_events(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_recap_id(state)?;
    let events = {
        let sim = borrow_state(state)?;
        find_recap(&sim.death_recaps, id)
            .map(|entry| entry.events.iter().take(10).cloned().collect::<Vec<_>>())
    };
    let Some(events) = events else {
        return Ok(0);
    };
    let array = create_table(state);
    for (index, event) in events.iter().enumerate() {
        let item = create_table(state);
        table_set(state, item, "currentHP", Val::Num(event.current_hp as f64));
        if let Some(amount) = event.amount {
            table_set(state, item, "amount", Val::Num(amount as f64));
        }
        if let Some(timestamp) = event.timestamp {
            table_set(state, item, "timestamp", Val::Num(timestamp));
        }
        if let Some(source_guid) = &event.source_guid {
            let value = create_string(state, source_guid);
            table_set(state, item, "sourceGUID", value);
        }
        set_table_array(state, array, index as i64 + 1, item);
    }
    state.push(array);
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn get_recap_max_health(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::FromStack;
    let id = Option::<u32>::from_stack(state, 1)?;
    let maximum = {
        let sim = borrow_state(state)?;
        let entry = match id {
            Some(id) => sim.death_recaps.iter().find(|entry| entry.recap_id == id),
            None => sim.death_recaps.last(),
        };
        // INFERRED: omitted recap selects the newest; missing recap returns zero.
        entry.map_or(0, |entry| entry.max_health)
    };
    state.push(Val::Num(maximum as f64));
    Ok(1)
}

fn c_death_recap_get_killing_blows(state: &mut LuaState) -> LuaResult<u32> {
    let blows: Vec<KillingBlowInfo> = borrow_state(state)?
        .death_recaps
        .last()
        .map(|e| e.killing_blows.clone())
        .unwrap_or_default();
    let array = create_table(state);
    for (i, blow) in blows.iter().enumerate() {
        let entry = push_killing_blow_table(state, blow);
        set_table_array(state, array, i as i64 + 1, entry);
    }
    state.push(array);
    Ok(1)
}

fn c_death_recap_get_most_recent_death_recap(state: &mut LuaState) -> LuaResult<u32> {
    let entry: Option<DeathRecapEntry> = borrow_state(state)?.death_recaps.last().cloned();
    match entry {
        None => {
            state.push(Val::Nil);
            Ok(1)
        }
        Some(recap) => {
            let t = create_table(state);
            table_set(state, t, "recapID", Val::Num(recap.recap_id as f64));
            let zone = create_string(state, &recap.zone_name);
            table_set(state, t, "zoneName", zone);
            let blows_array = create_table(state);
            for (i, blow) in recap.killing_blows.iter().enumerate() {
                let blow_t = push_killing_blow_table(state, blow);
                set_table_array(state, blows_array, i as i64 + 1, blow_t);
            }
            table_set(state, t, "killingBlows", blows_array);
            state.push(t);
            Ok(1)
        }
    }
}

fn push_killing_blow_table(state: &mut LuaState, blow: &KillingBlowInfo) -> Val {
    let t = create_table(state);
    table_set(state, t, "spellID", Val::Num(blow.spell_id as f64));
    let ability_name = create_string(state, &blow.ability_name);
    table_set(state, t, "abilityName", ability_name);
    let caster_name = create_string(state, &blow.caster_name);
    table_set(state, t, "casterName", caster_name);
    table_set(state, t, "amount", Val::Num(blow.amount as f64));
    table_set(state, t, "isOverkill", Val::Bool(blow.is_overkill));
    t
}
