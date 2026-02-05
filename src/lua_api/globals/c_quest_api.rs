//! C_Quest namespaces and quest-related API functions.
//!
//! Contains quest log, task quests, quest info, and quest line API functions.

use mlua::{Lua, Result, Value};

/// Register quest-related C_* namespaces.
pub fn register_c_quest_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    // C_QuestLog namespace - quest log utilities
    let c_quest_log = lua.create_table()?;
    c_quest_log.set(
        "IsQuestFlaggedCompleted",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_quest_log.set(
        "GetNumQuestLogEntries",
        lua.create_function(|_, ()| Ok((0i32, 0i32)))?,
    )?;
    c_quest_log.set(
        "GetInfo",
        lua.create_function(|_, _quest_index: i32| Ok(Value::Nil))?,
    )?;
    c_quest_log.set(
        "GetQuestIDForLogIndex",
        lua.create_function(|_, _quest_index: i32| Ok(0i32))?,
    )?;
    c_quest_log.set(
        "IsComplete",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_quest_log.set(
        "IsOnQuest",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_quest_log.set(
        "GetQuestObjectives",
        lua.create_function(|lua, _quest_id: i32| lua.create_table())?,
    )?;
    c_quest_log.set(
        "GetMaxNumQuestsCanAccept",
        lua.create_function(|_, ()| Ok(35i32))?,
    )?;
    c_quest_log.set(
        "GetTitleForQuestID",
        lua.create_function(|lua, _quest_id: i32| {
            Ok(Value::String(lua.create_string("Quest")?))
        })?,
    )?;
    c_quest_log.set(
        "GetQuestTagInfo",
        lua.create_function(|lua, _quest_id: i32| {
            let info = lua.create_table()?;
            info.set("tagID", 0)?;
            info.set("tagName", "Quest")?;
            info.set("worldQuestType", Value::Nil)?;
            info.set("quality", 1)?;
            info.set("isElite", false)?;
            info.set("displayExpiration", false)?;
            Ok(info)
        })?,
    )?;
    globals.set("C_QuestLog", c_quest_log)?;

    // C_TaskQuest namespace - world quest/task utilities
    let c_task_quest = lua.create_table()?;
    c_task_quest.set(
        "IsActive",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    c_task_quest.set(
        "GetQuestsOnMap",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_task_quest.set(
        "GetQuestInfoByQuestID",
        lua.create_function(|_, _quest_id: i32| Ok(Value::Nil))?,
    )?;
    c_task_quest.set(
        "GetQuestLocation",
        lua.create_function(|_, (_quest_id, _map_id): (i32, i32)| Ok((0.0f64, 0.0f64)))?,
    )?;
    c_task_quest.set(
        "GetQuestsForPlayerByMapID",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_task_quest.set(
        "GetQuestTimeLeftMinutes",
        lua.create_function(|_, _quest_id: i32| Ok(0i32))?,
    )?;
    globals.set("C_TaskQuest", c_task_quest)?;

    // C_QuestInfoSystem namespace - quest classification info
    let c_quest_info_system = lua.create_table()?;
    c_quest_info_system.set(
        "GetQuestClassification",
        lua.create_function(|_, _quest_id: i32| {
            // Returns Enum.QuestClassification value
            Ok(0) // Normal
        })?,
    )?;
    c_quest_info_system.set(
        "HasQuestRewardCurrencies",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    globals.set("C_QuestInfoSystem", c_quest_info_system)?;

    // C_QuestLine namespace - questline information
    let c_quest_line = lua.create_table()?;
    c_quest_line.set(
        "GetQuestLineInfo",
        lua.create_function(|_, (_quest_id, _map_id): (i32, Option<i32>)| Ok(Value::Nil))?,
    )?;
    c_quest_line.set(
        "GetQuestLineQuests",
        lua.create_function(|lua, _quest_line_id: i32| lua.create_table())?,
    )?;
    c_quest_line.set(
        "GetAvailableQuestLines",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    c_quest_line.set(
        "IsComplete",
        lua.create_function(|_, _quest_line_id: i32| Ok(false))?,
    )?;
    c_quest_line.set(
        "RequestQuestLinesForMap",
        lua.create_function(|_, _map_id: i32| Ok(()))?,
    )?;
    globals.set("C_QuestLine", c_quest_line)?;

    // C_QuestOffer namespace - quest offer/reward info
    let c_quest_offer = lua.create_table()?;
    c_quest_offer.set(
        "GetQuestOfferMajorFactionReputationRewards",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    globals.set("C_QuestOffer", c_quest_offer)?;

    Ok(())
}
