//! UI-related C_* namespace API stubs.
//!
//! Contains C_ namespaces for UI systems:
//! - C_VignetteInfo, C_AreaPoiInfo, C_PlayerChoice, C_MajorFactions
//! - C_UIWidgetManager, C_GossipInfo, C_Calendar, C_CovenantCallings
//! - C_WeeklyRewards, C_ContributionCollector, C_Scenario, C_Housing
//! - C_GameRules, C_ScriptedAnimations, C_Glue, C_UIColor, C_ClassColor
//! - C_SpecializationInfo, C_ArtifactUI, C_SuperTrack
//! - C_PlayerInteractionManager, C_PaperDollInfo, C_PerksProgram

use mlua::{Lua, Result, Value};

pub(super) fn register_all(lua: &Lua) -> Result<()> {
    register_c_vignette_info(lua)?;
    register_c_area_poi(lua)?;
    register_c_player_choice(lua)?;
    register_c_major_factions(lua)?;
    register_c_ui_widget(lua)?;
    register_c_gossip_info(lua)?;
    register_c_calendar(lua)?;
    register_c_covenant_callings(lua)?;
    register_c_weekly_rewards(lua)?;
    register_c_contribution_collector(lua)?;
    register_c_scenario(lua)?;
    register_c_housing(lua)?;
    register_c_game_rules(lua)?;
    register_c_scripted_animations(lua)?;
    register_c_glue(lua)?;
    register_c_ui_color(lua)?;
    register_c_class_color(lua)?;
    register_c_spec_info(lua)?;
    register_c_artifact_ui(lua)?;
    register_c_super_track(lua)?;
    register_c_player_interaction_manager(lua)?;
    register_c_paper_doll_info(lua)?;
    register_c_perks_program(lua)?;
    Ok(())
}

fn register_c_vignette_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetVignettes", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetVignetteInfo", lua.create_function(|_, _g: String| Ok(Value::Nil))?)?;
    t.set("GetVignettePosition", lua.create_function(|_, (_g, _m): (String, Option<i32>)| Ok(Value::Nil))?)?;
    t.set("GetVignetteGUID", lua.create_function(|_, _g: String| Ok(Value::Nil))?)?;
    lua.globals().set("C_VignetteInfo", t)?;
    Ok(())
}

fn register_c_area_poi(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetAreaPOIInfo", lua.create_function(|_, (_m, _id): (i32, i32)| Ok(Value::Nil))?)?;
    t.set("GetAreaPOISecondsLeft", lua.create_function(|_, _id: i32| Ok(0i32))?)?;
    t.set("IsAreaPOITimed", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("GetAreaPOIForMap", lua.create_function(|lua, _m: i32| lua.create_table())?)?;
    lua.globals().set("C_AreaPoiInfo", t)?;
    Ok(())
}

fn register_c_player_choice(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetCurrentPlayerChoiceInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetNumPlayerChoices", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetPlayerChoiceInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetPlayerChoiceOptionInfo", lua.create_function(|_, (_c, _o): (i32, i32)| Ok(Value::Nil))?)?;
    t.set("SendPlayerChoiceResponse", lua.create_function(|_, _id: i32| Ok(()))?)?;
    t.set("IsWaitingForPlayerChoiceResponse", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_PlayerChoice", t)?;
    Ok(())
}

fn register_c_major_factions(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetMajorFactionData", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetMajorFactionIDs", lua.create_function(|lua, _e: Option<i32>| lua.create_table())?)?;
    t.set("GetRenownLevels", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetCurrentRenownLevel", lua.create_function(|_, _id: i32| Ok(0i32))?)?;
    t.set("HasMaximumRenown", lua.create_function(|_, _id: i32| Ok(false))?)?;
    t.set("GetRenownRewardsForLevel", lua.create_function(|lua, (_f, _l): (i32, i32)| lua.create_table())?)?;
    lua.globals().set("C_MajorFactions", t)?;
    Ok(())
}

fn register_c_ui_widget(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetAllWidgetsBySetID", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetStatusBarWidgetVisualizationInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetTextWithStateWidgetVisualizationInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetIconAndTextWidgetVisualizationInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetCaptureBarWidgetVisualizationInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetDoubleStatusBarWidgetVisualizationInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetSpellDisplayVisualizationInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetWidgetSetInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetTopCenterWidgetSetID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetBelowMinimapWidgetSetID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetObjectiveTrackerWidgetSetID", lua.create_function(|_, ()| Ok(0i32))?)?;
    lua.globals().set("C_UIWidgetManager", t)?;
    Ok(())
}

fn register_c_gossip_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetNumOptions", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetOptions", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetText", lua.create_function(|_, ()| Ok(""))?)?;
    t.set("SelectOption", lua.create_function(|_, (_id, _t, _c): (i32, Option<String>, Option<bool>)| Ok(()))?)?;
    t.set("CloseGossip", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("GetNumActiveQuests", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetNumAvailableQuests", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetActiveQuests", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetAvailableQuests", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("SelectActiveQuest", lua.create_function(|_, _i: i32| Ok(()))?)?;
    t.set("SelectAvailableQuest", lua.create_function(|_, _i: i32| Ok(()))?)?;
    t.set("GetFriendshipReputation", lua.create_function(|lua, _fid: Option<i32>| {
        let info = lua.create_table()?;
        info.set("friendshipFactionID", 0)?;
        info.set("standing", 0)?;
        info.set("maxRep", 0)?;
        info.set("name", Value::Nil)?;
        info.set("text", Value::Nil)?;
        info.set("texture", Value::Nil)?;
        info.set("reaction", Value::Nil)?;
        info.set("reactionThreshold", 0)?;
        info.set("nextThreshold", Value::Nil)?;
        Ok(info)
    })?)?;
    t.set("GetFriendshipReputationRanks", lua.create_function(|lua, _fid: Option<i32>| {
        let info = lua.create_table()?;
        info.set("currentLevel", 0)?;
        info.set("maxLevel", 0)?;
        Ok(info)
    })?)?;
    t.set("ForceGossip", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_GossipInfo", t)?;
    Ok(())
}

fn register_c_calendar(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetDate", lua.create_function(|_, ()| Ok((1i32, 1i32, 1i32, 2024i32)))?)?;
    t.set("GetMonthInfo", lua.create_function(|lua, _o: Option<i32>| {
        let info = lua.create_table()?;
        info.set("month", 1)?;
        info.set("year", 2024)?;
        info.set("numDays", 31)?;
        info.set("firstWeekday", 1)?;
        Ok(info)
    })?)?;
    t.set("GetNumDayEvents", lua.create_function(|_, (_o, _d): (i32, i32)| Ok(0i32))?)?;
    t.set("GetDayEvent", lua.create_function(|_, (_o, _d, _i): (i32, i32, i32)| Ok(Value::Nil))?)?;
    t.set("OpenCalendar", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("CloseCalendar", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("SetMonth", lua.create_function(|_, _o: i32| Ok(()))?)?;
    t.set("SetAbsMonth", lua.create_function(|_, (_m, _y): (i32, i32)| Ok(()))?)?;
    t.set("GetMinDate", lua.create_function(|_, ()| Ok((1i32, 1i32, 2004i32)))?)?;
    t.set("GetMaxDate", lua.create_function(|_, ()| Ok((12i32, 31i32, 2030i32)))?)?;
    t.set("GetNumPendingInvites", lua.create_function(|_, ()| Ok(0i32))?)?;
    lua.globals().set("C_Calendar", t)?;
    Ok(())
}

fn register_c_covenant_callings(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("AreCallingsUnlocked", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("RequestCallings", lua.create_function(|_, ()| Ok(()))?)?;
    lua.globals().set("C_CovenantCallings", t)?;
    Ok(())
}

fn register_c_weekly_rewards(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("HasAvailableRewards", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("CanClaimRewards", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetActivities", lua.create_function(|lua, _t: Option<i32>| lua.create_table())?)?;
    t.set("GetNumCompletedDungeonRuns", lua.create_function(|_, ()| Ok(0i32))?)?;
    lua.globals().set("C_WeeklyRewards", t)?;
    Ok(())
}

fn register_c_contribution_collector(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetState", lua.create_function(|_, _id: i32| Ok(0i32))?)?;
    t.set("GetContributionCollector", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("GetManagedContributionsForCreatureID", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetContributionResult", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    t.set("IsAwaitingRewardQuestData", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_ContributionCollector", t)?;
    Ok(())
}

fn register_c_scenario(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetInfo", lua.create_function(|_, ()| Ok((Value::Nil, 0i32, 0i32, 0i32, false, false)))?)?;
    t.set("GetStepInfo", lua.create_function(|_, _s: Option<i32>| Ok((Value::Nil, Value::Nil, 0i32, false, false)))?)?;
    t.set("GetCriteriaInfo", lua.create_function(|_, _i: i32| Ok(Value::Nil))?)?;
    t.set("IsInScenario", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_Scenario", t)?;
    Ok(())
}

fn register_c_housing(lua: &Lua) -> Result<()> {
    let g = lua.globals();

    let t = lua.create_table()?;
    t.set("IsHoveringDecor", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetHoveredDecorInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetDecorDyeSlots", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    g.set("C_HousingCustomizeMode", t)?;

    let t = lua.create_table()?;
    t.set("GetDyeColorInfo", lua.create_function(|lua, _id: i32| {
        let info = lua.create_table()?;
        info.set("name", "Dye")?;
        info.set("dyeColorID", 0)?;
        info.set("baseColor", 0xFFFFFFu32)?;
        info.set("highlightColor", 0xFFFFFFu32)?;
        info.set("shadowColor", 0x000000u32)?;
        Ok(info)
    })?)?;
    g.set("C_DyeColor", t)?;

    let t = lua.create_table()?;
    t.set("IsHouseEditorActive", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetActiveHouseEditorMode", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("ActivateHouseEditorMode", lua.create_function(|_, _m: i32| Ok(()))?)?;
    t.set("GetHouseEditorModeAvailability", lua.create_function(|_, _m: i32| Ok(false))?)?;
    t.set("IsHouseEditorModeActive", lua.create_function(|_, _m: i32| Ok(false))?)?;
    g.set("C_HouseEditor", t)?;

    let t = lua.create_table()?;
    t.set("GetHoveredDecorInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("IsHoveringDecor", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetDecorInfo", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    g.set("C_HousingDecor", t)?;

    let t = lua.create_table()?;
    t.set("GetTrackedHouseGuid", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("IsInsideHouse", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsInsideHouseOrPlot", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsHousingServiceEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetPlayerOwnedHouses", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("C_Housing", t)?;

    let t = lua.create_table()?;
    t.set("IsDecorSelected", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetSelectedDecorInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    g.set("C_HousingBasicMode", t)?;

    Ok(())
}

fn register_c_game_rules(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsGameRuleActive", lua.create_function(|_, _r: Value| Ok(false))?)?;
    t.set("GetActiveGameMode", lua.create_function(|_, ()| Ok(0))?)?;
    t.set("GetGameRuleAsFloat", lua.create_function(|_, _r: Value| Ok(0.0f32))?)?;
    t.set("IsStandard", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("IsWoWHack", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_GameRules", t)?;
    Ok(())
}

fn register_c_scripted_animations(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetAllScriptedAnimationEffects", lua.create_function(|lua, ()| lua.create_table())?)?;
    lua.globals().set("C_ScriptedAnimations", t)?;
    Ok(())
}

fn register_c_glue(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsOnGlueScreen", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_Glue", t)?;
    Ok(())
}

fn register_c_ui_color(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetColors", lua.create_function(|lua, ()| lua.create_table())?)?;
    lua.globals().set("C_UIColor", t)?;
    Ok(())
}

fn register_c_class_color(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetClassColor", lua.create_function(|lua, _class: String| {
        let (r, g, b, a) = (1.0f32, 1.0f32, 1.0f32, 1.0f32);
        let color = lua.create_table()?;
        color.set("r", r)?;
        color.set("g", g)?;
        color.set("b", b)?;
        color.set("a", a)?;
        color.set("GetRGB", lua.create_function(move |_, ()| Ok((r, g, b)))?)?;
        color.set("GetRGBA", lua.create_function(move |_, ()| Ok((r, g, b, a)))?)?;
        color.set("GenerateHexColor", lua.create_function(move |lua, ()| {
            let hex = format!("{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
            Ok(Value::String(lua.create_string(&hex)?))
        })?)?;
        color.set("WrapTextInColorCode", lua.create_function(move |lua, (_s, text): (Value, String)| {
            let hex = format!("{:02x}{:02x}{:02x}", (r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
            let wrapped = format!("|cff{}{}|r", hex, text);
            Ok(Value::String(lua.create_string(&wrapped)?))
        })?)?;
        Ok(color)
    })?)?;
    lua.globals().set("C_ClassColor", t)?;
    Ok(())
}

fn register_c_spec_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetSpellsDisplay", lua.create_function(|lua, _id: i32| lua.create_table())?)?;
    t.set("GetInspectSelectedSpecialization", lua.create_function(|_, _u: Option<String>| Ok(0))?)?;
    t.set("CanPlayerUseTalentSpecUI", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("CanPlayerUseTalentUI", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("IsInitialized", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("GetSpecialization", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("GetSpecializationInfo", lua.create_function(|lua, idx: i32| {
        let spec_id = match idx { 1 => 71, 2 => 72, 3 => 73, _ => 71 };
        Ok(mlua::MultiValue::from_vec(vec![
            Value::Integer(spec_id),
            Value::String(lua.create_string("Arms")?),
            Value::String(lua.create_string("A battle-hardened master of weapons.")?),
            Value::Integer(132355),
            Value::String(lua.create_string("DAMAGER")?),
            Value::Integer(1),
        ]))
    })?)?;
    t.set("GetAllSelectedPvpTalentIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetPvpTalentSlotInfo", lua.create_function(|_, _s: i32| Ok(Value::Nil))?)?;
    t.set("GetNumSpecializationsForClassID", lua.create_function(|_, (_class_id, _sex): (Option<i32>, Option<i32>)| {
        Ok(_class_id.map_or(0, |_| 3i32))
    })?)?;
    lua.globals().set("C_SpecializationInfo", t)?;
    Ok(())
}

fn register_c_artifact_ui(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetArtifactItemID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetArtifactTier", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("IsAtForge", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetEquippedArtifactInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    lua.globals().set("C_ArtifactUI", t)?;
    Ok(())
}

fn register_c_super_track(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetSuperTrackedMapPin", lua.create_function(|_, ()| Ok((Value::Nil, Value::Nil)))?)?;
    t.set("SetSuperTrackedMapPin", lua.create_function(|_, (_m, _x, _y): (i32, f32, f32)| Ok(()))?)?;
    t.set("ClearSuperTrackedMapPin", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("GetSuperTrackedQuestID", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("SetSuperTrackedQuestID", lua.create_function(|_, _id: i32| Ok(()))?)?;
    t.set("IsSuperTrackingQuest", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsSuperTrackingMapPin", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetSuperTrackedVignette", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("IsSuperTrackingAnything", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetSuperTrackedContent", lua.create_function(|_, ()| Ok((Value::Nil, Value::Nil)))?)?;
    lua.globals().set("C_SuperTrack", t)?;
    Ok(())
}

fn register_c_player_interaction_manager(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsInteractingWithNpcOfType", lua.create_function(|_, _n: i32| Ok(false))?)?;
    t.set("ClearInteraction", lua.create_function(|_, _i: Option<i32>| Ok(()))?)?;
    t.set("GetCurrentInteraction", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    lua.globals().set("C_PlayerInteractionManager", t)?;
    Ok(())
}

fn register_c_paper_doll_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetStatsError", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetMinItemLevel", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("OffhandHasShield", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("OffhandHasWeapon", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsRangedSlotShown", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetArmorEffectiveness", lua.create_function(|_, _args: mlua::MultiValue| Ok(0.0_f64))?)?;
    t.set("GetArmorEffectivenessAgainstTarget", lua.create_function(|_, _args: mlua::MultiValue| Ok(Value::Nil))?)?;
    t.set("GetStaggerPercentage", lua.create_function(|_, _unit: Value| Ok((0.0_f64, Value::Nil)))?)?;
    t.set("CanCursorCanGoInSlot", lua.create_function(|_, _args: mlua::MultiValue| Ok(false))?)?;
    lua.globals().set("C_PaperDollInfo", t)?;
    Ok(())
}

fn register_c_perks_program(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsTradingPostAvailable", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_PerksProgram", t)?;
    Ok(())
}
