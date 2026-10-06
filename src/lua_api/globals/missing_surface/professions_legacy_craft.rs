use crate::c_api::crafting_execution::craft_recipe;
use crate::items;
use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use super::professions_tables::{item_icon, item_link_value, selected_profession};

const LEGACY_CRAFT_TOOL_FOCUS: &str = "Anvil";

type CraftGlobal = (&'static str, fn(&mut LuaState) -> LuaResult<u32>);

pub(super) fn register_legacy_craft_globals(state: &mut LuaState) -> LuaResult<()> {
    register_globals(state, LEGACY_CRAFT_FRAME_GLOBALS)?;
    register_globals(state, LEGACY_CRAFT_RECIPE_GLOBALS)?;
    register_globals(state, LEGACY_CRAFT_ACTION_GLOBALS)
}

const LEGACY_CRAFT_FRAME_GLOBALS: &[CraftGlobal] = &[
    ("CloseCraft", close_craft),
    ("CraftIsEnchanting", craft_is_enchanting),
    ("GetCraftButtonToken", get_craft_button_token),
    ("GetCraftDisplaySkillLine", get_craft_display_skill_line),
    ("GetCraftFilter", get_craft_filter),
    ("GetCraftName", get_craft_name),
    ("GetCraftSelectionIndex", get_craft_selection_index),
    ("GetCraftSlots", get_craft_slots),
    ("GetPetTrainingPoints", get_pet_training_points),
    ("SetCraftFilter", set_craft_filter),
];

const LEGACY_CRAFT_RECIPE_GLOBALS: &[CraftGlobal] = &[
    ("GetCraftCooldown", get_craft_cooldown),
    ("GetCraftDescription", get_craft_description),
    ("GetCraftIcon", get_craft_icon),
    ("GetCraftInfo", get_craft_info),
    ("GetCraftItemLink", get_craft_item_link),
    ("GetCraftNumMade", get_craft_num_made),
    ("GetCraftNumReagents", get_craft_num_reagents),
    ("GetCraftReagentInfo", get_craft_reagent_info),
    ("GetCraftReagentItemLink", get_craft_reagent_item_link),
    ("GetCraftRecipeLink", get_craft_recipe_link),
    ("GetCraftSpellFocus", get_craft_spell_focus),
    ("GetNumCrafts", get_num_crafts),
];

const LEGACY_CRAFT_ACTION_GLOBALS: &[CraftGlobal] = &[
    ("CollapseCraftSkillLine", collapse_craft_skill_line),
    ("DoCraft", do_craft),
    ("ExpandCraftSkillLine", expand_craft_skill_line),
    ("SelectCraft", select_craft),
];

fn register_globals(state: &mut LuaState, globals: &[CraftGlobal]) -> LuaResult<()> {
    for (name, function) in globals {
        table_set_rust_fn_static(state, state.global, name, *function)?;
    }
    Ok(())
}

fn get_num_crafts(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(legacy_craft_recipes().len() as f64));
    Ok(1)
}

fn get_craft_info(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    let recipe = legacy_craft_recipe(craft_index);

    let craft_name = recipe_string_field(state, recipe, |recipe| recipe.name);
    let craft_difficulty = recipe_string_field(state, recipe, legacy_craft_difficulty);

    state.push(craft_name);
    state.push(Val::Nil);
    state.push(craft_difficulty);
    state.push(Val::Num(legacy_craft_available_count(recipe) as f64));
    state.push(Val::Bool(true));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(7)
}

fn get_craft_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = selected_profession(state)
        .map(|profession| profession.name)
        .unwrap_or("Blacksmithing");
    let name = create_string(state, name);
    state.push(name);
    Ok(1)
}

fn get_craft_button_token(state: &mut LuaState) -> LuaResult<u32> {
    let token = create_string(state, "CREATE");
    state.push(token);
    Ok(1)
}

fn craft_is_enchanting(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_craft_display_skill_line(state: &mut LuaState) -> LuaResult<u32> {
    let profession =
        selected_profession(state).or_else(|| profession_data::get_profession_by_index(0));
    let Some(profession) = profession else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let name = create_string(state, profession.name);
    state.push(name);
    state.push(Val::Num(profession.skill_level as f64));
    state.push(Val::Num(profession.max_skill_level as f64));
    Ok(3)
}

fn select_craft(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    if legacy_craft_recipe(craft_index).is_some() {
        borrow_state_mut(state)?.crafting.selected_craft_index = Some(craft_index);
    }
    Ok(0)
}

fn get_craft_selection_index(state: &mut LuaState) -> LuaResult<u32> {
    let selected_index = borrow_state(state)?
        .crafting
        .selected_craft_index
        .unwrap_or(1);
    state.push(Val::Num(selected_index as f64));
    Ok(1)
}

fn get_craft_icon(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    let icon = legacy_craft_recipe(craft_index)
        .map(|recipe| item_icon(recipe.output_item_id))
        .unwrap_or(134400.0);
    state.push(Val::Num(icon));
    Ok(1)
}

fn get_craft_num_made(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    let quantity = legacy_craft_recipe(craft_index)
        .map(|recipe| recipe.output_quantity)
        .unwrap_or(1);
    state.push(Val::Num(quantity as f64));
    state.push(Val::Num(quantity as f64));
    Ok(2)
}

fn get_craft_description(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    let description = legacy_craft_recipe(craft_index)
        .and_then(|recipe| u32::try_from(recipe.recipe_id).ok())
        .and_then(crate::spell_descriptions::get_spell_description)
        .filter(|description| !description.is_empty());

    match description {
        Some(description) => {
            let description = create_string(state, description);
            state.push(description);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_craft_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn get_craft_num_reagents(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    let count = legacy_craft_recipe(craft_index)
        .map(|recipe| recipe.reagents.len())
        .unwrap_or(0);
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_craft_reagent_info(state: &mut LuaState) -> LuaResult<u32> {
    let reagent = reagent_from_stack(state)?;
    let item = reagent.and_then(|reagent| items::get_item(reagent.item_id));

    let reagent_name = match item {
        Some(item) => create_string(state, item.name),
        None => Val::Nil,
    };
    state.push(reagent_name);
    let icon = item.map(|item| item.icon_file_data_id);
    state.push(Val::Num(reagent_icon(icon) as f64));
    state.push(Val::Num(reagent_quantity(reagent) as f64));
    state.push(Val::Num(reagent_quantity(reagent) as f64));
    Ok(4)
}

fn get_craft_spell_focus(state: &mut LuaState) -> LuaResult<u32> {
    let focus = create_string(state, LEGACY_CRAFT_TOOL_FOCUS);
    state.push(focus);
    state.push(Val::Bool(true));
    Ok(2)
}

fn get_craft_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    let link = legacy_craft_recipe(craft_index)
        .and_then(|recipe| item_link_value(state, recipe.output_item_id));
    state.push(link.unwrap_or(Val::Nil));
    Ok(1)
}

fn get_craft_reagent_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let reagent = reagent_from_stack(state)?;
    let link = reagent.and_then(|reagent| item_link_value(state, reagent.item_id));
    state.push(link.unwrap_or(Val::Nil));
    Ok(1)
}

fn get_craft_recipe_link(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    let link = legacy_craft_recipe(craft_index).map(legacy_recipe_link);
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_craft_slots(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_craft_filter(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<i32>::from_stack(state, 1)?.unwrap_or(0);
    state.push(Val::Bool(index == 0));
    Ok(1)
}

fn get_pet_training_points(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(2)
}

fn set_craft_filter(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn expand_craft_skill_line(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn collapse_craft_skill_line(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn do_craft(state: &mut LuaState) -> LuaResult<u32> {
    let craft_index = i32::from_stack(state, 1)?;
    if let Some(recipe) = legacy_craft_recipe(craft_index) {
        let _ = craft_recipe(state, recipe.recipe_id, 1);
    }
    Ok(0)
}

fn close_craft(state: &mut LuaState) -> LuaResult<u32> {
    fire_named_event_state(state, "CRAFT_CLOSE", &[]);
    Ok(0)
}

fn legacy_craft_recipes() -> &'static [profession_data::RecipeEntry] {
    profession_data::BLACKSMITHING_RECIPES
}

fn legacy_craft_recipe(craft_index: i32) -> Option<&'static profession_data::RecipeEntry> {
    usize::try_from(craft_index.saturating_sub(1))
        .ok()
        .and_then(|index| legacy_craft_recipes().get(index))
}

fn reagent_from_stack(
    state: &mut LuaState,
) -> LuaResult<Option<&'static profession_data::ReagentSlot>> {
    let craft_index = i32::from_stack(state, 1)?;
    let reagent_index = i32::from_stack(state, 2)?.saturating_sub(1) as usize;
    Ok(legacy_craft_recipe(craft_index).and_then(|recipe| recipe.reagents.get(reagent_index)))
}

fn recipe_string_field(
    state: &mut LuaState,
    recipe: Option<&profession_data::RecipeEntry>,
    field: fn(&profession_data::RecipeEntry) -> &str,
) -> Val {
    recipe
        .map(|recipe| create_string(state, field(recipe)))
        .unwrap_or(Val::Nil)
}

fn legacy_craft_difficulty(recipe: &profession_data::RecipeEntry) -> &str {
    match recipe.difficulty {
        4.. => "optimal",
        3 => "medium",
        2 => "easy",
        _ => "trivial",
    }
}

fn legacy_craft_available_count(recipe: Option<&profession_data::RecipeEntry>) -> i32 {
    recipe.filter(|recipe| recipe.craftable).map_or(0, |_| 1)
}

fn reagent_icon(icon: Option<u32>) -> u32 {
    icon.unwrap_or(134400)
}

fn reagent_quantity(reagent: Option<&profession_data::ReagentSlot>) -> i32 {
    reagent.map(|reagent| reagent.quantity).unwrap_or(0)
}

fn legacy_recipe_link(recipe: &profession_data::RecipeEntry) -> String {
    format!(
        "|cff71d5ff|Henchant:{}|h[{}]|h|r",
        recipe.recipe_id, recipe.name
    )
}
