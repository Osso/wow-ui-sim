use crate::common;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::{run_extra_update_ticks, settle_headless_startup};

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_character_select_registers_undelete_event_on_load() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let registered: bool = env.eval("return CharacterSelect:IsEventRegistered('CHARACTER_UNDELETE_STARTED')").unwrap();
        assert!(registered);
        let errors = env.state().borrow().lua_errors.clone();
        assert!(
            !errors.iter().any(|error| error.contains("[OnLoad] CharacterSelect:") && error.contains("RegisterEvent")),
            "CharacterSelect event registration failed: {errors:#?}"
        );
    }
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_fonts_follow_screen_phase() {
    test_timeout! {
        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape(
            &["Blizzard_Fonts_Shared"], &[], |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_Fonts_Shared"));
                let game_font_script_absent: bool = env.eval("return UNIT_NAME_FONT_KOREAN == nil").unwrap();
                assert!(game_font_script_absent);
            },
        );
        common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_Fonts_Shared"], &[], |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_Fonts_Shared"));
                let game_font_script_loaded: bool = env.eval("return UNIT_NAME_FONT_KOREAN ~= nil").unwrap();
                assert!(game_font_script_loaded);
            },
        );
    }
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_declension_uses_glue_frame() {
    test_timeout! {
        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape(&["Blizzard_DeclensionFrame"], &[], |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_DeclensionFrame"));
            let is_glue: bool = env.eval("return DeclensionFrame ~= nil and DeclensionFrame:GetParent() == GlueParent").unwrap();
            assert!(is_glue, "DeclensionFrame must be the original glue frame");
            let source: String = env.eval("return debug.getinfo(DeclensionFrame_Update, 'S').source").unwrap();
            assert!(source.contains("DeclensionFrameGlue.lua"), "expected glue function, got {source}");
        });
    }
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_declension_consumer_registers_event() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let errors = env.state().borrow().lua_errors.clone();
        assert!(
            !errors.iter().any(|error| error.contains("DeclensionFrame:RegisterEvent") && error.contains("unknown event")),
            "DeclensionFrame event registration failed: {errors:#?}"
        );
        env.exec(r#"
            declensionProbe = CreateFrame('Frame')
            declensionProbe:RegisterEvent('FORCE_DECLINE_CHARACTER')
            declensionProbe:SetScript('OnEvent', function() declensionEventReceived = true end)
            assert(declensionProbe:IsEventRegistered('FORCE_DECLINE_CHARACTER'))
        "#).unwrap();
        run_extra_update_ticks(&env, 3);
        let emitted: bool = env.eval("return declensionEventReceived == true").unwrap();
        assert!(!emitted);
    }
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_login_state_is_available_to_glue() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        env.exec(r#"
            local s = C_Login.GetState()
            assert(type(s) == 'table')
            assert(type(s.auroraState) == 'number')
            assert(type(s.connectedToWoW) == 'boolean')
            assert(type(s.wowConnectionState) == 'number')
            assert(type(s.hasRealmList) == 'boolean')
            assert(s.waitingForRealmList == false)
            assert(s.superDistrictChoicePending == false)
            assert(s.keepingCharacterListOnRealmJoin == false)
            assert(s.waitingForRuleSets == false)
            assert(s.hasSelectedExperiencePreset == true)
        "#).unwrap();
    }
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_name_reservation_default_is_false() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let result: bool = env.eval("return IsNameReservationOnly() == false").unwrap();
        assert!(result);
    }
}

fn blizzard_ui_dir() -> std::path::PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced")
}

fn load_blizzard_screen(screen: ScreenKind) -> common::LockedEnv {
    common::lock_env(move || {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(screen);

        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        for (name, toc_path) in &addons {
            if let Err(err) = load_addon(&env.loader_env(), toc_path) {
                panic!("[load {name}] FAILED: {err}");
            }
        }

        env.apply_post_load_workarounds();
        settle_headless_startup(&env);
        env
    })
}

#[test]
fn character_select_screen_shows_character_select_not_login() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        let init_errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|msg| msg.contains("InitializeCharacterScreenData"))
            .cloned()
            .collect();
        assert!(
            init_errors.is_empty(),
            "character select boot should not error on InitializeCharacterScreenData: {init_errors:#?}"
        );

        let character_select_visible: bool = env
            .eval("return CharacterSelect ~= nil and CharacterSelect:IsShown()")
            .expect("CharacterSelect visibility should be queryable");
        assert!(
            character_select_visible,
            "character-select screen should show CharacterSelect"
        );

        let account_login_visible: bool = env
            .eval("return AccountLogin ~= nil and AccountLogin:IsShown()")
            .expect("AccountLogin visibility should be queryable");
        assert!(
            !account_login_visible,
            "character-select screen should not fall back to AccountLogin"
        );
    }
}

#[test]
fn character_select_boot_skips_core_missing_stub_errors() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        let errors = env.state().borrow().lua_errors.clone();
        let unexpected: Vec<String> = errors
            .into_iter()
            .filter(|msg| {
                msg.contains("KIOSK_ENABLED")
                    || msg.contains("ACCOUNT_DATA_INITIALIZED")
                    || msg.contains("CHARACTER_LIST_UPDATE")
                    || msg.contains("ACCOUNT_CHARACTER_LIST_RECIEVED")
                    || msg.contains("SHOULD_RECONNECT_TO_REALM_LIST")
                    || msg.contains("UPDATE_REALM_NAME_FOR_GUID")
                    || msg.contains("UPDATE_SELECTED_CHARACTER")
                    || msg.contains("CHAR_RESTORE_COMPLETE")
                    || msg.contains("RACE_FACTION_CHANGE_STARTED")
                    || msg.contains("CHARACTER_LIST_GROUP_CREATED")
                    || msg.contains("CHARACTER_LIST_RESTRICTIONS_RECEIVED")
                    || msg.contains("MAP_SCENE_CHARACTER_ON_MOUSE_ENTER")
                    || msg.contains("FORCE_RENAME_CHARACTER")
                    || msg.contains("ACCOUNT_DATA_RESTORED")
                    || msg.contains("RACE_FACTION_CHANGE_RESULT")
                    || msg.contains("CHAR_RENAME_IN_PROGRESS")
                    || msg.contains("KEY_BINDINGS_COPY_COMPLETE")
                    || msg.contains("CUSTOMIZE_CHARACTER_STARTED")
                    || msg.contains("SetWorldFrameStrata")
                    || msg.contains("GetMaxWarbandGroupCount")
                    || msg.contains("SetCharSelectModelFrame")
                    || msg.contains("SetCharSelectMapSceneFrame")
                    || msg.contains("SetInCharacterSelect")
                    || msg.contains("HasQueuedUpgrade")
                    || msg.contains("GetNumCharacters")
                    || msg.contains("CheckCharacterUndeleteCooldown")
                    || msg.contains("GetServerName")
                    || msg.contains("IsTimerunningEnabled")
                    || msg.contains("IsTrialBoostEnabled")
                    || msg.contains("GetCharacterTimerunningSeasonID")
                    || msg.contains("IsCharacterTimerunningConversionAllowed")
                    || msg.contains("GetLiveRegionCharacterCopySourceRegions")
                    || msg.contains("IsTimerunningSeasonActive")
                    || msg.contains("RequestAutoRealmJoin")
                    || msg.contains("RequestChangeRealmList")
                    || msg.contains("GetQueuedUpgradeGUID")
                    || msg.contains("ClearQueuedUpgrade")
                    || msg.contains("ApplyLevelUp")
                    || msg.contains("AssignUpgradeDistribution")
                    || msg.contains("DoesGUIDHavePendingFactionChange")
                    || msg.contains("GetSelectBackgroundModel")
                    || msg.contains("GetActiveClassTrialBoostType")
                    || msg.contains("GetAutomaticBoost")
                    || msg.contains("GetAutomaticBoostCharacter")
                    || msg.contains("GetCharacterServiceDisplayDataByVASType")
                    || msg.contains("CharacterCreateType")
                    || msg.contains("GetFactionGroupByIndex")
                    || msg.contains("RequestManualUnrevoke")
                    || msg.contains("SetAutomaticBoost")
                    || msg.contains("SetAutomaticBoostCharacter")
                    || msg.contains("TrialBoostCharacter")
                    || msg.contains("GetCharacterCreateType")
                    || msg.contains("GetRaceDataByID")
                    || msg.contains("GetRaceIDFromName")
            })
            .collect();

        assert!(
            unexpected.is_empty(),
            "character-select boot should not hit the current core stub/event gaps: {unexpected:#?}"
        );
    }
}

#[test]
fn character_select_hides_chat_frames() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        let chat_frame_visible: bool = env
            .eval("return ChatFrame1 ~= nil and ChatFrame1:IsShown()")
            .expect("ChatFrame1 visibility should be queryable");
        assert!(
            !chat_frame_visible,
            "character-select screen should not show the front-end chat frame"
        );

        let chat_dock_visible: bool = env
            .eval("return GeneralDockManager ~= nil and GeneralDockManager:IsShown()")
            .expect("GeneralDockManager visibility should be queryable");
        assert!(
            !chat_dock_visible,
            "character-select screen should not show the chat dock"
        );
    }
}

#[test]
fn character_select_populates_character_roster() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        let num_characters: i32 = env
            .eval("return GetNumCharacters()")
            .expect("GetNumCharacters should be queryable");
        assert!(
            num_characters >= 2,
            "character-select screen should expose two default characters"
        );

        let first_guid: Option<String> = env
            .eval("return GetCharacterGUID(1)")
            .expect("GetCharacterGUID(1) should be queryable");
        assert!(
            first_guid.is_some(),
            "character-select screen should expose a guid for the first character"
        );

        let name_field = if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Ptr {
            "characterName"
        } else {
            "name"
        };
        let first_name: Option<String> = env
            .eval(
                &format!("local info = CharacterSelectUtil and CharacterSelectUtil.GetCharacterInfoTable(1); return info and info.{name_field} or nil"),
            )
            .expect("CharacterSelectUtil.GetCharacterInfoTable(1) should be queryable");
        assert!(
            first_name.is_some(),
            "character-select screen should expose info for the first character"
        );

        let second_guid: Option<String> = env
            .eval("return GetCharacterGUID(2)")
            .expect("GetCharacterGUID(2) should be queryable");
        assert!(
            second_guid.is_some(),
            "character-select screen should expose a guid for the second character"
        );

        let second_name: Option<String> = env
            .eval(
                &format!("local info = CharacterSelectUtil and CharacterSelectUtil.GetCharacterInfoTable(2); return info and info.{name_field} or nil"),
            )
            .expect("CharacterSelectUtil.GetCharacterInfoTable(2) should be queryable");
        assert!(
            second_name.is_some(),
            "character-select screen should expose info for the second character"
        );
    }
}

#[test]
fn character_select_builds_scrollbox_entries() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        let provider_count: i32 = env
            .eval(
                r#"
                if CharacterSelectListUtil and CharacterSelectListUtil.BuildCharIndexToIDMapping then
                    CharacterSelectListUtil.BuildCharIndexToIDMapping()
                end
                local complete = CharacterSelectListUtil and CharacterSelectListUtil.CreateCompleteDataProvider
                    and CharacterSelectListUtil.CreateCompleteDataProvider()
                return complete and complete.GetSize and complete:GetSize() or -1
                "#,
            )
            .expect("character list data provider should be enumerable");
        let debug_state: String = env
            .eval(
                r#"
                local completeSize = -1
                if CharacterSelectListUtil
                    and CharacterSelectListUtil.CreateCompleteDataProvider
                    and CharacterSelectCharacterFrame
                then
                    local complete = CharacterSelectListUtil.CreateCompleteDataProvider()
                    completeSize = complete and complete.GetSize and complete:GetSize() or -1
                end
                local providerSize = completeSize
                local numCharacters = GetNumCharacters and GetNumCharacters(true) or -1
                local mappedFirst = CharacterSelectListUtil and CharacterSelectListUtil.GetCharIDFromIndex
                    and CharacterSelectListUtil.GetCharIDFromIndex(1) or -1
                local frameShown = CharacterSelectFrame and CharacterSelectFrame.IsShown and CharacterSelectFrame:IsShown() or false
                local selectedIndex = CharacterSelect and CharacterSelect.selectedIndex or -1
                local charSelectUIType = type(CharacterSelectUI)
                local visibilityType = type(VisibilityFramesContainer)
                local parentVisibilityType = CharacterSelectUI and type(CharacterSelectUI.VisibilityFramesContainer) or "nil"
                local characterListType = type(CharacterSelectCharacterFrame)
                local parentCharacterListType = CharacterSelectUI and CharacterSelectUI.VisibilityFramesContainer
                    and type(CharacterSelectUI.VisibilityFramesContainer.CharacterList) or "nil"
                return string.format(
                    "frameShown=%s selectedIndex=%d providerSize=%d completeSize=%d numCharacters=%d mappedFirst=%d CharacterSelectUI=%s VisibilityFramesContainer=%s ParentVisibility=%s CharacterSelectCharacterFrame=%s ParentCharacterList=%s",
                    tostring(frameShown),
                    selectedIndex,
                    providerSize,
                    completeSize,
                    numCharacters,
                    mappedFirst,
                    charSelectUIType,
                    visibilityType,
                    parentVisibilityType,
                    characterListType,
                    parentCharacterListType
                )
                "#,
            )
            .expect("character list debug state should be queryable");
        assert!(
            provider_count > 0,
            "character-select screen should populate the character list; {debug_state}"
        );
    }
}

#[test]
fn character_select_configuration_warnings_returns_a_table() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        let warnings_type: String = env
            .eval("return type(C_ConfigurationWarnings.GetConfigurationWarnings(false))")
            .expect("configuration warnings should be queryable");
        assert_eq!(
            warnings_type, "table",
            "configuration warnings API should return a table even when there are no warnings"
        );
    }
}

#[test]
fn character_select_tooltip_tolerates_missing_spec_id() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        let tooltip_result: bool = env
            .eval(
                r#"
                local info = CharacterSelectUtil and CharacterSelectUtil.GetCharacterInfoTable(1)
                assert(info, "character info should exist")
                info.specID = nil
                return CharacterSelectUtil.SetTooltipForCharacterInfo(info, 1)
                "#,
            )
            .expect("character tooltip should tolerate a missing specID");

        assert!(
            tooltip_result,
            "character tooltip should still render when specID is missing"
        );

        let errors = env.state().borrow().lua_errors.clone();
        let unexpected: Vec<String> = errors
            .into_iter()
            .filter(|msg| msg.contains("GetSpecializationInfoForSpecID"))
            .collect();

        assert!(
            unexpected.is_empty(),
            "missing specID should not trigger specialization lookup Lua errors: {unexpected:#?}"
        );
    }
}

#[test]
fn character_select_can_transition_to_character_create_without_lua_errors() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        env.exec(r#"GlueParent_SetScreen("charcreate")"#)
            .expect("character-create screen transition should execute");
        run_extra_update_ticks(&env, 3);

        let errors = env.state().borrow().lua_errors.clone();
        let unexpected: Vec<String> = errors
            .into_iter()
            .filter(|msg| msg.contains("Blizzard_CharacterCreate"))
            .collect();

        assert!(
            unexpected.is_empty(),
            "character-create screen transition should not hit CharacterCreate Lua errors: {unexpected:#?}"
        );

        let character_create_visible: bool = env
            .eval("return CharacterCreateFrame ~= nil and CharacterCreateFrame:IsShown()")
            .expect("CharacterCreateFrame visibility should be queryable");
        assert!(
            character_create_visible,
            "character-select screen should be able to show CharacterCreateFrame"
        );
    }
}

#[test]
fn character_create_action_updates_player_name_without_lua_errors() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);

        env.exec(
            r#"
            GlueParent_SetScreen("charcreate")
            CharacterCreateFrame.NameChoiceFrame.EditBox:SetText("Newhero")
            CharacterCreateFrame:CreateCharacter()
            "#,
        )
        .expect("character creation should execute");
        run_extra_update_ticks(&env, 3);

        let errors = env.state().borrow().lua_errors.clone();
        let unexpected: Vec<String> = errors
            .into_iter()
            .filter(|msg| msg.contains("Blizzard_CharacterCreate"))
            .collect();
        assert!(
            unexpected.is_empty(),
            "character creation should not hit CharacterCreate Lua errors: {unexpected:#?}"
        );

        let player_name: String = env
            .eval("return UnitName('player')")
            .expect("player name should be queryable");
        assert_eq!(player_name, "Newhero");
    }
}

#[test]
fn character_create_records_screen_state() {
    let env = load_blizzard_screen(ScreenKind::CharacterCreate);
    env.exec("C_CharacterCreation.SetInCharacterCreate(true)")
        .unwrap();
    assert!(env.state().borrow().in_character_create);
    env.exec("C_CharacterCreation.SetInCharacterCreate(false)")
        .unwrap();
    assert!(!env.state().borrow().in_character_create);
}

#[test]
fn character_create_screen_can_boot_directly() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterCreate);

        let errors = env.state().borrow().lua_errors.clone();
        let unexpected: Vec<String> = errors
            .into_iter()
            .filter(|msg| msg.contains("Blizzard_CharacterCreate"))
            .collect();
        assert!(
            unexpected.is_empty(),
            "direct character-create boot should not hit CharacterCreate Lua errors: {unexpected:#?}"
        );

        let character_create_visible: bool = env
            .eval("return CharacterCreateFrame ~= nil and CharacterCreateFrame:IsShown()")
            .expect("CharacterCreateFrame visibility should be queryable");
        assert!(character_create_visible, "direct boot should show CharacterCreateFrame");
    }
}

#[test]
fn character_create_screen_populates_races_classes_and_customizations() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterCreate);

        let (race_count, class_count, category_count, option_count, selected_sex, back_text, forward_text): (
            i32,
            i32,
            i32,
            i32,
            i32,
            String,
            String,
        ) = env
            .eval(
                r#"
                local races = C_CharacterCreation.GetAvailableRaces()
                local classes = C_CharacterCreation.GetAvailableClasses()
                local categories = C_CharacterCreation.GetAvailableCustomizations()
                local optionCount = 0
                for _, category in ipairs(categories) do
                    optionCount = optionCount + #category.options
                end
                return #races, #classes, #categories, optionCount, C_CharacterCreation.GetSelectedSex(), CharacterCreateFrame.BackButton:GetText() or "", CharacterCreateFrame.ForwardButton:GetText() or ""
                "#,
            )
            .expect("character create data should be queryable");

        assert!(race_count >= 20, "expected a full race list, got {race_count}");
        assert!(class_count >= 13, "expected a full class list, got {class_count}");
        assert!(
            category_count >= 3 && option_count >= 6,
            "expected populated customization categories/options, got {category_count} categories and {option_count} options"
        );
        assert!(
            selected_sex == 0 || selected_sex == 1,
            "selected sex should use Enum.UnitSex male/female values, got {selected_sex}"
        );
        assert!(!back_text.is_empty(), "back button text should be populated");
        assert!(!forward_text.is_empty(), "forward button text should be populated");
    }
}
