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
    register_achievement_stubs(lua)?;
    register_c_log(lua)?;
    register_c_campaign_info(lua)?;
    register_quest_global_functions(lua)?;
    register_chat_stubs(lua)?;
    register_c_macro(lua)?;
    register_c_wowlabs_matchmaking(lua)?;
    register_fading_frame_stubs(lua)?;
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
    t.set("GetNumMembers", lua.create_function(|_, ()| Ok(5i32))?)?;
    t.set("IsInGuild", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("GetGuildInfo", lua.create_function(|_, _unit: Option<String>| {
        Ok(("Stormwind Guard".to_string(), "Officer".to_string(), 2i32, ""))
    })?)?;
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
    t.set("CanCreateQuestGroup", lua.create_function(|_, _quest_id: i32| Ok(false))?)?;
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
    g.set("InCombatLockdown", lua.create_function(|_, ()| Ok(false))?)?;
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
    g.set("GetReleaseTimeRemaining", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("FCF_OnUpdate", lua.create_function(|_, _elapsed: Option<f64>| Ok(()))?)?;
    g.set("HelpOpenWebTicketButton_OnUpdate", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    g.set("GetLootSpecialization", lua.create_function(|_, ()| Ok(0i32))?)?;
    // UIParent PLAYER_ENTERING_WORLD handler stubs
    g.set("GetSpellConfirmationPromptsInfo", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("ResurrectGetOfferer", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    g.set("GetActiveLootRollIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("GetTutorialsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("BoostTutorial_AttemptLoad", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("ExpansionTrial_CheckLoadUI", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("SubscriptionInterstitial_LoadUI", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("ShowResurrectRequest", lua.create_function(|_, _offerer: String| Ok(()))?)?;
    g.set("GroupLootContainer_AddRoll", lua.create_function(|_, (_id, _dur): (Value, Value)| Ok(()))?)?;
    g.set("RemixArtifactTutorialUI_LoadUI", lua.create_function(|_, ()| Ok(()))?)?;
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

/// Achievement category API stubs needed by Blizzard_AchievementUI at parse time.
fn register_achievement_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("GetCategoryList", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("GetGuildCategoryList", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("GetStatisticsCategoryList", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set(
        "GetCategoryInfo",
        lua.create_function(|_, _id: Value| Ok((Value::Nil, -1i32, -1i32)))?,
    )?;
    g.set(
        "GetCategoryNumAchievements",
        lua.create_function(|_, _id: Value| Ok((0i32, 0i32, 0i32)))?,
    )?;
    g.set(
        "GetTotalAchievementPoints",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(0i32))?,
    )?;
    g.set(
        "GetLatestCompletedAchievements",
        lua.create_function(|_, _args: mlua::MultiValue| Ok(mlua::MultiValue::new()))?,
    )?;
    g.set(
        "GetAchievementInfo",
        lua.create_function(|_, _id: Value| Ok(Value::Nil))?,
    )?;
    g.set(
        "GetTrackedAchievements",
        lua.create_function(|_, ()| Ok(mlua::MultiValue::new()))?,
    )?;
    g.set(
        "GetNumCompletedAchievements",
        lua.create_function(|_, _for_guild: Option<bool>| Ok((0i32, 0i32)))?,
    )?;

    // C_Loot namespace
    let cl = lua.create_table()?;
    cl.set("GetLootRollDuration", lua.create_function(|_, _id: Value| Ok(0i32))?)?;
    g.set("C_Loot", cl)?;

    // C_ContentTracking namespace
    let ct = lua.create_table()?;
    ct.set("GetTrackedIDs", lua.create_function(|lua, _type: Value| lua.create_table())?)?;
    ct.set("IsTracking", lua.create_function(|_, (_type, _id): (Value, Value)| Ok(false))?)?;
    g.set("C_ContentTracking", ct)?;

    // C_AchievementTelemetry namespace
    let at = lua.create_table()?;
    at.set("LinkAchievementInWhisper", lua.create_function(|_, _id: Value| Ok(()))?)?;
    at.set("LinkAchievementInClub", lua.create_function(|_, _id: Value| Ok(()))?)?;
    g.set("C_AchievementTelemetry", at)?;

    Ok(())
}

fn register_c_log(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set(
        "LogMessage",
        lua.create_function(|_, _msg: Value| Ok(()))?,
    )?;
    lua.globals().set("C_Log", t)?;
    Ok(())
}

/// C_CampaignInfo namespace - campaign/war campaign data.
fn register_c_campaign_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetCampaignID", lua.create_function(|_, _quest_id: i32| Ok(0i32))?)?;
    t.set("GetCampaignInfo", lua.create_function(|_, _campaign_id: i32| Ok(Value::Nil))?)?;
    lua.globals().set("C_CampaignInfo", t)?;
    Ok(())
}

/// Quest-related global functions used by ObjectiveTracker.
fn register_quest_global_functions(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("IsInInstance", lua.create_function(|_, ()| Ok((false, "none")))?)?;
    g.set("IsQuestSequenced", lua.create_function(|_, _quest_id: i32| Ok(false))?)?;
    g.set("GetQuestLogCompletionText", lua.create_function(|_, _log_idx: i32| Ok(Value::Nil))?)?;
    g.set("GetQuestProgressBarPercent", lua.create_function(|_, _quest_id: i32| Ok(0.0f64))?)?;
    g.set("QuestMapFrame_GetFocusedQuestID", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("IsModifiedClick", lua.create_function(|_, _action: String| Ok(false))?)?;
    g.set("IsInJailersTower", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetNumAutoQuestPopUps", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("GetAutoQuestPopUp", lua.create_function(|_, _index: i32| Ok(Value::Nil))?)?;
    g.set("GetQuestLogSpecialItemInfo", lua.create_function(|_, _log_idx: i32| Ok(Value::Nil))?)?;
    register_quest_leaderboard_functions(lua, &g)?;
    Ok(())
}

/// GetNumQuestLeaderBoards / GetQuestLogLeaderBoard - quest objective data.
fn register_quest_leaderboard_functions(lua: &Lua, g: &mlua::Table) -> Result<()> {
    g.set(
        "GetNumQuestLeaderBoards",
        lua.create_function(|_, log_idx: i32| {
            Ok(match log_idx {
                1 => 2,
                2 => 1,
                3 => 2,
                _ => 0,
            })
        })?,
    )?;
    g.set(
        "GetQuestLogLeaderBoard",
        lua.create_function(|_, (obj_idx, log_idx, _suppress): (i32, i32, Option<bool>)| {
            Ok(quest_leaderboard_entry(log_idx, obj_idx))
        })?,
    )?;
    Ok(())
}

/// Chat-related global function stubs needed by Blizzard_ChatFrame.
fn register_chat_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    // GetChatTypeIndex: deterministic integer from chat type name
    g.set("GetChatTypeIndex", lua.create_function(|_, name: String| {
        let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        Ok((hash % 50 + 1) as i32)
    })?)?;
    // CreateSecureDelegate: no taint system, return the function as-is
    g.set("CreateSecureDelegate", lua.create_function(|_, func: mlua::Function| Ok(func))?)?;
    // GetChatWindowInfo: return defaults (only window 1 is shown)
    g.set("GetChatWindowInfo", lua.create_function(|_, id: i32| {
        let name = format!("ChatFrame{id}");
        let shown = id == 1;
        Ok((name, 14.0f64, 1.0f64, 1.0f64, 1.0f64, 1.0f64, shown, false, false, false))
    })?)?;
    g.set("GetDefaultLanguage", lua.create_function(|_, ()| Ok("Common"))?)?;
    g.set("GetAlternativeDefaultLanguage", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    Ok(())
}

/// C_Macro namespace - macro management stubs.
fn register_c_macro(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("SetMacroExecuteLineCallback", lua.create_function(|_, _cb: Value| Ok(()))?)?;
    t.set("GetMacroInfo", lua.create_function(|_, _id: Value| Ok(Value::Nil))?)?;
    t.set("GetNumMacros", lua.create_function(|_, ()| Ok((0i32, 0i32)))?)?;
    lua.globals().set("C_Macro", t)?;
    Ok(())
}

/// Return (text, objectiveType, finished) for a mock quest objective.
fn quest_leaderboard_entry(log_idx: i32, obj_idx: i32) -> (String, String, bool) {
    match (log_idx, obj_idx) {
        (1, 1) => ("Ironforge Relics collected: 3/5".into(), "item".into(), false),
        (1, 2) => ("Explore the Old Quarry".into(), "event".into(), false),
        (2, 1) => ("Stormwind Guards defended: 7/10".into(), "monster".into(), false),
        (3, 1) => ("Supplies gathered: 5/5".into(), "item".into(), true),
        (3, 2) => ("Deliver to Quartermaster".into(), "event".into(), false),
        _ => ("Unknown objective".into(), "event".into(), false),
    }
}

fn register_c_wowlabs_matchmaking(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetCurrentParty", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetPartyPlaylistEntry", lua.create_function(|_, ()| Ok(mlua::Value::Nil))?)?;
    t.set("ClearFastLogin", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("SetAutoQueueOnLogout", lua.create_function(|_, _flag: bool| Ok(()))?)?;
    lua.globals().set("C_WoWLabsMatchmaking", t)?;

    // C_WowLabsDataManager (note: different casing from C_WoWLabsMatchmaking)
    let dm = lua.create_table()?;
    dm.set("IsInPrematch", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_WowLabsDataManager", dm)?;
    Ok(())
}

/// FadingFrame_* global functions used by ZoneText.lua.
fn register_fading_frame_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    // FadingFrame_OnLoad initializes fading state on the frame
    g.set("FadingFrame_OnLoad", lua.create_function(|_, frame: mlua::Table| {
        frame.set("fadeInTime", 0.0f64)?;
        frame.set("fadeOutTime", 0.0f64)?;
        frame.set("holdTime", 0.0f64)?;
        Ok(())
    })?)?;
    g.set("FadingFrame_SetFadeInTime", lua.create_function(|_, (_frame, _t): (Value, f64)| Ok(()))?)?;
    g.set("FadingFrame_SetHoldTime", lua.create_function(|_, (_frame, _t): (Value, f64)| Ok(()))?)?;
    g.set("FadingFrame_SetFadeOutTime", lua.create_function(|_, (_frame, _t): (Value, f64)| Ok(()))?)?;
    g.set("FadingFrame_Show", lua.create_function(|_, _frame: Value| Ok(()))?)?;
    g.set("GetErrorCallstackHeight", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("SetChatWindowShown", lua.create_function(|_, (_id, _shown): (Value, Value)| Ok(()))?)?;
    Ok(())
}
