//! Recipe-name search over the simulator's existing learned recipe catalogue.
//! Locale collation, reagent searching and other recipe filters are not modeled here.

use crate::c_api::ensure_namespace;
use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, IntoStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let table = ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(state, table, "SetRecipeItemNameFilter", set_name_filter)?;
    table_set_rust_fn_static(state, table, "GetRecipeItemNameFilter", get_name_filter)?;
    table_set_rust_fn_static(
        state,
        table,
        "GetFilteredRecipeIDs",
        get_filtered_recipe_ids,
    )
}

fn set_name_filter(state: &mut LuaState) -> LuaResult<u32> {
    // Cached ProfessionsTemplates explicitly uses nil to clear its search box.
    let filter = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    borrow_state_mut(state)?.crafting.recipe_name_filter = filter;
    fire_named_event_state(state, "TRADE_SKILL_LIST_UPDATE", &[]);
    Ok(0)
}

fn get_name_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = borrow_state(state)?.crafting.recipe_name_filter.clone();
    let value = create_string(state, &filter);
    state.push(value);
    Ok(1)
}

fn filter_recipe_ids(filter: &str) -> Vec<i32> {
    let needle = filter.to_lowercase();
    profession_data::get_filtered_recipe_ids()
        .into_iter()
        .filter(|id| {
            profession_data::get_recipe(*id)
                .is_some_and(|recipe| recipe.name.to_lowercase().contains(&needle))
        })
        .collect()
}

fn get_filtered_recipe_ids(state: &mut LuaState) -> LuaResult<u32> {
    let filter = borrow_state(state)?.crafting.recipe_name_filter.clone();
    filter_recipe_ids(&filter).into_stack(state)
}
