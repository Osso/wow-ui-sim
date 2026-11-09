const REMOVED_NAMEPLATE_METHODS: &[&str] = &[
    "GetNamePlateEnemyClickThrough",
    "GetNamePlateEnemyPreferredClickInsets",
    "GetNamePlateEnemySize",
    "GetNamePlateFriendlyClickThrough",
    "GetNamePlateFriendlyPreferredClickInsets",
    "GetNamePlateFriendlySize",
    "GetNamePlateSelfClickThrough",
    "GetNamePlateSelfPreferredClickInsets",
    "GetNamePlateSelfSize",
    "GetNumNamePlateMotionTypes",
    "SetNamePlateEnemyClickThrough",
    "SetNamePlateEnemyPreferredClickInsets",
    "SetNamePlateEnemySize",
    "SetNamePlateFriendlyClickThrough",
    "SetNamePlateFriendlyPreferredClickInsets",
    "SetNamePlateFriendlySize",
    "SetNamePlateSelfClickThrough",
    "SetNamePlateSelfPreferredClickInsets",
    "SetNamePlateSelfSize",
];

const REMOVED_TRANSMOG_COLLECTION_OUTFIT_METHODS: &[&str] = &[
    "DeleteOutfit",
    "GetItemTransmogInfoListFromOutfitHyperlink",
    "GetNumMaxOutfits",
    "GetOutfitHyperlinkFromItemTransmogInfoList",
    "GetOutfitInfo",
    "GetOutfitItemTransmogInfoList",
    "GetOutfits",
    "ModifyOutfit",
    "NewOutfit",
    "RenameOutfit",
];

// The register's 21 removed C_Transmog rows contain 18 Lua API keys plus
// TransmogApplyWarningInfo and its two fields. Structure/field rows are
// metadata, not additional C_Transmog namespace entries, so rawget covers
// only the 18 API keys; the three metadata rows use auxiliary source checks.
const REMOVED_TRANSMOG_METHODS: &[&str] = &[
    "ApplyAllPending",
    "CanTransmogItem",
    "CanTransmogItemWithItem",
    "ClearAllPending",
    "ClearPending",
    "Close",
    "GetApplyCost",
    "GetApplyWarnings",
    "GetBaseCategory",
    "GetCreatureDisplayIDForSource",
    "GetPending",
    "GetSlotEffectiveCategory",
    "GetSlotInfo",
    "GetSlotUseError",
    "IsSlotBeingCollapsed",
    "IsTransmogEnabled",
    "LoadOutfit",
    "SetPending",
];

const REMOVED_TRANSMOG_WARNING_METADATA: &[&str] = &[
    "C_Transmog.TransmogApplyWarningInfo",
    "C_Transmog.TransmogApplyWarningInfo.itemLink",
    "C_Transmog.TransmogApplyWarningInfo.text",
];

const REMOVED_CATALOG_SHOP_METHODS: &[&str] = &["OpenCatalogShopInteraction"];
const REMOVED_EVENT_UTILS_METHODS: &[&str] = &["NotifySettingsLoaded"];
const REMOVED_HOUSE_EXTERIOR_METHODS: &[&str] = &["GetCurrentHouseExteriorTypeName"];
const REMOVED_HOUSING_BASIC_MODE_METHODS: &[&str] = &["IsNudgeEnabled", "SetNudgeEnabled"];
const REMOVED_HOUSING_DECOR_METHODS: &[&str] = &["GetMaxDecorPlaced"];
const REMOVED_PING_SECURE_METHODS: &[&str] = &[
    "GetCooldownInfo",
    "GetDefaultPingOptions",
    "GetTextureKitForType",
];
const REMOVED_PLAYER_INFO_METHODS: &[&str] = &[
    "CanPlayerUseEventScheduler",
    "IsExpansionLandingPageUnlockedForPlayer",
];
const REMOVED_PVP_METHODS: &[&str] = &[
    "CanDisplayDamage",
    "CanDisplayHealing",
    "CanDisplayKillingBlows",
];
const REMOVED_STORE_PUBLIC_METHODS: &[&str] = &["IsDisabledByParentalControls"];
const REMOVED_TASK_QUEST_METHODS: &[&str] =
    &["GetQuestIconUIWidgetSet", "GetQuestTooltipUIWidgetSet"];
const REMOVED_TEXTURE_METHODS: &[&str] = &["GetCraftingReagentQualityChatIcon"];
const REMOVED_TOOLTIP_INFO_METHODS: &[&str] = &["GetTransmogrifyItem"];

fn assert_removed_method_source_absent(
    namespace: &str,
    method: &str,
    path: &std::path::Path,
    source: &[u8],
) {
    let patterns = [
        format!("{namespace}.{method} ="),
        format!("{namespace}['{method}'] ="),
        format!("{namespace}[\"{method}\"] ="),
        format!("function {namespace}.{method}"),
        format!("function {namespace}:{method}"),
        format!("rawset({namespace}, '{method}'"),
        format!("rawset({namespace}, \"{method}\""),
    ];

    for pattern in patterns {
        assert!(
            !super::contains_ascii(source, &pattern),
            "removed API publication {pattern} appears in {}",
            path.display(),
        );
    }
}

fn assert_removed_namespace_source_absent(namespace: &str, methods: &[&str]) {
    for (path, source) in ptr_source_files() {
        for method in methods {
            assert_removed_method_source_absent(namespace, method, &path, &source);
        }
    }
}

fn assert_remaining_removed_source_publications_absent() {
    let namespaces: &[(&str, &[&str])] = &[
        ("C_CatalogShop", REMOVED_CATALOG_SHOP_METHODS),
        ("C_EventUtils", REMOVED_EVENT_UTILS_METHODS),
        ("C_HouseExterior", REMOVED_HOUSE_EXTERIOR_METHODS),
        ("C_HousingBasicMode", REMOVED_HOUSING_BASIC_MODE_METHODS),
        ("C_HousingDecor", REMOVED_HOUSING_DECOR_METHODS),
        ("C_PingSecure", REMOVED_PING_SECURE_METHODS),
        ("C_PlayerInfo", REMOVED_PLAYER_INFO_METHODS),
        ("C_PvP", REMOVED_PVP_METHODS),
        ("C_StorePublic", REMOVED_STORE_PUBLIC_METHODS),
        ("C_TaskQuest", REMOVED_TASK_QUEST_METHODS),
        ("C_Texture", REMOVED_TEXTURE_METHODS),
        ("C_TooltipInfo", REMOVED_TOOLTIP_INFO_METHODS),
    ];

    for (namespace, methods) in namespaces {
        assert_removed_namespace_source_absent(namespace, methods);
    }
}

fn assert_nameplate_source_omits_table_and_dynamic_publications() {
    for (path, source) in ptr_source_files() {
        for method in REMOVED_NAMEPLATE_METHODS {
            let table_literal_patterns = [
                format!("{method} ="),
                format!("[\"{method}\"] ="),
                format!("['{method}'] ="),
            ];
            let dynamic_name_patterns = [format!("\"{method}\""), format!("'{method}'")];

            for pattern in table_literal_patterns
                .iter()
                .chain(dynamic_name_patterns.iter())
            {
                assert!(
                    !super::contains_ascii(source, pattern),
                    "removed NamePlate method {method} appears in table/dynamic source pattern {pattern} in {}",
                    path.display(),
                );
            }
        }

        assert!(
            !source.split(|byte| *byte == b'\n').any(|line| {
                super::contains_ascii(line, "C_NamePlate[") && super::contains_ascii(line, "] =")
            }),
            "dynamic C_NamePlate publication appears in {}",
            path.display(),
        );
    }
}

/// Proves removed NamePlate methods stay absent while retained APIs remain callable.
#[test]
fn removed_nameplate_methods_are_absent_after_full_lod_load() {
    // Source checks only falsify; the full-LoD runtime probe below is the proof.
    assert_ptr_source_omits_qualified_methods("C_NamePlate", REMOVED_NAMEPLATE_METHODS);
    assert_nameplate_source_omits_table_and_dynamic_publications();

    let env = load_full_game_ui_with_all_lod();
    let (removed_published, retained_non_functions): (String, String) = env
        .eval(
            r#"
            local removed = {
                "GetNamePlateEnemyClickThrough",
                "GetNamePlateEnemyPreferredClickInsets",
                "GetNamePlateEnemySize",
                "GetNamePlateFriendlyClickThrough",
                "GetNamePlateFriendlyPreferredClickInsets",
                "GetNamePlateFriendlySize",
                "GetNamePlateSelfClickThrough",
                "GetNamePlateSelfPreferredClickInsets",
                "GetNamePlateSelfSize",
                "GetNumNamePlateMotionTypes",
                "SetNamePlateEnemyClickThrough",
                "SetNamePlateEnemyPreferredClickInsets",
                "SetNamePlateEnemySize",
                "SetNamePlateFriendlyClickThrough",
                "SetNamePlateFriendlyPreferredClickInsets",
                "SetNamePlateFriendlySize",
                "SetNamePlateSelfClickThrough",
                "SetNamePlateSelfPreferredClickInsets",
                "SetNamePlateSelfSize",
            }
            local retained = {
                "GetNamePlateForUnit",
                "GetNamePlates",
                "SetNamePlateSize",
            }
            local removedPublished = {}
            for _, name in ipairs(removed) do
                if rawget(C_NamePlate, name) ~= nil then
                    table.insert(removedPublished, name)
                end
            end
            local retainedNonFunctions = {}
            for _, name in ipairs(retained) do
                if type(rawget(C_NamePlate, name)) ~= "function" then
                    table.insert(retainedNonFunctions, name)
                end
            end
            return table.concat(removedPublished, ","),
                table.concat(retainedNonFunctions, ",")
            "#,
        )
        .expect("C_NamePlate runtime probe succeeds");

    assert_eq!(
        removed_published, "",
        "removed C_NamePlate methods were published"
    );
    assert_eq!(
        retained_non_functions, "",
        "retained C_NamePlate methods are not callable functions",
    );
}

/// Proves removed TransmogCollection outfit methods stay absent while appearance queries remain callable.
#[test]
fn removed_transmog_collection_outfit_methods_are_absent_after_full_lod_load() {
    // Source checks only falsify; the full-LoD runtime probe below is the proof.
    assert_ptr_source_omits_qualified_methods(
        "C_TransmogCollection",
        REMOVED_TRANSMOG_COLLECTION_OUTFIT_METHODS,
    );

    let env = load_full_game_ui_with_all_lod();
    let (removed_published, retained_non_functions): (String, String) = env
        .eval(
            r#"
            local removed = {
                "DeleteOutfit",
                "GetItemTransmogInfoListFromOutfitHyperlink",
                "GetNumMaxOutfits",
                "GetOutfitHyperlinkFromItemTransmogInfoList",
                "GetOutfitInfo",
                "GetOutfitItemTransmogInfoList",
                "GetOutfits",
                "ModifyOutfit",
                "NewOutfit",
                "RenameOutfit",
            }
            local retained = {
                "GetAppearanceSources",
                "GetAllAppearanceSources",
            }
            local removedPublished = {}
            for _, name in ipairs(removed) do
                if rawget(C_TransmogCollection, name) ~= nil then
                    table.insert(removedPublished, name)
                end
            end
            local retainedNonFunctions = {}
            for _, name in ipairs(retained) do
                if type(rawget(C_TransmogCollection, name)) ~= "function" then
                    table.insert(retainedNonFunctions, name)
                end
            end
            return table.concat(removedPublished, ","),
                table.concat(retainedNonFunctions, ",")
            "#,
        )
        .expect("C_TransmogCollection runtime probe succeeds");

    assert_eq!(
        removed_published, "",
        "removed C_TransmogCollection outfit methods were published"
    );
    assert_eq!(
        retained_non_functions, "",
        "retained C_TransmogCollection appearance methods are not callable functions",
    );
}

/// Proves removed C_Transmog APIs stay absent while metadata rows remain source-absent.
#[test]
fn removed_transmog_methods_are_absent_after_full_lod_load() {
    // Source checks only falsify; the full-LoD runtime probe below is the proof.
    assert_ptr_source_omits_qualified_methods("C_Transmog", REMOVED_TRANSMOG_METHODS);
    assert_ptr_source_omits_qualified_symbols(REMOVED_TRANSMOG_WARNING_METADATA);

    let env = load_full_game_ui_with_all_lod();
    let (removed_published, retained_non_functions): (String, String) = env
        .eval(
            r#"
            local removed = {
                "ApplyAllPending",
                "CanTransmogItem",
                "CanTransmogItemWithItem",
                "ClearAllPending",
                "ClearPending",
                "Close",
                "GetApplyCost",
                "GetApplyWarnings",
                "GetBaseCategory",
                "GetCreatureDisplayIDForSource",
                "GetPending",
                "GetSlotEffectiveCategory",
                "GetSlotInfo",
                "GetSlotUseError",
                "IsSlotBeingCollapsed",
                "IsTransmogEnabled",
                "LoadOutfit",
                "SetPending",
            }
            local retained = {
                "GetAllSetAppearancesByID",
                "GetAppliedAlteredAppearance",
                "GetAppliedSourceID",
                "IsAtTransmogNPC",
                "PlayerHasTransmogByItemInfo",
            }
            local removedPublished = {}
            for _, name in ipairs(removed) do
                if rawget(C_Transmog, name) ~= nil then
                    table.insert(removedPublished, name)
                end
            end
            local retainedNonFunctions = {}
            for _, name in ipairs(retained) do
                if type(rawget(C_Transmog, name)) ~= "function" then
                    table.insert(retainedNonFunctions, name)
                end
            end
            return table.concat(removedPublished, ","),
                table.concat(retainedNonFunctions, ",")
            "#,
        )
        .expect("C_Transmog runtime probe succeeds");

    assert_eq!(
        removed_published, "",
        "removed C_Transmog methods were published"
    );
    assert_eq!(
        retained_non_functions, "",
        "retained C_Transmog methods are not callable functions",
    );
}

/// Proves the remaining removed runtime APIs are absent after full LoD loading.
#[test]
fn removed_remaining_runtime_apis_are_absent_after_full_lod_load() {
    // Source checks only falsify; the full-LoD runtime probe below is the proof.
    assert_remaining_removed_source_publications_absent();

    let env = load_full_game_ui_with_all_lod();
    let (removed_published, retained_non_functions): (String, String) = env
        .eval(
            r#"
            local removed = {
                {"C_CatalogShop", "OpenCatalogShopInteraction"},
                {"C_EventUtils", "NotifySettingsLoaded"},
                {"C_HouseExterior", "GetCurrentHouseExteriorTypeName"},
                {"C_HousingBasicMode", "IsNudgeEnabled"},
                {"C_HousingBasicMode", "SetNudgeEnabled"},
                {"C_HousingDecor", "GetMaxDecorPlaced"},
                {"C_PingSecure", "GetCooldownInfo"},
                {"C_PingSecure", "GetDefaultPingOptions"},
                {"C_PingSecure", "GetTextureKitForType"},
                {"C_PlayerInfo", "CanPlayerUseEventScheduler"},
                {"C_PlayerInfo", "IsExpansionLandingPageUnlockedForPlayer"},
                {"C_PvP", "CanDisplayDamage"},
                {"C_PvP", "CanDisplayHealing"},
                {"C_PvP", "CanDisplayKillingBlows"},
                {"C_StorePublic", "IsDisabledByParentalControls"},
                {"C_TaskQuest", "GetQuestIconUIWidgetSet"},
                {"C_TaskQuest", "GetQuestTooltipUIWidgetSet"},
                {"C_Texture", "GetCraftingReagentQualityChatIcon"},
                {"C_TooltipInfo", "GetTransmogrifyItem"},
            }
            local retained = {
                {"C_CatalogShop", "IsShop2Enabled"},
                {"C_PingSecure", "CreateFrame"},
                {"C_PlayerInfo", "GetAlternateFormInfo"},
                {"C_StorePublic", "IsEnabled"},
            }
            local removedPublished = {}
            for _, entry in ipairs(removed) do
                local namespace = rawget(_G, entry[1])
                if type(namespace) == "table" and rawget(namespace, entry[2]) ~= nil then
                    table.insert(removedPublished, entry[1] .. "." .. entry[2])
                end
            end
            local retainedNonFunctions = {}
            for _, entry in ipairs(retained) do
                local namespace = rawget(_G, entry[1])
                if type(namespace) ~= "table" or type(rawget(namespace, entry[2])) ~= "function" then
                    table.insert(retainedNonFunctions, entry[1] .. "." .. entry[2])
                end
            end
            return table.concat(removedPublished, ","),
                table.concat(retainedNonFunctions, ",")
            "#,
        )
        .expect("remaining removed API runtime probe succeeds");

    assert_eq!(
        removed_published, "",
        "remaining removed runtime APIs were published"
    );
    assert_eq!(
        retained_non_functions, "",
        "retained runtime APIs are not callable functions",
    );
}

/// Proves removed legacy globals with no compatibility publication stay absent.
#[test]
fn removed_legacy_global_apis_are_absent_after_full_lod_load() {
    let env = load_full_game_ui_with_all_lod();
    let published: String = env
        .eval(
            r#"
            local names = {
                "IsConsumableSpell",
                "SetRaidTargetProtected",
                "SpellIsAlwaysShown",
                "StripHyperlinks",
            }
            local published = {}
            for _, name in ipairs(names) do
                if rawget(_G, name) ~= nil then
                    table.insert(published, name)
                end
            end
            return table.concat(published, ",")
            "#,
        )
        .expect("removed-global runtime probe succeeds");

    assert_eq!(published, "", "removed legacy globals were published");
}

/// Proves pinned deprecated-global wrappers remain published and forward.
#[test]
fn vendor_deprecated_globals_are_published_and_forward() {
    // Pinned PTR Deprecated_* addons intentionally publish these wrappers when
    // loadDeprecationFallbacks is enabled.
    let env = load_full_game_ui_with_all_lod();
    let (
        fallbacks_enabled,
        missing,
        action_forwarded,
        battlenet_forwarded,
        encounter_forwarded,
        combat_log_alias,
        combat_text_alias,
        death_recap_alias,
    ): (bool, String, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local action_bar = {
                "GetActionAutocast",
                "GetActionText",
                "GetActionTexture",
                "GetActionCount",
                "GetActionCooldown",
                "GetActionCharges",
                "GetActionLossOfControlCooldown",
                "HasAction",
                "IsAttackAction",
                "IsCurrentAction",
                "IsAutoRepeatAction",
                "IsUsableAction",
                "IsConsumableAction",
                "IsStackableAction",
                "IsItemAction",
                "IsEquippedAction",
                "ActionHasRange",
                "IsActionInRange",
                "SetActionUIButton",
                "GetBonusBarIndex",
                "GetBonusBarOffset",
                "GetExtraBarIndex",
                "GetMultiCastBarIndex",
                "GetOverrideBarIndex",
                "GetOverrideBarSkin",
                "GetTempShapeshiftBarIndex",
                "GetVehicleBarIndex",
                "HasBonusActionBar",
                "HasExtraActionBar",
                "HasOverrideActionBar",
                "HasTempShapeshiftActionBar",
                "HasVehicleActionBar",
                "IsPossessBarVisible",
                "ChangeActionBarPage",
                "GetActionBarPage",
            }
            local battlenet = {
                "BNSendGameData",
                "BNSendWhisper",
                "BNSetCustomMessage",
            }
            local combat_log = {
                "CombatLog_Object_IsA",
                "CombatLogAddFilter",
                "CombatLogClearEntries",
                "CombatLogGetCurrentEntry",
                "CombatLogGetCurrentEventInfo",
                "CombatLogGetNumEntries",
                "CombatLogGetRetentionTime",
                "CombatLogResetFilter",
                "CombatLogSetRetentionTime",
                "CombatLogShowCurrentEntry",
            }
            local combat_text = {
                "CombatTextSetActiveUnit",
                "GetCurrentCombatTextEventInfo",
            }
            local death_recap = {
                "DeathRecap_GetEvents",
                "DeathRecap_HasEvents",
                "GetDeathRecapLink",
            }
            local encounter = {
                "IsEncounterInProgress",
                "IsEncounterLimitingResurrections",
                "IsEncounterSuppressingRelease",
            }
            local groups = {
                {"ActionBar", action_bar, 35},
                {"BattleNet", battlenet, 3},
                {"CombatLog", combat_log, 10},
                {"CombatText", combat_text, 2},
                {"DeathRecap", death_recap, 3},
                {"InstanceEncounter", encounter, 3},
            }
            local missing = {}
            for _, group in ipairs(groups) do
                local label, names, expected = group[1], group[2], group[3]
                if #names ~= expected then
                    table.insert(missing, label .. ":count=" .. #names)
                end
                for _, name in ipairs(names) do
                    if type(rawget(_G, name)) ~= "function" then
                        table.insert(missing, name)
                    end
                end
            end

            local oldActionText = C_ActionBar.GetActionText
            local actionID
            C_ActionBar.GetActionText = function(value)
                actionID = value
                return "sentinel"
            end
            local actionOK, actionResult = pcall(GetActionText, 37)
            C_ActionBar.GetActionText = oldActionText
            local actionForwarded = actionOK and actionID == 37 and actionResult == "sentinel"

            local oldCustomMessage = C_BattleNet.SetCustomMessage
            local customMessage
            C_BattleNet.SetCustomMessage = function(value)
                customMessage = value
            end
            local battlenetOK = pcall(BNSetCustomMessage, "sentinel")
            C_BattleNet.SetCustomMessage = oldCustomMessage
            local battlenetForwarded = battlenetOK and customMessage == "sentinel"

            local oldEncounter = C_InstanceEncounter.IsEncounterInProgress
            C_InstanceEncounter.IsEncounterInProgress = function()
                return "sentinel"
            end
            local encounterOK, encounterResult = pcall(IsEncounterInProgress)
            C_InstanceEncounter.IsEncounterInProgress = oldEncounter
            local encounterForwarded = encounterOK and encounterResult == "sentinel"

            local combatLogAlias = rawequal(
                CombatLog_Object_IsA,
                C_CombatLog.DoesObjectMatchFilter
            )
            local combatTextAlias = rawequal(
                CombatTextSetActiveUnit,
                C_CombatText.SetActiveUnit
            )
            local deathRecapAlias = rawequal(
                DeathRecap_HasEvents,
                C_DeathRecap.HasRecapEvents
            )
            return GetCVarBool("loadDeprecationFallbacks"),
                table.concat(missing, ","),
                actionForwarded,
                battlenetForwarded,
                encounterForwarded,
                combatLogAlias,
                combatTextAlias,
                deathRecapAlias
            "#,
        )
        .expect("deprecated global runtime probe succeeds");

    assert!(
        fallbacks_enabled,
        "loadDeprecationFallbacks should be enabled"
    );
    assert_eq!(
        missing, "",
        "deprecated globals were not published as functions"
    );
    assert!(
        action_forwarded,
        "GetActionText did not forward to C_ActionBar"
    );
    assert!(
        battlenet_forwarded,
        "BNSetCustomMessage did not forward to C_BattleNet"
    );
    assert!(
        encounter_forwarded,
        "IsEncounterInProgress did not forward to C_InstanceEncounter"
    );
    assert!(
        combat_log_alias,
        "CombatLog_Object_IsA is not the vendor alias"
    );
    assert!(
        combat_text_alias,
        "CombatTextSetActiveUnit is not the vendor alias"
    );
    assert!(
        death_recap_alias,
        "DeathRecap_HasEvents is not the vendor alias"
    );
}

include!("legacy_compat.rs");
include!("vendor_deprecated_chat_spell.rs");
