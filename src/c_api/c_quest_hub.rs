//! `C_QuestHub` 12.0.7 host-backed race discovery.
//! INFERRED contract: see docs/specs/quest-hub-dragonriding-races-12-0-7.md.

use crate::c_api::helpers::ensure_namespace;
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
use crate::lua_api::methods::{borrow_state, create_table, table_set_num};
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register_c_quest_hub_surface(state: &mut LuaState) -> LuaResult<()> {
    let quest_hub = ensure_namespace(state, "C_QuestHub")?;
    register_patch_12_0_7_quest_hub_surface(state, quest_hub)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_patch_12_0_7_quest_hub_surface(
    state: &mut LuaState,
    quest_hub: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        quest_hub,
        "GetDragonridingRacesForAreaPOI",
        get_dragonriding_races_for_area_poi,
    )
}

#[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
fn register_patch_12_0_7_quest_hub_surface(
    _state: &mut LuaState,
    _quest_hub: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_dragonriding_races_for_area_poi(state: &mut LuaState) -> LuaResult<u32> {
    let area_poi_id = read_authenticated_area_poi_id(state)?;
    // INFERRED: live per-environment input, host order, empty default/unknown POI.
    let race_ids = borrow_state(state)?
        .quest_hub_dragonriding_races
        .get(&area_poi_id)
        .cloned()
        .unwrap_or_default();
    // INFERRED: exactly one fresh detached array of numeric race POI IDs,
    // matching the related GetDragonridingRacesForMap element type.
    let races = create_table(state);
    let Val::Table(races_ref) = races else {
        unreachable!("create_table must return a table");
    };
    for (index, race_id) in race_ids.into_iter().enumerate() {
        table_set_num(
            state,
            races_ref,
            (index + 1) as f64,
            Val::Num(f64::from(race_id)),
        );
    }
    state.push(races);
    Ok(1)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn read_authenticated_area_poi_id(state: &LuaState) -> LuaResult<i32> {
    // INFERRED AllowedWhenUntainted: authenticate ALL arguments, including
    // ignored extras, before validating; never clear or replace caller taint.
    let authenticated = state.stack[state.base..state.top]
        .iter()
        .map(|value| rilua::table_security::unwrap_secret(state, *value))
        .collect::<LuaResult<Vec<_>>>()?;
    // INFERRED: required actual number, finite integral signed-i32 POI ID;
    // reject coercion and lossy casts. Public authenticated extras are ignored.
    match authenticated.first() {
        Some(Val::Num(number))
            if number.is_finite()
                && number.fract() == 0.0
                && *number >= f64::from(i32::MIN)
                && *number <= f64::from(i32::MAX) =>
        {
            Ok(*number as i32)
        }
        _ => Err(rilua::runtime_error(
            "C_QuestHub.GetDragonridingRacesForAreaPOI: areaPoiID must be a finite integral i32 number",
        )),
    }
}
