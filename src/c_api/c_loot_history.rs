//! Empty-state model for `C_LootHistory`.

use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_loot_history(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }

    let namespace = ensure_namespace(state, "C_LootHistory")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAllEncounterInfos",
        get_all_encounter_infos,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetInfoForEncounter",
        get_info_for_encounter,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetLootHistoryTime",
        get_loot_history_time,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetSortedDropsForEncounter",
        get_sorted_drops_for_encounter,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetSortedInfoForDrop",
        get_sorted_info_for_drop,
    )?;
    Ok(())
}

fn get_all_encounter_infos(state: &mut LuaState) -> LuaResult<u32> {
    let encounters = create_table(state);
    state.push(encounters);
    Ok(1)
}

fn get_info_for_encounter(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn get_loot_history_time(state: &mut LuaState) -> LuaResult<u32> {
    let history_time = borrow_state(state)?.loot_history.history_time;
    state.push(Val::Num(history_time));
    Ok(1)
}

fn get_sorted_drops_for_encounter(state: &mut LuaState) -> LuaResult<u32> {
    let drops = create_table(state);
    state.push(drops);
    Ok(1)
}

fn get_sorted_info_for_drop(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}
