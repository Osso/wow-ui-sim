//! Explicit dye catalog and consumable ownership from held/banked bag stacks.

use crate::c_api::helpers::{ensure_namespace, global_val, set_table_array};
use crate::lua_api::SimState;
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, create_table, table_set_static,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct DyeColorState {
    pub categories: BTreeMap<i32, String>,
    pub colors: BTreeMap<i32, DyeColorRecord>,
}

#[derive(Clone, Debug)]
pub struct DyeColorRecord {
    pub category_id: i32,
    pub name: String,
    pub sort_order: i32,
    pub swatch_start: [f64; 3],
    pub swatch_end: [f64; 3],
    pub item_id: Option<u32>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_DyeColor")?;
    for (name, function) in [
        ("GetAllDyeColorCategories", categories as rilua::RustFn),
        ("GetAllDyeColors", colors),
        ("GetDyeColorCategoryInfo", category_info),
        ("GetDyeColorInfo", color_info),
        ("GetDyeColorsInCategory", category_colors),
        ("IsDyeColorOwned", is_owned),
    ] {
        table_set_rust_fn_static(state, ns, name, function)?;
    }
    Ok(())
}

fn read_id(state: &LuaState) -> LuaResult<i32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Num(value) if value > 0.0 && value <= f64::from(i32::MAX) && value.fract() == 0.0 => {
            Ok(value as i32)
        }
        _ => Err(rilua::runtime_error(
            "dye selector must be a positive integer",
        )),
    }
}

fn read_owned_only(state: &LuaState, index: i32) -> LuaResult<bool> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, index))?;
    match value {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(rilua::runtime_error(
            "ownedColorsOnly must be a boolean or nil",
        )),
    }
}

fn consumable_count(sim: &SimState, color: &DyeColorRecord) -> i64 {
    let Some(item_id) = color.item_id else {
        return 0;
    };
    // Existing bag store contains both carried and bank bag IDs; guild storage is excluded.
    sim.bag_items
        .values()
        .filter(|item| item.item_id == item_id)
        .map(|item| i64::from(item.stack_count.max(0)))
        .sum()
}

fn push_ids(state: &mut LuaState, ids: &[i32]) -> LuaResult<u32> {
    let table = create_table(state);
    state.push(table);
    // INFERRED deterministic ascending ID order; native ordering is undocumented.
    for (index, id) in ids.iter().enumerate() {
        set_table_array(state, table, (index + 1) as i64, Val::Num(f64::from(*id)));
    }
    Ok(1)
}

fn categories(state: &mut LuaState) -> LuaResult<u32> {
    let ids = borrow_state(state)?
        .dye_colors
        .categories
        .keys()
        .copied()
        .collect::<Vec<_>>();
    push_ids(state, &ids)
}

fn selected_colors(sim: &SimState, category: Option<i32>, owned_only: bool) -> Vec<i32> {
    sim.dye_colors
        .colors
        .iter()
        .filter(|(_, record)| category.is_none_or(|id| record.category_id == id))
        .filter(|(_, record)| !owned_only || consumable_count(sim, record) > 0)
        .map(|(id, _)| *id)
        .collect()
}

fn colors(state: &mut LuaState) -> LuaResult<u32> {
    let owned_only = read_owned_only(state, 1)?;
    let ids = selected_colors(&*borrow_state(state)?, None, owned_only);
    push_ids(state, &ids)
}

fn category_colors(state: &mut LuaState) -> LuaResult<u32> {
    let category = read_id(state)?;
    let owned_only = read_owned_only(state, 2)?;
    let ids = selected_colors(&*borrow_state(state)?, Some(category), owned_only);
    push_ids(state, &ids)
}

fn category_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let name = borrow_state(state)?.dye_colors.categories.get(&id).cloned();
    let Some(name) = name else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let row = create_table(state);
    state.push(row);
    table_set_static(state, row, "ID", Val::Num(f64::from(id)));
    let name = create_string(state, &name);
    table_set_static(state, row, "name", name);
    Ok(1)
}

fn color_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let record = {
        let sim = borrow_state(state)?;
        sim.dye_colors
            .colors
            .get(&id)
            .map(|record| (record.clone(), consumable_count(&sim, record)))
    };
    let Some((record, count)) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_color_info(state, id, &record, count)?;
    Ok(1)
}

fn push_color_info(
    state: &mut LuaState,
    id: i32,
    record: &DyeColorRecord,
    count: i64,
) -> LuaResult<()> {
    let row = create_table(state);
    state.push(row); // Root DTO across both Lua ColorMixin callbacks.
    for (field, value) in [
        ("ID", id),
        ("dyeColorCategoryID", record.category_id),
        ("sortOrder", record.sort_order),
    ] {
        table_set_static(state, row, field, Val::Num(f64::from(value)));
    }
    let name = create_string(state, &record.name);
    table_set_static(state, row, "name", name);
    table_set_static(
        state,
        row,
        "itemID",
        record
            .item_id
            .map_or(Val::Nil, |id| Val::Num(f64::from(id))),
    );
    table_set_static(state, row, "numOwned", Val::Num(count as f64));
    for (field, rgb) in [
        ("swatchColorStart", record.swatch_start),
        ("swatchColorEnd", record.swatch_end),
    ] {
        let factory = global_val(state, "CreateColor");
        let color = call_function_state(
            state,
            factory,
            &[
                Val::Num(rgb[0]),
                Val::Num(rgb[1]),
                Val::Num(rgb[2]),
                Val::Num(1.0),
            ],
        )?;
        table_set_static(state, row, field, color);
    }
    Ok(())
}

fn is_owned(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let owned = {
        let sim = borrow_state(state)?;
        sim.dye_colors
            .colors
            .get(&id)
            .is_some_and(|color| consumable_count(&sim, color) > 0)
    };
    state.push(Val::Bool(owned));
    Ok(1)
}
