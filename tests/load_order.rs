//! Tests for Blizzard addon load order.
//!
//! Verifies that transitive dependencies are loaded before the addons that
//! need them, even when the dependency chain crosses base UI addon boundaries.

use crate::common;

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Blizzard_ObjectAPI (which defines ItemMixin) must load before Blizzard_FrameXML
/// (which uses ItemMixin in EventToastManager.lua:669).
#[test]
fn test_object_api_loads_before_frame_xml() {
    test_timeout! {
        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);

        let names: Vec<&str> = addons.iter().map(|(n, _)| n.as_str()).collect();

        let obj_api_pos = names.iter().position(|&n| n == "Blizzard_ObjectAPI");
        let frame_xml_pos = names.iter().position(|&n| n == "Blizzard_FrameXML");

        assert!(
            obj_api_pos.is_some(),
            "Blizzard_ObjectAPI should be in the addon list"
        );
        assert!(
            frame_xml_pos.is_some(),
            "Blizzard_FrameXML should be in the addon list"
        );

        assert!(
            obj_api_pos.unwrap() < frame_xml_pos.unwrap(),
            "Blizzard_ObjectAPI (pos {}) must load before Blizzard_FrameXML (pos {})\n\
             Load order: {:?}",
            obj_api_pos.unwrap(),
            frame_xml_pos.unwrap(),
            &names[..std::cmp::min(names.len(), 10)],
        );
    }
}

/// ItemMixin (from Blizzard_ObjectAPI) must be defined when Blizzard_FrameXML loads.
/// EventToastManager.lua:669 does `CreateFromMixins(..., ItemMixin)` at file scope.
#[test]
fn test_item_mixin_available_for_event_toast_manager() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);

        for (name, toc_path) in &addons {
            load_addon(&env.loader_env(), toc_path).ok();
            if name == "Blizzard_FrameXML" {
                break;
            }
        }

        let has_item_mixin: bool = env
            .eval("return type(ItemMixin) == 'table'")
            .unwrap_or(false);
        assert!(
            has_item_mixin,
            "ItemMixin should be defined before Blizzard_FrameXML finishes loading"
        );
    }
}

/// Blizzard_UIPanels_Game (which defines PaperDollItemSlotButton_OnLoad) must
/// load before Blizzard_MainMenuBarBagButtons (whose OnLoad calls it).
/// ActionBar depends on UIPanels_Game, and ActionBar sorts before BagButtons
/// alphabetically, so the dependency chain pulls UIPanels_Game in first.
#[test]
fn test_uipanels_game_loads_before_bag_buttons() {
    test_timeout! {
        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);

        let names: Vec<&str> = addons.iter().map(|(n, _)| n.as_str()).collect();

        let uipanels_pos = names.iter().position(|&n| n == "Blizzard_UIPanels_Game");
        let bags_pos = names.iter().position(|&n| n == "Blizzard_MainMenuBarBagButtons");

        assert!(uipanels_pos.is_some(), "Blizzard_UIPanels_Game should be in the addon list");
        assert!(bags_pos.is_some(), "Blizzard_MainMenuBarBagButtons should be in the addon list");

        assert!(
            uipanels_pos.unwrap() < bags_pos.unwrap(),
            "Blizzard_UIPanels_Game (pos {}) must load before Blizzard_MainMenuBarBagButtons (pos {})",
            uipanels_pos.unwrap(),
            bags_pos.unwrap(),
        );
    }
}

/// Blizzard_ItemButton and Blizzard_FrameXMLUtil must both load before
/// Blizzard_FrameXML, which uses surfaces from each addon.
#[test]
fn test_item_button_and_framexmlutil_load_before_framexml() {
    test_timeout! {
        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);
        let names: Vec<&str> = addons.iter().map(|(n, _)| n.as_str()).collect();

        let framexml_position = names
            .iter()
            .position(|name| *name == "Blizzard_FrameXML")
            .expect("Blizzard_FrameXML should be in the addon list");

        for addon in ["Blizzard_ItemButton", "Blizzard_FrameXMLUtil"] {
            let addon_position = names
                .iter()
                .position(|name| *name == addon)
                .unwrap_or_else(|| panic!("{addon} should be in the addon list"));

            assert!(
                addon_position < framexml_position,
                "{addon} (pos {addon_position}) must load before Blizzard_FrameXML (pos {framexml_position})"
            );
        }
    }
}

/// Startup consumers must pull their load-on-demand publisher addons without
/// promoting unrelated load-on-demand addons into eager discovery.
#[test]
fn test_startup_publishers_load_before_consumers() {
    test_timeout! {
        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);
        let names: Vec<&str> = addons.iter().map(|(name, _)| name.as_str()).collect();

        for (publisher, consumer) in [
            ("Blizzard_TimeManager", "Blizzard_Game"),
            ("Blizzard_CooldownBroadcaster", "Blizzard_Game"),
            ("Blizzard_BoostTutorial", "Blizzard_Game"),
            ("Blizzard_CombatLog", "Blizzard_Game"),
            ("Blizzard_RaidFrame", "Blizzard_RaidUI"),
        ] {
            let publisher_position = names
                .iter()
                .position(|name| *name == publisher)
                .unwrap_or_else(|| panic!("{publisher} should be discovered"));
            let consumer_position = names
                .iter()
                .position(|name| *name == consumer)
                .unwrap_or_else(|| panic!("{consumer} should be discovered"));

            assert!(
                publisher_position < consumer_position,
                "{publisher} must load before {consumer}"
            );
        }

        for publisher in [
            "Blizzard_MacroUI",
            "Blizzard_TrainerUI",
            "Blizzard_AchievementUI",
        ] {
            assert!(
                names.contains(&publisher),
                "standalone startup publisher {publisher} should be discovered"
            );
        }

        assert!(
            !names.contains(&"Deprecated_PaperDoll"),
            "unrelated load-on-demand addons must remain excluded"
        );
    }
}

/// Snapshot of the full resolved Blizzard addon load order.
///
/// If the topological sort algorithm changes and reorders addons, this test
/// catches it. Update the snapshot deliberately when the order changes for a
/// good reason (e.g. new addon added, wow-ui-source updated, dependency changed).
///
/// To regenerate: `cargo test --test integration load_order::dump_load_order -- --ignored --nocapture`
#[test]
fn test_blizzard_addon_load_order_snapshot() {
    test_timeout! {
        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);
        let names: Vec<&str> = addons.iter().map(|(n, _)| n.as_str()).collect();

        #[rustfmt::skip]
        let expected: &[&str] = &[
            "Blizzard_ProjectConstants",
            "Blizzard_LoadLocale",
            "Blizzard_Fonts_Shared",
            "Blizzard_ScriptErrors",
            "Blizzard_SharedXMLBase",
            "Blizzard_PrintHandler",
            "Blizzard_Menu",
            "Blizzard_Colors",
            "Blizzard_HelpPlate",
            "Blizzard_Narration",
            "Blizzard_SharedXML",
            "Blizzard_SharedXMLGame",
            "Blizzard_FrameXMLBase",
            "Blizzard_ObjectAPI",
            "Blizzard_StaticPopup",
            "Blizzard_FrameXMLUtil",
            "Blizzard_UIPanelTemplates",
            "Blizzard_ManagedFrameSystem",
            "Blizzard_GameMenuEsc",
            "Blizzard_UIParentUtil",
            "Blizzard_EditMode",
            "Blizzard_GarrisonBase",
            "Blizzard_GameTooltip",
            "Blizzard_RaidWarning",
            "Blizzard_VisualAlerts",
            "Blizzard_PrivateAurasUI",
            "Blizzard_AuraContainer",
            "Blizzard_AuthChallengeUI",
            "Blizzard_CatalogShopSharedUtil",
            "Blizzard_CatalogShopSharedTemplates",
            "Blizzard_CatalogShop",
            "Blizzard_AsyncRequest",
            "Blizzard_CatalogShopRefundFlow",
            "Blizzard_CatalogShopTopUpFlow",
            "Blizzard_ClassTrialSecure",
            "Blizzard_CombatLogBase",
            "Blizzard_CombatLogProcessor",
            "Blizzard_CommunitiesSecure",
            "Blizzard_DebugUtil",
            "Blizzard_UIErrorsFrame",
            "Blizzard_UIParentPanelManager",
            "Blizzard_ItemButton",
            "Blizzard_AutoComplete",
            "Blizzard_MoneyFrame",
            "Blizzard_AccessibilityTemplates",
            "Blizzard_StaticPopup_Game",
            "Blizzard_Settings_Shared",
            "Blizzard_TextStatusBar",
            "Blizzard_SettingsDefinitions_Shared",
            "Blizzard_SettingsDefinitions_Frame",
            "Blizzard_UnitPopupShared",
            "Blizzard_AddFriend",
            "Blizzard_SocialUIShared",
            "Blizzard_UnitPopup",
            "Blizzard_TransmogShared",
            "Blizzard_LFGUtil",
            "Blizzard_MirrorTimer",
            "Blizzard_FrameXML",
            "Blizzard_SimpleCheckout",
            "Blizzard_StoreUI",
            "Blizzard_QuickKeybind",
            "Blizzard_POIButton",
            "Blizzard_UIPanels_Game",
            "Blizzard_Flyout",
            "Blizzard_GuildControlUI",
            "Blizzard_TimerunningUtil",
            "Blizzard_ChatFrameBase",
            "Blizzard_VoiceToggleButton",
            "Blizzard_ChatFrame",
            "Blizzard_Communities",
            "Blizzard_MicroMenu",
            "Blizzard_PingUI",
            "Blizzard_ActionBar",
            "Blizzard_Minimap",
            "Blizzard_BuffFrame",
            "Blizzard_SpellDiminishUI",
            "Blizzard_ColorPickerFrame",
            "Blizzard_StatusTrayManager",
            "Blizzard_UnitFrame",
            "Blizzard_ScriptErrorsFrame",
            "Blizzard_RestrictedAddOnEnvironment",
            "Blizzard_EnvironmentCleanup",
            "Blizzard_TimeManager",
            "Blizzard_CooldownBroadcaster",
            "middleclass",
            "Blizzard_Dispatcher",
            "Blizzard_BoostTutorial",
            "Blizzard_CombatLog",
            "Blizzard_Game",
            "Blizzard_MapCanvasSecureUtil",
            "Blizzard_SecureTransferUI",
            "Blizzard_WowTokenUI",
            "Blizzard_AccountStore",
            "Blizzard_AchievementUI",
            "Blizzard_OverrideActionBar",
            "Blizzard_ActionBarController",
            "Blizzard_ActionStatus",
            "Blizzard_AddOnList",
            "Blizzard_AddOnPerformance",
            "Blizzard_AnimatedShine",
            "Blizzard_ArdenwealdGardening",
            "Blizzard_SocialToast",
            "Blizzard_BNet",
            "Blizzard_ButtonPulse",
            "Blizzard_CUFProfiles",
            "Blizzard_Channels",
            "Blizzard_ChatBubble",
            "Blizzard_ChatFrameUtil",
            "Blizzard_ClassMenu",
            "Blizzard_ClassTrial",
            "Blizzard_ClientSavedVariables",
            "Blizzard_UIModes",
            "Blizzard_ClientSceneVisManager",
            "Blizzard_CodeOfConduct",
            "Blizzard_CombatAudioAlerts",
            "Blizzard_CommandLineUtil",
            "Blizzard_RecentAllies",
            "Blizzard_FriendsFrame",
            "Blizzard_RaidFrame",
            "Blizzard_CompactRaidFrames",
            "Blizzard_Console",
            "Blizzard_UIFrameManager",
            "Blizzard_MawBuffs",
            "Blizzard_SpellSearch",
            "Blizzard_SharedTalentUI",
            "Blizzard_TieredEntranceTraits",
            "Blizzard_UIWidgets",
            "Blizzard_ObjectiveTracker",
            "Blizzard_ContentTracking",
            "Blizzard_CooldownViewer",
            "Blizzard_CovenantToasts",
            "Blizzard_DamageMeter",
            "Blizzard_DeclensionFrame",
            "Blizzard_PagedContent",
            "Blizzard_DelvesCompanionConfiguration",
            "Blizzard_DelvesToast",
            "Blizzard_Deprecated",
            "Blizzard_DeprecatedActionBar",
            "Blizzard_DeprecatedAuraFilters",
            "Blizzard_DeprecatedAutoComplete",
            "Blizzard_DeprecatedBattleNet",
            "Blizzard_DeprecatedChatInfo",
            "Blizzard_DeprecatedCombatLog",
            "Blizzard_DeprecatedCurrencyScript",
            "Blizzard_DeprecatedGlue",
            "Blizzard_DeprecatedGuildScript",
            "Blizzard_DeprecatedInstanceEncounter",
            "Blizzard_DeprecatedItemScript",
            "Blizzard_DeprecatedItemSocketInfo",
            "Blizzard_DeprecatedLFG",
            "Blizzard_DeprecatedPartyInfo",
            "Blizzard_DeprecatedPetInfo",
            "Blizzard_DeprecatedPvpScript",
            "Blizzard_DeprecatedRaidWarning",
            "Blizzard_DeprecatedSoundScript",
            "Blizzard_DeprecatedSpecialization",
            "Blizzard_DeprecatedSpellBook",
            "Blizzard_DeprecatedSpellScript",
            "Blizzard_DeprecatedTradeInfo",
            "Blizzard_DeprecatedUnitScript",
            "Blizzard_DeprecatedWorldElapsedTimerTypes",
            "Blizzard_DurabilityFrame",
            "Blizzard_Deprecated_ArenaUI",
            "Blizzard_EncounterTimeline",
            "Blizzard_EncounterWarnings",
            "Blizzard_ExpansionLandingPage",
            "Blizzard_FrameEffects",
            "Blizzard_FrameStack",
            "Blizzard_FramerateFrame",
            "Blizzard_FullscreenBrowser",
            "Blizzard_GameMenu",
            "Blizzard_GlobalFXModelScenes",
            "Blizzard_GroupFinder",
            "Blizzard_GuildRename",
            "Blizzard_HelpFrame",
            "Blizzard_HousingEventHandler",
            "Blizzard_HousingTemplates",
            "Blizzard_TutorialManager",
            "Blizzard_Tutorials",
            "Blizzard_HousingTutorials",
            "Blizzard_IME",
            "Blizzard_MacroUI",
            "Blizzard_MailFrame",
            "Blizzard_MainMenuBarBagButtons",
            "Blizzard_MajorFactions",
            "Blizzard_MapCanvas",
            "Blizzard_MatchmakingQueueDisplay",
            "Blizzard_MoneyReceipt",
            "Blizzard_NamePlates",
            "Blizzard_Notification",
            "Blizzard_PVPMatch",
            "Blizzard_PerformanceBar",
            "Blizzard_PersonalResourceDisplay",
            "Blizzard_PetBattleUI",
            "Blizzard_PhotoSharing",
            "Blizzard_QuestNavigation",
            "Blizzard_QueueStatusFrame",
            "Blizzard_QuickJoin",
            "Blizzard_RPE_TurnStrafe",
            "Blizzard_RaidUI",
            "Blizzard_RecruitAFriend",
            "Blizzard_ReportFrameShared",
            "Blizzard_ReportFrame",
            "Blizzard_SavedSets",
            "Blizzard_ShakeUtil",
            "Blizzard_SharedMapDataProviders",
            "Blizzard_SharedWidgetFrames",
            "Blizzard_SocialUI",
            "Blizzard_SplashFrame",
            "Blizzard_StableUI",
            "Blizzard_Subtitles",
            "Blizzard_TokenUI",
            "Blizzard_TrainerUI",
            "Blizzard_TransformManipulator",
            "Blizzard_Transmog",
            "Blizzard_UIParent",
            "Blizzard_WeeklyRewardsUtil",
            "Blizzard_WorldMap",
            "Blizzard_ZoneAbility",
        ];

        assert_eq!(
            names, expected,
            "Blizzard addon load order changed. If intentional, update the snapshot.\n\
             To regenerate: cargo test --test integration load_order::dump_load_order -- --ignored --nocapture"
        );
    }
}

/// Helper to regenerate the snapshot above.
#[test]
#[ignore]
fn dump_load_order() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons(&ui);
    for (name, _) in &addons {
        eprintln!("    \"{name}\",");
    }
    eprintln!("Total: {} addons", addons.len());
    panic!("dump complete — copy output into test_blizzard_addon_load_order_snapshot");
}

/// PaperDollItemSlotButton_OnLoad must exist when Blizzard_MainMenuBarBagButtons loads.
#[test]
fn test_paperdoll_onload_exists_for_bag_buttons() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons(&ui);

        for (name, toc_path) in &addons {
            load_addon(&env.loader_env(), toc_path).ok();
            if name == "Blizzard_MainMenuBarBagButtons" {
                break;
            }
        }

        let exists: bool = env
            .eval("return type(PaperDollItemSlotButton_OnLoad) == 'function'")
            .unwrap_or(false);
        assert!(
            exists,
            "PaperDollItemSlotButton_OnLoad should be defined before Blizzard_MainMenuBarBagButtons loads"
        );
    }
}
