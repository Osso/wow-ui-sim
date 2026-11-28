//! Retail legacy quest reward item reads (secure-publication rows 5642/5644/5649).
//! `QuestUtils_GetBestQualityItemRewardIndex` selects an item from supplied rows;
//! `QuestInfo.lua` and `QuestFrameTemplates.xml` consume the item links.
//! INFERRED (R-ag10-18): empty indices or missing item identity return no values.

use crate::c_api::c_tooltip_info_item_sources::{active_dialog_quest_id, quest_item_id};
use crate::lua_api::globals::quest_surface::push_quest_reward_item;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_api::state::QuestRewardItem;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn selected_quest(state: &LuaState) -> LuaResult<Option<i32>> {
    Ok(borrow_state(state)?
        .selected_quest_log_id
        .and_then(|id| i32::try_from(id).ok()))
}

fn query_quest(state: &LuaState, arg: i32) -> LuaResult<Option<i32>> {
    Ok(Option::<i32>::from_stack(state, arg)?.or(selected_quest(state)?))
}

fn choice_rows(state: &LuaState, quest: Option<i32>) -> LuaResult<Vec<QuestRewardItem>> {
    Ok(borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|row| Some(row.quest_id) == quest)
        .map(|row| row.reward_choices.clone())
        .unwrap_or_default())
}

fn get_num_quest_log_choices(state: &mut LuaState) -> LuaResult<u32> {
    let count = choice_rows(state, query_quest(state, 1)?)?.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_num_quest_choices(state: &mut LuaState) -> LuaResult<u32> {
    let count = choice_rows(state, active_dialog_quest_id(state)?)?.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_quest_log_choice_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = index(state, 1)? else {
        return Ok(0);
    };
    let row = choice_rows(state, query_quest(state, 2)?)?
        .get(index)
        .cloned();
    let item_id = row.as_ref().and_then(|row| row.item_id);
    let count = push_quest_reward_item(state, row)?;
    if count == 0 {
        return Ok(0);
    }
    state.push(item_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(count + 1)
}

fn index(state: &LuaState, arg: i32) -> LuaResult<Option<usize>> {
    Ok(i32::from_stack(state, arg)?
        .checked_sub(1)
        .and_then(|i| usize::try_from(i).ok()))
}

fn item_link(state: &mut LuaState, quest: Option<i32>) -> LuaResult<u32> {
    let selector = String::from_stack(state, 1)?.to_ascii_lowercase();
    let (Some(quest), Some(index)) = (quest, index(state, 2)?) else {
        return Ok(0);
    };
    let link = quest_item_id(state, quest, &selector, index)?
        .and_then(crate::c_api::item_spell::item_link_for_id);
    let Some(link) = link else { return Ok(0) };
    let value = create_string(state, &link);
    state.push(value);
    Ok(1)
}

fn get_quest_item_link(state: &mut LuaState) -> LuaResult<u32> {
    item_link(state, active_dialog_quest_id(state)?)
}

fn get_quest_log_item_link(state: &mut LuaState) -> LuaResult<u32> {
    item_link(state, query_quest(state, 3)?)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetNumQuestLogChoices", get_num_quest_log_choices)?;
    LuaApiMut::register_function(lua, "GetNumQuestChoices", get_num_quest_choices)?;
    LuaApiMut::register_function(lua, "GetQuestLogChoiceInfo", get_quest_log_choice_info)?;
    LuaApiMut::register_function(lua, "GetQuestItemLink", get_quest_item_link)?;
    LuaApiMut::register_function(lua, "GetQuestLogItemLink", get_quest_log_item_link)
}
