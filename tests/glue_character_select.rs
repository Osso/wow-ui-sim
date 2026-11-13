use crate::common;

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
use wow_ui_sim::c_api::c_character_creation::{StartingZoneChoiceInput, StartingZoneChoicesInput};
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
fn ptr_enus_glue_excludes_declension_addon() {
    test_timeout! {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::CharacterSelect);
        assert!(
            !addons.iter().any(|(name, _)| name == "Blizzard_DeclensionFrame"),
            "ruRU-only Blizzard_DeclensionFrame must not be selected for enUS"
        );
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

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_name_reservation_reads_feature_status_byte() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_mode(ScreenKind::CharacterSelect);

    wow_ui_sim::lua_api::globals::real::name_reservation::set_feature_status(&env, 0x80);

    let enabled: bool = env
        .eval("return IsNameReservationOnly()")
        .expect("Forever Glue name-reservation query should be callable");

    assert!(enabled);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_trial_boost_status_defaults_false_on_glue() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_mode(ScreenKind::CharacterSelect);

    let (return_count, is_boolean, enabled): (i32, bool, bool) = env
        .eval(
            r##"
            local return_count = select("#", C_CharacterServices.IsTrialBoostEnabled())
            local enabled = C_CharacterServices.IsTrialBoostEnabled()
            return return_count, type(enabled) == "boolean", enabled
            "##,
        )
        .expect("Forever Glue trial-boost status should be registered");

    assert_eq!(return_count, 1);
    assert!(is_boolean);
    assert!(!enabled);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_name_reservation_default_is_false_on_glue() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_mode(ScreenKind::CharacterSelect);

    let (return_count, result_type, result): (i32, String, bool) = env
        .eval(
            r##"
            local return_count = select("#", IsNameReservationOnly())
            local result = IsNameReservationOnly()
            return return_count, type(result), result
            "##,
        )
        .expect("Forever Glue name-reservation query should be registered");

    assert_eq!(return_count, 1);
    assert_eq!(result_type, "boolean");
    assert!(!result);
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
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn character_create_mode_is_glue_only_and_uses_lua_truthiness() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    let game_surface_has_setter: bool = env
        .eval(
            "return C_CharacterCreation ~= nil and C_CharacterCreation.SetInCharacterCreate ~= nil",
        )
        .expect("Game surface member should be queryable");
    assert!(!game_surface_has_setter);

    env.set_screen_mode(ScreenKind::CharacterSelect);
    let glue_surface_has_setter: bool = env
        .eval("return type(C_CharacterCreation.SetInCharacterCreate) == 'function'")
        .expect("Glue surface member should be queryable");
    assert!(glue_surface_has_setter);

    env.exec(
        r##"
        characterCreateEventCount = 0
        local eventProbe = CreateFrame("Frame")
        eventProbe:RegisterEvent("PLAYER_LOGIN")
        eventProbe:SetScript("OnEvent", function()
            characterCreateEventCount = characterCreateEventCount + 1
        end)

        assert(select("#", C_CharacterCreation.SetInCharacterCreate(true)) == 0)
        assert(select("#", C_CharacterCreation.SetInCharacterCreate(0)) == 0)
        assert(select("#", C_CharacterCreation.SetInCharacterCreate("enter")) == 0)
        assert(select("#", C_CharacterCreation.SetInCharacterCreate({})) == 0)
        assert(not pcall(C_CharacterCreation.SetInCharacterCreate))
        assert(not pcall(C_CharacterCreation.SetInCharacterCreate, nil))
        assert(characterCreateEventCount == 0)
        "##,
    )
    .expect("Glue character-create input should accept Lua truthy values");
    assert!(env.state().borrow().in_character_create);

    env.exec(
        "C_CharacterCreation.SetInCharacterCreate(false); C_CharacterCreation.SetInCharacterCreate(false); assert(characterCreateEventCount == 0)",
    )
    .unwrap();
    assert!(!env.state().borrow().in_character_create);

    env.set_screen_mode(ScreenKind::Game);
    let game_surface_has_setter: bool = env
        .eval(
            "return C_CharacterCreation ~= nil and C_CharacterCreation.SetInCharacterCreate ~= nil",
        )
        .expect("Game surface member should remain queryable");
    assert!(!game_surface_has_setter);
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn character_create_default_facing_uses_host_race_sex_data_in_blizzard_consumer() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterCreate);
        let (return_count, missing_facing): (i32, f64) = env
            .eval(
                "return select('#', C_CharacterCreation.GetDefaultCharacterCreateFacing()), C_CharacterCreation.GetDefaultCharacterCreateFacing()",
            )
            .expect("GetDefaultCharacterCreateFacing should return one number");
        assert_eq!(return_count, 1);
        assert_eq!(missing_facing, 0.0);

        env.exec(
            r##"
            local getFacing = C_CharacterCreation.GetCharacterCreateFacing
            local setFacing = C_CharacterCreation.SetCharacterCreateFacing
            assert(type(getFacing) == "function" and type(setFacing) == "function")
            assert(select("#", getFacing()) == 1)
            assert(math.abs(getFacing()) < 0.001)
            assert(select("#", setFacing("370")) == 0)
            assert(math.abs(getFacing() - 10) < 0.001)
            setFacing(270)
            assert(math.abs(getFacing() + 90) < 0.001)

            local usage = "Usage: C_CharacterCreation.SetCharacterCreateFacing(degrees)"
            local function expect_exact_error(value, detail)
                local ok, message = pcall(setFacing, value)
                assert(not ok)
                assert(message == "bad argument #1 to 'SetCharacterCreateFacing' (" .. detail .. ")")
            end
            local missing_ok, missing_message = pcall(setFacing)
            assert(not missing_ok)
            assert(missing_message == "bad argument #1 to 'SetCharacterCreateFacing' (" .. usage .. ")")
            expect_exact_error(nil, usage)
            local finite_error = "must be a finite number - " .. usage
            expect_exact_error(0 / 0, finite_error)
            expect_exact_error(math.huge, finite_error)
            expect_exact_error(-math.huge, finite_error)

            local function expect_range_error(value)
                local ok, message = pcall(setFacing, value)
                assert(not ok)
                assert(message:find("outside of expected range", 1, true))
                assert(message:find(usage, 1, true))
            end
            expect_range_error("not a number")
            expect_range_error(false)
            expect_range_error(3.4028236e38)
            "##,
        )
        .expect("native facing conversion and argument errors should match");

        env.exec(
            "C_CharacterCreation.SetSelectedRace(2); C_CharacterCreation.SetSelectedSex(1)",
        )
        .expect("select a character-create race and sex");
        let selection: (i64, i64) = env
            .eval(
                "return C_CharacterCreation.GetSelectedRace(), C_CharacterCreation.GetSelectedSex()",
            )
            .expect("selected race and sex should be queryable");
        env.state()
            .borrow_mut()
            .character_create_default_facing_by_race_sex
            .insert(selection, 137.5);

        let facing: f64 = env
            .eval(
                r#"
                local source = debug.getinfo(CharacterCreateMixin.ResetSubjectRotation, "S").source
                assert(source:find("Blizzard_CharacterCreate/", 1, true))
                CharacterCreateFrame:ResetSubjectRotation(nil, true)
                return C_CharacterCreation.GetCharacterCreateFacing()
                "#,
            )
            .expect("the Blizzard reset consumer should load the supplied default facing");
        assert!((facing - 137.5).abs() < 0.001);

        env.set_screen_mode(ScreenKind::Game);
        let game_members: (bool, bool, bool) = env
            .eval(
                "return C_CharacterCreation.GetDefaultCharacterCreateFacing == nil, C_CharacterCreation.GetCharacterCreateFacing == nil, C_CharacterCreation.SetCharacterCreateFacing == nil",
            )
            .expect("Game surface visibility should be queryable");
        assert_eq!(game_members, (true, true, true), "facing APIs are Glue-only");
    }
}

#[test]
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
fn queued_upgrade_presence_drives_blizzard_character_select_consumer() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let (is_queued, return_count): (bool, i32) = env
            .eval(
                "return C_CharacterServices.HasQueuedUpgrade(), select('#', C_CharacterServices.HasQueuedUpgrade())",
            )
            .expect("HasQueuedUpgrade should return one boolean");
        assert!(!is_queued);
        assert_eq!(return_count, 1);

        env.state()
            .borrow_mut()
            .character_services
            .queued_upgrade_guid_type_nonzero = true;
        let is_queued: bool = env
            .eval("return C_CharacterServices.HasQueuedUpgrade()")
            .expect("host-supplied queued-upgrade state should be visible");
        assert!(is_queued);

        let refreshed: bool = env
            .eval(
                r#"
                local oldRefresh = CharacterServicesMaster_OnCharacterListUpdate
                local oldWaiting = StoreFrame_WaitingForCharacterListUpdate
                local oldPopupCheck = StaticPopup_CheckQueuedDialogs
                local refreshCount = 0
                CharacterServicesMaster_OnCharacterListUpdate = function()
                    refreshCount = refreshCount + 1
                end
                StoreFrame_WaitingForCharacterListUpdate = function() return false end
                StaticPopup_CheckQueuedDialogs = function() end
                CharacterSelectFrameMixin.OnUpdate(CharacterSelect, 0)
                CharacterServicesMaster_OnCharacterListUpdate = oldRefresh
                StoreFrame_WaitingForCharacterListUpdate = oldWaiting
                StaticPopup_CheckQueuedDialogs = oldPopupCheck
                return refreshCount == 1
                "#,
            )
            .expect("the Blizzard character-select update should observe queued upgrades");
        assert!(refreshed);

        env.set_screen_mode(ScreenKind::Game);
        let game_member: bool = env
            .eval("return C_CharacterServices.HasQueuedUpgrade == nil")
            .expect("Game surface visibility should be queryable");
        assert!(game_member, "queued-upgrade presence is Glue-only");
    }
}

#[test]
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
fn free_promotional_upgrade_uses_shared_character_services() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    let (registered, return_count, available, store_public_absent): (bool, i32, bool, bool) = env
        .eval(
            r##"
            local return_count = select("#", C_SharedCharacterServices.HasFreePromotionalUpgrade())
            local available = C_SharedCharacterServices.HasFreePromotionalUpgrade()
            return type(C_SharedCharacterServices.HasFreePromotionalUpgrade) == "function",
                return_count, available,
                C_StorePublic == nil or C_StorePublic.HasFreePromotionalUpgrade == nil
            "##,
        )
        .expect("shared promotional-upgrade query should be callable");
    assert!(registered);
    assert_eq!(return_count, 1);
    assert!(!available);
    assert!(store_public_absent);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_free_promotional_upgrade_is_available_on_glue() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_mode(ScreenKind::CharacterSelect);
    let (return_count, available): (i32, bool) = env
        .eval(
            r##"
            local return_count = select("#", C_SharedCharacterServices.HasFreePromotionalUpgrade())
            return return_count, C_SharedCharacterServices.HasFreePromotionalUpgrade()
            "##,
        )
        .expect("Forever Glue should expose the shared promotional-upgrade query");
    assert_eq!(return_count, 1);
    assert!(!available);
}

#[test]
// The Camelot CharacterSelectList consumer is TOC-selected only by `AllowLoadGameType camelot`.
#[cfg(feature = "client-wowforever")]
fn super_district_title_uses_host_selection_and_info() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let (selected_id, return_count): (i32, i32) = env
            .eval(
                "return C_Login.GetSelectedSuperDistrictID(), select('#', C_Login.GetSelectedSuperDistrictID())",
            )
            .expect("selected superdistrict should be queryable");
        assert_eq!(selected_id, 0);
        assert_eq!(return_count, 1);

        let unknown_return_count: i32 = env
            .eval("return select('#', C_DistrictUtils.GetSuperDistrictInfo(0))")
            .expect("unknown superdistrict lookup should be queryable");
        assert_eq!(unknown_return_count, 0);

        let default_title: String = env
            .eval(
                r#"
                local list = CharacterSelectUI.VisibilityFramesContainer.CharacterList
                CharacterSelectListMixin.OnShow(list)
                return list.SuperdistrictText:GetText()
                "#,
            )
            .expect("the cached CharacterSelectList consumer should display its missing-info title");
        assert_eq!(default_title, "");

        let district_id = 73;
        env.state().borrow_mut().super_district_info_by_id.insert(
            district_id,
            wow_ui_sim::c_api::c_district_utils::SuperDistrictInfo {
                super_district_id: district_id,
                display_name: "Supplied region".to_string(),
                description: "Host district description".to_string(),
                icon_file_data_id: 9173,
                disallow_login: true,
                no_realms_available: false,
                override_bg_atlas: "district-background".to_string(),
                override_bg_glow_atlas: "district-glow".to_string(),
            },
        );

        let selection_result: bool = env
            .eval(
                r##"
                local returns = select("#", SelectSuperDistrict("73.9"))
                local selected = C_Login.GetSelectedSuperDistrictID()
                SelectSuperDistrict(999)
                return returns == 0 and selected == 73
                    and C_Login.GetSelectedSuperDistrictID() == 73
                "##,
            )
            .expect("valid selection should truncate and invalid IDs should preserve state");
        assert!(selection_result);

        let argument_errors: bool = env
            .eval(
                r#"
                local function error_contains(callback, expected)
                    local ok, message = pcall(callback)
                    local error_text = tostring(message)
                    return not ok
                        and string.find(error_text, "bad argument #1", 1, true) ~= nil
                        and string.find(error_text, expected, 1, true) ~= nil
                end
                return error_contains(function() SelectSuperDistrict() end,
                                      "Usage: SelectSuperDistrict(superDistrictID)")
                    and error_contains(function() SelectSuperDistrict(nil) end,
                                      "Usage: SelectSuperDistrict(superDistrictID)")
                    and error_contains(function() SelectSuperDistrict("not numeric") end,
                        "outside of expected range -2147483648 to 2147483647 - Usage: SelectSuperDistrict(superDistrictID)")
                    and error_contains(function() SelectSuperDistrict(2147483648) end,
                        "outside of expected range -2147483648 to 2147483647 - Usage: SelectSuperDistrict(superDistrictID)")
                    and error_contains(function() C_DistrictUtils.GetSuperDistrictInfo() end,
                        "Usage: local info = C_DistrictUtils.GetSuperDistrictInfo(superDistrictID)")
                    and error_contains(function() C_DistrictUtils.GetSuperDistrictInfo(nil) end,
                        "Usage: local info = C_DistrictUtils.GetSuperDistrictInfo(superDistrictID)")
                    and error_contains(function() C_DistrictUtils.GetSuperDistrictInfo(false) end,
                        "outside of expected range -2147483648 to 2147483647 - Usage: local info = C_DistrictUtils.GetSuperDistrictInfo(superDistrictID)")
                    and error_contains(function() C_DistrictUtils.GetSuperDistrictInfo(2147483648) end,
                        "outside of expected range -2147483648 to 2147483647 - Usage: local info = C_DistrictUtils.GetSuperDistrictInfo(superDistrictID)")
                "#,
            )
            .expect("superdistrict argument failures should retain native error details");
        assert!(argument_errors);

        let record_matches: bool = env
            .eval(
                r##"
                local info = C_DistrictUtils.GetSuperDistrictInfo("73.9")
                return select("#", C_DistrictUtils.GetSuperDistrictInfo("73.9")) == 1
                    and type(info) == "table"
                    and info.superDistrictID == 73
                    and info.displayName == "Supplied region"
                    and info.description == "Host district description"
                    and info.iconFileDataID == 9173
                    and info.disallowLogin == true
                    and info.noRealmsAvailable == false
                    and info.overrideBgAtlas == "district-background"
                    and info.overrideBgGlowAtlas == "district-glow"
                "##,
            )
            .expect("host-supplied district fields should be projected");
        assert!(record_matches);

        let supplied_title: String = env
            .eval(
                r#"
                local list = CharacterSelectUI.VisibilityFramesContainer.CharacterList
                CharacterSelectListMixin.OnShow(list)
                return list.SuperdistrictText:GetText()
                "#,
            )
            .expect("the cached CharacterSelectList consumer should use the host display name");
        assert_eq!(supplied_title, "Supplied region");

        env.set_screen_mode(ScreenKind::Login);
        let reset_id: i32 = env
            .eval("return C_Login.GetSelectedSuperDistrictID()")
            .expect("login start should reset the selected superdistrict");
        assert_eq!(reset_id, 0);

        env.set_screen_mode(ScreenKind::Game);
        let (glue_members_absent, district_info_available): (bool, bool) = env
            .eval(
                "return C_Login.GetSelectedSuperDistrictID == nil and SelectSuperDistrict == nil, C_DistrictUtils ~= nil and C_DistrictUtils.GetSuperDistrictInfo ~= nil",
            )
            .expect("Game surface visibility should be queryable");
        assert!(glue_members_absent);
        assert_eq!(district_info_available, cfg!(feature = "client-ptr"));
    }
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_does_not_register_forever_name_reservation_global() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_mode(ScreenKind::CharacterSelect);

    let absent: bool = env
        .eval("return IsNameReservationOnly == nil")
        .expect("Retail Glue global visibility should be queryable");

    assert!(absent);
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_does_not_register_super_district_glue_apis() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.set_screen_mode(ScreenKind::CharacterSelect);
    let members_absent: bool = env
        .eval(
            "return C_Login.GetSelectedSuperDistrictID == nil and SelectSuperDistrict == nil and (C_DistrictUtils == nil or C_DistrictUtils.GetSuperDistrictInfo == nil)",
        )
        .expect("Retail superdistrict API visibility should be queryable");
    assert!(members_absent);
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

        let name_field = if matches!(wow_ui_sim::client_profile::ACTIVE,
            wow_ui_sim::client_profile::ClientProfile::Ptr | wow_ui_sim::client_profile::ClientProfile::WowForever) {
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

character_select_tooltip_fixture! {
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
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_character_tooltip_projects_two_supplied_character_names() {
    let env = load_blizzard_screen(ScreenKind::CharacterSelect);
    for name in ["Aldren", "Mirelle"] {
        env.state()
            .borrow_mut()
            .bnet_friends
            .iter_mut()
            .flat_map(|friend| friend.game_accounts.iter_mut())
            .find(|account| account.client_program == "WoW")
            .expect("supplied character account")
            .character_name = name.into();
        env.exec(&format!(
            r#"
            local info = CharacterSelectUtil.GetCharacterInfoTable(1)
            assert(info and info.characterName == {name:?})
            assert(info.fullName == {name:?})
            info.specID = nil
            GlueTooltip:ClearLines()
            assert(CharacterSelectUtil.SetTooltipForCharacterInfo(info, 1))
            assert(GlueTooltip:GetLeftLine(1):GetText() == {name:?})
            "#,
        ))
        .expect("original Camelot tooltip uses supplied character display name");
    }
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_character_tooltip_composes_supplied_surname_and_separator() {
    use wow_ui_sim::lua_api::state::CharacterNameSeparator;
    let env = load_blizzard_screen(ScreenKind::CharacterSelect);
    for (surname, separator, expected) in [
        ("Dawn", CharacterNameSeparator::Space, "Aldren Dawn"),
        ("Vale", CharacterNameSeparator::Hyphen, "Aldren-Vale"),
        ("", CharacterNameSeparator::Space, "Aldren"),
    ] {
        {
            let state = env.state();
            let mut sim = state.borrow_mut();
            let account = sim
                .bnet_friends
                .iter_mut()
                .flat_map(|friend| friend.game_accounts.iter_mut())
                .find(|account| account.client_program == "WoW")
                .unwrap();
            account.character_name = "Aldren".into();
            account.surname = surname.into();
            account.name_separator = separator;
        }
        env.exec(&format!(
            r#"
            local info = CharacterSelectUtil.GetCharacterInfoTable(1)
            assert(info.characterName == 'Aldren' and info.surname == {surname:?})
            assert(info.fullName == {expected:?})
            GlueTooltip:ClearLines()
            assert(CharacterSelectUtil.SetTooltipForCharacterInfo(info, 1))
            assert(GlueTooltip:GetLeftLine(1):GetText() == {expected:?})
        "#
        ))
        .expect("original Camelot tooltip uses composed name and surname");
    }
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_basic_character_info_requires_guid_and_unknown_returns_no_values() {
    let env = load_blizzard_screen(ScreenKind::CharacterSelect);
    for call in ["GetBasicCharacterInfo()", "GetBasicCharacterInfo(nil)"] {
        let error = env.exec(call).expect_err("required GUID").to_string();
        assert!(
            error.contains("Usage: local basicInfo = GetBasicCharacterInfo(guid)"),
            "{error}"
        );
    }
    let arity: i32 = env
        .eval("return select('#', GetBasicCharacterInfo('Player-unknown'))")
        .unwrap();
    assert_eq!(arity, 0);
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
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn character_creation_interaction_records_native_byte_without_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    for arguments in ["", "nil", "false, {}, 17, 'unused'"] {
        env.state()
            .borrow_mut()
            .character_creation
            .interaction_state = 9;
        let count: i32 = env
            .eval(&format!(
                "return select('#', C_CharacterCreation.OnPlayerInteraction({arguments}))"
            ))
            .unwrap();
        assert_eq!(count, 0, "R78 Q8: no arguments read and zero returns");
        assert_eq!(env.state().borrow().character_creation.interaction_state, 1);
    }
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn character_creation_visual_kits_clear_without_a_model() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    for arguments in ["", "nil", "false, {}, 17, 'unused'"] {
        {
            let mut sim = env.state().borrow_mut();
            sim.character_creation.visual_kit_active = true;
            sim.character_creation.visual_kit_value = u64::MAX;
        }
        let count: i32 = env
            .eval(&format!(
                "return select('#', C_CharacterCreation.StopAllSpellVisualKitsOnCharacter({arguments}))"
            ))
            .unwrap();
        assert_eq!(count, 0);
        let sim = env.state().borrow();
        assert!(!sim.character_creation.visual_kit_active);
        assert_eq!(sim.character_creation.visual_kit_value, 0);
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn character_creation_hidden_state_validates_and_has_no_model_effect() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    {
        let mut sim = env.state().borrow_mut();
        sim.events.drain();
        sim.character_creation.interaction_state = 23;
        sim.character_creation.visual_kit_active = true;
        sim.character_creation.visual_kit_value = 55;
    }
    env.exec(r#"
        __hiddenStateEvents = 0
        local listener = CreateFrame("Frame")
        listener:RegisterAllEvents()
        listener:SetScript("OnEvent", function() __hiddenStateEvents = __hiddenStateEvents + 1 end)
        local api = C_CharacterCreation.SetPlayerModelHiddenState
        for _, invoke in ipairs({ function() api() end, function() api(nil) end }) do
            local ok, err = pcall(invoke)
            assert(not ok)
            assert(string.find(err, "Usage: C_CharacterCreation.SetPlayerModelHiddenState(hiddenState)", 1, true))
        end
        for _, value in ipairs({false, true, 1, 0, "", {}, function() end}) do
            assert(select('#', api(value)) == 0)
        end
        assert(__hiddenStateEvents == 0)
    "#).unwrap();
    let sim = env.state().borrow();
    assert!(sim.events.is_empty());
    assert!(sim.character_creation.last_call.is_none());
    assert_eq!(sim.character_creation.interaction_state, 23);
    assert!(sim.character_creation.visual_kit_active);
    assert_eq!(sim.character_creation.visual_kit_value, 55);
}

#[cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]
fn submit_real_character_creation(env: &WowLuaEnv) {
    let errors_before = env.state().borrow().lua_errors.len();
    // Drive the same screen transition as the existing Glue tests, then the
    // real edit box and inherited button script. Do not patch the mixin or
    // invoke CreateCharacter directly. No server outcome is supplied here.
    let submission = env.exec(
        r#"
        local original = C_CharacterCreation.CreateCharacter
        assert(type(original) == "function")
        __creationSubmitCalls = {}
        C_CharacterCreation.CreateCharacter = function(...)
            __creationSubmitCalls[#__creationSubmitCalls + 1] = { n = select('#', ...), ... }
            return original(...)
        end
        GlueParent_SetScreen("charcreate")
        assert(CharacterCreateFrame:IsShown())
        local button = CharacterCreateFrame.ForwardButton
        local click = button:GetScript("OnClick")
        assert(type(click) == "function", "real ForwardButton must have OnClick")
        assert(button:IsEnabled(), "initial customization navigation must be enabled")
        click(button, "LeftButton")
        CharacterCreateFrame.NameChoiceFrame.EditBox:SetText("Newhero")
        for step = 1, 3 do
            if CharacterCreateFrame.creatingCharacter then break end
            assert(button:IsEnabled(), "real Create navigation must be enabled")
            click(button, "LeftButton")
        end
        assert(CharacterCreateFrame.creatingCharacter == true,
            "the real Create button must reach the submission consumer")
    "#,
    );
    let first_error = env.state().borrow().lua_errors[errors_before..]
        .first()
        .map(|error| error.lines().next().unwrap_or_default().to_owned());
    if let Some(first_error) = first_error {
        const EXPECTED_R79_STOP: &str = concat!(
            "bad argument #1 to 'SetCameraZoomLevel' ",
            "(Usage: C_CharacterCreation.SetCameraZoomLevel(zoomLevel [, keepCustomZoom]))"
        );
        assert_eq!(
            first_error, EXPECTED_R79_STOP,
            "unexpected real-control first error"
        );
        // Assert the exact boundary, then remain RED: this is not error tolerance.
        panic!("R79 real Create-button RED at verified first error: {first_error}");
    }
    submission.expect("real character creation controls should submit");
    let observed: (i32, i32, String) = env
        .eval(
            r#"
        local args = __creationSubmitCalls[1]
        assert(args, "the pass-through spy must observe the real native call")
        return #__creationSubmitCalls, args.n, args[1]
    "#,
        )
        .unwrap();
    assert_eq!(observed.0, 1);
    assert_eq!(
        observed.1,
        if cfg!(feature = "client-wowforever") {
            4
        } else {
            3
        }
    );
    assert_eq!(observed.2, "Newhero");
    env.exec(if cfg!(feature = "client-wowforever") {
        r#"
            local args = __creationSubmitCalls[1]
            assert(args[2] == CharacterCreateFrame:GetSelectedSurname())
            assert(args[3] == ZoneChoiceFrame.useNPE)
            assert(args[4] == RaceAndClassFrame:GetCreateCharacterFaction())
        "#
    } else {
        r#"
            local args = __creationSubmitCalls[1]
            assert(args[2] == ZoneChoiceFrame.useNPE)
            assert(args[3] == RaceAndClassFrame:GetCreateCharacterFaction())
        "#
    })
    .unwrap();
    let sim = env.state().borrow();
    let call = sim
        .character_creation
        .last_call
        .as_ref()
        .expect("host-only call record must observe the real submission");
    assert_eq!(call.name, b"Newhero");
    assert_eq!(call.surname.is_some(), cfg!(feature = "client-wowforever"));
    assert!(
        sim.lua_errors[errors_before..].is_empty(),
        "new submit Lua errors: {:?}",
        &sim.lua_errors[errors_before..]
    );
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn character_creation_submit_argument_contract_and_usage() {
    use wow_ui_sim::c_api::c_character_creation_submit::FactionArgument;
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    let forever = cfg!(feature = "client-wowforever");
    let usage = if forever {
        "Usage: C_CharacterCreation.CreateCharacter(name, surname, useNPE [, raceFaction])"
    } else {
        "Usage: C_CharacterCreation.CreateCharacter(name, useNPE [, raceFaction])"
    };
    for bad in ["nil", "false", "{}", "function() end"] {
        let args = if forever {
            format!("{bad}, 'Surname', false")
        } else {
            format!("{bad}, false")
        };
        assert_creation_usage(&env, &args, usage);
        if forever {
            assert_creation_usage(&env, &format!("'Name', {bad}, false"), usage);
        }
    }
    for bad in ["false", "{}", "function() end"] {
        let args = if forever {
            format!("'Name', 'Surname', false, {bad}")
        } else {
            format!("'Name', false, {bad}")
        };
        assert_creation_usage(&env, &args, usage);
    }
    for suffix in ["", ", nil"] {
        let arguments = if forever {
            format!("'Name', 'Surname'{suffix}")
        } else {
            format!("'Name'{suffix}")
        };
        assert_creation_usage(&env, &arguments, usage);
    }
    for (truthy, expected) in [("false", false), ("0", true), ("''", true), ("{}", true)] {
        let args = if forever {
            format!("123, 456, {truthy}, 7, 'extra'")
        } else {
            format!("123, {truthy}, 7, 'extra'")
        };
        env.exec(&format!("C_CharacterCreation.CreateCharacter({args})"))
            .unwrap();
        let call = env
            .state()
            .borrow()
            .character_creation
            .last_call
            .clone()
            .unwrap();
        assert_eq!(call.name, b"123");
        assert_eq!(call.surname, forever.then(|| b"456".to_vec()));
        assert_eq!(call.use_npe, expected);
        assert_eq!(call.race_faction, Some(FactionArgument::Number(7.0)));
    }
    for faction in ["", ", nil", ", 'Alliance'"] {
        let args = if forever {
            format!("'Name', '', false{faction}")
        } else {
            format!("'Name', false{faction}")
        };
        env.exec(&format!("C_CharacterCreation.CreateCharacter({args})"))
            .unwrap();
        let call = env
            .state()
            .borrow()
            .character_creation
            .last_call
            .clone()
            .unwrap();
        let expected =
            (faction == ", 'Alliance'").then(|| FactionArgument::String(b"Alliance".to_vec()));
        assert_eq!(call.race_faction, expected);
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_creation_usage(env: &WowLuaEnv, arguments: &str, usage: &str) {
    let error = env
        .exec(&format!("C_CharacterCreation.CreateCharacter({arguments})"))
        .expect_err("wrong argument type must raise Usage");
    assert!(error.to_string().contains(usage), "{error}");
}

// Stays RED until the R79 DB2-backed customization provider lands; it runs
// under optional-tests until then and returns to the normal suite when GREEN.
#[test]
#[cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]
fn character_creation_submit_real_controls_wait_for_host_result() {
    // Current reachability RED (#228): customization category cameraZoomLevel is nil.
    // Exact first line: bad argument #1 to 'SetCameraZoomLevel'
    // (Usage: C_CharacterCreation.SetCameraZoomLevel(zoomLevel [, keepCustomZoom])).
    // Do not tolerate/skip it or weaken the evidenced zoom argument gate.
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let original_name: String = env.eval("return UnitName('player')").unwrap();
        let original_count: i32 = env.eval("return GetNumCharacters()").unwrap();

        submit_real_character_creation(&env);
        run_extra_update_ticks(&env, 3);
        let pending_ui: (String, bool, String) = env.eval(r#"
            return GlueParent.currentScreen, CharacterCreateFrame:IsShown(),
                CharacterCreateFrame.NameChoiceFrame.EditBox:GetText()
        "#).unwrap();
        assert_eq!(pending_ui, ("charcreate".to_string(), true, "Newhero".to_string()),
            "without a host result the real UI must remain on the submitted creation screen");

        // lt-a912 requires server outcomes to be host-supplied, never invented.
        // The old workaround instead renames the existing player immediately.
        let current_name: String = env.eval("return UnitName('player')").unwrap();
        assert_eq!(current_name, original_name,
            "submission must not rename the existing player while awaiting a host result");
        let current_count: i32 = env.eval("return GetNumCharacters()").unwrap();
        assert_eq!(current_count, original_count,
            "submission without a host result must not manufacture a roster entry");
    }
}

#[cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]
fn enter_character_create_for_starting_zone_test(env: &WowLuaEnv) -> usize {
    use wow_ui_sim::c_api::c_character_creation_customizations::Category;

    // This host fixture is not native customization data; its explicit zoom
    // lets the original Forward path reach the starting-zone consumer.
    env.state().borrow_mut().character_customizations = Some(vec![Category::new(0)]);
    env.exec(
        r#"
        GlueParent_SetScreen("charcreate")
        assert(CharacterCreateFrame:IsShown())
    "#,
    )
    .expect("the original CharacterCreate screen should open");
    env.state().borrow().lua_errors.len()
}

#[cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]
fn click_character_create_forward(env: &WowLuaEnv) {
    env.exec(
        r#"
        local button = CharacterCreateFrame.ForwardButton
        local click = button:GetScript("OnClick")
        assert(type(click) == "function" and button:IsEnabled())
        click(button, "LeftButton")
    "#,
    )
    .expect("the real Forward button should run");
}

#[cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]
fn assert_no_new_character_create_lua_errors(env: &WowLuaEnv, errors_before: usize) {
    let errors = env.state().borrow().lua_errors[errors_before..].to_vec();
    assert!(
        errors.is_empty(),
        "unexpected character-create Lua errors: {errors:#?}"
    );
}

#[test]
#[cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]
fn character_creation_default_starting_zone_reaches_customize_screen() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let errors_before = enter_character_create_for_starting_zone_test(&env);
        click_character_create_forward(&env);
        assert_no_new_character_create_lua_errors(&env, errors_before);

        let next_step: (bool, bool, bool) = env.eval(r#"
            local zone = CharacterCreateFrame.ZoneChoiceFrame
            return CharCustomizeFrame:IsShown(), zone.useNPE, zone:ShouldShow()
        "#).unwrap();
        assert_eq!(next_step, (true, false, false));
        env.exec(r##"
            local function pack(...) return { n = select("#", ...), ... } end
            local choices = pack(C_CharacterCreation.GetStartingZoneChoices())
            assert(choices.n == 2)
            local first, second = choices[1], choices[2]
            assert(type(first) == "table" and second == nil)
            assert(first.zoneName == "" and first.zoneImageAtlas == nil)
            assert(first.isNPE == false)
        "##).unwrap();
    }
}

#[test]
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn retail_character_creation_two_starting_zone_choices_forward_to_create() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let errors_before = enter_character_create_for_starting_zone_test(&env);
        env.state().borrow_mut().character_create_starting_zone_choices = StartingZoneChoicesInput {
            normal_choice: Some(StartingZoneChoiceInput {
                zone_name: "Fixture Zone".to_string(),
                zone_image_atlas: Some("charactercreate-startingzone-exilesreach".to_string()),
            }),
            restricted_to_npe: None,
        };

        click_character_create_forward(&env);
        assert_no_new_character_create_lua_errors(&env, errors_before);
        env.exec(r##"
            local function pack(...) return { n = select("#", ...), ... } end
            local choices = pack(C_CharacterCreation.GetStartingZoneChoices())
            assert(choices.n == 2)
            local first, second = choices[1], choices[2]
            assert(type(first) == "table" and type(second) == "table")
            assert(first.zoneName == "Fixture Zone")
            assert(first.zoneImageAtlas == "charactercreate-startingzone-exilesreach")
            assert(first.isNPE == false)
            assert(second.zoneName == "" and second.zoneImageAtlas == nil)
            assert(second.isNPE == true)
            local zone = CharacterCreateFrame.ZoneChoiceFrame
            assert(zone:ShouldShow() and zone.useNPE == true)
            assert(zone.NormalStartingZone.ZoneNameButton.Label:GetText() == "Fixture Zone")
        "##).unwrap();

        let errors_before = env.state().borrow().lua_errors.len();
        click_character_create_forward(&env);
        assert_no_new_character_create_lua_errors(&env, errors_before);
        env.exec(r#"
            local normal = CharacterCreateFrame.ZoneChoiceFrame.NormalStartingZone.ZoneNameButton.Button
            assert(type(normal:GetScript("OnClick")) == "function")
            normal:Click()
            assert(CharacterCreateFrame.ZoneChoiceFrame.useNPE == false)
            CharacterCreateFrame.NameChoiceFrame.EditBox:SetText("Newhero")
        "#).unwrap();

        let errors_before = env.state().borrow().lua_errors.len();
        click_character_create_forward(&env);
        assert_no_new_character_create_lua_errors(&env, errors_before);
        assert!(env.eval::<bool>("return CharacterCreateFrame.creatingCharacter").unwrap());
        let use_npe = env.state().borrow().character_creation.last_call.as_ref().unwrap().use_npe;
        assert!(!use_npe);
    }
}

#[test]
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn retail_character_creation_npe_restriction_skips_zone_choice_and_creates() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let errors_before = enter_character_create_for_starting_zone_test(&env);
        env.state().borrow_mut().character_create_starting_zone_choices = StartingZoneChoicesInput {
            normal_choice: Some(StartingZoneChoiceInput {
                zone_name: "Restricted Fixture Zone".to_string(),
                zone_image_atlas: Some("charactercreate-startingzone-exilesreach".to_string()),
            }),
            restricted_to_npe: Some(true),
        };

        click_character_create_forward(&env);
        assert_no_new_character_create_lua_errors(&env, errors_before);
        env.exec(r##"
            local function pack(...) return { n = select("#", ...), ... } end
            local choices = pack(C_CharacterCreation.GetStartingZoneChoices())
            assert(choices.n == 2)
            local first, second = choices[1], choices[2]
            assert(type(first) == "table" and second == nil)
            assert(first.zoneName == "" and first.zoneImageAtlas == nil)
            assert(first.isNPE == true)
            local zone = CharacterCreateFrame.ZoneChoiceFrame
            assert(zone.useNPE == true and not zone:ShouldShow())
            CharacterCreateFrame.NameChoiceFrame.EditBox:SetText("Newhero")
        "##).unwrap();

        let errors_before = env.state().borrow().lua_errors.len();
        click_character_create_forward(&env);
        assert_no_new_character_create_lua_errors(&env, errors_before);
        assert!(env.eval::<bool>("return CharacterCreateFrame.creatingCharacter").unwrap());
        let use_npe = env.state().borrow().character_creation.last_call.as_ref().unwrap().use_npe;
        assert!(use_npe);
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn record_creation_results(env: &WowLuaEnv) {
    env.exec(
        r#"
        __creationResults = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("CHARACTER_CREATION_RESULT")
        listener:SetScript("OnEvent", function(_, _, ...)
            __creationResults[#__creationResults + 1] = { n = select('#', ...), ... }
        end)
    "#,
    )
    .unwrap();
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn character_creation_host_result_handler_preservation_without_submit() {
    test_timeout! {
        use wow_ui_sim::c_api::c_character_creation_submit::deliver_result;
        // Isolate the already-loaded real handler while submit is blocked by
        // R78b. This is handler preservation, not an end-to-end Create GREEN.
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        record_creation_results(&env);
        env.exec(r#"
            assert(CharacterCreateFrame:IsEventRegistered("CHARACTER_CREATION_RESULT"))
            SetCVar("showCreateCharacterRealmConfirmDialog", 1)
        "#).unwrap();
        let errors_before = env.state().borrow().lua_errors.len();
        deliver_result(&env, Some("CHAR_CREATE_NAME_IN_USE"), Some("discarded-guid")).unwrap();
        env.exec(r#"
            local args = __creationResults[1]
            assert(args.n == 3 and args[1] == false)
            assert(args[2] == "CHAR_CREATE_NAME_IN_USE" and args[3] == nil)
            local popup = StaticPopup_FindVisible("CHARACTER_CREATE_FAILURE")
            assert(popup and popup:IsShown())
            assert(type(CHAR_CREATE_NAME_IN_USE) == "string" and #CHAR_CREATE_NAME_IN_USE > 0)
            assert(popup:GetTextFontString():GetText() == CHAR_CREATE_NAME_IN_USE)
            StaticPopup_Hide("CHARACTER_CREATE_FAILURE")
        "#).unwrap();
        deliver_result(&env, None, Some("Player-1-00000042")).unwrap();
        env.exec(r#"
            local args = __creationResults[2]
            assert(args.n == 3 and args[1] == true and args[2] == nil)
            assert(args[3] == "Player-1-00000042")
            assert(GetCVar("showCreateCharacterRealmConfirmDialog") == "0")
            assert(GlueParent.currentScreen == "charselect")
        "#).unwrap();
        deliver_result(&env, Some(""), None).unwrap();
        env.exec(r#"
            assert(#__creationResults == 3)
            local args = __creationResults[3]
            assert(args.n == 3 and args[1] == true and args[2] == nil and args[3] == nil)
        "#).unwrap();
        let sim = env.state().borrow();
        assert!(sim.lua_errors[errors_before..].is_empty(), "new result-handler errors: {:?}", &sim.lua_errors[errors_before..]);
    }
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_character_creation_verified_members_are_glue_only() {
    let env = WowLuaEnv::new().unwrap();
    for screen in [
        ScreenKind::CharacterCreate,
        ScreenKind::Game,
        ScreenKind::Login,
    ] {
        env.set_screen_mode(screen);
        let expected = if screen.is_glue() { "function" } else { "nil" };
        let types: (String, String) = env
            .eval(
                r#"
            return type(C_CharacterCreation.CreateCharacter),
                type(C_CharacterCreation.StopAllSpellVisualKitsOnCharacter)
        "#,
            )
            .unwrap();
        assert_eq!(types, (expected.to_owned(), expected.to_owned()));
        if screen.is_glue() {
            // VERIFIED no-model/no-pending-error call, without a return-count claim.
            env.exec("C_CharacterCreation.StopAllSpellVisualKitsOnCharacter()")
                .unwrap();
        }
    }
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_character_creation_camera_zoom_getter_is_glue_only_and_reads_no_arguments() {
    let env = WowLuaEnv::new().unwrap();
    for screen in [
        ScreenKind::CharacterCreate,
        ScreenKind::Game,
        ScreenKind::Login,
    ] {
        env.set_screen_mode(screen);
        let expected = if screen.is_glue() { "function" } else { "nil" };
        assert_eq!(
            env.eval::<String>("return type(C_CharacterCreation.GetCurrentCameraZoom)")
                .unwrap(),
            expected
        );
        if screen.is_glue() {
            env.exec("C_CharacterCreation.GetCurrentCameraZoom(); C_CharacterCreation.GetCurrentCameraZoom(nil, {}, 'unused')").unwrap();
        }
    }
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_character_creation_camera_zoom_getter_reads_host_stored_value() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    for zoom in [17, 83] {
        env.state().borrow_mut().character_create_camera_zoom = zoom;
        let read: i32 = env
            .eval("return C_CharacterCreation.GetCurrentCameraZoom()")
            .unwrap();
        assert_eq!(read, zoom);
    }
    // Stored-value identity is VERIFIED; no default or exact return-count assertion.
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_character_creation_interaction_write_and_no_model_visibility() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    // Calling the inferred catalog entries is fixture setup, not catalog proof.
    for arguments in ["", "nil"] {
        let error = env
            .exec(&format!(
                "C_CharacterCreation.SetPlayerModelHiddenState({arguments})"
            ))
            .expect_err("PTR hidden state requires a non-nil first argument");
        assert!(
            error
                .to_string()
                .contains("Usage: C_CharacterCreation.SetPlayerModelHiddenState(hiddenState)"),
            "{error}"
        );
    }
    env.state()
        .borrow_mut()
        .ptr_character_creation
        .interaction_state = 42;
    env.exec(
        r#"
        local events = 0
        local listener = CreateFrame("Frame")
        listener:RegisterAllEvents()
        listener:SetScript("OnEvent", function() events = events + 1 end)
        C_CharacterCreation.SetPlayerModelHiddenState(false)
        C_CharacterCreation.SetPlayerModelHiddenState(true)
        C_CharacterCreation.SetPlayerModelHiddenState(0)
        C_CharacterCreation.SetPlayerModelHiddenState("")
        C_CharacterCreation.SetPlayerModelHiddenState({})
        assert(events == 0)
        listener:UnregisterAllEvents()
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .ptr_character_creation
            .interaction_state,
        42
    );
    env.exec("C_CharacterCreation.OnPlayerInteraction()")
        .unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .ptr_character_creation
            .interaction_state,
        1
    );
    // No Lua return count, inferred catalog, or nonexistent model-alpha claim.
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_character_creation_submit_requires_three_non_nil_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    const USAGE: &str =
        "Usage: C_CharacterCreation.CreateCharacter(name, surname, useNPE [, raceFaction])";
    for arguments in [
        "",
        "nil, 'Surname', false",
        "'Name'",
        "'Name', nil, false",
        "'Name', 'Surname'",
        "'Name', 'Surname', nil",
    ] {
        let error = env
            .exec(&format!("C_CharacterCreation.CreateCharacter({arguments})"))
            .expect_err("PTR requires present, non-nil name, surname, and useNPE");
        assert!(error.to_string().contains(USAGE), "{error}");
    }
    // R78 PTR verifies that missing/nil optional faction advances. Do not assert
    // types, conversions, Lua return count, or native request fields here.
    for arguments in ["'Name', 'Surname', false", "'Name', 'Surname', false, nil"] {
        env.exec(&format!("C_CharacterCreation.CreateCharacter({arguments})"))
            .unwrap();
    }
}

#[test]
#[cfg(all(feature = "client-ptr", feature = "optional-tests"))]
fn ptr_character_creation_submit_real_controls_do_not_invent_outcome() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let original_name: String = env.eval("return UnitName('player')").unwrap();
        let original_count: i32 = env.eval("return GetNumCharacters()").unwrap();
        let errors_before = env.state().borrow().lua_errors.len();
        let submission = env.exec(r#"
            GlueParent_SetScreen("charcreate")
            assert(CharacterCreateFrame:IsShown())
            local original = C_CharacterCreation.CreateCharacter
            __ptrCreateCalls = {}
            C_CharacterCreation.CreateCharacter = function(...)
                __ptrCreateCalls[#__ptrCreateCalls + 1] = { n = select('#', ...), ... }
                return original(...)
            end
            __ptrCreateEvents = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("CHARACTER_CREATION_RESULT")
            listener:RegisterEvent("CHARACTER_LIST_UPDATE")
            listener:SetScript("OnEvent", function(_, event)
                __ptrCreateEvents[#__ptrCreateEvents + 1] = event
            end)
            local button = CharacterCreateFrame.ForwardButton
            local click = button:GetScript("OnClick")
            assert(type(click) == "function" and button:IsEnabled())
            click(button, "LeftButton")
            CharacterCreateFrame.NameChoiceFrame.EditBox:SetText("Newhero")
            CharacterCreateFrame.NameChoiceFrame.EditBoxSurname:SetText("Example")
            for step = 1, 3 do
                if CharacterCreateFrame.creatingCharacter then break end
                assert(button:IsEnabled())
                click(button, "LeftButton")
            end
            assert(CharacterCreateFrame.creatingCharacter == true,
                "PTR real Create button must reach the submission consumer")
            assert(#__ptrCreateCalls == 1)
            local args = __ptrCreateCalls[1]
            assert(args.n == 4 and args[1] == "Newhero")
            assert(args[2] == CharacterCreateFrame:GetSelectedSurname())
            assert(args[3] == ZoneChoiceFrame.useNPE)
            assert(args[4] == RaceAndClassFrame:GetCreateCharacterFaction())
            assert(#__ptrCreateEvents == 0, "submit must not invent result/list events")
        "#);
        let first_error = env.state().borrow().lua_errors[errors_before..]
            .first().map(|error| error.lines().next().unwrap_or_default().to_owned());
        if let Some(first_error) = first_error {
            const EXPECTED_PTR_STOP: &str = concat!(
                "bad argument #1 to 'SetCameraZoomLevel' ",
                "(Usage: C_CharacterCreation.SetCameraZoomLevel(zoomLevel [, keepCustomZoom]))"
            );
            assert_eq!(first_error, EXPECTED_PTR_STOP, "unexpected PTR first error");
            panic!("PTR real Create-button reachability RED: {first_error}");
        }
        submission.expect("PTR real controls must submit");
        assert_eq!(env.eval::<String>("return UnitName('player')").unwrap(), original_name);
        assert_eq!(env.eval::<i32>("return GetNumCharacters()").unwrap(), original_count);
    }
}

#[test]
#[cfg(not(any(
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "client-ptr"
)))]
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
#[cfg(feature = "client-wowforever")]
fn forever_race_data_publishes_empty_racial_abilities() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let abilities_are_empty_tables: bool = env
        .eval(
            r#"
            local race = C_CharacterCreation.GetRaceDataByID(1)
            local listedRace = C_CharacterCreation.GetAvailableRaces()[1]
            return type(race.racialAbilities) == "table"
                and next(race.racialAbilities) == nil
                and type(listedRace.racialAbilities) == "table"
                and next(listedRace.racialAbilities) == nil
            "#,
        )
        .expect("race data should publish empty racial-ability tables");

    assert!(abilities_are_empty_tables);
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_race_data_publishes_supplied_racial_ability_fields() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.exec(
        r#"
        __wow_character_create_races[1].racialAbilities = {
            { description = "Supplied description", name = "Supplied ability", icon = 0 },
        }
        "#,
    )
    .expect("host-supplied racial ability should be accepted");

    let (row_count, description, name, icon_is_nil): (i32, String, String, bool) = env
        .eval(
            r#"
            local ability = C_CharacterCreation.GetRaceDataByID(1).racialAbilities[1]
            return #C_CharacterCreation.GetRaceDataByID(1).racialAbilities,
                ability.description, ability.name, ability.icon == nil
            "#,
        )
        .expect("race data should publish supplied racial abilities");

    assert_eq!(row_count, 1);
    assert_eq!(description, "Supplied description");
    assert_eq!(name, "Supplied ability");
    assert!(icon_is_nil);
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

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn trial_boost_starting_level_is_registered_only_on_glue_screens() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let absent_on_game: bool = env
        .eval(
            "local api = rawget(_G, 'C_CharacterCreation'); return api == nil or rawget(api, 'GetTrialBoostStartingLevel') == nil",
        )
        .expect("Game-screen API availability should be queryable");
    assert!(
        absent_on_game,
        "trial boost API must be absent on Game screens"
    );

    env.set_screen_mode(ScreenKind::CharacterCreate);
    let available_on_glue: bool = env
        .eval(
            "local api = rawget(_G, 'C_CharacterCreation'); return api ~= nil and type(rawget(api, 'GetTrialBoostStartingLevel')) == 'function'",
        )
        .expect("Glue-screen API availability should be queryable");
    assert!(
        available_on_glue,
        "trial boost API must be available on Glue screens"
    );

    env.set_screen_mode(ScreenKind::Game);
    let absent_after_return_to_game: bool = env
        .eval(
            "local api = rawget(_G, 'C_CharacterCreation'); return api == nil or rawget(api, 'GetTrialBoostStartingLevel') == nil",
        )
        .expect("Game-screen API availability should be queryable after transition");
    assert!(
        absent_after_return_to_game,
        "trial boost API must be removed when returning to Game screens"
    );
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_class_trial_tooltip_uses_exported_character_service_level() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterCreate);
        env.exec(
            "CharacterCreateRaceAndClassMixin.OnShow(CharacterCreateFrame.RaceAndClassFrame)",
        )
        .expect("original class-trial tooltip consumer should execute");

        let (level, tooltip_text): (i32, String) = env
            .eval(
                "local button = CharacterCreateFrame.RaceAndClassFrame.ClassTrialCheckButton; return C_CharacterCreation.GetTrialBoostStartingLevel(), button.tooltipLines[1].text",
            )
            .expect("class-trial level and tooltip should be queryable");
        let expected_tooltip: String = env
            .eval("return CHARACTER_TYPE_FRAME_TRIAL_BOOST_CHARACTER_TOOLTIP:format(100)")
            .expect("class-trial tooltip template should be queryable");

        assert_eq!(level, 100, "Retail 12.1.5.70783 exports level 100 for selector 2");
        assert_eq!(tooltip_text, expected_tooltip);
    }
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_class_trial_tooltip_uses_empty_host_table_default() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterCreate);
        env.exec(
            "CharacterCreateRaceAndClassMixin.OnShow(CharacterCreateFrame.RaceAndClassFrame)",
        )
        .expect("original Camelot class-trial tooltip consumer should execute");

        let (level, tooltip_text): (i32, String) = env
            .eval(
                "local button = CharacterCreateFrame.RaceAndClassFrame.ClassTrialCheckButton; return C_CharacterCreation.GetTrialBoostStartingLevel(), button.tooltipLines[1].text",
            )
            .expect("empty host-table level and original tooltip should be queryable");
        let expected_tooltip: String = env
            .eval("return CHARACTER_TYPE_FRAME_TRIAL_BOOST_CHARACTER_TOOLTIP:format(0)")
            .expect("Camelot class-trial tooltip template should be queryable");

        assert_eq!(level, 0, "the simulator's empty Forever table yields zero");
        assert_eq!(tooltip_text, expected_tooltip);
    }
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_trial_boost_query_returns_exported_level_and_one_value() {
    let env = load_blizzard_screen(ScreenKind::CharacterCreate);
    let (level, result_count): (i32, i32) = env
        .eval(
            "return C_CharacterCreation.GetTrialBoostStartingLevel(), select('#', C_CharacterCreation.GetTrialBoostStartingLevel())",
        )
        .expect("trial boost query should return its exported level");

    assert_eq!(level, 100, "Retail selector 2 exports level 100");
    assert_eq!(result_count, 1, "the getter publishes exactly one value");
}

#[test]
#[cfg(feature = "client-retail")]
fn retail_trial_boost_host_selector_selects_first_matching_row() {
    let env = load_blizzard_screen(ScreenKind::CharacterCreate);
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.trial_boost_level.replace_rows(vec![
            wow_ui_sim::c_api::c_character_creation_trial_boost::CharacterServiceInfoRow {
                service_type: 1,
                boost_type: 4,
                boost_level: 71,
            },
            wow_ui_sim::c_api::c_character_creation_trial_boost::CharacterServiceInfoRow {
                service_type: 1,
                boost_type: 4,
                boost_level: 72,
            },
            wow_ui_sim::c_api::c_character_creation_trial_boost::CharacterServiceInfoRow {
                service_type: 1,
                boost_type: 8,
                boost_level: 90,
            },
        ]);
        sim.trial_boost_level.set_selector(4);
    }
    let selected_level: i32 = env
        .eval("return C_CharacterCreation.GetTrialBoostStartingLevel()")
        .expect("host-selected row level should be queryable");
    assert_eq!(selected_level, 71, "the first row for selector 4 wins");

    env.state().borrow_mut().trial_boost_level.set_selector(99);
    let unmatched_level: i32 = env
        .eval("return C_CharacterCreation.GetTrialBoostStartingLevel()")
        .expect("unmatched selector should return a level");
    assert_eq!(unmatched_level, 0, "an unmatched selector returns zero");
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_trial_boost_zero_fallback_is_unchanged() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let game_level: i32 = env
        .eval("return C_CharacterCreation.GetTrialBoostStartingLevel()")
        .expect("existing PTR fallback should remain callable");
    env.set_screen_mode(ScreenKind::CharacterCreate);
    let glue_level: i32 = env
        .eval("return C_CharacterCreation.GetTrialBoostStartingLevel()")
        .expect("existing PTR fallback should remain callable on Glue");

    assert_eq!(game_level, 0);
    assert_eq!(glue_level, 0);
}
#[test]
#[cfg(feature = "client-ptr")]
fn ptr_character_creation_camera_zoom_setter_verified_inputs_and_clamp() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    for arguments in ["", "nil"] {
        let error = env
            .exec(&format!(
                "C_CharacterCreation.SetCameraZoomLevel({arguments})"
            ))
            .expect_err("zoomLevel is required");
        assert!(
            error.to_string().contains(
                "Usage: C_CharacterCreation.SetCameraZoomLevel(zoomLevel [, keepCustomZoom])"
            ),
            "{error}"
        );
    }
    for zoom in ["2147483648", "-2147483649"] {
        let error = env
            .exec(&format!("C_CharacterCreation.SetCameraZoomLevel({zoom})"))
            .expect_err("zoomLevel must fit signed int32");
        assert!(
            error
                .to_string()
                .contains("outside of expected range -2147483648 to 2147483647"),
            "{error}"
        );
    }
    for (input, expected) in [(17.9, 17), (-17.9, 0), (150.0, 100), (51.7, 51)] {
        env.exec(&format!("C_CharacterCreation.SetCameraZoomLevel({input})"))
            .unwrap();
        assert_eq!(env.state().borrow().character_create_camera_zoom, expected);
    }
    for keep in ["", ", nil", ", false", ", true"] {
        {
            let mut sim = env.state().borrow_mut();
            sim.character_create_camera_zoom = 63;
            sim.character_create_camera_zoom_is_custom = true;
        }
        env.exec(&format!("C_CharacterCreation.SetCameraZoomLevel(24{keep})"))
            .unwrap();
        assert_eq!(
            env.state().borrow().character_create_camera_zoom,
            if keep == ", true" { 63 } else { 24 }
        );
    }
    // No catalog, numeric-string/secret, truthy non-boolean, or return-count claim.
}

#[test]
#[cfg(feature = "client-ptr")]
fn ptr_character_creation_camera_zoom_marker_changes_only_with_zoom() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::CharacterCreate);
    // R83c #320: equal, changed, and explicitly preserved custom zoom.
    for (requested, keep, expected_zoom, expected_custom) in [
        (63, false, 63, true),
        (24, false, 24, false),
        (24, true, 63, true),
    ] {
        {
            let mut sim = env.state().borrow_mut();
            sim.character_create_camera_zoom = 63;
            sim.character_create_camera_zoom_is_custom = true;
        }
        env.exec(&format!(
            "C_CharacterCreation.SetCameraZoomLevel({requested}, {keep})"
        ))
        .unwrap();
        let sim = env.state().borrow();
        assert_eq!(sim.character_create_camera_zoom, expected_zoom);
        assert_eq!(sim.character_create_camera_zoom_is_custom, expected_custom);
    }
}

#[test]
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn character_creation_category_zoom_through_original_consumer() {
    test_timeout! {
        use wow_ui_sim::c_api::c_character_creation_customizations::{Category, PublishedValue as V};
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        // Explicit host scenario, not a collector or native category-ID claim.
        // R83d requires every host record to supply signed32 zoom explicitly.
        env.state().borrow_mut().character_customizations = Some(
            [-1, 0, 37, 101, 73]
                .into_iter().enumerate().map(|(index, zoom)| Category {
                    stored_order_key: index as i64,
                    fields: vec![
                        ("id".into(), V::Number(100.0 + index as f64)),
                        ("name".into(), V::Text(format!("Host category {index}").into_bytes())),
                        ("subcategory".into(), V::Boolean(false)),
                        ("undressModel".into(), V::Boolean(false)),
                        ("needsNativeFormCategory".into(), V::Boolean(false)),
                    ],
                    camera_zoom_level: zoom,
                    camera_distance_offset: Some(0.0),
                    ..Category::new(zoom)
                }).collect()
        );
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(r#"
            GlueParent_SetScreen("charcreate")
            -- Original customize-entry setup: Retail CharacterCreate.lua:540-544,
            -- Forever Camelot CharacterCreate.lua:637-641; bounded scope approved #712.
            local RaceAndClassFrame = CharacterCreateFrame.RaceAndClassFrame
            assert(RaceAndClassFrame.selectedRaceData)
            CharCustomizeFrame:SetSelectedData(RaceAndClassFrame.selectedRaceData,
                RaceAndClassFrame.selectedSexID, C_CharacterCreation.IsViewingAlteredForm())
            CharacterCreateFrame:UpdateCharCustomizationFrame(true)
            local categories = CharCustomizeFrame:GetCategories()
            assert(#categories == 5, "original consumer must receive seeded categories")
            local inputs = {-1, 0, 37, 101, 73}
            local expected = {0, 0, 37, 100, 73}
            for index, category in ipairs(categories) do
                assert(type(category.cameraZoomLevel) == "number",
                    "R83d: every published category must have numeric cameraZoomLevel")
                assert(category.cameraZoomLevel == inputs[index])
                CharCustomizeFrame:SetSelectedSubcategory(nil)
                CharCustomizeFrame:SetSelectedCategory(category, false)
                local zoom = CharacterCreateFrame:GetCurrentCameraZoom()
                assert(zoom == expected[index], "consumer must apply the 0-100 clamp")
                assert(CharCustomizeFrame.SmallButtons.ZoomInButton:IsEnabled() == (zoom < 100))
                assert(CharCustomizeFrame.SmallButtons.ZoomOutButton:IsEnabled() == (zoom > 0))
            end
        "#).expect("R83d original category consumer must publish numeric zoom and clamp it");
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before,
            "original category consumer must not introduce Lua errors");
    }
}
