//! Item tooltip getters backed by supplied quest and guild-bank state.
//! `TooltipInfoDocumentation.lua:304–317, 762–775, 798–813` declare the getters.
//! Quest row-to-item mapping follows FrameXML consumers by inference.

use crate::c_api::tooltip_getter_args::one_based_index;
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::QuestLogEntry;
use crate::lua_bridge::FromStack;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

const QUEST_ITEM_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetQuestItem(type, itemIndex [, allowCollectionText])";
const QUEST_LOG_ITEM_USAGE: &str = "Usage: local data = C_TooltipInfo.GetQuestLogItem(type, itemIndex [, questID [, allowCollectionText]])";

pub(crate) fn get_quest_item(state: &mut LuaState) -> LuaResult<u32> {
    let selector = String::from_stack(state, 1)?.to_ascii_lowercase();
    let index = one_based_index(state, 2, QUEST_ITEM_USAGE)?;
    let _allow_collection_text = Option::<bool>::from_stack(state, 3)?;
    let Some(index) = index else { return Ok(0) };
    let Some(quest_id) = active_dialog_quest_id(state)? else {
        return Ok(0);
    };
    let Some(item_id) = quest_item_id(state, quest_id, &selector, index)? else {
        return Ok(0);
    };
    publish_item_tooltip(state, item_id)
}

pub(crate) fn get_quest_log_item(state: &mut LuaState) -> LuaResult<u32> {
    let selector = String::from_stack(state, 1)?.to_ascii_lowercase();
    let index = one_based_index(state, 2, QUEST_LOG_ITEM_USAGE)?;
    let requested_quest = Option::<i32>::from_stack(state, 3)?;
    let _allow_collection_text = Option::<bool>::from_stack(state, 4)?;
    let Some(index) = index else { return Ok(0) };
    let quest_id = match requested_quest {
        Some(id) => Some(id),
        None => borrow_state(state)?
            .selected_quest_log_id
            .and_then(|id| i32::try_from(id).ok()),
    };
    let Some(quest_id) = quest_id.filter(|id| *id > 0) else {
        return Ok(0);
    };
    let Some(item_id) = quest_item_id(state, quest_id, &selector, index)? else {
        return Ok(0);
    };
    publish_item_tooltip(state, item_id)
}

pub(crate) fn get_guild_bank_item(state: &mut LuaState) -> LuaResult<u32> {
    let tab = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let item_id = borrow_state(state)?
        .guild_bank_items
        .get(&(tab, slot))
        .map(|item| item.item_id);
    let Some(item_id) = item_id else { return Ok(0) };
    publish_item_tooltip(state, item_id)
}

pub(crate) fn active_dialog_quest_id(state: &LuaState) -> LuaResult<Option<i32>> {
    let sim = borrow_state(state)?;
    let Some((quest_id, _)) = sim.quest_log_entries.readiness.dialog else {
        return Ok(None);
    };
    let selected = sim
        .selected_quest_log_id
        .and_then(|id| i32::try_from(id).ok());
    Ok((selected == Some(quest_id)
        && crate::c_api::quest_readiness::has_active_npc_quest(&sim, quest_id))
    .then_some(quest_id))
}

pub(crate) fn quest_item_id(
    state: &LuaState,
    quest_id: i32,
    selector: &str,
    index: usize,
) -> LuaResult<Option<u32>> {
    let sim = borrow_state(state)?;
    let Some(entry) = sim
        .quest_log_entries
        .entries
        .iter()
        .find(|row| row.quest_id == quest_id)
    else {
        return Ok(None);
    };
    Ok(item_id_from_entry(entry, selector, index))
}

fn item_id_from_entry(entry: &QuestLogEntry, selector: &str, index: usize) -> Option<u32> {
    // Quest objectives contain no supplied item IDs; only reward and choice rows resolve.
    let item = match selector {
        "reward" => entry.reward_items.get(index),
        "choice" => entry.reward_choices.get(index),
        _ => None,
    }?;
    item.item_id.filter(|id| *id != 0)
}

fn publish_item_tooltip(state: &mut LuaState, item_id: u32) -> LuaResult<u32> {
    let Some(item) = crate::items::get_item(item_id) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = tooltip_for_item_context(state, item_id, Some(item));
    state.push(tooltip);
    Ok(1)
}
