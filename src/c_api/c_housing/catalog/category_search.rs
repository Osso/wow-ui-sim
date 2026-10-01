//! Explicit category searches; featured-parent and ordering rules are simulator policies.
//! Public-only parsing is conservative, not full AllowedWhenUntainted secret parity.

use super::HousingCatalogState;
use super::input::{read_public_integer, read_selector};
use crate::c_api::helpers::{global_val, set_table_array};
use crate::lua_api::methods::{borrow_state, create_table, table_get_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

struct SearchFilters {
    stored_only: bool,
    include_featured: bool,
    editor_mode: Option<i32>,
}

pub(super) fn search_categories(state: &mut LuaState) -> LuaResult<u32> {
    let filters = parse_search_filters(state)?;
    let featured_id = find_featured_category_id(state)?;
    let matches = find_categories(&borrow_state(state)?.housing.catalog, &filters, featured_id);
    publish_ids(state, &sort_ids(matches))
}

pub(super) fn search_subcategories(state: &mut LuaState) -> LuaResult<u32> {
    let filters = parse_search_filters(state)?;
    let featured_id = find_featured_category_id(state)?;
    let matches = find_subcategories(&borrow_state(state)?.housing.catalog, &filters, featured_id);
    publish_ids(state, &sort_ids(matches))
}

fn parse_search_filters(state: &mut LuaState) -> LuaResult<SearchFilters> {
    // Reuse the public-table and VM access guard before reading any fields.
    let params = read_selector(state)?;
    let stored_only = parse_boolean(
        table_get_static(state, params, "withStoredEntriesOnly"),
        "withStoredEntriesOnly",
    )?;
    let include_featured = parse_boolean(
        table_get_static(state, params, "includeFeaturedCategory"),
        "includeFeaturedCategory",
    )?;
    let editor_mode = match table_get_static(state, params, "editorModeContext") {
        Val::Nil => None,
        value => Some(read_public_integer(value, "editorModeContext")?),
    };
    Ok(SearchFilters {
        stored_only,
        include_featured,
        editor_mode,
    })
}

fn parse_boolean(value: Val, field: &'static str) -> LuaResult<bool> {
    match value {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(rilua::runtime_error(format!(
            "housing catalog {field} must be a public boolean or nil; secret access is not modeled"
        ))),
    }
}

fn find_featured_category_id(state: &mut LuaState) -> LuaResult<i32> {
    let constants = global_val(state, "Constants");
    let catalog = table_get_static(state, constants, "HousingCatalogConsts");
    let id = table_get_static(state, catalog, "HOUSING_CATALOG_FEATURED_CATEGORY_ID");
    read_public_integer(
        id,
        "Constants.HousingCatalogConsts.HOUSING_CATALOG_FEATURED_CATEGORY_ID",
    )
}

fn filter_record(filters: &SearchFilters, stored: bool, contexts: &[i32], featured: bool) -> bool {
    if filters.stored_only && !stored {
        return false;
    }
    if !filters.include_featured && featured {
        return false;
    }
    match filters.editor_mode {
        Some(mode) => contexts.contains(&mode),
        None => true,
    }
}

fn find_categories(
    catalog: &HousingCatalogState,
    filters: &SearchFilters,
    featured_id: i32,
) -> Vec<(i32, i32)> {
    let mut matches = Vec::new();
    for (&id, record) in &catalog.categories {
        if filter_record(
            filters,
            record.any_stored_entries,
            &record.editor_mode_contexts,
            id == featured_id,
        ) {
            matches.push((record.order_index, id));
        }
    }
    matches
}

fn find_subcategories(
    catalog: &HousingCatalogState,
    filters: &SearchFilters,
    featured_id: i32,
) -> Vec<(i32, i32)> {
    let mut matches = Vec::new();
    for (&id, record) in &catalog.subcategories {
        if filter_record(
            filters,
            record.any_stored_entries,
            &record.editor_mode_contexts,
            record.parent_category_id == featured_id,
        ) {
            matches.push((record.order_index, id));
        }
    }
    matches
}

fn sort_ids(mut matches: Vec<(i32, i32)>) -> Vec<i32> {
    matches.sort_unstable();
    matches.into_iter().map(|(_, id)| id).collect()
}

fn publish_ids(state: &mut LuaState, ids: &[i32]) -> LuaResult<u32> {
    let result = create_table(state);
    state.push(result);
    for (index, &id) in ids.iter().enumerate() {
        set_table_array(state, result, (index + 1) as i64, Val::Num(f64::from(id)));
    }
    Ok(1)
}
