use crate::c_api::crafting_execution::craft_recipe;
use crate::items;
use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use super::professions_tables::{item_icon, item_link_value, selected_profession};

type TradeSkillGlobal = (&'static str, fn(&mut LuaState) -> LuaResult<u32>);

pub(super) fn register_legacy_trade_skill_globals(state: &mut LuaState) -> LuaResult<()> {
    register_globals(state, LEGACY_TRADE_SKILL_FRAME_GLOBALS)?;
    register_globals(state, LEGACY_TRADE_SKILL_RECIPE_GLOBALS)?;
    register_globals(state, LEGACY_TRADE_SKILL_FILTER_GLOBALS)?;
    register_globals(state, LEGACY_TRADE_SKILL_ACTION_GLOBALS)
}

const LEGACY_TRADE_SKILL_FRAME_GLOBALS: &[TradeSkillGlobal] = &[
    ("GetFirstTradeSkill", get_first_trade_skill),
    ("GetNumTradeSkills", get_num_trade_skills),
    ("GetTradeskillRepeatCount", get_tradeskill_repeat_count),
    ("GetTradeSkillLine", get_trade_skill_line),
    (
        "GetTradeSkillSelectionIndex",
        get_trade_skill_selection_index,
    ),
    ("IsTradeSkillLinked", is_trade_skill_linked),
    ("StopTradeSkillRepeat", stop_trade_skill_repeat),
];

const LEGACY_TRADE_SKILL_RECIPE_GLOBALS: &[TradeSkillGlobal] = &[
    ("GetTradeSkillCooldown", get_trade_skill_cooldown),
    ("GetTradeSkillDescription", get_trade_skill_description),
    ("GetTradeSkillIcon", get_trade_skill_icon),
    ("GetTradeSkillInfo", get_trade_skill_info),
    ("GetTradeSkillItemLink", get_trade_skill_item_link),
    ("GetTradeSkillListLink", get_trade_skill_list_link),
    ("GetTradeSkillNumMade", get_trade_skill_num_made),
    ("GetTradeSkillNumReagents", get_trade_skill_num_reagents),
    ("GetTradeSkillReagentInfo", get_trade_skill_reagent_info),
    (
        "GetTradeSkillReagentItemLink",
        get_trade_skill_reagent_item_link,
    ),
    ("GetTradeSkillRecipeLink", get_trade_skill_recipe_link),
    ("GetTradeSkillTools", get_trade_skill_tools),
];

const LEGACY_TRADE_SKILL_FILTER_GLOBALS: &[TradeSkillGlobal] = &[
    ("GetOnlyShowMakeable", get_only_show_makeable),
    ("GetOnlyShowSkillUps", get_only_show_skill_ups),
    (
        "GetTradeSkillInvSlotFilter",
        get_trade_skill_inv_slot_filter,
    ),
    ("GetTradeSkillInvSlots", get_trade_skill_inv_slots),
    (
        "GetTradeSkillSubClassFilter",
        get_trade_skill_sub_class_filter,
    ),
    ("GetTradeSkillSubClasses", get_trade_skill_sub_classes),
    (
        "SetTradeSkillInvSlotFilter",
        set_trade_skill_inv_slot_filter,
    ),
    (
        "SetTradeSkillItemLevelFilter",
        set_trade_skill_item_level_filter,
    ),
    (
        "SetTradeSkillItemNameFilter",
        set_trade_skill_item_name_filter,
    ),
    (
        "SetTradeSkillSubClassFilter",
        set_trade_skill_sub_class_filter,
    ),
    ("TradeSkillOnlyShowMakeable", trade_skill_only_show_makeable),
    (
        "TradeSkillOnlyShowSkillUps",
        trade_skill_only_show_skill_ups,
    ),
];

const LEGACY_TRADE_SKILL_ACTION_GLOBALS: &[TradeSkillGlobal] = &[
    ("CloseTradeSkill", close_trade_skill),
    ("CollapseTradeSkillSubClass", collapse_trade_skill_sub_class),
    ("DoTradeSkill", do_trade_skill),
    ("ExpandTradeSkillSubClass", expand_trade_skill_sub_class),
    ("SelectTradeSkill", select_trade_skill),
];

fn register_globals(state: &mut LuaState, globals: &[TradeSkillGlobal]) -> LuaResult<()> {
    for (name, function) in globals {
        table_set_rust_fn_static(state, state.global, name, *function)?;
    }
    Ok(())
}

fn get_num_trade_skills(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(legacy_trade_skill_recipes().len() as f64));
    Ok(1)
}

fn get_first_trade_skill(state: &mut LuaState) -> LuaResult<u32> {
    let first_index = if legacy_trade_skill_recipes().is_empty() {
        0.0
    } else {
        1.0
    };
    state.push(Val::Num(first_index));
    Ok(1)
}

fn select_trade_skill(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    if legacy_trade_skill_recipe(trade_skill_index).is_some() {
        borrow_state_mut(state)?.crafting.selected_trade_skill_index = Some(trade_skill_index);
    }
    Ok(0)
}

fn get_trade_skill_selection_index(state: &mut LuaState) -> LuaResult<u32> {
    let selected_index = borrow_state(state)?
        .crafting
        .selected_trade_skill_index
        .unwrap_or(0);
    state.push(Val::Num(selected_index as f64));
    Ok(1)
}

fn get_trade_skill_line(state: &mut LuaState) -> LuaResult<u32> {
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

fn get_trade_skill_info(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let recipe = legacy_trade_skill_recipe(trade_skill_index);
    let recipe_name = recipe_string_field(state, recipe, |recipe| recipe.name);
    let difficulty = recipe_string_field(state, recipe, legacy_trade_skill_difficulty);

    state.push(recipe_name);
    state.push(difficulty);
    state.push(Val::Num(legacy_trade_skill_available_count(recipe) as f64));
    state.push(Val::Bool(true));
    state.push(Val::Nil);
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(11)
}

fn get_trade_skill_icon(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let icon = legacy_trade_skill_recipe(trade_skill_index)
        .map(|recipe| item_icon(recipe.output_item_id))
        .unwrap_or(134400.0);
    state.push(Val::Num(icon));
    Ok(1)
}

fn get_trade_skill_num_made(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let quantity = legacy_trade_skill_recipe(trade_skill_index)
        .map(|recipe| recipe.output_quantity)
        .unwrap_or(1);
    state.push(Val::Num(quantity as f64));
    state.push(Val::Num(quantity as f64));
    Ok(2)
}

fn get_trade_skill_num_reagents(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let count = legacy_trade_skill_recipe(trade_skill_index)
        .map(|recipe| recipe.reagents.len())
        .unwrap_or(0);
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_trade_skill_reagent_info(state: &mut LuaState) -> LuaResult<u32> {
    let reagent = reagent_from_stack(state)?;
    let item = reagent.and_then(|reagent| items::get_item(reagent.item_id));
    let reagent_name = item
        .map(|item| create_string(state, item.name))
        .unwrap_or(Val::Nil);
    let icon = item.map(|item| item.icon_file_data_id).unwrap_or(134400);
    let quantity = reagent.map(|reagent| reagent.quantity).unwrap_or(0);

    state.push(reagent_name);
    state.push(Val::Num(icon as f64));
    state.push(Val::Num(quantity as f64));
    state.push(Val::Num(quantity as f64));
    Ok(4)
}

fn get_trade_skill_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let link = legacy_trade_skill_recipe(trade_skill_index)
        .and_then(|recipe| item_link_value(state, recipe.output_item_id));
    state.push(link.unwrap_or(Val::Nil));
    Ok(1)
}

fn get_trade_skill_reagent_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let reagent = reagent_from_stack(state)?;
    let link = reagent.and_then(|reagent| item_link_value(state, reagent.item_id));
    state.push(link.unwrap_or(Val::Nil));
    Ok(1)
}

fn get_trade_skill_recipe_link(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let link = legacy_trade_skill_recipe(trade_skill_index).map(legacy_recipe_link);
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_trade_skill_list_link(state: &mut LuaState) -> LuaResult<u32> {
    let profession =
        selected_profession(state).or_else(|| profession_data::get_profession_by_index(0));
    let link = profession.map(|profession| {
        format!(
            "|cff71d5ff|Htrade:{}:{}:{}|h[{}]|h|r",
            profession.profession_id,
            profession.skill_level,
            profession.max_skill_level,
            profession.name
        )
    });

    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_trade_skill_description(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let description = legacy_trade_skill_recipe(trade_skill_index)
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

fn get_trade_skill_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn get_trade_skill_tools(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_tradeskill_repeat_count(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

fn is_trade_skill_linked(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_only_show_makeable(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_only_show_skill_ups(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_trade_skill_inv_slots(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_trade_skill_sub_classes(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_trade_skill_inv_slot_filter(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

fn get_trade_skill_sub_class_filter(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

fn set_trade_skill_inv_slot_filter(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn set_trade_skill_sub_class_filter(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn set_trade_skill_item_name_filter(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn set_trade_skill_item_level_filter(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn trade_skill_only_show_makeable(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn trade_skill_only_show_skill_ups(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn expand_trade_skill_sub_class(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn collapse_trade_skill_sub_class(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn stop_trade_skill_repeat(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn do_trade_skill(state: &mut LuaState) -> LuaResult<u32> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let count = Option::<i32>::from_stack(state, 2)?.unwrap_or(1).max(1);
    if let Some(recipe) = legacy_trade_skill_recipe(trade_skill_index) {
        let _ = craft_recipe(state, recipe.recipe_id, count);
    }
    Ok(0)
}

fn close_trade_skill(state: &mut LuaState) -> LuaResult<u32> {
    fire_named_event_state(state, "TRADE_SKILL_CLOSE", &[]);
    Ok(0)
}

fn legacy_trade_skill_recipes() -> &'static [profession_data::RecipeEntry] {
    profession_data::BLACKSMITHING_RECIPES
}

fn legacy_trade_skill_recipe(
    trade_skill_index: i32,
) -> Option<&'static profession_data::RecipeEntry> {
    usize::try_from(trade_skill_index.saturating_sub(1))
        .ok()
        .and_then(|index| legacy_trade_skill_recipes().get(index))
}

fn reagent_from_stack(
    state: &mut LuaState,
) -> LuaResult<Option<&'static profession_data::ReagentSlot>> {
    let trade_skill_index = i32::from_stack(state, 1)?;
    let reagent_index = i32::from_stack(state, 2)?.saturating_sub(1) as usize;
    Ok(legacy_trade_skill_recipe(trade_skill_index)
        .and_then(|recipe| recipe.reagents.get(reagent_index)))
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

fn legacy_trade_skill_difficulty(recipe: &profession_data::RecipeEntry) -> &str {
    match recipe.difficulty {
        4.. => "optimal",
        3 => "medium",
        2 => "easy",
        _ => "trivial",
    }
}

fn legacy_trade_skill_available_count(recipe: Option<&profession_data::RecipeEntry>) -> i32 {
    recipe.filter(|recipe| recipe.craftable).map_or(0, |_| 1)
}

fn legacy_recipe_link(recipe: &profession_data::RecipeEntry) -> String {
    format!(
        "|cff71d5ff|Henchant:{}|h[{}]|h|r",
        recipe.recipe_id, recipe.name
    )
}
