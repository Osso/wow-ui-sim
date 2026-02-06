//! C_Quest namespaces and quest-related API functions.
//!
//! Contains quest log, task quests, quest info, and quest line API functions.

use mlua::{Lua, Result, Value};

/// Register quest-related C_* namespaces.
pub fn register_c_quest_api(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("C_QuestLog", register_c_quest_log(lua)?)?;
    globals.set("C_TaskQuest", register_c_task_quest(lua)?)?;
    globals.set("C_QuestInfoSystem", register_c_quest_info_system(lua)?)?;
    globals.set("C_QuestLine", register_c_quest_line(lua)?)?;
    globals.set("C_QuestOffer", register_c_quest_offer(lua)?)?;
    Ok(())
}

/// C_QuestLog namespace - quest log utilities.
fn register_c_quest_log(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    register_quest_log_queries(lua, &t)?;
    register_quest_log_info(lua, &t)?;
    Ok(t)
}

/// Quest log query methods (status checks, objectives, counts).
fn register_quest_log_queries(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("IsQuestFlaggedCompleted", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("GetNumQuestLogEntries", lua.create_function(|_, ()| Ok((0i32, 0i32)))?)?;
    t.set("GetInfo", lua.create_function(|_, _idx: i32| Ok(Value::Nil))?)?;
    t.set("GetQuestIDForLogIndex", lua.create_function(|_, _idx: i32| Ok(0i32))?)?;
    t.set("IsComplete", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("IsOnQuest", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("GetQuestObjectives", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetMaxNumQuestsCanAccept", lua.create_function(|_, ()| Ok(35i32))?)?;
    t.set("ReadyForTurnIn", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("GetQuestAdditionalHighlights", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("SetMapForQuestPOIs", lua.create_function(|_, _map_id: i32| Ok(()))?)?;
    t.set("GetZoneStoryInfo", lua.create_function(|_, _map_id: i32| Ok((Value::Nil, Value::Nil)))?)?;
    Ok(())
}

/// Quest log info methods (titles, tags).
fn register_quest_log_info(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetTitleForQuestID", lua.create_function(|lua, _id: i32| {
        Ok(Value::String(lua.create_string("Quest")?))
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
    Ok(())
}

/// C_TaskQuest namespace - world quest/task utilities.
fn register_c_task_quest(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set(
        "IsActive",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    t.set(
        "GetQuestsOnMap",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    t.set(
        "GetQuestInfoByQuestID",
        lua.create_function(|_, _quest_id: i32| Ok(Value::Nil))?,
    )?;
    t.set(
        "GetQuestLocation",
        lua.create_function(|_, (_quest_id, _map_id): (i32, i32)| Ok((0.0f64, 0.0f64)))?,
    )?;
    t.set(
        "GetQuestsForPlayerByMapID",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    t.set(
        "GetQuestTimeLeftMinutes",
        lua.create_function(|_, _quest_id: i32| Ok(0i32))?,
    )?;
    Ok(t)
}

/// C_QuestInfoSystem namespace - quest classification info.
fn register_c_quest_info_system(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set(
        "GetQuestClassification",
        lua.create_function(|_, _quest_id: i32| {
            // Returns Enum.QuestClassification value
            Ok(0) // Normal
        })?,
    )?;
    t.set(
        "HasQuestRewardCurrencies",
        lua.create_function(|_, _quest_id: i32| Ok(false))?,
    )?;
    Ok(t)
}

/// C_QuestLine namespace - questline information.
fn register_c_quest_line(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set(
        "GetQuestLineInfo",
        lua.create_function(|_, (_quest_id, _map_id): (i32, Option<i32>)| Ok(Value::Nil))?,
    )?;
    t.set(
        "GetQuestLineQuests",
        lua.create_function(|lua, _quest_line_id: i32| lua.create_table())?,
    )?;
    t.set(
        "GetAvailableQuestLines",
        lua.create_function(|lua, _map_id: i32| lua.create_table())?,
    )?;
    t.set(
        "IsComplete",
        lua.create_function(|_, _quest_line_id: i32| Ok(false))?,
    )?;
    t.set(
        "RequestQuestLinesForMap",
        lua.create_function(|_, _map_id: i32| Ok(()))?,
    )?;
    Ok(t)
}

/// C_QuestOffer namespace - quest offer/reward info.
fn register_c_quest_offer(lua: &Lua) -> Result<mlua::Table> {
    let t = lua.create_table()?;
    t.set(
        "GetQuestOfferMajorFactionReputationRewards",
        lua.create_function(|lua, ()| lua.create_table())?,
    )?;
    Ok(t)
}
