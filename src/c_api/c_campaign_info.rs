//! Supplied campaign chapter associations.

use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_CampaignInfo")?;
    table_set_rust_fn_static(state, ns, "GetChapterIDs", get_chapter_ids)?;
    table_set_rust_fn_static(state, ns, "GetCampaignID", get_campaign_id)?;
    table_set_rust_fn_static(state, ns, "GetCampaignInfo", get_campaign_info)?;
    table_set_rust_fn_static(state, ns, "GetState", get_state)
}

fn get_chapter_ids(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let chapters = borrow_state(state)?
        .quest_log_entries
        .campaign_chapter_ids
        .get(&id)
        .cloned();
    let Some(chapters) = chapters else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let result = create_table(state);
    for (index, chapter) in chapters.iter().enumerate() {
        super::helpers::set_table_array(
            state,
            result,
            (index + 1) as i64,
            Val::Num(f64::from(*chapter)),
        );
    }
    state.push(result);
    Ok(1)
}

fn get_campaign_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest = i32::from_stack(state, 1)?;
    let id = borrow_state(state)?
        .quest_log_entries
        .campaign_ids
        .get(&quest)
        .copied()
        .unwrap_or(0);
    state.push(Val::Num(f64::from(id)));
    Ok(1)
}

fn get_campaign_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let exists = borrow_state(state)?
        .quest_log_entries
        .campaign_ids
        .values()
        .any(|campaign| *campaign == id && id != 0);
    let result = if exists {
        create_table(state)
    } else {
        Val::Nil
    };
    state.push(result);
    Ok(1)
}

fn get_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let result = {
        let sim = borrow_state(state)?;
        let log = &sim.quest_log_entries;
        let quests: Vec<_> = log
            .campaign_ids
            .iter()
            .filter_map(|(quest, campaign)| (*campaign == id && id != 0).then_some(*quest))
            .collect();
        let complete = |quest: &i32| {
            log.completed_quest_ids.contains(quest)
                || log
                    .entries
                    .iter()
                    .any(|entry| entry.quest_id == *quest && entry.is_complete && !entry.is_failed)
        };
        if !quests.is_empty() && quests.iter().all(complete) {
            1
        } else if quests.iter().any(|quest| {
            complete(quest) || log.entries.iter().any(|entry| entry.quest_id == *quest)
        }) {
            2
        } else {
            0
        }
    };
    state.push(Val::Num(f64::from(result)));
    Ok(1)
}
