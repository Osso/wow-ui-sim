//! Snapshot publication of the explicit active-brawl input.

use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

use super::PvpBrawlInfo;

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetActiveBrawlInfo",
        get_active_brawl_info,
    )
}

fn get_active_brawl_info(state: &mut LuaState) -> LuaResult<u32> {
    let record = borrow_state(state)?.active_brawl.clone();
    let result = match record {
        Some(record) => build_brawl_table(state, &record),
        None => Val::Nil,
    };
    state.push(result);
    Ok(1)
}

fn build_brawl_table(state: &mut LuaState, record: &PvpBrawlInfo) -> Val {
    let table = create_table(state);
    for (key, value) in [
        ("brawlID", record.brawl_id as f64),
        ("minLevel", record.min_level as f64),
        ("maxLevel", record.max_level as f64),
        ("brawlType", record.brawl_type as f64),
        ("minItemLevel", record.min_item_level),
    ] {
        table_set(state, table, key, Val::Num(value));
    }
    for (key, value) in [
        ("name", record.name.as_str()),
        ("shortDescription", record.short_description.as_str()),
        ("longDescription", record.long_description.as_str()),
    ] {
        let value = create_string(state, value);
        table_set(state, table, key, value);
    }
    for (key, value) in [
        ("canQueue", record.can_queue),
        ("groupsAllowed", record.groups_allowed),
        ("crossFactionAllowed", record.cross_faction_allowed),
        ("includesAllArenas", record.includes_all_arenas),
        ("shouldHideRewardIcon", record.should_hide_reward_icon),
    ] {
        table_set(state, table, key, Val::Bool(value));
    }
    if let Some(time) = record.time_left_until_next_change {
        table_set(state, table, "timeLeftUntilNextChange", Val::Num(time));
    }
    let maps = build_map_names(state, &record.map_names);
    table_set(state, table, "mapNames", maps);
    table
}

fn build_map_names(state: &mut LuaState, names: &[String]) -> Val {
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, name) in names.iter().enumerate() {
        let value = create_string(state, name);
        table_set_num(state, table, (index + 1) as f64, value);
    }
    array
}
