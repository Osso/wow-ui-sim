//! `C_GossipInfo` probe surface backed by `SimState.gossip`.
//!
//! Migrates 4 entries off the namespace stub tables:
//!
//! - `C_GossipInfo.GetOptions()` — array of `GossipOptionUIInfo` tables
//! - `C_GossipInfo.GetActiveQuests()` — array of `GossipQuestUIInfo` tables
//! - `C_GossipInfo.GetAvailableQuests()` — array of `GossipQuestUIInfo` tables
//! - `C_GossipInfo.GetText()` — greeting/body text for the open gossip frame
//! - `C_GossipInfo.SelectAvailableQuest(questID)` — opens QUEST_DETAIL
//! - `C_GossipInfo.SelectActiveQuest(questID)` — opens QUEST_PROGRESS
//! - `C_GossipInfo.CloseGossip()` — closes the active gossip dialog

use super::{ensure_namespace, set_table_array};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, create_table_with_capacity,
    table_set,
};
use crate::lua_api::sim_substates::{GossipOption, GossipQuestRow};
use crate::lua_bridge::FromStack;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const GOSSIP_FRIENDSHIP_HASH_FIELDS: usize = 5;
const GOSSIP_FRIENDSHIP_RANKS_HASH_FIELDS: usize = 2;
const GOSSIP_OPTION_HASH_FIELDS: usize = 8;
const GOSSIP_QUEST_HASH_FIELDS: usize = 12;

pub(super) fn register_gossip_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_GossipInfo")?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    crate::c_api::c_gossip_info_poi::register(state)?;
    table_set_rust_fn_static(state, table_ref, "GetOptions", c_gossip_info_get_options)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetActiveQuests",
        c_gossip_info_get_active_quests,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAvailableQuests",
        c_gossip_info_get_available_quests,
    )?;
    table_set_rust_fn_static(state, table_ref, "GetText", c_gossip_info_get_text)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFriendshipReputation",
        c_gossip_info_get_friendship_reputation,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetFriendshipReputationRanks",
        c_gossip_info_get_friendship_reputation_ranks,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SelectAvailableQuest",
        c_gossip_info_select_available_quest,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SelectActiveQuest",
        c_gossip_info_select_active_quest,
    )?;
    table_set_rust_fn_static(state, table_ref, "CloseGossip", c_gossip_info_close_gossip)?;
    Ok(())
}

fn c_gossip_info_get_options(state: &mut LuaState) -> LuaResult<u32> {
    let options = borrow_state(state)?.gossip.options.clone();
    let array = create_table(state);
    for (i, opt) in options.iter().enumerate() {
        let entry = push_option_table(state, opt);
        set_table_array(state, array, i as i64 + 1, entry);
    }
    state.push(array);
    Ok(1)
}

fn c_gossip_info_get_active_quests(state: &mut LuaState) -> LuaResult<u32> {
    let rows = {
        let sim = borrow_state(state)?;
        let mut rows = sim.gossip.active_quests.clone();
        if crate::c_api::quest_objectives::ENABLED {
            for row in &mut rows {
                row.is_complete = Some(crate::c_api::quest_readiness::is_ready(
                    &sim,
                    row.quest_id as i32,
                ));
            }
        }
        rows
    };
    push_quest_array(state, &rows)
}

fn c_gossip_info_get_available_quests(state: &mut LuaState) -> LuaResult<u32> {
    let rows = borrow_state(state)?.gossip.available_quests.clone();
    push_quest_array(state, &rows)
}

fn c_gossip_info_get_text(state: &mut LuaState) -> LuaResult<u32> {
    let text = borrow_state(state)?.gossip.text.clone();
    let text = create_string(state, &text);
    state.push(text);
    Ok(1)
}

fn c_gossip_info_select_available_quest(state: &mut LuaState) -> LuaResult<u32> {
    let Some(quest_id) = quest_id_arg(state)? else {
        return Ok(0);
    };
    {
        let mut sim = borrow_state_mut(state)?;
        let Some(row) = sim
            .gossip
            .available_quests
            .iter()
            .find(|row| row.quest_id == quest_id)
            .cloned()
        else {
            return Ok(0);
        };
        let offer = sim
            .quest_log_entries
            .snapshot_gossip_offer(&row)
            .map_err(|_| rilua::runtime_error("selected quest ID exceeds the rich-log range"))?;
        sim.quest_log_entries.readiness.dialog = None;
        sim.pending_quest_offer = Some(offer);
        sim.selected_quest_log_id = Some(quest_id);
    }

    dispatch_event_now(state, "QUEST_DETAIL", &[])?;
    Ok(0)
}

fn c_gossip_info_select_active_quest(state: &mut LuaState) -> LuaResult<u32> {
    let Some(quest_id) = quest_id_arg(state)? else {
        return Ok(0);
    };
    if crate::c_api::quest_objectives::ENABLED {
        return crate::c_api::quest_readiness::select_active_dialog(state, quest_id);
    }
    let Some(is_complete) = ({
        let mut sim = borrow_state_mut(state)?;
        let complete = sim
            .gossip
            .active_quests
            .iter()
            .find(|row| row.quest_id == quest_id)
            .map(|row| row.is_complete.unwrap_or(false));
        if complete.is_some() {
            sim.selected_quest_log_id = Some(quest_id);
        }
        complete
    }) else {
        return Ok(0);
    };
    let event = if is_complete {
        "QUEST_COMPLETE"
    } else {
        "QUEST_PROGRESS"
    };
    dispatch_event_now(state, event, &[])?;
    Ok(0)
}

fn c_gossip_info_close_gossip(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.gossip.active = false;
        sim.quest_log_entries.readiness.dialog = None;
        sim.gossip.options.clear();
        sim.gossip.active_quests.clear();
        sim.gossip.available_quests.clear();
        sim.gossip.num_options = 0;
        sim.gossip.num_active_quests = 0;
        sim.gossip.num_available_quests = 0;
    }

    dispatch_event_now(state, "GOSSIP_CLOSED", &[])?;
    Ok(0)
}

fn quest_id_arg(state: &mut LuaState) -> LuaResult<Option<u32>> {
    Ok(Option::<f64>::from_stack(state, 1)?
        .filter(|id| *id > 0.0)
        .map(|id| id as u32))
}

fn c_gossip_info_get_friendship_reputation(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table_with_capacity(state, GOSSIP_FRIENDSHIP_HASH_FIELDS);
    table_set(state, table, "friendshipFactionID", Val::Num(0.0));
    table_set(state, table, "reaction", Val::Num(0.0));
    table_set(state, table, "currentReactionThreshold", Val::Num(0.0));
    table_set(state, table, "nextReactionThreshold", Val::Num(0.0));
    table_set(state, table, "currentStanding", Val::Num(0.0));
    state.push(table);
    Ok(1)
}

fn c_gossip_info_get_friendship_reputation_ranks(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table_with_capacity(state, GOSSIP_FRIENDSHIP_RANKS_HASH_FIELDS);
    table_set(state, table, "currentLevel", Val::Num(0.0));
    table_set(state, table, "maxLevel", Val::Num(0.0));
    state.push(table);
    Ok(1)
}

fn push_option_table(state: &mut LuaState, opt: &GossipOption) -> Val {
    let t = create_table_with_capacity(state, GOSSIP_OPTION_HASH_FIELDS);
    table_set(
        state,
        t,
        "gossipOptionID",
        Val::Num(opt.gossip_option_id as f64),
    );
    table_set(state, t, "orderIndex", Val::Num(opt.order_index as f64));
    let name = create_string(state, &opt.name);
    table_set(state, t, "name", name);
    table_set(state, t, "flags", Val::Num(opt.flags as f64));
    table_set(state, t, "icon", Val::Num(opt.icon as f64));
    match opt.spell_id {
        Some(id) => table_set(state, t, "spellID", Val::Num(id as f64)),
        None => table_set(state, t, "spellID", Val::Nil),
    }
    table_set(
        state,
        t,
        "selectOptionWhenOnlyOption",
        Val::Bool(opt.select_option_when_only_option),
    );
    // rewards is always empty in the sim
    let rewards = create_table(state);
    table_set(state, t, "rewards", rewards);
    t
}

fn push_quest_table(state: &mut LuaState, row: &GossipQuestRow) -> Val {
    let t = create_table_with_capacity(state, GOSSIP_QUEST_HASH_FIELDS);
    table_set(state, t, "questID", Val::Num(row.quest_id as f64));
    table_set(state, t, "questInfoID", Val::Num(row.quest_info_id as f64));
    table_set(state, t, "questLevel", Val::Num(row.quest_level as f64));
    let title = create_string(state, &row.title);
    table_set(state, t, "title", title);
    table_set(state, t, "isImportant", Val::Bool(row.is_important));
    table_set(state, t, "isLegendary", Val::Bool(row.is_legendary));
    table_set(state, t, "isMeta", Val::Bool(row.is_meta));
    table_set(state, t, "isTrivial", Val::Bool(row.is_trivial));
    table_set(state, t, "isIgnored", Val::Bool(row.is_ignored));
    match row.frequency {
        Some(f) => table_set(state, t, "frequency", Val::Num(f as f64)),
        None => table_set(state, t, "frequency", Val::Nil),
    }
    match row.is_complete {
        Some(b) => table_set(state, t, "isComplete", Val::Bool(b)),
        None => table_set(state, t, "isComplete", Val::Nil),
    }
    match row.repeatable {
        Some(b) => table_set(state, t, "repeatable", Val::Bool(b)),
        None => table_set(state, t, "repeatable", Val::Nil),
    }
    t
}

fn push_quest_array(state: &mut LuaState, rows: &[GossipQuestRow]) -> LuaResult<u32> {
    let array = create_table(state);
    for (i, row) in rows.iter().enumerate() {
        let entry = push_quest_table(state, row);
        set_table_array(state, array, i as i64 + 1, entry);
    }
    state.push(array);
    Ok(1)
}
