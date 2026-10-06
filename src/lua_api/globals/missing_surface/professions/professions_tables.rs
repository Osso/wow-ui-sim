use super::{
    BLACKSMITHING_PROFESSION, COOKING_INVENTORY_SLOTS, COOKING_PROFESSION, FISHING_INVENTORY_SLOTS,
    FISHING_PROFESSION, MINING_PROFESSION, PROF0_INVENTORY_SLOTS, PROF1_INVENTORY_SLOTS,
    SELECTED_PROFESSION_KEY, TRADE_SKILL_NAMESPACE, ensure_namespace, is_profession_unlearned,
    set_table_array,
};
use crate::items;
use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, create_table_with_capacity, table_get, table_set,
    table_set_static,
};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const PROFESSION_TABLE_HASH_FIELDS: usize = 9;
const CATEGORY_TABLE_HASH_FIELDS: usize = 4;
const RECIPE_INFO_HASH_FIELDS: usize = 9;
const REAGENT_INFO_HASH_FIELDS: usize = 5;
const RECIPE_SCHEMATIC_HASH_FIELDS: usize = 6;

pub(super) fn skill_line_id_table(state: &mut LuaState) -> Val {
    let table = create_table(state);
    let mut slot = 0i64;
    for profession in profession_data::PROFESSIONS.iter() {
        if !is_profession_unlearned(state, profession.skill_line_id) {
            slot += 1;
            set_table_array(
                state,
                table,
                slot,
                Val::Num(profession.profession_id as f64),
            );
        }
    }
    table
}

pub(super) fn recipe_id_table(state: &mut LuaState, recipe_ids: &[i32]) -> Val {
    let table = create_table(state);
    for (index, recipe_id) in recipe_ids.iter().enumerate() {
        set_table_array(
            state,
            table,
            (index + 1) as i64,
            Val::Num(*recipe_id as f64),
        );
    }
    table
}

pub(super) fn all_profession_tables(state: &mut LuaState) -> Val {
    let table = create_table(state);
    let mut slot = 0i64;
    for profession in profession_data::PROFESSIONS.iter() {
        if !is_profession_unlearned(state, profession.skill_line_id) {
            slot += 1;
            let value = profession_table(state, Some(profession));
            set_table_array(state, table, slot, value);
        }
    }
    table
}

pub(super) fn profession_table(
    state: &mut LuaState,
    profession: Option<&profession_data::ProfessionInfo>,
) -> Val {
    let table = create_table_with_capacity(state, PROFESSION_TABLE_HASH_FIELDS);
    set_number_field(
        state,
        table,
        "professionID",
        profession
            .map(|profession| profession.profession_id)
            .unwrap_or(0) as f64,
    );
    if let Some(profession) = profession {
        populate_profession_table(state, table, profession);
    }
    table
}

pub(super) fn category_table(
    state: &mut LuaState,
    category: Option<&profession_data::RecipeCategory>,
) -> Val {
    let Some(category) = category else {
        return Val::Nil;
    };

    let table = create_table_with_capacity(state, CATEGORY_TABLE_HASH_FIELDS);
    let name = create_string(state, category.name);
    table_set_static(
        state,
        table,
        "categoryID",
        Val::Num(category.category_id as f64),
    );
    table_set_static(state, table, "name", name);
    table_set_static(
        state,
        table,
        "parentCategoryID",
        Val::Num(category.parent_category_id as f64),
    );
    table_set_static(state, table, "uiOrder", Val::Num(category.ui_order as f64));
    table
}

pub(super) fn recipe_info_table(
    state: &mut LuaState,
    recipe: Option<&profession_data::RecipeEntry>,
) -> Val {
    let table = create_table_with_capacity(state, RECIPE_INFO_HASH_FIELDS);
    match recipe {
        Some(recipe) => populate_recipe_info_table(state, table, recipe),
        None => populate_missing_recipe_info_table(state, table),
    }
    table
}

pub(super) fn reagent_info_table(
    state: &mut LuaState,
    reagent: Option<&profession_data::ReagentSlot>,
) -> Val {
    let Some(reagent) = reagent else {
        return Val::Nil;
    };

    let table = create_table_with_capacity(state, REAGENT_INFO_HASH_FIELDS);
    table_set_static(state, table, "itemID", Val::Num(reagent.item_id as f64));
    table_set_static(
        state,
        table,
        "numRequired",
        Val::Num(reagent.quantity as f64),
    );
    table_set_static(
        state,
        table,
        "quantityRequired",
        Val::Num(reagent.quantity as f64),
    );
    table_set_static(state, table, "reagentType", Val::Num(1.0));
    let name = items::get_item(reagent.item_id)
        .map(|item| item.name)
        .unwrap_or("Unknown");
    let name = create_string(state, name);
    table_set_static(state, table, "name", name);
    table
}

pub(super) fn recipe_schematic_table(
    state: &mut LuaState,
    recipe: Option<&profession_data::RecipeEntry>,
) -> Val {
    let Some(recipe) = recipe else {
        return Val::Nil;
    };

    let table = create_table_with_capacity(state, RECIPE_SCHEMATIC_HASH_FIELDS);
    state.push(table);
    populate_recipe_schematic_table(state, table, recipe);
    table
}

fn reagent_slot_schematic_table(
    state: &mut LuaState,
    recipe: &profession_data::RecipeEntry,
) -> Val {
    let slots = borrow_state(state)
        .expect("crafting state installed")
        .crafting
        .reagents
        .recipe_slots
        .get(&recipe.recipe_id)
        .cloned()
        .expect("catalog recipe has a host schematic");
    crate::c_api::crafting_tables::schematic_slots(state, &slots)
}

pub(super) fn profession_for_recipe(
    recipe_id: i32,
) -> Option<&'static profession_data::ProfessionInfo> {
    profession_data::get_recipe(recipe_id).and_then(|_| profession_data::get_profession_by_index(0))
}

fn populate_profession_table(
    state: &mut LuaState,
    table: Val,
    profession: &profession_data::ProfessionInfo,
) {
    set_number_field(state, table, "profession", profession.profession as f64);
    set_string_field(state, table, "professionName", profession.name);
    set_string_field(
        state,
        table,
        "parentProfessionName",
        profession.parent_profession_name,
    );
    set_number_field(state, table, "skillLevel", profession.skill_level as f64);
    set_number_field(
        state,
        table,
        "maxSkillLevel",
        profession.max_skill_level as f64,
    );
    set_number_field(
        state,
        table,
        "skillModifier",
        profession.skill_modifier as f64,
    );
    set_number_field(state, table, "skillLineID", profession.skill_line_id as f64);
    set_number_field(state, table, "iconFileID", profession.icon as f64);
}

fn populate_missing_recipe_info_table(state: &mut LuaState, table: Val) {
    set_number_field(state, table, "recipeID", 0.0);
    table_set_static(state, table, "name", Val::Nil);
    set_bool_field(state, table, "craftable", false);
}

fn populate_recipe_info_table(
    state: &mut LuaState,
    table: Val,
    recipe: &profession_data::RecipeEntry,
) {
    set_number_field(state, table, "recipeID", recipe.recipe_id as f64);
    set_string_field(state, table, "name", recipe.name);
    set_bool_field(state, table, "learned", recipe.learned);
    set_bool_field(state, table, "craftable", recipe.craftable);
    set_number_field(state, table, "difficulty", recipe.difficulty as f64);
    set_number_field(state, table, "categoryID", recipe.category_id as f64);
    set_number_field(state, table, "itemLevel", recipe.item_level as f64);
    set_number_field(state, table, "maxTrivialLevel", recipe.difficulty as f64);
    set_bool_field(state, table, "favorite", false);
}

fn populate_recipe_schematic_table(
    state: &mut LuaState,
    table: Val,
    recipe: &profession_data::RecipeEntry,
) {
    let reagent_slot_schematics = reagent_slot_schematic_table(state, recipe);
    set_number_field(state, table, "recipeID", recipe.recipe_id as f64);
    set_string_field(state, table, "name", recipe.name);
    if recipe.output_item_id == 0 {
        table_set_static(state, table, "outputItemID", Val::Nil);
    } else {
        set_number_field(state, table, "outputItemID", recipe.output_item_id as f64);
    }
    set_number_field(state, table, "quantityMin", recipe.output_quantity as f64);
    set_number_field(state, table, "quantityMax", recipe.output_quantity as f64);
    table_set_static(
        state,
        table,
        "reagentSlotSchematics",
        reagent_slot_schematics,
    );
}

pub(super) fn set_number_field(state: &mut LuaState, table: Val, key: &str, value: f64) {
    table_set(state, table, key, Val::Num(value));
}

fn set_bool_field(state: &mut LuaState, table: Val, key: &str, value: bool) {
    table_set(state, table, key, Val::Bool(value));
}

fn set_string_field(state: &mut LuaState, table: Val, key: &str, value: &str) {
    let string = create_string(state, value);
    table_set(state, table, key, string);
}

pub(super) fn selected_profession(
    state: &mut LuaState,
) -> Option<&'static profession_data::ProfessionInfo> {
    if let Ok(sim) = borrow_state(state)
        && let Some(id) = sim.crafting.selected_profession_id
    {
        let prof = profession_data::get_profession(id)?;
        if is_profession_unlearned(state, prof.skill_line_id) {
            return None;
        }
        return Some(prof);
    }
    let table_ref = ensure_namespace(state, TRADE_SKILL_NAMESPACE).ok()?;
    let selected = table_get(state, Val::Table(table_ref), SELECTED_PROFESSION_KEY);
    let Val::Num(skill_line_id) = selected else {
        return first_learned_profession(state);
    };
    let prof = profession_data::get_profession(skill_line_id as i32)?;
    if is_profession_unlearned(state, prof.skill_line_id) {
        return None;
    }
    Some(prof)
}

fn first_learned_profession(state: &LuaState) -> Option<&'static profession_data::ProfessionInfo> {
    profession_data::PROFESSIONS
        .iter()
        .find(|p| !is_profession_unlearned(state, p.skill_line_id))
}

pub(super) fn profession_slots(profession: i32) -> &'static [i32] {
    match profession {
        BLACKSMITHING_PROFESSION => PROF0_INVENTORY_SLOTS,
        MINING_PROFESSION => PROF1_INVENTORY_SLOTS,
        COOKING_PROFESSION => COOKING_INVENTORY_SLOTS,
        FISHING_PROFESSION => FISHING_INVENTORY_SLOTS,
        _ => &[],
    }
}

pub(super) fn profession_for_inventory_slot(slot: i32) -> Option<i32> {
    match slot {
        20..=22 => Some(BLACKSMITHING_PROFESSION),
        23..=25 => Some(MINING_PROFESSION),
        26 | 27 => Some(COOKING_PROFESSION),
        28 => Some(FISHING_PROFESSION),
        _ => None,
    }
}

pub(super) fn reagent_index_from_stack(state: &mut LuaState) -> LuaResult<usize> {
    let index = i32::from_stack(state, 2)?;
    Ok(index.saturating_sub(1) as usize)
}

pub(super) fn item_link_value(state: &mut LuaState, item_id: u32) -> Option<Val> {
    if item_id == 0 {
        return None;
    }

    let item = items::get_item(item_id)?;
    Some(create_string(
        state,
        &format!(
            "|cffffffff|Hitem:{item_id}::::::::80:::::|h[{}]|h|r",
            item.name
        ),
    ))
}

pub(super) fn item_icon(item_id: u32) -> f64 {
    items::get_item(item_id)
        .map(|item| item.icon_file_data_id)
        .filter(|icon| *icon != 0)
        .unwrap_or(134400) as f64
}
