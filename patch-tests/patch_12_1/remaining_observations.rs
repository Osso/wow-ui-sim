use super::{load_full_game_ui_with_all_lod, missing_runtime_functions, present_runtime_symbols};

const EXPECTED_PTR_SOURCE_PUBLICATIONS: &[&str] = &[
    "CompactUnitFrameLayoutTemplates_LayoutFrameElement",
    "CompactUnitFrameUtil.ApplyConfig",
    "GameRulesUtil.IsPlayerAtEffectiveMaxLevel",
    "ManageFramePositions",
    "RaidWarningUtil.AddMessage",
    "RaidWarningUtil.ClearBossEmotes",
    "TextureUtil.AnimateTexCoords",
];

const ABSENT_NATIVE_MEMBERS: &[&str] = &[
    "C_Ping.GetContextualPingTypeForUnit",
    "C_RecruitAFriend.IsEnabled",
    "C_UnitAuras.TriggerPrivateAuraShowDispelType",
];

/// Global aliases published by the selected PTR deprecated RaidWarning addon.
const PTR_DEPRECATED_GLOBALS: &[&str] = &[
    "RaidNotice_AddMessage",
    "RaidNotice_Clear",
    "RaidNotice_FadeInit",
    "RaidNotice_UpdateSlot",
];

const LUA_BUILTIN_FUNCTIONS: &[&str] = &["getglobal", "setglobal"];

/// Records runtime misses without treating them as native API absence.
const NON_CALLABLE_REMOVED_SYMBOLS: &[&str] = &[
    "AddFrameLock",
    "AnimatedShine_OnUpdate",
    "AnimateTexCoords",
    "BattleTagInviteFrame_Show",
    "BetterDate",
    "BFAMissionFrame_EscapePressed",
    "BoostTutorial_AttemptLoad",
    "BuildColoredListString",
    "BuildIconArray",
    "BuildListString",
    "BuildMultilineTooltip",
    "BuildNewLineListString",
    "ButtonPulse_OnUpdate",
    "ClassTrainerFrame_Hide",
    "ClassTrainerFrame_Show",
    "CloseCalendarMenus",
    "CommunitiesFrame_IsEnabled",
    "CompactUnitFrame_GetOptionDisplayOnlyHealerPowerBars",
    "CompactUnitFrame_GetOptionDisplayPowerBar",
    "CompactUnitFrame_GetOptionShowDispelIndicatorOverlay",
    "ConvertRGBtoColorString",
    "DisplayTypeUnassignedSupported",
    "EventUtil.AreVariablesLoaded",
    "ExpansionTrial_CheckLoadUI",
    "FriendsFrame_CloseQuickJoinHelpTip",
    "FriendsFrameAddFriendButton_OnClick",
    "FriendsFriends_InitButton",
    "FriendsFriends_SetSelection",
    "FriendsFriendsButton_SetSelected",
    "FriendsFriendsFrame_Close",
    "GetDungeonNameWithDifficulty",
    "GetSocialColoredName",
    "GetSortedSelfResurrectOptions",
    "HelpPlatesSupported",
    "HousingControlsUtil.CanActivateHousingControls",
    "IsFrameLockActive",
    "IsFrameSmartShown",
    "IsLevelAtEffectiveMaxLevel",
    "IsPlayerAtEffectiveMaxLevel",
    "KeyBindingFrame_LoadUI",
    "LocalizePlayerFrame",
    "LocalizezhCN",
    "LocalizezhTW",
    "MajorFactions_LoadUI",
    "NPE_CheckTutorials",
    "NPETutorial_AttemptToBegin",
    "OpenAchievementFrameToAchievement",
    "OrderHallMissionFrame_EscapePressed",
    "OrderHallTalentFrame_EscapePressed",
    "OutfitterUI_LoadUI",
    "QuickJoin_JoinQueueButtonOnClick",
    "RaidBossEmoteFrame_OnEvent",
    "RaidBossEmoteFrame_OnLoad",
    "RaidBrowser_IsEmpowered",
    "RaidNotice_ClearSlot",
    "RaidNotice_OnUpdate",
    "RaidNotice_SetSlot",
    "RaidWarningFrame_OnEvent",
    "RaidWarningFrame_OnLoad",
    "RecentTimeDate",
    "RefreshAuras",
    "RegisterNewFrameLock",
    "RemoveFrameLock",
    "ReverseQuestObjective",
    "SetDesaturation",
    "SetFrameLock",
    "ShowResurrectRequest",
    "SmartHide",
    "SmartShow",
    "TalentFrame_LoadUI",
    "TargetFrame_UpdateBuffAnchor",
    "TargetFrame_UpdateDebuffAnchor",
    "ToggleLFGFrame",
    "ToggleRafPanel",
    "ToggleRaidBrowser",
    "ToggleWoWHackCharacterUI",
    "TokenFrame_LoadUI",
    "TrialAccountCapReached_Inform",
    "UIParent_ManageFramePositions",
    "UIParent_OnEvent",
    "UIParent_OnHide",
    "UIParent_OnLoad",
    "UIParent_OnShow",
    "UIParent_Shared_OnEvent",
    "UIParent_Shared_OnLoad",
    "UIParent_UpdateTopFramePositions",
    "UIParentLoadAddOn",
    "UnitHasMana",
    "UpdateFrameLock",
    "UpdateUIElementsForClientScene",
    "WorldFrame_OnLoad",
    "WorldFrame_OnUpdate",
    "WoWHackSpellsUI_LoadUI",
];

/// Verifies every final 12.1 row against its full-load PTR publication type.
#[test]
fn remaining_symbols_match_observed_publications() {
    let env = load_full_game_ui_with_all_lod();
    let fallbacks_enabled: bool = env
        .eval("return GetCVarBool('loadDeprecationFallbacks')")
        .expect("deprecation fallback CVar query succeeds");
    assert!(
        fallbacks_enabled,
        "PTR source aliases require deprecation fallbacks"
    );

    let compatibility_mismatches: String = env
        .eval(
            r#"
            local function resolve(path)
                local value = _G
                for part in string.gmatch(path, "[^.]+") do
                    value = value and value[part]
                end
                return value
            end

            local function check_callable(path, ...)
                local value = resolve(path)
                if type(value) ~= "function" then
                    return path .. " expected=function observed=" .. type(value)
                end
                local ok, errorMessage = pcall(value, ...)
                if not ok then
                    return path .. " call failed: " .. tostring(errorMessage)
                end
            end

            local mismatches = {}
            for _, probe in ipairs({
                { "GetInventorySlotInfo", "MainHandSlot" },
                { "C_CVar.GetCVar", "UnitNameFocused" },
                { "C_Housing.IsInsideOwnHouse" },
                { "C_SuperTrack.GetNextWaypointForMap", 1 },
            }) do
                local path = probe[1]
                local errorMessage = check_callable(path, unpack(probe, 2))
                if errorMessage then
                    table.insert(mismatches, errorMessage)
                end
            end

            return table.concat(mismatches, ",")
            "#,
        )
        .expect("pre-removal compatibility observation succeeds");
    assert_eq!(
        compatibility_mismatches, "",
        "pinned Blizzard sources require compatibility symbols before strict removal"
    );

    let missing_source_publications =
        missing_runtime_functions(&env, EXPECTED_PTR_SOURCE_PUBLICATIONS);
    assert_eq!(
        missing_source_publications, "",
        "PTR source-declared publications must remain callable"
    );

    let missing_deprecated_globals = missing_runtime_functions(&env, PTR_DEPRECATED_GLOBALS);
    assert_eq!(
        missing_deprecated_globals, "",
        "the selected PTR deprecated RaidWarning aliases remain callable"
    );

    let missing_lua_builtins = missing_runtime_functions(&env, LUA_BUILTIN_FUNCTIONS);
    assert_eq!(missing_lua_builtins, "", "Lua built-ins remain callable");

    let present_native_members = present_runtime_symbols(&env, ABSENT_NATIVE_MEMBERS);
    assert_eq!(
        present_native_members, "",
        "native-absent members must stay absent"
    );
}

/// Verifies runtime misses while keeping native publication status unresolved.
#[test]
fn unresolved_removed_symbols_are_not_callable_in_ptr_runtime() {
    let env = load_full_game_ui_with_all_lod();
    let non_callable = missing_runtime_functions(&env, NON_CALLABLE_REMOVED_SYMBOLS);

    assert_eq!(
        non_callable,
        NON_CALLABLE_REMOVED_SYMBOLS.join(","),
        "runtime misses do not establish native API absence"
    );
}
