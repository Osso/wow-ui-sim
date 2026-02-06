//! Additional C_* namespace stubs.
//!
//! Contains stub implementations for C_* namespaces that are referenced by
//! Blizzard UI code but don't need real logic in the simulator:
//! - C_AchievementInfo - Achievement data
//! - C_ClassTalents - Class talent configuration
//! - C_Guild - Guild membership
//! - C_GuildInfo - Guild display info
//! - C_LFGList - Looking for Group listings
//! - C_LossOfControl - Loss of control effects
//! - C_Mail - Mailbox system
//! - C_StableInfo - Hunter pet stables
//! - C_Tutorial - Tutorial flags
//! - C_ActionBar - Action bar queries

use mlua::{Lua, Result, Value};

/// Register all additional C_* namespace stubs.
pub fn register_c_stubs_api(lua: &Lua) -> Result<()> {
    register_c_achievement_info(lua)?;
    register_c_class_talents(lua)?;
    register_c_guild(lua)?;
    register_c_guild_info(lua)?;
    register_c_lfg_list(lua)?;
    register_c_loss_of_control(lua)?;
    register_c_mail(lua)?;
    register_c_stable_info(lua)?;
    register_c_tutorial(lua)?;
    register_c_action_bar(lua)?;
    Ok(())
}

fn register_c_achievement_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetRewardItemID", lua.create_function(|_, _achievement_id: i32| Ok(Value::Nil))?)?;
    t.set("GetAchievementInfo", lua.create_function(|_, _achievement_id: i32| Ok(Value::Nil))?)?;
    lua.globals().set("C_AchievementInfo", t)?;
    Ok(())
}

fn register_c_class_talents(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetActiveConfigID", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetConfigIDsBySpecID", lua.create_function(|lua, _spec_id: Option<i32>| lua.create_table())?)?;
    t.set("GetStarterBuildActive", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetHasStarterBuild", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_ClassTalents", t)?;
    Ok(())
}

fn register_c_guild(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNumMembers", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("IsInGuild", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetGuildInfo", lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?)?;
    t.set("GetMemberInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    lua.globals().set("C_Guild", t)?;
    Ok(())
}

fn register_c_guild_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetGuildTabardInfo", lua.create_function(|_, _unit: Option<String>| Ok(Value::Nil))?)?;
    t.set("GetGuildNewsInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    lua.globals().set("C_GuildInfo", t)?;
    Ok(())
}

fn register_c_lfg_list(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetActiveEntryInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("HasActiveEntryInfo", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetSearchResultInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    lua.globals().set("C_LFGList", t)?;
    Ok(())
}

fn register_c_loss_of_control(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetActiveLossOfControlData", lua.create_function(|_, _index: Option<i32>| Ok(Value::Nil))?)?;
    t.set("GetActiveLossOfControlDataCount", lua.create_function(|_, ()| Ok(0i32))?)?;
    lua.globals().set("C_LossOfControl", t)?;
    Ok(())
}

fn register_c_mail(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNumItems", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("HasNewMail", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsCommandPending", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_Mail", t)?;
    Ok(())
}

fn register_c_stable_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNumStablePets", lua.create_function(|_, ()| Ok(0i32))?)?;
    lua.globals().set("C_StableInfo", t)?;
    Ok(())
}

fn register_c_tutorial(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetTutorialStatus", lua.create_function(|_, _tutorial_id: Option<i32>| Ok(false))?)?;
    t.set("SetTutorialFlag", lua.create_function(|_, _tutorial_id: i32| Ok(()))?)?;
    lua.globals().set("C_Tutorial", t)?;
    Ok(())
}

fn register_c_action_bar(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetBonusBarIndexForSlot", lua.create_function(|_, _slot: i32| Ok(0i32))?)?;
    t.set("IsOnBarOrSpecialBar", lua.create_function(|_, _slot: i32| Ok(false))?)?;
    t.set("FindSpellActionButtons", lua.create_function(|lua, _spell_id: i32| lua.create_table())?)?;
    t.set("GetCurrentActionBarByClass", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("HasFlyoutActionButtons", lua.create_function(|_, _flyout_id: i32| Ok(false))?)?;
    lua.globals().set("C_ActionBar", t)?;
    Ok(())
}
