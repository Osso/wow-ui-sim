//! Recipe-list filters over the simulator's recipe catalogue: name search,
//! learned/unlearned, makeable, category and item level filters. Skill-up and
//! first-craft flags are stored only: the catalogue has no relative difficulty
//! or first-craft state. Source-type and inventory-slot filters are stored,
//! but the catalogue carries no source or slot data, so they never exclude a
//! recipe. See
//! `docs/specs/trade-skill-recipe-filters.md`.

use std::collections::BTreeSet;

use crate::c_api::ensure_namespace;
use crate::lua_api::globals::profession_data::{self, RecipeEntry};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, create_table};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state_types::crafting::RecipeListFilters;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

fn filters_show(filters: &RecipeListFilters, recipe: &RecipeEntry) -> bool {
    let learned_visible = if recipe.learned {
        filters.show_learned
    } else {
        filters.show_unlearned
    };
    learned_visible
        && (!filters.only_makeable || recipe.craftable)
        && !filters.filtered_category_ids.contains(&recipe.category_id)
        && filters
            .item_level_range
            .is_none_or(|(min, max)| (min..=max).contains(&recipe.item_level))
}

type Method = (&'static str, rilua::vm::closure::RustFn);

const METHODS: &[Method] = &[
    ("SetRecipeItemNameFilter", set_name_filter),
    ("GetRecipeItemNameFilter", get_name_filter),
    ("GetFilteredRecipeIDs", get_filtered_recipe_ids),
];

/// Filter controls called only by the mainline-family Professions UI.
const MAINLINE_METHODS: &[Method] = &[
    ("GetShowLearned", get_show_learned),
    ("SetShowLearned", set_show_learned),
    ("GetShowUnlearned", get_show_unlearned),
    ("SetShowUnlearned", set_show_unlearned),
    ("GetOnlyShowMakeableRecipes", get_only_makeable),
    ("SetOnlyShowMakeableRecipes", set_only_makeable),
    ("GetOnlyShowSkillUpRecipes", get_only_skill_up),
    ("SetOnlyShowSkillUpRecipes", set_only_skill_up),
    ("GetOnlyShowFirstCraftRecipes", get_only_first_craft),
    ("SetOnlyShowFirstCraftRecipes", set_only_first_craft),
    ("IsRecipeSourceTypeFiltered", is_source_type_filtered),
    ("SetRecipeSourceTypeFilter", set_source_type_filter),
    ("ClearRecipeSourceTypeFilter", clear_source_type_filter),
    ("GetSourceTypeFilter", get_source_type_mask),
    ("SetSourceTypeFilter", set_source_type_mask),
    (
        "GetAllFilterableInventorySlotsCount",
        filterable_inventory_slot_count,
    ),
    (
        "GetFilterableInventorySlotName",
        filterable_inventory_slot_name,
    ),
    ("IsInventorySlotFiltered", is_inventory_slot_filtered),
    ("SetInventorySlotFilter", set_inventory_slot_filter),
    ("ClearInventorySlotFilter", clear_inventory_slot_filter),
    ("AreAnyInventorySlotsFiltered", any_inventory_slots_filtered),
    ("IsRecipeCategoryFiltered", is_category_filtered),
    ("SetRecipeCategoryFilter", set_category_filter),
    ("ClearRecipeCategoryFilter", clear_category_filter),
    ("AnyRecipeCategoriesFiltered", any_categories_filtered),
    ("SetRecipeItemLevelFilter", set_item_level_filter),
];

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};
    let table = ensure_namespace(state, "C_TradeSkillUI")?;
    let mainline = matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    );
    let extra: &[Method] = if mainline { MAINLINE_METHODS } else { &[] };
    for (name, func) in METHODS.iter().chain(extra) {
        table_set_rust_fn_static(state, table, name, *func)?;
    }
    Ok(())
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

fn filter_recipe_ids(name_filter: &str, filters: &RecipeListFilters) -> Vec<i32> {
    let needle = name_filter.to_lowercase();
    profession_data::get_all_recipe_ids()
        .into_iter()
        .filter(|id| {
            profession_data::get_recipe(*id).is_some_and(|recipe| {
                filters_show(filters, recipe) && recipe.name.to_lowercase().contains(&needle)
            })
        })
        .collect()
}

fn get_filtered_recipe_ids(state: &mut LuaState) -> LuaResult<u32> {
    let ids = {
        let sim = borrow_state(state)?;
        filter_recipe_ids(
            &sim.crafting.recipe_name_filter,
            &sim.crafting.recipe_filters,
        )
    };
    let table = create_table(state);
    for (index, id) in ids.into_iter().enumerate() {
        super::helpers::set_table_array(state, table, (index + 1) as i64, Val::Num(id as f64));
    }
    state.push(table);
    Ok(1)
}

fn read_filters<T>(
    state: &mut LuaState,
    read: impl FnOnce(&RecipeListFilters) -> T,
) -> LuaResult<T> {
    Ok(read(&borrow_state(state)?.crafting.recipe_filters))
}

/// Applies a filter change and announces the recipe-list refresh.
fn update_filters(
    state: &mut LuaState,
    update: impl FnOnce(&mut RecipeListFilters),
) -> LuaResult<u32> {
    update(&mut borrow_state_mut(state)?.crafting.recipe_filters);
    fire_named_event_state(state, "TRADE_SKILL_LIST_UPDATE", &[]);
    Ok(0)
}

fn push_bool(state: &mut LuaState, value: bool) -> LuaResult<u32> {
    state.push(Val::Bool(value));
    Ok(1)
}

fn arg_truthy(state: &LuaState, index: usize) -> bool {
    !matches!(stack_val(state, index), Val::Nil | Val::Bool(false))
}

fn get_show_learned(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_filters(state, |f| f.show_learned)?;
    push_bool(state, value)
}

fn set_show_learned(state: &mut LuaState) -> LuaResult<u32> {
    let value = arg_truthy(state, 1);
    update_filters(state, |f| f.show_learned = value)
}

fn get_show_unlearned(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_filters(state, |f| f.show_unlearned)?;
    push_bool(state, value)
}

fn set_show_unlearned(state: &mut LuaState) -> LuaResult<u32> {
    let value = arg_truthy(state, 1);
    update_filters(state, |f| f.show_unlearned = value)
}

fn get_only_makeable(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_filters(state, |f| f.only_makeable)?;
    push_bool(state, value)
}

fn set_only_makeable(state: &mut LuaState) -> LuaResult<u32> {
    let value = arg_truthy(state, 1);
    update_filters(state, |f| f.only_makeable = value)
}

fn get_only_skill_up(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_filters(state, |f| f.only_skill_up)?;
    push_bool(state, value)
}

fn set_only_skill_up(state: &mut LuaState) -> LuaResult<u32> {
    let value = arg_truthy(state, 1);
    update_filters(state, |f| f.only_skill_up = value)
}

fn get_only_first_craft(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_filters(state, |f| f.only_first_craft)?;
    push_bool(state, value)
}

fn set_only_first_craft(state: &mut LuaState) -> LuaResult<u32> {
    let value = arg_truthy(state, 1);
    update_filters(state, |f| f.only_first_craft = value)
}

fn is_source_type_filtered(state: &mut LuaState) -> LuaResult<u32> {
    let source = i32::from_stack(state, 1)?;
    let value = read_filters(state, |f| f.filtered_source_types.contains(&source))?;
    push_bool(state, value)
}

/// The second argument is the filtered state, as `SetSourceChecked` passes it.
fn set_source_type_filter(state: &mut LuaState) -> LuaResult<u32> {
    let source = i32::from_stack(state, 1)?;
    let filtered = arg_truthy(state, 2);
    update_filters(state, |f| {
        toggle(&mut f.filtered_source_types, source, filtered)
    })
}

fn clear_source_type_filter(state: &mut LuaState) -> LuaResult<u32> {
    update_filters(state, |f| f.filtered_source_types.clear())
}

/// Bit `n - 1` set means source index `n` is filtered out.
fn get_source_type_mask(state: &mut LuaState) -> LuaResult<u32> {
    let mask = read_filters(state, |f| {
        f.filtered_source_types
            .iter()
            .filter(|source| (1..=32).contains(*source))
            .fold(0u32, |mask, source| mask | (1 << (source - 1)))
    })?;
    state.push(Val::Num(f64::from(mask)));
    Ok(1)
}

fn set_source_type_mask(state: &mut LuaState) -> LuaResult<u32> {
    let mask = u32::from_stack(state, 1)?;
    update_filters(state, |f| {
        f.filtered_source_types = (1..=32)
            .filter(|bit| mask & (1 << (bit - 1)) != 0)
            .collect();
    })
}

/// The catalogue maps no recipe output to an inventory slot.
fn filterable_inventory_slot_count(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn filterable_inventory_slot_name(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    Err(runtime_error(format!(
        "inventory slot filter index {index} out of range (0 filterable slots)"
    )))
}

fn is_inventory_slot_filtered(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let value = read_filters(state, |f| f.filtered_inventory_slots.contains(&slot))?;
    push_bool(state, value)
}

/// The second argument is the shown (checked) state; an exclusive call leaves
/// only that slot shown.
fn set_inventory_slot_filter(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let shown = arg_truthy(state, 2);
    let exclusive = arg_truthy(state, 3);
    update_filters(state, |f| {
        if exclusive {
            f.filtered_inventory_slots.clear();
        }
        toggle(&mut f.filtered_inventory_slots, slot, !shown);
    })
}

fn clear_inventory_slot_filter(state: &mut LuaState) -> LuaResult<u32> {
    update_filters(state, |f| f.filtered_inventory_slots.clear())
}

fn any_inventory_slots_filtered(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_filters(state, |f| !f.filtered_inventory_slots.is_empty())?;
    push_bool(state, value)
}

fn is_category_filtered(state: &mut LuaState) -> LuaResult<u32> {
    let category = i32::from_stack(state, 1)?;
    let value = read_filters(state, |f| f.filtered_category_ids.contains(&category))?;
    push_bool(state, value)
}

fn set_category_filter(state: &mut LuaState) -> LuaResult<u32> {
    let category = i32::from_stack(state, 1)?;
    let filtered = arg_truthy(state, 2);
    update_filters(state, |f| {
        toggle(&mut f.filtered_category_ids, category, filtered)
    })
}

fn clear_category_filter(state: &mut LuaState) -> LuaResult<u32> {
    update_filters(state, |f| f.filtered_category_ids.clear())
}

fn any_categories_filtered(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_filters(state, |f| !f.filtered_category_ids.is_empty())?;
    push_bool(state, value)
}

fn set_item_level_filter(state: &mut LuaState) -> LuaResult<u32> {
    let min = i32::from_stack(state, 1)?;
    let max = i32::from_stack(state, 2)?;
    // The Professions UI clears the range with (0, 0).
    let range = (min, max) != (0, 0);
    update_filters(state, |f| f.item_level_range = range.then_some((min, max)))
}

fn toggle(set: &mut BTreeSet<i32>, value: i32, present: bool) {
    if present {
        set.insert(value);
    } else {
        set.remove(&value);
    }
}
