//! C_Quest namespaces and quest-related API functions.
//!
//! Contains quest log, task quests, quest info, and quest line API functions.
//! Mock quest data provides 3 watched quests for the ObjectiveTracker.

use mlua::{Lua, Result, Value};

/// Mock quest definitions: (questID, logIndex, title).
const MOCK_QUESTS: &[(i32, i32, &str)] = &[
    (80000, 1, "The Lost Expedition"),
    (80001, 2, "Defending the Gates"),
    (80002, 3, "Supply Run"),
];

/// Register quest-related C_* namespaces.
pub fn register_c_quest_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("C_QuestLog", register_c_quest_log(lua)?)?;
    globals.set("C_TaskQuest", register_c_task_quest(lua)?)?;
    globals.set("C_QuestInfoSystem", register_c_quest_info_system(lua)?)?;
    globals.set("C_QuestLine", register_c_quest_line(lua)?)?;
    globals.set("C_QuestOffer", register_c_quest_offer(lua)?)?;
    globals.set("C_QuestSession", register_c_quest_session(lua)?)?;
    Ok(())
}

/// C_QuestLog namespace - quest log utilities.
fn register_c_quest_log(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    register_quest_log_queries(lua, &t)?;
    register_quest_log_info(lua, &t)?;
    register_quest_log_requests(lua, &t)?;
    register_quest_log_watch(lua, &t)?;
    register_quest_log_status(lua, &t)?;
    t.set("HasActiveThreats", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetBountySetInfoForMapID", lua.create_function(|_, _map_id: i32| Ok(Value::Nil))?)?;
    t.set("IsUnitOnQuest", lua.create_function(|_, (_unit, _quest_id): (String, i32)| Ok(false))?)?;
    Ok(t)
}

/// Quest log query methods (counts, GetInfo, objectives).
fn register_quest_log_queries(lua: &Lua, t: &mlua::Table) -> Result<()> {
    let num_quests = MOCK_QUESTS.len() as i32;
    t.set("GetNumQuestLogEntries", lua.create_function(move |_, ()| Ok((num_quests, 0i32)))?)?;
    t.set("GetInfo", lua.create_function(|lua, idx: i32| create_quest_info(lua, idx))?)?;
    t.set("GetQuestIDForLogIndex", lua.create_function(|_, idx: i32| {
        Ok(MOCK_QUESTS.iter().find(|q| q.1 == idx).map_or(0, |q| q.0))
    })?)?;
    t.set("GetLogIndexForQuestID", lua.create_function(|_, quest_id: i32| {
        Ok(MOCK_QUESTS.iter().find(|q| q.0 == quest_id).map(|q| q.1))
    })?)?;
    t.set("GetQuestObjectives", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetMaxNumQuestsCanAccept", lua.create_function(|_, ()| Ok(35i32))?)?;
    t.set("GetMaxNumQuests", lua.create_function(|_, ()| Ok(35i32))?)?;
    t.set("SetMapForQuestPOIs", lua.create_function(|_, _map_id: i32| Ok(()))?)?;
    t.set("GetZoneStoryInfo", lua.create_function(|_, _map_id: i32| Ok((Value::Nil, Value::Nil)))?)?;
    t.set("GetQuestAdditionalHighlights", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("IsQuestReplayable", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("GetQuestWatchType", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    Ok(())
}

/// Create a quest info table for a given log index.
fn create_quest_info(lua: &Lua, idx: i32) -> Result<Value> {
    let quest = MOCK_QUESTS.iter().find(|q| q.1 == idx);
    let Some(&(quest_id, _, title)) = quest else {
        return Ok(Value::Nil);
    };
    let info = lua.create_table()?;
    info.set("title", title)?;
    info.set("questLogIndex", idx)?;
    info.set("questID", quest_id)?;
    info.set("campaignID", 0)?;
    info.set("level", 70)?;
    info.set("difficultyLevel", 70)?;
    info.set("suggestedGroup", 0)?;
    info.set("isHeader", false)?;
    info.set("isCollapsed", false)?;
    info.set("isTask", false)?;
    info.set("isBounty", false)?;
    info.set("isStory", false)?;
    info.set("isOnMap", true)?;
    info.set("hasLocalPOI", false)?;
    info.set("isHidden", false)?;
    info.set("isAutoComplete", false)?;
    info.set("overridesSortOrder", false)?;
    info.set("startEvent", false)?;
    info.set("isScaling", false)?;
    info.set("readyForTranslation", false)?;
    Ok(Value::Table(info))
}

/// Quest data request stubs (async data loading).
/// In WoW, these trigger server requests. We stub them as no-ops.
fn register_quest_log_requests(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("RequestLoadQuestByID", lua.create_function(|_, _id: i32| Ok(()))?)?;
    Ok(())
}

/// Quest log info methods (titles, tags).
fn register_quest_log_info(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetTitleForQuestID", lua.create_function(|lua, id: i32| {
        let title = MOCK_QUESTS.iter().find(|q| q.0 == id).map_or("Quest", |q| q.2);
        Ok(Value::String(lua.create_string(title)?))
    })?)?;
    t.set("GetQuestTagInfo", lua.create_function(|lua, _id: i32| {
        let info = lua.create_table()?;
        info.set("tagID", 0)?;
        info.set("tagName", "Quest")?;
        info.set("worldQuestType", Value::Nil)?;
        info.set("quality", 1)?;
        info.set("isElite", false)?;
        info.set("displayExpiration", false)?;
        Ok(info)
    })?)?;
    t.set("GetRequiredMoney", lua.create_function(|_, _id: i32| Ok(0i32))?)?;
    Ok(())
}

/// Quest watch list methods (tracked quests for ObjectiveTracker).
fn register_quest_log_watch(lua: &Lua, t: &mlua::Table) -> Result<()> {
    let num_watches = MOCK_QUESTS.len() as i32;
    t.set("GetNumQuestWatches", lua.create_function(move |_, ()| Ok(num_watches))?)?;
    t.set("GetQuestIDForQuestWatchIndex", lua.create_function(|_, idx: i32| {
        Ok(MOCK_QUESTS.get((idx - 1) as usize).map(|q| q.0))
    })?)?;
    t.set("AddQuestWatch", lua.create_function(|_, _id: i32| Ok(()))?)?;
    t.set("RemoveQuestWatch", lua.create_function(|_, _id: i32| Ok(()))?)?;
    t.set("SortQuestWatches", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("GetNumWorldQuestWatches", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetQuestIDForWorldQuestWatchIndex", lua.create_function(|_, _idx: i32| Ok(Value::Nil))?)?;
    Ok(())
}

/// Quest status check methods.
fn register_quest_log_status(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("IsQuestFlaggedCompleted", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsComplete", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsOnQuest", lua.create_function(|_, id: i32| {
        Ok(MOCK_QUESTS.iter().any(|q| q.0 == id))
    })?)?;
    t.set("ReadyForTurnIn", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsFailed", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsPushableQuest", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsQuestDisabledForSession", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsRepeatableQuest", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsImportantQuest", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsMetaQuest", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsOnMap", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("GetNextWaypointText", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetTimeAllowed", lua.create_function(|_, _id: i32| Ok((Value::Nil, Value::Nil)))?)?;
    Ok(())
}

/// C_TaskQuest namespace - world quest/task utilities.
fn register_c_task_quest(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("IsActive", lua.create_function(|_, _quest_id: i32| Ok(false))?)?;
    t.set("GetQuestsOnMap", lua.create_function(|lua, _map_id: i32| lua.create_table())?)?;
    t.set("GetQuestInfoByQuestID", lua.create_function(|_, _quest_id: i32| Ok(Value::Nil))?)?;
    t.set("GetQuestLocation", lua.create_function(|_, (_qid, _mid): (i32, i32)| Ok((0.0f64, 0.0f64)))?)?;
    t.set("GetQuestsForPlayerByMapID", lua.create_function(|lua, _map_id: i32| lua.create_table())?)?;
    t.set("GetQuestTimeLeftMinutes", lua.create_function(|_, _quest_id: i32| Ok(0i32))?)?;
    Ok(t)
}

/// C_QuestInfoSystem namespace - quest classification info.
fn register_c_quest_info_system(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    // Returns Enum.QuestClassification.Normal (0)
    t.set("GetQuestClassification", lua.create_function(|_, _quest_id: i32| Ok(0i32))?)?;
    t.set("HasQuestRewardCurrencies", lua.create_function(|_, _quest_id: i32| Ok(false))?)?;
    Ok(t)
}

/// C_QuestLine namespace - questline information.
fn register_c_quest_line(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("GetQuestLineInfo", lua.create_function(|_, (_qid, _mid): (i32, Option<i32>)| Ok(Value::Nil))?)?;
    t.set("GetQuestLineQuests", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetAvailableQuestLines", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("IsComplete", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("RequestQuestLinesForMap", lua.create_function(|_, _id: i32| Ok(()))?)?;
    Ok(t)
}

/// C_QuestSession namespace - quest session/party sync system.
fn register_c_quest_session(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("Exists", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("HasJoined", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetAvailableSessionCommand", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("HasPendingCommand", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetPendingCommand", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetSessionBeginDetails", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    Ok(t)
}

/// C_QuestOffer namespace - quest offer/reward info.
fn register_c_quest_offer(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set("GetQuestOfferMajorFactionReputationRewards", lua.create_function(|lua, ()| lua.create_table())?)?;
    Ok(t)
}
