//! `C_DeathRecap` probe surface backed by `SimState.death_recaps`.
//!
//! - `C_DeathRecap.GetKillingBlows()` returns an array of `KillingBlowInfo`
//!   tables from the most recent death recap entry, or an empty array when no
//!   deaths have been recorded.
//! - `C_DeathRecap.GetMostRecentDeathRecap()` returns the most recent
//!   `DeathRecapEntry` as a table (`recapID`, `zoneName`, `killingBlows`), or
//!   nil when the list is empty.

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_api::state::{DeathRecapEntry, KillingBlowInfo};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_death_recap_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_DeathRecap")?;
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
