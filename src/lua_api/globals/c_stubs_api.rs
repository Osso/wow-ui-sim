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
    register_unit_frame_global_stubs(lua)?;
    register_powerbar_prediction_colors(lua)?;
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
    t.set("EnableActionRangeCheck", lua.create_function(|_, (_action, _enable): (Value, bool)| Ok(()))?)?;
    t.set("IsAssistedCombatAction", lua.create_function(|_, _action: Value| Ok(false))?)?;
    lua.globals().set("C_ActionBar", t)?;
    Ok(())
}

/// Global function stubs needed by Blizzard_UnitFrame.
fn register_unit_frame_global_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("IsResting", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("IsPVPTimerRunning", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetPVPTimer", lua.create_function(|_, ()| Ok(0.0f64))?)?;
    g.set("GetReadyCheckStatus", lua.create_function(|_, _unit: String| Ok(Value::Nil))?)?;
    g.set("HasLFGRestrictions", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("IsInRaid", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetNumGroupMembers", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("GetRaidRosterInfo", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    g.set("PartialPlayTime", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("NoPlayTime", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetBillingTimeRested", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("SetPortraitToTexture", lua.create_function(|_, (_tex, _path): (Value, Value)| Ok(()))?)?;
    g.set("GetUnitTotalModifiedMaxHealthPercent", lua.create_function(|_, _unit: String| Ok(0.0f64))?)?;
    g.set("IsThreatWarningEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetThreatStatusColor", lua.create_function(|_, _status: i32| Ok((1.0f64, 1.0f64, 1.0f64)))?)?;
    g.set("LE_REALM_RELATION_VIRTUAL", 3i32)?;
    g.set("PlaySound", lua.create_function(|_, (_id, _ch): (Value, Option<String>)| Ok(()))?)?;
    g.set("PlaySoundFile", lua.create_function(|_, (_path, _ch): (Value, Option<String>)| Ok(()))?)?;
    g.set("StopSound", lua.create_function(|_, _handle: Value| Ok(()))?)?;
    g.set("IsInGroup", lua.create_function(|_, _category: Option<i32>| Ok(false))?)?;
    g.set("IsActiveBattlefieldArena", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetNumArenaOpponents", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("GetBattlefieldEstimatedWaitTime", lua.create_function(|_, _index: Value| Ok(0i32))?)?;
    g.set("PetUsesPetFrame", lua.create_function(|_, ()| Ok(true))?)?;
    g.set("UnitIsPossessed", lua.create_function(|_, _unit: String| Ok(false))?)?;
    Ok(())
}

/// POWERBAR_PREDICTION_COLOR_* globals used by PowerBarColorUtil.lua at parse time.
fn register_powerbar_prediction_colors(lua: &Lua) -> Result<()> {
    let get_rgba = lua.create_function(|_, this: mlua::Table| {
        Ok((
            this.get::<f64>("r")?,
            this.get::<f64>("g")?,
            this.get::<f64>("b")?,
            this.get::<f64>("a")?,
        ))
    })?;
    let get_rgb = lua.create_function(|_, this: mlua::Table| {
        Ok((
            this.get::<f64>("r")?,
            this.get::<f64>("g")?,
            this.get::<f64>("b")?,
        ))
    })?;

    let g = lua.globals();
    let colors: &[(&str, f64, f64, f64)] = &[
        ("POWERBAR_PREDICTION_COLOR_MANA", 0.0, 0.0, 1.0),
        ("POWERBAR_PREDICTION_COLOR_RAGE", 1.0, 0.0, 0.0),
        ("POWERBAR_PREDICTION_COLOR_FOCUS", 1.0, 0.5, 0.25),
        ("POWERBAR_PREDICTION_COLOR_ENERGY", 1.0, 1.0, 0.0),
        ("POWERBAR_PREDICTION_COLOR_RUNIC_POWER", 0.0, 0.82, 1.0),
        ("POWERBAR_PREDICTION_COLOR_LUNAR_POWER", 0.3, 0.52, 0.9),
        ("POWERBAR_PREDICTION_COLOR_MAELSTROM", 0.0, 0.5, 1.0),
        ("POWERBAR_PREDICTION_COLOR_INSANITY", 0.4, 0.0, 0.8),
        ("POWERBAR_PREDICTION_COLOR_FURY", 0.788, 0.259, 0.992),
        ("POWERBAR_PREDICTION_COLOR_PAIN", 1.0, 0.612, 0.0),
    ];
    for &(name, r, green, b) in colors {
        let t = lua.create_table()?;
        t.set("r", r)?;
        t.set("g", green)?;
        t.set("b", b)?;
        t.set("a", 0.5f64)?;
        t.set("GetRGBA", get_rgba.clone())?;
        t.set("GetRGB", get_rgb.clone())?;
        g.set(name, t)?;
    }
    Ok(())
}
