//! Owned custom-set storage; native validation and persistence are not modeled.

use std::collections::BTreeMap;

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
    table_set_num,
};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

#[derive(Debug, Default)]
pub struct CustomSets {
    last_id: i32,
    records: BTreeMap<i32, CustomSet>,
}

#[derive(Debug, Clone)]
struct CustomSet {
    name: String,
    icon: i32,
    items: Vec<ItemTransmogInfo>,
}

#[derive(Debug, Clone)]
struct ItemTransmogInfo {
    appearance_id: i32,
    secondary_appearance_id: i32,
    illusion_id: i32,
}

pub(crate) fn register(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    for (name, handler) in [
        (
            "NewCustomSet",
            new_custom_set as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("GetCustomSets", get_custom_sets),
        ("GetCustomSetInfo", get_custom_set_info),
        ("GetCustomSetItemTransmogInfoList", get_custom_set_items),
        ("ModifyCustomSet", modify_custom_set),
        ("RenameCustomSet", rename_custom_set),
        ("DeleteCustomSet", delete_custom_set),
    ] {
        table_set_rust_fn_static(state, table, name, handler)?;
    }
    Ok(())
}

fn read_item_id(state: &mut LuaState, row: Val, key: &str) -> LuaResult<i32> {
    let Val::Num(value) = table_get(state, row, key) else {
        return Err(rilua::runtime_error(format!(
            "ItemTransmogInfo.{key} requires a number"
        )));
    };
    Ok(value as i32)
}

fn read_items(state: &mut LuaState, index: i32) -> LuaResult<Vec<ItemTransmogInfo>> {
    let Val::Table(list) = stack_val(state, index) else {
        return Err(rilua::runtime_error(
            "itemTransmogInfoList requires a table",
        ));
    };
    let count = state
        .gc
        .tables
        .get(list)
        .ok_or_else(|| rilua::runtime_error("invalid item list"))?
        .len(&state.gc.string_arena);
    let mut items = Vec::with_capacity(count);
    for index in 1..=count {
        let row = state
            .gc
            .tables
            .get(list)
            .ok_or_else(|| rilua::runtime_error("invalid item list"))?
            .get_int(index as i64);
        if !matches!(row, Val::Table(_)) {
            return Err(rilua::runtime_error(format!(
                "itemTransmogInfoList[{index}] requires a table"
            )));
        }
        items.push(ItemTransmogInfo {
            appearance_id: read_item_id(state, row, "appearanceID")?,
            secondary_appearance_id: read_item_id(state, row, "secondaryAppearanceID")?,
            illusion_id: read_item_id(state, row, "illusionID")?,
        });
    }
    Ok(items)
}

fn new_custom_set(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let icon = i32::from_stack(state, 2)?;
    let items = read_items(state, 3)?;
    let id = {
        let mut sim = borrow_state_mut(state)?;
        let sets = &mut sim.transmog_custom_sets;
        let id = sets
            .last_id
            .checked_add(1)
            .ok_or_else(|| rilua::runtime_error("custom set ID space exhausted"))?;
        sets.records.insert(id, CustomSet { name, icon, items });
        sets.last_id = id;
        id
    };
    state.push(Val::Num(id as f64));
    Ok(1)
}

fn get_custom_sets(state: &mut LuaState) -> LuaResult<u32> {
    let ids: Vec<_> = borrow_state(state)?
        .transmog_custom_sets
        .records
        .keys()
        .copied()
        .collect();
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, id) in ids.into_iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, Val::Num(id as f64));
    }
    state.push(array);
    Ok(1)
}

fn get_custom_set_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let info = borrow_state(state)?
        .transmog_custom_sets
        .records
        .get(&id)
        .map(|record| (record.name.clone(), record.icon));
    let Some((name, icon)) = info else {
        return Ok(0);
    };
    let name = create_string(state, &name);
    state.push(name);
    state.push(Val::Num(icon as f64));
    Ok(2)
}

fn get_custom_set_items(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let items = borrow_state(state)?
        .transmog_custom_sets
        .records
        .get(&id)
        .map(|record| record.items.clone());
    let Some(items) = items else { return Ok(0) };
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, item) in items.into_iter().enumerate() {
        let row = create_table(state);
        table_set(
            state,
            row,
            "appearanceID",
            Val::Num(item.appearance_id as f64),
        );
        table_set(
            state,
            row,
            "secondaryAppearanceID",
            Val::Num(item.secondary_appearance_id as f64),
        );
        table_set(state, row, "illusionID", Val::Num(item.illusion_id as f64));
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(array);
    Ok(1)
}

fn missing_set(id: i32) -> rilua::LuaError {
    rilua::runtime_error(format!("unknown custom set ID {id}"))
}

fn modify_custom_set(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let items = read_items(state, 2)?;
    borrow_state_mut(state)?
        .transmog_custom_sets
        .records
        .get_mut(&id)
        .ok_or_else(|| missing_set(id))?
        .items = items;
    Ok(0)
}

fn rename_custom_set(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .transmog_custom_sets
        .records
        .get_mut(&id)
        .ok_or_else(|| missing_set(id))?
        .name = name;
    Ok(0)
}

fn delete_custom_set(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .transmog_custom_sets
        .records
        .remove(&id)
        .ok_or_else(|| missing_set(id))?;
    Ok(0)
}
