//! Perks vendor categories from the pinned client catalogue.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_string, create_table, table_set_num, table_set_static};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_PerksProgram")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAvailableCategoryIDs",
        available_category_ids,
    )?;
    table_set_rust_fn_static(state, namespace, "GetCategoryInfo", category_info)
}

fn available_category_ids(state: &mut LuaState) -> LuaResult<u32> {
    let result = create_table(state);
    let Val::Table(table) = result else {
        unreachable!()
    };
    for (index, &(id, _, _)) in sim_data::perks_categories::CATEGORIES.iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, Val::Num(id as f64));
    }
    state.push(result);
    Ok(1)
}

fn category_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let Some(&(_, name, scene)) = sim_data::perks_categories::CATEGORIES
        .iter()
        .find(|row| row.0 == id)
    else {
        return Ok(0);
    };
    let result = create_table(state);
    table_set_static(state, result, "ID", Val::Num(id as f64));
    let name = create_string(state, name);
    table_set_static(state, result, "displayName", name);
    table_set_static(
        state,
        result,
        "defaultUIModelSceneID",
        Val::Num(scene as f64),
    );
    state.push(result);
    Ok(1)
}
