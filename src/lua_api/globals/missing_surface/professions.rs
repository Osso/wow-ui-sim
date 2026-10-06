#[cfg(feature = "client-mists")]
#[path = "professions_legacy_craft.rs"]
mod professions_legacy_craft;
#[cfg(feature = "client-mists")]
#[path = "professions_legacy_trade_skill.rs"]
mod professions_legacy_trade_skill;
#[cfg(feature = "client-mists")]
#[path = "professions_legacy_trainer.rs"]
mod professions_legacy_trainer;
mod professions_registration;
mod professions_tables;
mod professions_tracking;

use super::{ensure_namespace, set_table_array};
use crate::c_api::crafting_execution::{craft_recipe, recipe_is_craftable};
use crate::lua_api::globals::{profession_data, spellbook_data};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
};
use crate::lua_api::script_helpers::{fire_named_event_state, protected_lua_pcall_state};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
#[cfg(feature = "client-mists")]
use professions_legacy_craft::register_legacy_craft_globals;
#[cfg(feature = "client-mists")]
use professions_legacy_trade_skill::register_legacy_trade_skill_globals;
#[cfg(feature = "client-mists")]
use professions_legacy_trainer::register_legacy_trainer_globals;
use professions_registration::{register_crafting_order_namespace, register_trade_skill_namespace};
use professions_tables::{
    all_profession_tables, category_table, item_icon, item_link_value,
    profession_for_inventory_slot, profession_for_recipe, profession_slots, profession_table,
    reagent_index_from_stack, reagent_info_table, recipe_id_table, recipe_info_table,
    recipe_schematic_table, selected_profession, set_number_field, skill_line_id_table,
};
use professions_tracking::{
    c_trade_skill_ui_get_recipes_tracked, c_trade_skill_ui_is_recipe_learned,
    c_trade_skill_ui_is_recipe_tracked, c_trade_skill_ui_set_recipe_tracked,
};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(super) const TRADE_SKILL_NAMESPACE: &str = "C_TradeSkillUI";
pub(super) const CRAFTING_ORDERS_NAMESPACE: &str = "C_CraftingOrders";
pub(super) const SELECTED_PROFESSION_KEY: &str = "_selectedProfessionID";
pub(super) const BLACKSMITHING_PROFESSION: i32 = 1;
pub(super) const MINING_PROFESSION: i32 = 6;
pub(super) const COOKING_PROFESSION: i32 = 9;
pub(super) const FISHING_PROFESSION: i32 = 10;
pub(super) const PROF0_INVENTORY_SLOTS: &[i32] = &[20, 21, 22];
pub(super) const PROF1_INVENTORY_SLOTS: &[i32] = &[23, 24, 25];
pub(super) const COOKING_INVENTORY_SLOTS: &[i32] = &[26, 27];
pub(super) const FISHING_INVENTORY_SLOTS: &[i32] = &[28];
const PROFESSION_INVENTORY_SLOTS: &[i32] = &[20, 21, 22, 23, 24, 25, 26, 27, 28];

pub(super) fn register_profession_surface(state: &mut LuaState) -> LuaResult<()> {
    register_trade_skill_namespace(state)?;
    register_crafting_order_namespace(state)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetProfessions",
        get_professions_global,
    )?;
    table_set_rust_fn_static(
        state,
        state.global,
        "GetProfessionInfo",
        get_profession_info_global,
    )?;
    table_set_rust_fn_static(state, state.global, "AbandonSkill", abandon_skill)?;
    #[cfg(feature = "client-mists")]
    register_legacy_craft_globals(state)?;
    #[cfg(feature = "client-mists")]
    register_legacy_trade_skill_globals(state)?;
    #[cfg(feature = "client-mists")]
    register_legacy_trainer_globals(state)?;
    Ok(())
}

fn abandon_skill(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line_id = i32::from_stack(state, 1)?;
    abandon_profession_impl(state, skill_line_id);
    fire_named_event_state(state, "SKILL_LINES_CHANGED", &[]);
    Ok(0)
}

pub(crate) fn abandon_profession_impl(state: &mut LuaState, skill_line_id: i32) {
    let mut sim = match borrow_state_mut(state) {
        Ok(s) => s,
        Err(_) => return,
    };
    sim.crafting.unlearned_profession_ids.insert(skill_line_id);
    if sim.crafting.selected_profession_id == Some(skill_line_id) {
        sim.crafting.selected_profession_id = None;
    }
}

pub(crate) fn relearn_profession_impl(state: &mut LuaState, skill_line_id: i32) {
    let mut sim = match borrow_state_mut(state) {
        Ok(s) => s,
        Err(_) => return,
    };
    sim.crafting.unlearned_profession_ids.remove(&skill_line_id);
}

fn c_crafting_orders_get_order_claim_info(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    set_number_field(state, table, "claimsRemaining", 0.0);
    set_number_field(state, table, "secondsToRecharge", 0.0);
    state.push(table);
    Ok(1)
}

pub(crate) fn open_trade_skill_for_skill_line(
    state: &mut LuaState,
    skill_line_id: i32,
) -> LuaResult<bool> {
    if profession_data::get_profession(skill_line_id).is_none() {
        return Ok(false);
    }

    select_profession(state, skill_line_id)?;
    ensure_professions_frame_loaded(state);
    fire_named_event_state(state, "TRADE_SKILL_LIST_UPDATE", &[]);
    show_professions_frame(state);
    fire_named_event_state(state, "TRADE_SKILL_NAME_UPDATE", &[]);
    Ok(true)
}

fn ensure_professions_frame_loaded(state: &mut LuaState) {
    if !matches!(
        LuaApiMut::get_global_val(state, "ProfessionsFrame"),
        Val::Nil
    ) {
        return;
    }
    call_global_function(state, "ProfessionsFrame_LoadUI", &[]);
}

fn show_professions_frame(state: &mut LuaState) {
    let frame = LuaApiMut::get_global_val(state, "ProfessionsFrame");
    if matches!(frame, Val::Nil) {
        return;
    }

    let show_ui_panel = LuaApiMut::get_global_val(state, "ShowUIPanel");
    if matches!(show_ui_panel, Val::Function(_)) {
        let _ = protected_lua_pcall_state(state, show_ui_panel, &[frame]);
        return;
    }

    let show_method = table_get(state, frame, "Show");
    if matches!(show_method, Val::Function(_)) {
        let _ = protected_lua_pcall_state(state, show_method, &[frame]);
    }
}

fn call_global_function(state: &mut LuaState, name: &str, args: &[Val]) {
    let function = LuaApiMut::get_global_val(state, name);
    if !matches!(function, Val::Function(_)) {
        return;
    }
    let _ = protected_lua_pcall_state(state, function, args);
}

fn c_trade_skill_ui_get_all_profession_trade_skill_lines(state: &mut LuaState) -> LuaResult<u32> {
    let table = skill_line_id_table(state);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_all_recipe_ids(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_ids = profession_data::get_all_recipe_ids();
    let table = recipe_id_table(state, &recipe_ids);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_base_profession_info(state: &mut LuaState) -> LuaResult<u32> {
    let table = profession_table(state, profession_data::get_profession_by_index(0));
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_category_info(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let table = category_table(state, profession_data::get_category(category_id));
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_child_profession_info(state: &mut LuaState) -> LuaResult<u32> {
    let profession = selected_profession(state);
    let table = profession_table(state, profession);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_child_profession_infos(state: &mut LuaState) -> LuaResult<u32> {
    let table = all_profession_tables(state);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_crafting_order_count(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

/// `C_TradeSkillUI.GetDependentReagents(reagent) -> CraftingReagent[]`.
///
/// Always returns an array (possibly empty). Returning nil here would crash
/// callers that wrap the result in `ipairs`, e.g.
/// `ProfessionsRecipeTransactionMixin:AreDependentReagentsAllocated`.
fn c_trade_skill_ui_get_dependent_reagents(state: &mut LuaState) -> LuaResult<u32> {
    let reagent = Val::from_stack(state, 1).unwrap_or(Val::Nil);
    let item_id = reagent_item_id(state, reagent);
    let dependents = dependent_reagents_for_item(item_id);
    let table = dependent_reagent_table(state, dependents);
    state.push(table);
    Ok(1)
}

fn reagent_item_id(state: &mut LuaState, reagent: Val) -> Option<u32> {
    match reagent {
        Val::Table(_) => match table_get(state, reagent, "itemID") {
            Val::Num(n) if n > 0.0 => Some(n as u32),
            _ => None,
        },
        _ => None,
    }
}

fn dependent_reagents_for_item(item_id: Option<u32>) -> &'static [profession_data::ReagentSlot] {
    item_id
        .map(profession_data::find_reagent_dependents)
        .unwrap_or(&[])
}

fn dependent_reagent_table(
    state: &mut LuaState,
    dependents: &[profession_data::ReagentSlot],
) -> Val {
    let table = create_table(state);
    for (index, dep) in dependents.iter().enumerate() {
        let entry = create_table(state);
        table_set(state, entry, "itemID", Val::Num(dep.item_id as f64));
        table_set(
            state,
            entry,
            "quantityRequired",
            Val::Num(dep.quantity as f64),
        );
        set_table_array(state, table, (index + 1) as i64, entry);
    }
    table
}

fn c_trade_skill_ui_get_categories(state: &mut LuaState) -> LuaResult<u32> {
    let category_ids = profession_data::get_category_ids();
    for category_id in &category_ids {
        state.push(Val::Num(*category_id as f64));
    }
    Ok(category_ids.len() as u32)
}

fn c_trade_skill_ui_get_filtered_recipe_ids(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_ids = profession_data::get_filtered_recipe_ids();
    let table = recipe_id_table(state, &recipe_ids);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_profession_info_by_recipe_id(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let profession = profession_for_recipe(recipe_id);
    let table = profession_table(state, profession);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_profession_by_inventory_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    match profession_for_inventory_slot(slot) {
        Some(profession) => state.push(Val::Num(profession as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_trade_skill_ui_get_profession_inventory_slots(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    for (index, slot) in PROFESSION_INVENTORY_SLOTS.iter().enumerate() {
        set_table_array(state, table, (index + 1) as i64, Val::Num(*slot as f64));
    }
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_profession_skill_line_id(state: &mut LuaState) -> LuaResult<u32> {
    let profession_id = i32::from_stack(state, 1)?;
    let skill_line_id = profession_data::PROFESSIONS
        .iter()
        .find(|profession| profession.profession == profession_id)
        .map(|profession| profession.skill_line_id)
        .unwrap_or(profession_id);
    state.push(Val::Num(skill_line_id as f64));
    Ok(1)
}

fn c_trade_skill_ui_get_trade_skill_display_name(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line_id = i32::from_stack(state, 1)?;
    let display_name = profession_data::get_profession_by_skill_line_id(skill_line_id)
        .map(|profession| profession.name)
        .unwrap_or("");
    let name = create_string(state, display_name);
    state.push(name);
    Ok(1)
}

fn c_trade_skill_ui_get_profession_slots(state: &mut LuaState) -> LuaResult<u32> {
    let profession = i32::from_stack(state, 1)?;
    let slots = profession_slots(profession);
    let table = create_table(state);
    for (index, slot) in slots.iter().enumerate() {
        set_table_array(state, table, (index + 1) as i64, Val::Num(*slot as f64));
    }
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_professions(state: &mut LuaState) -> LuaResult<u32> {
    let table = skill_line_id_table(state);
    state.push(table);
    Ok(1)
}

fn is_profession_unlearned(state: &LuaState, skill_line_id: i32) -> bool {
    borrow_state(state)
        .map(|s| s.crafting.unlearned_profession_ids.contains(&skill_line_id))
        .unwrap_or(false)
}

fn get_professions_global(state: &mut LuaState) -> LuaResult<u32> {
    // Slots 1-2 are the two primaries. Surviving primaries collapse to fill from slot 1.
    let mut slot1 = Val::Nil;
    let mut slot2 = Val::Nil;
    let mut primary_slot = 0usize;
    for (index, profession) in profession_data::PROFESSIONS.iter().enumerate() {
        if profession.parent_profession_name.is_empty()
            && !is_profession_unlearned(state, profession.skill_line_id)
        {
            primary_slot += 1;
            let index_val = Val::Num((index + 1) as f64);
            match primary_slot {
                1 => slot1 = index_val,
                2 => slot2 = index_val,
                _ => break,
            }
        }
    }
    state.push(slot1);
    state.push(slot2);
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(5)
}

fn get_profession_info_global(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let Some(profession) = visible_global_profession(state, index) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_global_profession_info(state, profession);
    Ok(11)
}

fn visible_global_profession(
    state: &mut LuaState,
    index: i32,
) -> Option<&'static profession_data::ProfessionInfo> {
    let profession = global_profession(index)?;
    (!is_profession_unlearned(state, profession.skill_line_id)).then_some(profession)
}

fn push_global_profession_info(state: &mut LuaState, profession: &profession_data::ProfessionInfo) {
    let name = create_string(state, profession.name);
    let icon = create_string(state, profession_icon_path(profession.profession_id));
    let skill_line_name = if profession.parent_profession_name.is_empty() {
        create_string(state, profession.name)
    } else {
        create_string(state, profession.parent_profession_name)
    };
    let spellbook_skill_line = profession_spellbook_skill_line(profession.name).unwrap_or(0);
    let spell_offset = if spellbook_skill_line > 0 {
        spellbook_data::skill_line_offset(spellbook_skill_line) as f64
    } else {
        0.0
    };
    let num_spells = profession_spell_count(profession.name);

    state.push(name);
    state.push(icon);
    state.push(Val::Num(profession.skill_level as f64));
    state.push(Val::Num(profession.max_skill_level as f64));
    state.push(Val::Num(num_spells as f64));
    state.push(Val::Num(spell_offset));
    state.push(Val::Num(profession.skill_line_id as f64));
    state.push(Val::Num(profession.skill_modifier as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(skill_line_name);
}

fn c_trade_skill_ui_get_num_recipes(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(profession_data::BLACKSMITHING_RECIPES.len() as f64));
    Ok(1)
}

fn c_trade_skill_ui_get_num_trade_skills(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(profession_data::BLACKSMITHING_RECIPES.len() as f64));
    Ok(1)
}

fn global_profession(index: i32) -> Option<&'static profession_data::ProfessionInfo> {
    usize::try_from(index.saturating_sub(1))
        .ok()
        .and_then(profession_data::get_profession_by_index)
}

fn profession_spellbook_skill_line(name: &str) -> Option<i32> {
    (1..=spellbook_data::num_skill_lines()).find(|index| {
        spellbook_data::get_skill_line(*index)
            .map(|skill_line| skill_line.name == name)
            .unwrap_or(false)
    })
}

fn profession_spell_count(name: &str) -> i32 {
    profession_spellbook_skill_line(name)
        .and_then(spellbook_data::get_skill_line)
        .map(|skill_line| skill_line.spells.len() as i32)
        .unwrap_or(0)
}

fn profession_icon_path(profession_id: i32) -> &'static str {
    match profession_id {
        164 => "Interface\\Icons\\Trade_BlackSmithing",
        186 => "Interface\\Icons\\Trade_Mining",
        _ => "Interface\\Icons\\INV_Scroll_04",
    }
}

fn c_trade_skill_ui_get_recipe_info(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let recipe = profession_data::get_recipe(recipe_id);
    let table = recipe_info_table(state, recipe);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_recipe_description(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let description = u32::try_from(recipe_id)
        .ok()
        .and_then(crate::spell_descriptions::get_spell_description)
        .unwrap_or("");
    let description = create_string(state, description);
    state.push(description);
    Ok(1)
}

fn c_trade_skill_ui_get_recipe_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let link = profession_data::get_recipe(recipe_id)
        .and_then(|recipe| item_link_value(state, recipe.output_item_id));
    state.push(link.unwrap_or(Val::Nil));
    Ok(1)
}

fn c_trade_skill_ui_get_recipe_item_name_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = create_string(state, "");
    state.push(filter);
    Ok(1)
}

fn c_trade_skill_ui_get_recipe_output_item_data(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let recipe = profession_data::get_recipe(recipe_id);
    let table = create_table(state);
    if let Some(recipe) = recipe {
        let link = item_link_value(state, recipe.output_item_id).unwrap_or(Val::Nil);
        table_set(state, table, "hyperlink", link);
        set_number_field(state, table, "icon", item_icon(recipe.output_item_id));
        if recipe.output_item_id == 0 {
            table_set(state, table, "itemID", Val::Nil);
        } else {
            set_number_field(state, table, "itemID", recipe.output_item_id as f64);
        }
    }
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_recipe_num_reagents(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let count = profession_data::get_recipe(recipe_id)
        .map(|recipe| recipe.reagents.len())
        .unwrap_or(0);
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn c_trade_skill_ui_get_recipe_reagent_info(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let reagent_index = reagent_index_from_stack(state)?;
    let reagent = profession_data::get_recipe(recipe_id)
        .and_then(|recipe| recipe.reagents.get(reagent_index));
    let table = reagent_info_table(state, reagent);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_get_recipe_reagent_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let reagent_index = reagent_index_from_stack(state)?;
    let link = profession_data::get_recipe(recipe_id)
        .and_then(|recipe| recipe.reagents.get(reagent_index))
        .and_then(|reagent| item_link_value(state, reagent.item_id));
    state.push(link.unwrap_or(Val::Nil));
    Ok(1)
}

fn c_trade_skill_ui_get_recipe_schematic(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let recipe = profession_data::get_recipe(recipe_id);
    let table = recipe_schematic_table(state, recipe);
    state.push(table);
    Ok(1)
}

fn c_trade_skill_ui_is_recipe_craftable(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let count = Option::<i32>::from_stack(state, 2)?.unwrap_or(1).max(1);
    let craftable = recipe_is_craftable(state, recipe_id, count);
    state.push(Val::Bool(craftable));
    Ok(1)
}

fn c_trade_skill_ui_is_recipe_in_skill_line(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let skill_line_id = i32::from_stack(state, 2)?;
    let in_skill_line = profession_data::get_recipe(recipe_id)
        .and_then(|_| profession_data::get_profession(skill_line_id))
        .map(|profession| profession.profession == BLACKSMITHING_PROFESSION)
        .unwrap_or(false);
    state.push(Val::Bool(in_skill_line));
    Ok(1)
}

fn c_trade_skill_ui_craft_recipe(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let count = Option::<i32>::from_stack(state, 2)?.unwrap_or(1).max(1);
    let success = craft_recipe(state, recipe_id, count)?;
    state.push(Val::Bool(success));
    Ok(1)
}

fn c_trade_skill_ui_get_trade_skill_line(state: &mut LuaState) -> LuaResult<u32> {
    let profession =
        selected_profession(state).or_else(|| profession_data::get_profession_by_index(0));
    let (skill_line_id, skill_level, max_skill_level) = profession
        .map(|profession| {
            (
                profession.skill_line_id as f64,
                profession.skill_level as f64,
                profession.max_skill_level as f64,
            )
        })
        .unwrap_or((0.0, 0.0, 0.0));
    state.push(Val::Num(skill_line_id));
    state.push(Val::Nil);
    state.push(Val::Num(skill_level));
    state.push(Val::Num(max_skill_level));
    Ok(4)
}

fn c_trade_skill_ui_get_trade_skill_list_link(state: &mut LuaState) -> LuaResult<u32> {
    let profession =
        selected_profession(state).or_else(|| profession_data::get_profession_by_index(0));
    let link = profession
        .map(|profession| {
            format!(
                "|cff71d5ff|Htrade:{}:{}:{}|h[{}]|h|r",
                profession.profession_id,
                profession.skill_level,
                profession.max_skill_level,
                profession.name
            )
        })
        .unwrap_or_default();
    let link = create_string(state, &link);
    state.push(link);
    Ok(1)
}

fn c_trade_skill_ui_get_trade_skill_texture(state: &mut LuaState) -> LuaResult<u32> {
    let profession_id = Option::<i32>::from_stack(state, 1)?;
    let icon = profession_id
        .and_then(profession_data::get_profession)
        .or_else(|| profession_data::get_profession_by_index(0))
        .map(|profession| profession.icon as f64)
        .unwrap_or(0.0);
    state.push(Val::Num(icon));
    Ok(1)
}

fn c_trade_skill_ui_is_data_source_changing(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_trade_skill_ui_is_npc_crafting(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_trade_skill_ui_is_runeforging(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_trade_skill_ui_is_trade_skill_guild(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_trade_skill_ui_is_trade_skill_guild_member(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_trade_skill_ui_is_trade_skill_linked(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    state.push(Val::Nil);
    Ok(2)
}

fn c_trade_skill_ui_is_trade_skill_ready(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_trade_skill_ui_open_trade_skill(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line_id = i32::from_stack(state, 1)?;
    let opened = open_trade_skill_for_skill_line(state, skill_line_id)?;
    state.push(Val::Bool(opened));
    Ok(1)
}

fn c_trade_skill_ui_close_trade_skill(state: &mut LuaState) -> LuaResult<u32> {
    fire_named_event_state(state, "TRADE_SKILL_CLOSE", &[]);
    Ok(0)
}

fn c_trade_skill_ui_set_profession_child_skill_line_id(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line_id = i32::from_stack(state, 1)?;
    if profession_data::get_profession(skill_line_id).is_none() {
        return Ok(0);
    }

    select_profession(state, skill_line_id)?;
    Ok(0)
}

fn select_profession(state: &mut LuaState, skill_line_id: i32) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, TRADE_SKILL_NAMESPACE)?;
    borrow_state_mut(state)?.crafting.selected_profession_id = Some(skill_line_id);
    table_set(
        state,
        Val::Table(table_ref),
        SELECTED_PROFESSION_KEY,
        Val::Num(skill_line_id as f64),
    );
    Ok(())
}
