//! Game menu and global game stub APIs.
//!
//! Contains game-system stubs and menu-related functions:
//! - C_ExternalEventURL, C_StorePublic, Kiosk, GameRulesUtil
//! - Game menu stubs (Logout, Quit, StaticPopup, etc.)
//! - C_CatalogShop, C_SplashScreen, C_ArtifactUI/C_AzeriteItem
//! - C_Commentator, C_ChallengeMode, C_Club, C_ClubFinder
//! - Global game stubs (combat, action, account, unit stats, store)
//! - C_Garrison, MinimapUtil, C_CraftingOrders, ExpansionLandingPage

use mlua::{Lua, Result, Value};

pub(super) fn register_all(lua: &Lua) -> Result<()> {
    register_c_external_event_url(lua)?;
    register_c_store_public(lua)?;
    register_kiosk(lua)?;
    register_game_rules_util(lua)?;
    register_game_menu_stubs(lua)?;
    register_c_catalog_shop(lua)?;
    register_c_commentator(lua)?;
    register_c_challenge_mode(lua)?;
    register_c_club(lua)?;
    register_c_club_finder(lua)?;
    register_c_artifact_and_azerite(lua)?;
    register_global_game_stubs(lua)?;
    register_c_garrison(lua)?;
    register_minimap_util(lua)?;
    register_c_crafting_orders(lua)?;
    register_expansion_landing_page(lua)?;
    register_minimap_globals(lua)?;
    register_c_equipment_set(lua)?;
    register_c_adventure_journal(lua)?;
    register_c_summon_info(lua)?;
    register_c_ui(lua)?;
    register_c_item_upgrade(lua)?;
    Ok(())
}

fn register_c_item_upgrade(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("CanUpgradeItem", lua.create_function(|_, _loc: Value| Ok(false))?)?;
    lua.globals().set("C_ItemUpgrade", t)?;
    Ok(())
}

fn register_c_external_event_url(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("HasURL", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsNew", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("LaunchURL", lua.create_function(|_, ()| Ok(()))?)?;
    lua.globals().set("C_ExternalEventURL", t)?;
    Ok(())
}

fn register_c_store_public(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsDisabledByParentalControls", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("EventStoreUISetShown", lua.create_function(|_, _shown: bool| Ok(()))?)?;
    lua.globals().set("C_StorePublic", t)?;
    Ok(())
}

fn register_kiosk(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("Kiosk", t)?;
    Ok(())
}

fn register_game_rules_util(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetActiveAccountStore", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("ShouldShowAddOns", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("ShouldShowSplashScreen", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("ShouldShowExpansionLandingPageButton", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("GameRulesUtil", t)?;
    Ok(())
}

fn register_game_menu_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    let nop = lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?;

    g.set("CurrentVersionHasNewUnseenSettings", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("StaticPopup_Visible", lua.create_function(|_, _w: String| Ok(Value::Nil))?)?;
    g.set("IsRestrictedAccount", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("Logout", nop.clone())?;
    g.set("Quit", nop.clone())?;
    g.set("ForceLogout", nop.clone())?;
    g.set("ForceQuit", nop.clone())?;
    g.set("ShowMacroFrame", nop.clone())?;
    g.set("ToggleHelpFrame", nop.clone())?;
    g.set("ToggleStoreUI", nop.clone())?;
    g.set("UpdateMicroButtons", nop.clone())?;
    g.set("CanAutoSetGamePadCursorControl", lua.create_function(|_, _e: bool| Ok(false))?)?;
    g.set("SetGamePadCursorControl", nop.clone())?;
    g.set("SetPortraitTexture", lua.create_function(|lua, (tex, unit): (Value, Value)| {
        use crate::lua_api::frame::FrameHandle;
        let texture_path = class_icon_path_for_unit(lua, &unit);
        if let Value::UserData(ud) = &tex {
            if let Ok(handle) = ud.borrow::<FrameHandle>() {
                let mut state = handle.state.borrow_mut();
                if let Some(frame) = state.widgets.get_mut(handle.id) {
                    frame.texture = Some(texture_path);
                }
            }
        }
        Ok(())
    })?)?;
    g.set("ChangeActionBarPage", nop.clone())?;
    g.set("StaticPopup_UpdateAll", nop.clone())?;
    g.set("StaticPopup_Show", nop.clone())?;
    g.set("StaticPopup_Hide", nop.clone())?;
    g.set("IsTutorialFlagged", lua.create_function(|_, _f: i32| Ok(false))?)?;

    register_c_splash_screen(lua)?;
    Ok(())
}

fn register_c_catalog_shop(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsShop2Enabled", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_CatalogShop", t)?;
    Ok(())
}

fn register_c_splash_screen(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("RequestLatestSplashScreen", lua.create_function(|_, _f: Option<bool>| Ok(()))?)?;
    t.set("AcknowledgeSplashScreen", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("CanViewSplashScreen", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("SendSplashScreenCloseTelem", lua.create_function(|_, ()| Ok(()))?)?;
    lua.globals().set("C_SplashScreen", t)?;
    lua.globals().set("IsCharacterNewlyBoosted", lua.create_function(|_, ()| Ok(false))?)?;
    Ok(())
}

fn register_c_artifact_and_azerite(lua: &Lua) -> Result<()> {
    let art = lua.create_table()?;
    art.set("IsEquippedArtifactMaxed", lua.create_function(|_, ()| Ok(true))?)?;
    art.set("IsEquippedArtifactDisabled", lua.create_function(|_, ()| Ok(false))?)?;
    art.set("GetEquippedArtifactInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    art.set("GetArtifactItemID", lua.create_function(|_, ()| Ok(0i32))?)?;
    art.set("GetArtifactTier", lua.create_function(|_, ()| Ok(0i32))?)?;
    art.set("IsAtForge", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_ArtifactUI", art)?;

    let az = lua.create_table()?;
    az.set("FindActiveAzeriteItem", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    az.set("IsAzeriteItemAtMaxLevel", lua.create_function(|_, ()| Ok(true))?)?;
    az.set("IsAzeriteItemEnabled", lua.create_function(|_, _i: Value| Ok(false))?)?;
    lua.globals().set("C_AzeriteItem", az)?;

    let aze = lua.create_table()?;
    aze.set("IsAzeriteEmpoweredItem", lua.create_function(|_, _loc: Value| Ok(false))?)?;
    aze.set("IsAzeriteEmpoweredItemByID", lua.create_function(|_, _id: Value| Ok(false))?)?;
    lua.globals().set("C_AzeriteEmpoweredItem", aze)?;
    Ok(())
}

fn register_c_commentator(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetMode", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("IsSpectating", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_Commentator", t)?;
    Ok(())
}

fn register_c_challenge_mode(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsChallengeModeActive", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetActiveChallengeMapID", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetActiveKeystoneInfo", lua.create_function(|_, ()| Ok((0i32, Value::Nil, false)))?)?;
    t.set("GetCompletionInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetDeathCount", lua.create_function(|_, ()| Ok((0i32, 0i32)))?)?;
    t.set("GetLeaverPenaltyWarningTimeLeft", lua.create_function(|_, ()| Ok(0.0f64))?)?;
    lua.globals().set("C_ChallengeMode", t)?;
    Ok(())
}

fn register_c_club(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetSubscribedClubs", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetClubInfo", lua.create_function(|_, _id: i64| Ok(Value::Nil))?)?;
    t.set("GetStreams", lua.create_function(|lua, _id: i64| lua.create_table())?)?;
    t.set("GetClubMembers", lua.create_function(|lua, _id: i64| lua.create_table())?)?;
    t.set("FocusMembers", lua.create_function(|_, _id: i64| Ok(()))?)?;
    t.set("UnfocusMembers", lua.create_function(|_, _id: i64| Ok(()))?)?;
    t.set("SetClubPresenceSubscription", lua.create_function(|_, _id: i64| Ok(()))?)?;
    t.set("ClearClubPresenceSubscription", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("GetInvitationsForSelf", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("IsRestricted", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("ShouldAllowClubType", lua.create_function(|_, _ct: Value| Ok(false))?)?;
    lua.globals().set("C_Club", t)?;
    Ok(())
}

fn register_c_club_finder(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsCommunityFinderEnabled", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsListingEnabledFromFlags", lua.create_function(|_, _f: Option<i32>| Ok(false))?)?;
    t.set("PlayerGetClubInvitationList", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("PlayerRequestPendingClubsList", lua.create_function(|_, _t: Option<i32>| Ok(()))?)?;
    t.set("GetPlayerApplicantLocaleFlags", lua.create_function(|_, ()| Ok(0i32))?)?;
    lua.globals().set("C_ClubFinder", t)?;
    Ok(())
}

fn register_global_game_stubs(lua: &Lua) -> Result<()> {
    register_global_combat_stubs(lua)?;
    register_global_action_stubs(lua)?;
    register_global_account_stubs(lua)?;
    register_actionbar_hotkey_color(lua)?;
    register_unit_stat_constants(lua)?;
    register_store_frame_functions(lua)?;
    register_communities_dialog_stubs(lua)?;
    Ok(())
}

/// Stub dialog frames checked by CommunitiesAddDialogInsecure.lua.
fn register_communities_dialog_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    for name in ["CommunitiesAddDialog", "CommunitiesCreateDialog"] {
        if g.get::<Value>(name)?.is_nil() {
            let stub = lua.create_table()?;
            let attrs = lua.create_table()?;
            stub.set("__attrs", attrs)?;
            stub.set("IsShown", lua.create_function(|_, ()| Ok(false))?)?;
            stub.set("Hide", lua.create_function(|_, ()| Ok(()))?)?;
            stub.set("GetAttribute", lua.create_function(get_attr)?)?;
            stub.set("SetAttribute", lua.create_function(set_attr)?)?;
            g.set(name, stub)?;
        }
    }
    g.set("CommunitiesAvatarPicker_IsShown",
        lua.create_function(|_, ()| Ok(false))?)?;
    g.set("CommunitiesAvatarPicker_CloseDialog",
        lua.create_function(|_, ()| Ok(()))?)?;
    Ok(())
}

fn get_attr(_: &Lua, (this, key): (mlua::Table, String)) -> Result<Value> {
    let attrs: mlua::Table = this.get("__attrs")?;
    attrs.get::<Value>(key.as_str())
}

fn set_attr(_: &Lua, (this, key, val): (mlua::Table, String, Value)) -> Result<()> {
    let attrs: mlua::Table = this.get("__attrs")?;
    attrs.set(key.as_str(), val)
}

/// LE_UNIT_STAT_* constants and SPELL_STAT*_NAME strings for PaperDollFrame.
fn register_unit_stat_constants(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("LE_UNIT_STAT_STRENGTH", 1i32)?;
    g.set("LE_UNIT_STAT_AGILITY", 2i32)?;
    g.set("LE_UNIT_STAT_STAMINA", 3i32)?;
    g.set("LE_UNIT_STAT_INTELLECT", 4i32)?;
    g.set("SPELL_STAT1_NAME", "Strength")?;
    g.set("SPELL_STAT2_NAME", "Agility")?;
    g.set("SPELL_STAT3_NAME", "Stamina")?;
    g.set("SPELL_STAT4_NAME", "Intellect")?;
    g.set("NUM_STATS", 4i32)?;
    Ok(())
}

/// StoreFrame_IsShown function stub (used by MicroButtons).
fn register_store_frame_functions(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("StoreFrame_IsShown", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetRepairAllCost", lua.create_function(|_, ()| Ok((0i64, false)))?)?;
    g.set("GetGuildRenameRequired", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetNumGuildPerks", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("RequestGuildRewards", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("AchievementFrame_ToggleAchievementFrame", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("ToggleAchievementFrame", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("SwitchAchievementSearchTab", lua.create_function(|_, _tab: Value| Ok(()))?)?;
    Ok(())
}

fn register_global_combat_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("GetTotemInfo", lua.create_function(|_, _s: i32| Ok((false, Value::Nil, 0.0f64, 0.0f64, Value::Nil)))?)?;
    g.set("GetNegativeCorruptionEffectInfo", lua.create_function(|lua, ()| lua.create_table())?)?;
    g.set("GetCorruption", lua.create_function(|_, ()| Ok(0.0f64))?)?;
    g.set("GetCorruptionResistance", lua.create_function(|_, ()| Ok(0.0f64))?)?;
    g.set("UnitHasVehicleUI", lua.create_function(|_, _u: Option<String>| Ok(false))?)?;
    g.set("HasArtifactEquipped", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("IsInActiveWorldPVP", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("IsWatchingHonorAsXP", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("DoEmote", lua.create_function(|_, _emote: Option<String>| Ok(()))?)?;
    Ok(())
}

fn register_global_action_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("IsEquippedAction", lua.create_function(|_, _s: Option<i32>| Ok(false))?)?;
    g.set("IsConsumableAction", lua.create_function(|_, _s: Option<i32>| Ok(false))?)?;
    g.set("IsStackableAction", lua.create_function(|_, _s: Option<i32>| Ok(false))?)?;
    g.set("IsItemAction", lua.create_function(|_, _s: Option<i32>| Ok(false))?)?;
    // IsCurrentAction has a stateful implementation in action_bar_api.rs — don't overwrite it.
    g.set("IsAutoRepeatAction", lua.create_function(|_, _s: Option<i32>| Ok(false))?)?;
    g.set("IsAttackAction", lua.create_function(|_, _s: Option<i32>| Ok(false))?)?;
    // HasAction, GetActionInfo, GetActionTexture, IsUsableAction, GetActionCooldown
    // have stateful implementations in action_bar_api.rs — don't overwrite them.
    g.set("GetActionText", lua.create_function(|_, _s: Option<i32>| Ok(Value::Nil))?)?;
    g.set("GetActionCount", lua.create_function(|_, _s: Option<i32>| Ok(0i32))?)?;
    g.set("GetActionCharges", lua.create_function(|_, _s: Option<i32>| Ok((0i32, 0i32, 0.0f64, 0.0f64)))?)?;
    g.set("GetActionLossOfControlCooldown", lua.create_function(|_, _s: Option<i32>| Ok((0.0f64, 0.0f64)))?)?;
    g.set("GetCursorInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    Ok(())
}

fn register_global_account_stubs(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("GetExpansionTrialInfo", lua.create_function(|_, ()| Ok((false, 0i32)))?)?;
    g.set("UnitTrialBankedLevels", lua.create_function(|_, _u: Option<String>| Ok(0i32))?)?;
    g.set("IsInGuild", lua.create_function(|_, ()| Ok(true))?)?;
    g.set("GetGuildLogoInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    g.set("HasCompletedAnyAchievement", lua.create_function(|_, ()| Ok(true))?)?;
    g.set("CanShowAchievementUI", lua.create_function(|_, ()| Ok(true))?)?;
    g.set("CanShowEncounterJournal", lua.create_function(|_, ()| Ok(true))?)?;
    g.set("SortQuestSortTypes", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("SortQuests", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("QuestMapUpdateAllQuests", lua.create_function(|_, ()| Ok(0i32))?)?;
    g.set("QuestPOIUpdateIcons", lua.create_function(|_, ()| Ok(()))?)?;
    Ok(())
}

fn register_actionbar_hotkey_color(lua: &Lua) -> Result<()> {
    let color = lua.create_table()?;
    color.set("r", 0.6f64)?;
    color.set("g", 0.6f64)?;
    color.set("b", 0.6f64)?;
    color.set("a", 1.0f64)?;
    color.set("GetRGB", lua.create_function(|_, this: mlua::Table| {
        Ok((this.get::<f64>("r")?, this.get::<f64>("g")?, this.get::<f64>("b")?))
    })?)?;
    lua.globals().set("ACTIONBAR_HOTKEY_FONT_COLOR", color)?;
    Ok(())
}

fn register_c_garrison(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetLandingPageGarrisonType", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("IsLandingPageMinimapButtonVisible", lua.create_function(|_, _gt: i32| Ok(false))?)?;
    t.set("GetFollowerShipments", lua.create_function(|lua, _id: Value| lua.create_table())?)?;
    lua.globals().set("C_Garrison", t)?;
    Ok(())
}

fn register_minimap_util(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("SetTrackingFilterByFilterIndex", lua.create_function(|_, (_i, _v): (i32, bool)| Ok(()))?)?;
    t.set("GetFilterIndexForFilterID", lua.create_function(|_, _id: i32| Ok(Value::Nil))?)?;
    lua.globals().set("MinimapUtil", t)?;
    Ok(())
}

fn register_c_crafting_orders(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetPersonalOrdersInfo", lua.create_function(|lua, ()| lua.create_table())?)?;
    lua.globals().set("C_CraftingOrders", t)?;
    Ok(())
}

fn register_expansion_landing_page(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("IsOverlayApplied", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetLandingPageType", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetOverlayMinimapDisplayInfo", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    lua.globals().set("ExpansionLandingPage", t)?;
    Ok(())
}

fn register_minimap_globals(lua: &Lua) -> Result<()> {
    let g = lua.globals();
    g.set("HasNewMail", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("GetLatestThreeSenders", lua.create_function(|_, ()| Ok(mlua::MultiValue::new()))?)?;
    g.set("DoesFollowerMatchCurrentGarrisonType", lua.create_function(|_, _ft: Value| Ok(false))?)?;
    g.set("ShowGarrisonLandingPage", lua.create_function(|_, _gt: Value| Ok(()))?)?;
    g.set("ToggleExpansionLandingPage", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("CovenantCalling_CheckCallings", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("ToggleMajorFactionRenown", lua.create_function(|_, _fid: Value| Ok(()))?)?;
    g.set("GetGameTime", lua.create_function(|lua, ()| {
        // Return local (hour, minute) via Lua's os.date to match the system clock.
        let hour: i32 = lua.load("tonumber(os.date('%H'))").eval()?;
        let min: i32 = lua.load("tonumber(os.date('%M'))").eval()?;
        Ok((hour, min))
    })?)?;
    Ok(())
}

fn register_c_equipment_set(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetEquipmentSetIDs", lua.create_function(|lua, ()| lua.create_table())?)?;
    t.set("GetNumEquipmentSets", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetEquipmentSetInfo", lua.create_function(|_, _id: Value| Ok(Value::Nil))?)?;
    t.set("GetEquipmentSetID", lua.create_function(|_, _name: Value| Ok(Value::Nil))?)?;
    t.set("GetIgnoredSlots", lua.create_function(|lua, _id: Value| lua.create_table())?)?;
    t.set("GetEquipmentSetAssignedSpec", lua.create_function(|_, _id: Value| Ok(0i32))?)?;
    lua.globals().set("C_EquipmentSet", t)?;
    Ok(())
}

fn register_c_adventure_journal(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("CanBeShown", lua.create_function(|_, ()| Ok(true))?)?;
    t.set("UpdateSuggestions", lua.create_function(|_, ()| Ok(()))?)?;
    t.set("GetNumAvailableSuggestions", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetPrimaryOffset", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("SetPrimaryOffset", lua.create_function(|_, _off: i32| Ok(()))?)?;
    lua.globals().set("C_AdventureJournal", t)?;
    Ok(())
}

fn register_c_ui(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("ShouldUIParentAvoidNotch", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("GetTopLeftNotchSafeRegion", lua.create_function(|_, ()| Ok((0.0f64, 0.0f64, 0.0f64, 0.0f64)))?)?;
    lua.globals().set("C_UI", t)?;
    Ok(())
}

fn register_c_summon_info(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;
    t.set("GetSummonConfirmTimeLeft", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetSummonReason", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("IsSummonSkippingStartExperience", lua.create_function(|_, ()| Ok(false))?)?;
    lua.globals().set("C_SummonInfo", t)?;
    Ok(())
}

/// Look up the unit's class via `UnitClass` and return the class icon texture path.
fn class_icon_path_for_unit(lua: &Lua, unit: &Value) -> String {
    let class_file = lua.globals().get::<mlua::Function>("UnitClass").ok()
        .and_then(|f| f.call::<mlua::MultiValue>(unit.clone()).ok())
        .and_then(|mv| mv.into_iter().nth(1))
        .and_then(|v| match v {
            Value::String(s) => s.to_str().ok().map(|s| s.to_owned()),
            _ => None,
        });
    match class_file.as_deref() {
        Some("WARRIOR") => r"Interface\Icons\ClassIcon_Warrior",
        Some("PALADIN") => r"Interface\Icons\ClassIcon_Paladin",
        Some("HUNTER") => r"Interface\Icons\ClassIcon_Hunter",
        Some("ROGUE") => r"Interface\Icons\ClassIcon_Rogue",
        Some("PRIEST") => r"Interface\Icons\ClassIcon_Priest",
        Some("DEATHKNIGHT") => r"Interface\Icons\ClassIcon_DeathKnight",
        Some("SHAMAN") => r"Interface\Icons\ClassIcon_Shaman",
        Some("MAGE") => r"Interface\Icons\ClassIcon_Mage",
        Some("WARLOCK") => r"Interface\Icons\ClassIcon_Warlock",
        Some("MONK") => r"Interface\Icons\ClassIcon_Monk",
        Some("DRUID") => r"Interface\Icons\ClassIcon_Druid",
        Some("DEMONHUNTER") => r"Interface\Icons\ClassIcon_DemonHunter",
        Some("EVOKER") => r"Interface\Icons\ClassIcon_Evoker",
        _ => r"Interface\CharacterFrame\TempPortrait",
    }
    .to_string()
}
