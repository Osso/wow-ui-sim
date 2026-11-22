use crate::common;

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
use wow_ui_sim::c_api::c_character_creation::{StartingZoneChoiceInput, StartingZoneChoicesInput};
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
use wow_ui_sim::c_api::store_catalog_model::{StoreProductInput, StoreProductSharedData};
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
#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
fn retail_character_services_consumer_uses_test_display_provider() {
    test_timeout! {
        let ui = common::blizzard_addon_harness::blizzard_ui_dir();
        let glue_dir = ui.join("Blizzard_GlueXML");
        let toc_path = wow_ui_sim::loader::find_toc_file(&glue_dir)
            .expect("GlueXML has a selected profile TOC");
        let toc = wow_ui_sim::toc::TocFile::from_file(&toc_path)
            .expect("selected GlueXML TOC parses");
        assert!(toc.allows_screen(ScreenKind::CharacterSelect));
        assert!(toc.files.iter().any(|file| {
            file.file_name().is_some_and(|name| name == "CharacterServices.lua")
        }));

        common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape(
            &["Blizzard_GlueXML"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_GlueXML"));
                env.exec(
                    r#"
                    local info = debug.getinfo(GetAvailableBoostTypesForCharacterByGUID, "S")
                    assert(info and info.source:find("CharacterServices.lua", 1, true))

                    local displayData = {
                        level = 80,
                        flowTitle = "Fixture Upgrade",
                        iconTextureKit = "FixtureUpgrade",
                    }
                    C_SharedCharacterServices.GetUpgradeDistributions = function()
                        return { [5] = { amount = 1 } }
                    end
                    C_CharacterServices.GetCharacterServiceDisplayData = function(boostType)
                        assert(boostType == 5)
                        return displayData
                    end
                    GetBasicCharacterInfo = function(guid)
                        assert(guid == "fixture-character")
                        return { experienceLevel = 60 }
                    end
                    GetServiceCharacterInfo = function(guid)
                        assert(guid == "fixture-character")
                        return {
                            boostInProgress = false,
                            isTrialBoost = false,
                            isRevokedCharacterUpgrade = false,
                            vasServiceInProgress = false,
                        }
                    end

                    local function texture()
                        return { SetAtlas = function(self, atlas) self.atlas = atlas end }
                    end
                    local function noop() end
                    characterServiceButton = {
                        Icon = texture(),
                        Highlight = { Icon = texture() },
                        Timer = { StartTimer = noop, StopTimer = noop },
                        Ring = { Show = noop, Hide = noop },
                        Number = { Show = noop, Hide = noop, SetText = noop },
                        SetAlpha = noop,
                        Show = noop,
                    }
                    CharacterSelect.VASPools = {
                        Acquire = function(_, template)
                            assert(template == "CharacterBoostTemplate")
                            return characterServiceButton
                        end,
                        GetNumActive = function() return 1 end,
                    }
                    "#,
                )
                .expect("original character-service consumers should load with test providers");

                let (available_count, boost_type, blocked_count, title, atlas):
                    (i32, i32, i32, String, String) = env
                    .eval(
                        r#"
                        local available = GetAvailableBoostTypesForCharacterByGUID("fixture-character")
                        local available_count, boost_type = #available, available[1]
                        C_CharacterServices.GetCharacterServiceDisplayData(5).level = 60
                        local blocked = GetAvailableBoostTypesForCharacterByGUID("fixture-character")
                        C_CharacterServices.GetCharacterServiceDisplayData(5).level = 80
                        CharacterServicesMaster_UpdateBoostButtons(
                            { { isVAS = false, serviceID = 5 } },
                            { [5] = { amount = 1 } }
                        )
                        return available_count, boost_type, #blocked,
                            CharacterVASMixin.NarrationGetName(characterServiceButton),
                            characterServiceButton.Icon.atlas
                        "#,
                    )
                    .expect("original character-service consumers should read the supplied fields");

                assert_eq!(available_count, 1);
                assert_eq!(boost_type, 5);
                assert_eq!(blocked_count, 0, "the original filter must use displayData.level");
                assert_eq!(title, "Fixture Upgrade");
                assert_eq!(atlas, "FixtureUpgrade-small");

                env.exec(
                    "C_CharacterServices.GetCharacterServiceDisplayData = function() return nil end",
                )
                .expect("nil-provider fixture should install");
                let nil_consumer = env
                    .exec("GetAvailableBoostTypesForCharacterByGUID('fixture-character')");
                assert!(
                    nil_consumer.is_err(),
                    "the original consumer dereferences flowData.level when a boost exists"
                );
            },
        );
    }
}

#[test]
#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
fn retail_catalog_shop_owner_precedes_character_select_toggle() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let (owner_loaded, original_inbound): (bool, bool) = env
            .eval(
                r#"
                local info = debug.getinfo(CatalogShopInboundInterface.SetShown, "S")
                return C_AddOns.IsAddOnLoaded("Blizzard_CatalogShop"),
                    info ~= nil and info.source:find("Blizzard_CatalogShop_Inbound.lua", 1, true) ~= nil
                "#,
            )
            .expect("the selected CatalogShop inbound owner should be queryable");
        assert!(owner_loaded);
        assert!(original_inbound);

        env.set_server_mirrored_cvar("shop2Enabled", "1")
            .expect("Shop2 should be enabled for this fixture");
        env.exec(
            r#"
            CharacterSelectUtil.IsStoreAvailable = function() return true end
            ToggleStoreUI("character-select-probe")
            "#,
        )
        .expect("original CharacterSelect toggle should call the CatalogShop inbound owner");

        let (action, context_key): (String, String) = env
            .eval(
                "return CatalogShopFrame:GetAttribute('action'), CatalogShopFrame:GetAttribute('contextkey')",
            )
            .expect("CatalogShop action attributes should be readable");
        assert_eq!(action, "Show");
        assert_eq!(context_key, "character-select-probe");
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

#[cfg(feature = "client-wowforever")]
fn seed_name_reservation_consumer_inputs(env: &WowLuaEnv) {
    env.exec(
        r#"
        CharacterSelectUtil.IsAccountLocked = function() return false end
        GetNumCharacters = function() return 1 end
        CharacterSelect.undeleting = false
        CharacterSelect.hasPendingTrialBoost = false
        AccountReactivationInProgressDialog = { IsShown = function() return false end }
        GoldReactivateConfirmationDialog = { IsShown = function() return false end }
        TokenReactivateConfirmationDialog = { IsShown = function() return false end }
        CharSelectServicesFlowFrame = { ShouldDisableButtons = function() return false end }
        Kiosk = { IsEnabled = function() return false end }
        GetCharacterSelection = function() return 1 end
        GetCharacterGUID = function() return "Player-0000-0000-0000-000000000001" end
        GetServiceCharacterInfo = function()
            return { isExpansionTrialCharacter = false, isTrialBoost = false,
                boostInProgress = false, isRevokedCharacterUpgrade = false }
        end
        IsCharacterTimerunning = function() return false end
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_name_reservation_blocks_original_character_select_consumer() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        let original_consumer: bool = env
            .eval(
                r#"local info = debug.getinfo(CharacterSelect_AllowedToEnterWorld, "S")
                return info and info.source:find("Mainline/CharacterSelect.lua", 1, true) ~= nil"#,
            )
            .unwrap();
        assert!(original_consumer, "name reservation must run through the selected Blizzard consumer");
        seed_name_reservation_consumer_inputs(&env);
        wow_ui_sim::lua_api::globals::real::glue_feature_status::deliver_status(
            &env,
            wow_ui_sim::lua_api::globals::real::glue_feature_status::GlueFeatureStatus {
                name_reservation_only: true,
            },
        )
        .unwrap();
        let can_enter_world: bool = env.eval("return CharacterSelect_AllowedToEnterWorld()").unwrap();
        assert!(!can_enter_world, "VERIFIED name-reservation state must block the original CharacterSelect consumer");
    }
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

// INFERRED (R88ak 5.5.4.70631 applied to the 5.5.4.70268 cache): the selected
// Classic GlueGameLogo consumer exercises the verified getter and event facts.
#[cfg(feature = "client-mists")]
#[test]
fn mists_glue_game_logo_reads_current_mode_host_input() {
    let env = load_blizzard_screen(ScreenKind::CharacterSelect);
    assert_mists_game_logo_starts_without_current_mode(&env);
    assert_mists_game_logo_refreshes_from_current_mode(&env);
}

#[cfg(feature = "client-mists")]
fn assert_mists_game_logo_starts_without_current_mode(env: &common::LockedEnv) {
    let errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|error| error.contains("GetCurrentGameModeRecordID"))
        .cloned()
        .collect();
    assert!(
        errors.is_empty(),
        "GlueGameLogo consumer errors: {errors:#?}"
    );

    let (returns, getter_is_nil, logo_is_nil): (i32, bool, bool) = env
        .eval(
            r#"
            return select('#', C_GameRules.GetCurrentGameModeRecordID()),
                C_GameRules.GetCurrentGameModeRecordID() == nil,
                CharacterSelectLogo.gameModeRecordID == nil
            "#,
        )
        .expect("Mists GlueGameLogo current-mode getter should be callable");
    assert_eq!(returns, 1);
    assert!(getter_is_nil && logo_is_nil);
}

#[cfg(feature = "client-mists")]
fn assert_mists_game_logo_refreshes_from_current_mode(env: &common::LockedEnv) {
    env.exec(
        r##"
        gameModeUpdates = 0
        gameModeUpdateArgs = -1
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("GAME_MODE_DISPLAY_INFO_UPDATED")
        listener:SetScript("OnEvent", function(_, _, ...)
            gameModeUpdates = gameModeUpdates + 1
            gameModeUpdateArgs = select("#", ...)
        end)
        SetCVar("currentGameMode", "17")
        assert(select("#", C_GameRules.GetCurrentGameModeRecordID()) == 1)
        assert(C_GameRules.GetCurrentGameModeRecordID() == 17)
        assert(CharacterSelectLogo.gameModeRecordID == 17)
        assert(gameModeUpdates == 1 and gameModeUpdateArgs == 0)
        SetCVar("currentGameMode", "-1")
        assert(C_GameRules.GetCurrentGameModeRecordID() == nil)
        assert(CharacterSelectLogo.gameModeRecordID == nil)
        assert(gameModeUpdates == 2 and gameModeUpdateArgs == 0)
        "##,
    )
    .expect("Mists GlueGameLogo should refresh from the current-mode host input");
}

// UNKNOWN (Mists 5.5.4.70268 UI outcome): Classic CharacterSelect calls
// GetCurrentGameModeDisplayInfo at line 2845; R88ak does not establish its host data or screen effect.
#[cfg(not(feature = "client-mists"))]
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

// VERIFIED (Mists cache 5.5.4.70268): Classic CharacterSelect uses
// CharacterSelectCharacterFrame with a ScrollBar, not the Mainline ScrollBox.
#[cfg(not(feature = "client-mists"))]
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
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn character_select_configuration_warning_popups_track_seen_records() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        exercise_configuration_warning_popups(&env);
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn exercise_configuration_warning_popups(env: &common::LockedEnv) {
    env.exec(
        r#"
        A_Admin.SetConfigurationWarnings({
            { warning = 3, text = "Graphics driver is blocked" },
            { warning = 4, text = "Graphics driver is out of date" },
        })
        CheckSystemRequirements(false)
        StaticPopup_CheckQueuedDialogs()
        local first = StaticPopup_FindVisible("CONFIGURATION_WARNING")
        assert(first and first:IsShown(), "first configuration warning should show")
        assert(first:GetTextFontString():GetText() == "Graphics driver is blocked")
        assert(first.data.configurationWarning == 3)
        first.button1:Click()
        StaticPopup_CheckQueuedDialogs()
        local second = StaticPopup_FindVisible("CONFIGURATION_WARNING")
        assert(second and second:IsShown(), "second configuration warning should show")
        assert(second:GetTextFontString():GetText() == "Graphics driver is out of date")
        assert(second.data.configurationWarning == 4)
        second.button1:Click()
        StaticPopup_CheckQueuedDialogs()
        CheckSystemRequirements(false)
        StaticPopup_CheckQueuedDialogs()
        assert(StaticPopup_FindVisible("CONFIGURATION_WARNING") == nil)
        CharacterSelect_CheckDialogStates()
        assert(CharacterSelectUI.VisibilityFramesContainer.ConfigurationWarnings:IsShown())
        "#,
    )
    .expect("original warning popup consumers should preserve and consume warning state");
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

const RETAIL_CHARACTER_CREATE_PRESERVATION_LUA: &str = r#"
local source = "Blizzard_CharacterCreate/"
assert(debug.getinfo(CharacterCreateMixin.OnLoad, "S").source:find(source, 1, true), "OnLoad source")
assert(debug.getinfo(CharacterCreateMixin.UpdateForwardButton, "S").source:find(source, 1, true), "forward source")
assert(debug.getinfo(CharacterCreateRaceAndClassMixin.SetCharacterRace, "S").source:find(source, 1, true), "race source")
local frame = CharacterCreateFrame
local race = frame.RaceAndClassFrame
local overlayKeys = { "TopBackgroundOverlay", "LeftBackgroundOverlay", "RightBackgroundOverlay", "BottomBackgroundOverlay" }
assert(#frame.BGTex == #overlayKeys, "XML BGTex array count: " .. tostring(#frame.BGTex))
for index, key in ipairs(overlayKeys) do
    local overlay = frame[key]
    assert(overlay ~= nil, "missing XML overlay " .. key)
    assert(rawequal(frame.BGTex[index], overlay), "BGTex order/identity at " .. index)
    assert(overlay:GetObjectType() == "Texture", "XML BGTex element type at " .. index)
    for previous = 1, index - 1 do
        assert(not rawequal(frame.BGTex[index], frame.BGTex[previous]), "duplicate BGTex member at " .. index)
    end
end
local function assertOverlayAlpha(expected, state)
    for index = 1, #overlayKeys do
        assert(math.abs(frame.BGTex[index]:GetAlpha() - expected) < 0.001, state .. " overlay alpha at " .. index)
    end
end
race:SetCharacterClass(1)
race:SetCharacterRace(1)
assert(race.selectedRaceData.fileName == "Human", "selected Human record")
assert(race.selectedFaction == "Alliance", "Human must select Alliance")
race:SetCharacterRace(2)
assert(race.selectedRaceData.fileName == "Orc", "selected Orc record")
assert(race.selectedFaction == "Horde", "Orc must select Horde")
race:SetCharacterRace(24)
assert(race.selectedRaceData.fileName == "Pandaren", "selected Pandaren record")
assertOverlayAlpha(0.75, "Pandaren")
race:SetCharacterRace(2)
assertOverlayAlpha(0.6, "Horde Orc")
race:SetCharacterClass(12)
assert(race.selectedClassData.fileName == "DEMONHUNTER", "selected Demon Hunter record")
assertOverlayAlpha(0.7, "Demon Hunter")
race:SetCharacterClass(6)
assert(race.selectedClassData.fileName == "DEATHKNIGHT", "selected Death Knight record")
assertOverlayAlpha(0.8, "Death Knight")
local backSuffix = "  " .. BACK
local backText = frame.BackButton:GetText()
assert(backText:sub(-#backSuffix) == backSuffix, "Back must retain the original BACK string")
assert(backText:find("common-icon-backarrow", 1, true), "Back must use the original back arrow")
local classRaceMode = frame.currentMode
assert(classRaceMode == 1, "original OnLoad must select class/race mode")
local classRaceForward = frame.ForwardButton:GetText()
assert(classRaceForward:sub(1, #CUSTOMIZE) == CUSTOMIZE, "class/race mode must use original CUSTOMIZE string")
assert(classRaceForward:find("common-icon-forwardarrow", 1, true), "class/race mode must use the original forward arrow")
-- CharacterCreate.lua defines class/race as 1 and customization as 2; exercise the original label updater without its camera transition.
local customizeMode = classRaceMode + 1
assert(customizeMode == 2, "source-defined mode order")
frame.currentMode = customizeMode
frame:UpdateForwardButton()
local customizeForward = frame.ForwardButton:GetText()
local customizeLabel = frame.ZoneChoiceFrame:ShouldShow() and NEXT or FINISH
assert(customizeForward:sub(1, #customizeLabel) == customizeLabel, "customization mode must use its original forward string")
assert(customizeLabel ~= CUSTOMIZE and customizeForward ~= classRaceForward, "forward label must change with the original mode")
local customizeBack = frame.BackButton:GetText()
assert(customizeBack:sub(-#backSuffix) == backSuffix, "Back label remains the original BACK string")
assert(customizeBack:find("common-icon-backarrow", 1, true), "customization mode must retain the original back arrow")
frame.currentMode = classRaceMode
frame:UpdateForwardButton()
assert(frame.currentMode == classRaceMode, "restore original class/race mode input")
assert(frame.ForwardButton:GetText():sub(1, #CUSTOMIZE) == CUSTOMIZE, "class/race forward string must be restored")
"#;

#[test]
#[cfg(feature = "client-retail")]
fn retail_character_create_transition_preserves_original_surfaces() {
    test_timeout! {
        let ui = blizzard_ui_dir();
        let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::CharacterSelect);
        let toc_path = addons.iter().find(|(name, _)| name == "Blizzard_CharacterCreate")
            .map(|(_, path)| path).expect("Glue should select CharacterCreate");
        let toc = wow_ui_sim::toc::TocFile::from_file(toc_path).expect("Glue TOC should parse");
        assert!(toc.allows_screen(ScreenKind::CharacterSelect));
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        env.exec("GlueParent_SetScreen('charcreate')").expect("original screen transition should run");
        run_extra_update_ticks(&env, 3);
        env.exec(RETAIL_CHARACTER_CREATE_PRESERVATION_LUA).expect("original consumer state should be preserved");
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
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn retail_character_creation_default_starting_zone_reaches_customize_screen() {
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
#[cfg(all(feature = "optional-tests", feature = "client-wowforever"))]
fn forever_character_creation_default_starting_zone_reaches_customize_screen() {
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
        let (self_found_allowed, return_count): (bool, i64) = env
            .eval(r##"
                local function pack(...) return { n = select("#", ...), ... } end
                local result = pack(C_GameRules.IsSelfFoundAllowed())
                return result[1], result.n
            "##)
            .unwrap();
        assert_eq!((self_found_allowed, return_count), (false, 1));
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
fn retail_character_creation_two_starting_zone_choices_reach_forward_consumer() {
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
    }
}

#[test]
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn retail_character_creation_npe_restriction_skips_zone_choice() {
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
        "##).unwrap();
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

// UNKNOWN (Mists 5.5.4.70268 native consumer): this test expects
// `CharacterCreateFrame.NameChoiceFrame`, which R88ak does not establish.
#[cfg(not(feature = "client-mists"))]
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

// UNKNOWN (Mists 5.5.4.70268): this probe targets
// C_CharacterCreation.SetInCharacterCreate; the selected Classic Lua uses the
// legacy global SetInCharacterCreate instead.
#[cfg(not(feature = "client-mists"))]
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

// VERIFIED (Mists cache 5.5.4.70268): Classic CharacterCreate names this
// control `CharCreateBackButton`, not `CharacterCreateFrame.BackButton`.
#[cfg(not(feature = "client-mists"))]
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

#[test]
#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
fn retail_glue_tooltip_preserves_shared_nineslice() {
    test_timeout! {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        env.exec(r#"
            -- Mainline/GlueTooltip.xml:3,18 inherits SharedTooltipTemplate
            -- and SharedTooltipTemplates.xml:19 supplies the NineSlice.
            local source = debug.getinfo(SharedTooltip_SetBackdropStyle, 'S')
            assert(source.source:gsub('\\', '/'):find('Blizzard_SharedXML/SharedTooltipTemplates.lua', 1, true))
            assert(source.linedefined == 47)
            local child = assert(GlueTooltip.NineSlice)
            assert(child:GetParent() == GlueTooltip)
            assert(child:IsUsingParentLevel() and child:GetFrameLevel() == GlueTooltip:GetFrameLevel())
            SharedTooltip_SetBackdropStyle(GlueTooltip, nil, true)
            assert(not child:IsShown())
            SharedTooltip_SetBackdropStyle(GlueTooltip, nil, false)
            assert(child:IsShown() and GlueTooltip.NineSlice == child)
        "#).expect("original GlueTooltip backdrop should use its inherited NineSlice");
        assert!(!env.state().borrow().lua_errors.iter().any(|error| error.contains("SharedTooltipTemplates.lua")));
    }
}

// INFERRED applicability only; Retail consumer/assertion/input semantics are unchanged.
#[cfg(feature = "optional-tests")]
fn r88q_profile_active() -> bool {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}

// R88q original-consumer regression; inputs are explicit host fixtures.
#[cfg(all(
    feature = "optional-tests",
    any(
        feature = "profile-retail",
        feature = "client-ptr",
        feature = "client-wowforever"
    )
))]
mod storeglue_r88q_tuple {
    use super::*;

    fn env_with_records() -> common::LockedEnv {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        seed_purchase_records(&env);
        seed_r88q_product_inputs(&env);
        env
    }

    // INFERRED host input supplies two distinct states to the selected consumer.
    fn seed_purchase_records(env: &WowLuaEnv) {
        env.exec(
            r#"
            A_Admin.SetStoreGluePurchaseState('Player-1-a1', 2003, 3, {17, 0, -4})
            A_Admin.SetStoreGluePurchaseState('Player-2-b2', 189, 4, {0, 29})
        "#,
        )
        .unwrap();
    }

    #[test]
    fn purchase_tuple_and_error_sentinel_reach_original_glue_consumer() {
        test_timeout! {
            let env = env_with_records();
            env.exec(r#"
                local function check(guid, productID, expectedState, expectedErrors)
                    local consumerState, consumerErrors, consumerProduct =
                        CharacterSelectListUtil.GetVASInfoForGUID(guid)
                    local product, state, errors = C_StoreGlue.GetVASPurchaseStateInfo(guid)
                    assert(select('#', C_StoreGlue.GetVASPurchaseStateInfo(guid)) == 3)
                    assert(product == productID and state == expectedState)
                    assert(type(errors) == 'table' and #errors == #expectedErrors)
                    for i, value in ipairs(expectedErrors) do assert(errors[i] == value) end
                    assert(consumerState == expectedState and #consumerErrors == #expectedErrors)
                    for i, value in ipairs(expectedErrors) do assert(consumerErrors[i] == value) end
                    if productID ~= 0 then assert(consumerProduct.productID == productID) end
                end
                check('Player-1-a1', 2003, 3, {17,0,-4})
                check('Player-2-b2', 189, 4, {})
                check('Player-3-c3', 0, 0, {})
            "#).expect("R88q.q1 stored tuple/errors must reach original consumer");
        }
    }
}

// Host input structs are simulator seams, not claims about native packet names
// or record acquisition. Tests invoke the original selected Glue consumers.
#[cfg(feature = "optional-tests")]
mod storeglue_r88q_host {
    use super::r88q_routes::{
        self as c_store_glue, CompletionInput, VasPurchaseRecord, VasQueueCharacter,
    };
    use super::*;

    const A: &str = "Player-1-a1";
    const B: &str = "Player-2-b2";
    const ABSENT: &str = "Player-3-c3";

    fn record(product_id: i32, state: i32, errors: &[i32]) -> VasPurchaseRecord {
        VasPurchaseRecord {
            product_id,
            state,
            errors: errors.to_vec(),
        }
    }

    fn fixture() -> common::LockedEnv {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        seed_r88q_product_inputs(&env);
        c_store_glue::seed(
            &env,
            vec![
                (A.into(), record(2003, 3, &[17, 0, -4])),
                (B.into(), record(189, 4, &[0, 29])),
            ],
            vec![
                VasQueueCharacter {
                    guid: A.into(),
                    correlation: vec![21],
                },
                VasQueueCharacter {
                    guid: B.into(),
                    correlation: vec![43],
                },
            ],
            &[A, B, ABSENT],
        );
        assert_original_owners(&env);
        env
    }

    fn assert_records(env: &WowLuaEnv) {
        env.exec(R88Q_LUA_03).unwrap();
    }

    fn install_listener(env: &WowLuaEnv) {
        c_store_glue::install_callbacks(env);
        env.exec(R88Q_LUA_04).unwrap();
    }

    fn assert_original_owners(env: &WowLuaEnv) {
        env.exec(R88Q_LUA_05).unwrap();
    }

    #[test]
    fn shared_boolean_writers_drive_original_glue_update() {
        test_timeout! {
            if !r88q_profile_active() { return; }
            let env = fixture();
            assert_records(&env);
            assert_original_owners(&env);
            env.exec(R88Q_LUA_06).unwrap();
            assert_records(&env);
            refresh_request_controls(&env);
        }
    }

    // q5 request-side assertions are separate from response-event expectations.
    fn refresh_request_controls(env: &WowLuaEnv) {
        let before = c_store_glue::submission_count(env);
        env.exec("assert(select('#', C_StoreGlue.UpdateVASPurchaseStates(nil, false, {})) == 0)")
            .unwrap();
        assert_eq!(c_store_glue::submission_count(env), before + 1);
        c_store_glue::set_connection(env, false);
        env.exec("C_StoreGlue.UpdateVASPurchaseStates()").unwrap();
        assert_eq!(c_store_glue::submission_count(env), before + 1);
        assert_records(env); // Request did not synchronously mutate either record or fabricate absence.
    }

    fn make_queue_rows(env: &WowLuaEnv) {
        env.exec(R88Q_LUA_07).unwrap();
    }

    #[test]
    fn queue_requests_and_responses_drive_original_rows_and_event_owner() {
        test_timeout! {
            if !r88q_profile_active() { return; }
            let env = fixture();
            assert_records(&env);
            install_listener(&env);
            make_queue_rows(&env);
            let request_counts = queue_requests(&env);
            c_store_glue::receive_queue_response(&env, &[99], 91).unwrap();
            c_store_glue::receive_queue_response(&env, &[21], -17).unwrap();
            c_store_glue::receive_queue_response(&env, &[43], 29).unwrap();
            env.exec(R88Q_LUA_08).unwrap();
            assert_eq!(request_counts, [2, 2, 3, 3, 3]);
        }
    }

    fn queue_requests(env: &WowLuaEnv) -> [u64; 5] {
        env.exec(R88Q_LUA_09).unwrap();
        let before = [
            c_store_glue::request_sequence(env),
            c_store_glue::submission_count(env) as u64,
        ];
        c_store_glue::set_record_state(env, A, 4);
        env.exec("R88qRows['Player-1-a1']:UpdateCharacterDisplayInfo()")
            .unwrap();
        let after = [
            c_store_glue::request_sequence(env),
            c_store_glue::submission_count(env) as u64,
        ];
        env.exec("C_StoreGlue.RequestCharacterQueueTime('Player-3-c3')")
            .unwrap();
        [
            before[0],
            before[1],
            after[0],
            after[1],
            c_store_glue::request_sequence(env),
        ]
    }

    #[test]
    fn bulk_receive_replaces_before_original_glue_event_observation() {
        test_timeout! {
            if !r88q_profile_active() { return; }
            let env = fixture();
            assert_records(&env);
            install_listener(&env);
            c_store_glue::receive_bulk(&env, vec![
                (A.into(), record(189, 6, &[31, 0])),
                (B.into(), record(2003, 2, &[-23])),
            ]).unwrap();
            env.exec(R88Q_LUA_10).unwrap();
            c_store_glue::receive_bulk(&env, vec![]).unwrap();
            env.exec(R88Q_LUA_11).unwrap();
        }
    }

    #[test]
    fn removal_copies_separate_errors_before_post_removal_state_notification() {
        test_timeout! {
            if !r88q_profile_active() { return; }
            let env = fixture();
            assert_records(&env);
            install_listener(&env);
            env.exec(R88Q_REMOVAL_BEFORE).unwrap();
            c_store_glue::receive_removal(&env, A, vec![51, 0]).unwrap();
            assert_eq!(c_store_glue::purchase_errors(&env), vec![51, 0]);
            env.exec(R88Q_LUA_12).unwrap();
            c_store_glue::receive_removal(&env, B, vec![-37]).unwrap();
            assert_eq!(c_store_glue::purchase_errors(&env), vec![-37]);
            env.exec(R88Q_LUA_13).unwrap();
            c_store_glue::receive_removal(&env, ABSENT, vec![88]).unwrap();
            // VERIFIED R88w.w4 supersedes the old neutral copy-eligibility input.
            assert_eq!(c_store_glue::purchase_errors(&env), vec![88]);
            env.exec("assert(#R88qEvents == 2)").unwrap();
        }
    }

    fn completion(first: i32, second: i32, first_flag: bool, second_flag: bool) -> CompletionInput {
        CompletionInput {
            first,
            second,
            first_flag,
            second_flag,
            data: vec![],
        }
    }

    fn completion_gate_cases(env: &WowLuaEnv) {
        for input in [
            completion(0, 0, true, false),
            completion(1, 9, true, false),
            completion(1, 0, false, false),
            completion(1, 0, true, true),
        ] {
            env.exec("C_StoreSecure.SetVASProductReady(false)").unwrap();
            c_store_glue::receive_completion(env, input).unwrap();
        }
        c_store_glue::receive_completion(env, completion(7, 0, true, false)).unwrap();
        c_store_glue::receive_completion(env, completion(0, 0, true, false)).unwrap();
    }

    // Insert immediately after fixture()/assert_records, before listener/receive helpers.
    // Initial readiness is captured, then the ORIGINAL consumer runs before assertions.
    // This does not call ClearVASProductReady or replace any API/consumer function.
    fn exercise_initial_completion_consumer(env: &WowLuaEnv) {
        env.exec(
            r#"
            local initialReady = C_StoreGlue.GetVASProductReady()
            CharacterServicesMaster.pendingGuid = 'initial-not-ready'
            CharacterSelectFrameMixin.OnUpdate(CharacterSelect, 0)
            assert(not initialReady)
            assert(CharacterServicesMaster.pendingGuid == 'initial-not-ready')
        "#,
        )
        .unwrap();
    }

    #[test]
    fn completion_notification_is_zero_payload_and_not_proof_of_ready() {
        test_timeout! {
            if !r88q_profile_active() { return; }
            let env = fixture();
            assert_records(&env);
            exercise_initial_completion_consumer(&env);
            install_listener(&env);
            assert_records(&env);
            completion_gate_cases(&env);
            env.exec(R88Q_LUA_14).unwrap();
            assert_records(&env);
        }
    }
}

// Original consumer calls and assertions, unchanged while extracting constants.

#[cfg(feature = "optional-tests")]
const R88Q_LUA_03: &str = r#"
            local stateA, errorsA, productA = CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1')
            local stateB, errorsB, productB = CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2')
            local stateNone, errorsNone = CharacterSelectListUtil.GetVASInfoForGUID('Player-3-c3')
            assert(stateA == 3 and productA.productID == 2003 and #errorsA == 3)
            assert(errorsA[1] == 17 and errorsA[2] == 0 and errorsA[3] == -4)
            assert(stateB == 4 and productB.productID == 189 and #errorsB == 0)
            assert(stateNone == 0 and type(errorsNone) == 'table' and #errorsNone == 0)
        "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_04: &str = r#"
            R88qEvents = {}
            R88qListener = CreateFrame('Frame')
            for _, event in ipairs({'STORE_VAS_PURCHASE_COMPLETE', 'VAS_CHARACTER_QUEUE_STATUS_UPDATE',
                    'VAS_CHARACTER_STATE_CHANGED'}) do
                R88qListener:RegisterEvent(event)
            end
            R88qListener:SetScript('OnEvent', function(self, event, ...)
                local pa, sa, ea = C_StoreGlue.GetVASPurchaseStateInfo('Player-1-a1')
                local pb, sb, eb = C_StoreGlue.GetVASPurchaseStateInfo('Player-2-b2')
                local pn, sn, en = C_StoreGlue.GetVASPurchaseStateInfo('Player-3-c3')
                table.insert(R88qEvents, {event=event, argc=select('#',...), args={...},
                    pa=pa, sa=sa, ea=ea, pb=pb, sb=sb, eb=eb, pn=pn, sn=sn, en=en,
                    ready=C_StoreGlue.GetVASProductReady()})
            end)
        "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_05: &str = r#"
            for _, callback in ipairs({CharacterSelectFrameMixin.OnShow,
                    CharacterSelectFrameMixin.OnUpdate, CharacterSelectFrameMixin.OnEvent}) do
                assert(debug.getinfo(callback, 'S').source:find('CharacterSelect.lua', 1, true))
            end
            assert(debug.getinfo(CharacterSelectListUtil.GetVASInfoForGUID, 'S').source:
                find('CharacterSelectListUtil.lua', 1, true))
        "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_06: &str = r#"
                C_StoreSecure.SetDisconnectOnLogout(false)
                assert(not C_StoreGlue.GetDisconnectOnLogout())
                C_StoreSecure.SetDisconnectOnLogout(0, 'ignored')
                assert(C_StoreGlue.GetDisconnectOnLogout())
                C_StoreSecure.SetDisconnectOnLogout(false)
                assert(not C_StoreGlue.GetDisconnectOnLogout())
                C_StoreSecure.SetVASProductReady(false)
                CharacterServicesMaster.pendingGuid = 'untouched-when-not-ready'
                CharacterSelectFrameMixin.OnUpdate(CharacterSelect, 0)
                assert(CharacterServicesMaster.pendingGuid == 'untouched-when-not-ready')
                C_StoreSecure.SetVASProductReady('')
                local readyBeforeUpdate = C_StoreGlue.GetVASProductReady()
                CharacterSelectFrameMixin.OnUpdate(CharacterSelect, 0)
                assert(readyBeforeUpdate)
                assert(CharacterServicesMaster.pendingGuid == nil)
                assert(select('#', C_StoreGlue.ClearVASProductReady(false)) == 0)
                assert(not C_StoreGlue.GetVASProductReady())
            "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_07: &str = r#"
            R88qRows = {}
            for index, guid in ipairs({'Player-1-a1','Player-2-b2','Player-3-c3'}) do
                local row = CreateFrame('Button', nil, UIParent, 'CharacterSelectListCharacterTemplate')
                row.characterID = index -- Explicit UI fixture row identity, not GUID acquisition.
                local inner = row.InnerContent
                inner.characterInfo = {guid=guid, name='Fixture', className='Warrior', classFilename='WARRIOR',
                    experienceLevel=80, areaName='Fixture Zone'}
                inner.isAccountLocked = false
                R88qRows[guid] = inner
                assert(debug.getinfo(inner.UpdateCharacterDisplayInfo, 'S').source:
                    find('CharacterSelectListElements.lua', 1, true))
            end
        "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_08: &str = r#"
                assert(#R88qEvents == 2)
                for i, expected in ipairs({{'Player-1-a1',-17},{'Player-2-b2',29}}) do
                    local event = R88qEvents[i]
                    assert(event.event == 'VAS_CHARACTER_QUEUE_STATUS_UPDATE' and event.argc == 2)
                    assert(event.args[1] == expected[1] and event.args[2] == expected[2])
                    assert(CharacterSelectUtil.GetVASQueueTime(expected[1]) == expected[2])
                end
                assert(CharacterSelectUtil.GetVASQueueTime('Player-3-c3') == nil)
                local sa, ea, pa = CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1')
                local sb, eb, pb = CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2')
                local sn, en = CharacterSelectListUtil.GetVASInfoForGUID('Player-3-c3')
                assert(sa == 4 and pa.productID == 2003 and ea[1] == 17 and ea[2] == 0 and ea[3] == -4)
                assert(sb == 4 and pb.productID == 189 and #eb == 0)
                assert(sn == 0 and type(en) == 'table' and #en == 0)
            "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_09: &str = r#"
            R88qRows['Player-1-a1']:UpdateCharacterDisplayInfo()
            R88qRows['Player-2-b2']:UpdateCharacterDisplayInfo()
            R88qRows['Player-3-c3']:UpdateCharacterDisplayInfo()
            R88qRows['Player-2-b2']:UpdateCharacterDisplayInfo()
            assert(#R88qEvents == 0)
        "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_10: &str = r#"
                local stateA = CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1')
                local stateB = CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2')
                local stateAbsent = CharacterSelectListUtil.GetVASInfoForGUID('Player-3-c3')
                assert(#R88qEvents == 1 and R88qEvents[1].event == 'VAS_CHARACTER_STATE_CHANGED')
                local event = R88qEvents[1]
                assert(event.pa == 189 and event.sa == 6 and event.ea[1] == 31 and event.ea[2] == 0)
                assert(event.pb == 2003 and event.sb == 2 and event.eb[1] == -23)
                assert(event.pn == 0 and event.sn == 0 and #event.en == 0)
                assert(stateA == 6)
                assert(stateB == 2 and stateAbsent == 0)
            "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_11: &str = r#"
                assert(#R88qEvents == 2 and R88qEvents[2].event == 'VAS_CHARACTER_STATE_CHANGED')
                assert(R88qEvents[2].pa == 0 and R88qEvents[2].sa == 0 and #R88qEvents[2].ea == 0)
                assert(R88qEvents[2].pb == 0 and R88qEvents[2].sb == 0 and #R88qEvents[2].eb == 0)
            "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_12: &str = r#"
                assert(#R88qEvents == 1 and R88qEvents[1].event == 'VAS_CHARACTER_STATE_CHANGED')
                assert(R88qBeforeA[1] == 3 and R88qBeforeA[3].productID == 2003)
                assert(R88qBeforeA[2][1] == 17 and R88qBeforeA[2][2] == 0 and R88qBeforeA[2][3] == -4)
                assert(R88qBeforeB[1] == 4 and R88qBeforeB[3].productID == 189 and #R88qBeforeB[2] == 0)
                assert(R88qBeforeAbsent[1] == 0 and #R88qBeforeAbsent[2] == 0)
                local event = R88qEvents[1]
                assert(event.pa == 0 and event.sa == 0 and #event.ea == 0)
                assert(event.pb == 189 and event.sb == 4 and #event.eb == 0)
                assert(event.pn == 0 and event.sn == 0 and #event.en == 0)
            "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_13: &str = r#"
                assert(#R88qEvents == 2 and R88qEvents[2].event == 'VAS_CHARACTER_STATE_CHANGED')
                assert(R88qEvents[2].pa == 0 and R88qEvents[2].pb == 0)
            "#;

#[cfg(feature = "optional-tests")]
const R88Q_LUA_14: &str = r#"
                CharacterServicesMaster.pendingGuid = 'completion-consumer-probe'
                CharacterSelectFrameMixin.OnUpdate(CharacterSelect, 0)
                assert(#R88qEvents == 6)
                for _, event in ipairs(R88qEvents) do
                    assert(event.event == 'STORE_VAS_PURCHASE_COMPLETE' and event.argc == 0)
                end
                for i=1,4 do assert(not R88qEvents[i].ready) end
                assert(R88qEvents[5].ready and R88qEvents[6].ready)
                assert(CharacterServicesMaster.pendingGuid == nil)
            "#;

#[cfg(feature = "optional-tests")]
const R88Q_REMOVAL_BEFORE: &str = r#"
    R88qBeforeA = {CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1')}
    R88qBeforeB = {CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2')}
    R88qBeforeAbsent = {CharacterSelectListUtil.GetVASInfoForGUID('Player-3-c3')}
"#;

// INFERRED (simulator mapping): fixture product rows supply details to the
// original Glue consumer through C_StoreSecure.GetProductInfo. R88q.q1 proves
// that the consumer passes the product ID; it does not prove native acquisition.
#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
fn seed_r88q_product_inputs(env: &WowLuaEnv) {
    let products = [2003, 189].map(|product_id| StoreProductInput {
        product_id,
        supported: true,
        session_eligible: true,
        display_eligible: true,
        dynamic_bundle: false,
        product_type: None,
        shared_data: StoreProductSharedData::default(),
    });
    env.state()
        .borrow_mut()
        .store_glue
        .store_secure
        .catalog
        .products
        .extend(products);
}

// Other profiles still expose the compatibility bootstrap's Lua product table.
#[cfg(all(feature = "optional-tests", not(feature = "client-retail")))]
fn seed_r88q_product_inputs(env: &WowLuaEnv) {
    env.exec(
        "__wow_store_state.productsByID[189] = {productID=189, sharedData={name='Fixture B'}}",
    )
    .expect("seed explicit host product input for original Glue consumers");
}
#[cfg(feature = "optional-tests")]
mod r88q_routes {
    pub use super::r88q_receive::{
        receive_bulk, receive_completion, receive_queue_response, receive_removal,
    };
    use super::*;
    pub use wow_ui_sim::c_api::store_glue_model::{
        CompletionInput, VasPurchaseRecord, VasQueueCharacter,
    };
    pub fn seed(
        env: &WowLuaEnv,
        rows: Vec<(String, VasPurchaseRecord)>,
        queues: Vec<VasQueueCharacter>,
        _aliases: &[&str],
    ) {
        let mut sim = env.state().borrow_mut();
        let s = &mut sim.store_glue;
        s.purchase_states = rows.into_iter().collect();
        s.queue_characters = queues;
        s.opaque_guid_aliases.clear();
        s.connection_present = true;
        s.request_sequence = 0;
        s.submissions.clear();
    }
    pub fn install_callbacks(env: &WowLuaEnv) {
        let mut sim = env.state().borrow_mut();
        let c = &mut sim.store_glue.callbacks;
        c.purchase_complete = true;
        c.queue_status = true;
        c.state_changed = true;
    }
    pub fn request_sequence(env: &WowLuaEnv) -> u64 {
        env.state().borrow().store_glue.request_sequence
    }
    pub fn submission_count(env: &WowLuaEnv) -> usize {
        env.state().borrow().store_glue.submissions.len()
    }
    pub fn set_connection(env: &WowLuaEnv, present: bool) {
        env.state().borrow_mut().store_glue.connection_present = present;
    }
    pub fn set_record_state(env: &WowLuaEnv, guid: &str, state: i32) {
        env.state()
            .borrow_mut()
            .store_glue
            .purchase_states
            .get_mut(guid)
            .unwrap()
            .state = state;
    }
    pub fn purchase_errors(env: &WowLuaEnv) -> Vec<i32> {
        env.state().borrow().store_glue.purchase_errors.clone()
    }
}

#[cfg(feature = "optional-tests")]
mod r88q_receive {
    pub use wow_ui_sim::c_api::c_store_glue::{
        receive_bulk, receive_completion, receive_queue_response, receive_removal,
    };
}
// Supplemental direct API contract, not the frozen original-consumer RED selector.
// Each independent fixture proves two distinct original consumer records plus
// absence before exercising the supplementary contract.
#[cfg(feature = "optional-tests")]
mod storeglue_r88q_supplemental {
    use super::*;
    use rilua::LuaApiMut;
    use wow_ui_sim::c_api::store_glue_model::{RestrictedExecution, VasPurchaseRecord};

    fn active() -> bool {
        r88q_profile_active()
    }
    fn fixture() -> common::LockedEnv {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        seed_r88q_product_inputs(&env);
        seed(&env);
        env.exec(R88Q_CONTRACT_RECORDS).unwrap();
        env
    }
    fn seed(env: &WowLuaEnv) {
        let mut sim = env.state().borrow_mut();
        let s = &mut sim.store_glue;
        s.purchase_states.clear();
        for (guid, product_id, state, errors) in [
            ("Player-1-a1", 2003, 3, vec![17, 0, -4]),
            ("Player-2-b2", 189, 4, vec![0, 29]),
            ("Player-4-d4", -27, -9, vec![-8, 0, 9]),
        ] {
            s.purchase_states.insert(
                guid.into(),
                VasPurchaseRecord {
                    product_id,
                    state,
                    errors,
                },
            );
        }
        seed_aliases(s);
    }
    fn seed_aliases(s: &mut wow_ui_sim::c_api::store_glue_model::StoreGlueState) {
        // R88w: bare numeric text is not an opaque alias. No raw-text aliases.
        s.opaque_guid_aliases.clear();
        s.pending_error = None;
        s.restricted_execution = None;
        s.secret_access_mode = None;
    }
    #[test]
    fn supplemental_guid_arguments_signed_errors_and_dual_publication() {
        test_timeout! {
            if !active() { return; }
            let env=fixture();
            env.exec(R88Q_CONTRACT_ARGUMENTS).unwrap();
            env.exec(R88Q_CONTRACT_SIGNED_PUBLICATION).unwrap();
        }
    }
    #[test]
    fn supplemental_execution_metadata_precedes_parser_and_store() {
        test_timeout! {
            if !active() { return; }
            let env=fixture();
            restrict(&env);
            env.exec(R88Q_CONTRACT_RESTRICTED).unwrap();
            let sim=env.state().borrow();
            assert!(!sim.store_glue.vas_product_ready && sim.store_glue.disconnect_on_logout);
        }
    }
    fn restrict(env: &WowLuaEnv) {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.vas_product_ready = false;
        sim.store_glue.disconnect_on_logout = true;
        sim.store_glue.restricted_execution = Some(RestrictedExecution::Macro {
            function: "host-restricted-call".into(),
            callback_present: false,
        });
    }
    #[test]
    fn supplemental_pending_error_does_not_rollback_successful_store() {
        test_timeout! {
            if !active() { return; }
            let env=fixture();
            pending_ready(&env);
            pending_disconnect(&env);
            pending_clear(&env);
            env.state().borrow_mut().store_glue.pending_error=None;
            env.exec(R88Q_CONTRACT_RECORDS).unwrap();
        }
    }
    fn set_pending(env: &WowLuaEnv) {
        env.state().borrow_mut().store_glue.pending_error = Some("host-pending-message".into());
    }
    fn pending_ready(env: &WowLuaEnv) {
        env.state().borrow_mut().store_glue.vas_product_ready = false;
        set_pending(env);
        env.exec("local ok,e=pcall(C_StoreSecure.SetVASProductReady,true); assert(not ok and e:find('SetVASProductReady(): host-pending-message',1,true))").unwrap();
        assert!(env.state().borrow().store_glue.vas_product_ready);
        assert!(env.state().borrow().store_glue.pending_error.is_some());
    }
    fn pending_disconnect(env: &WowLuaEnv) {
        env.state().borrow_mut().store_glue.disconnect_on_logout = false;
        set_pending(env);
        env.exec("local ok,e=pcall(C_StoreSecure.SetDisconnectOnLogout,true); assert(not ok and e:find('SetDisconnectOnLogout(): host-pending-message',1,true))").unwrap();
        assert!(env.state().borrow().store_glue.disconnect_on_logout);
    }
    fn pending_clear(env: &WowLuaEnv) {
        env.state().borrow_mut().store_glue.vas_product_ready = true;
        set_pending(env);
        env.exec("local ok,e=pcall(C_StoreGlue.ClearVASProductReady); assert(not ok and e:find('ClearVASProductReady(): host-pending-message',1,true))").unwrap();
        assert!(!env.state().borrow().store_glue.vas_product_ready);
    }
    #[test]
    fn supplemental_secret_denials_use_only_explicit_access_metadata() {
        test_timeout! {
            if !active() { return; }
            let env=fixture();
            // None is a neutral host seam. No native behavior is asserted for it.
            assert!(env.state().borrow().store_glue.secret_access_mode.is_none());
            publish_secret(&env);
            for (mode,reason) in [(0,"Secret values are not allowed"),(73,"Unknown secret access mode")] {
                env.state().borrow_mut().store_glue.secret_access_mode=Some(mode);
                env.exec(&format!("local ok,e=pcall(C_StoreGlue.GetVASPurchaseStateInfo,R88qSecret); assert(not ok and e:find({reason:?},1,true))")).unwrap();
            }
            env.state().borrow_mut().store_glue.secret_access_mode=Some(1);
            env.exec(R88Q_CONTRACT_SECRET_MODE_ONE).unwrap();
            env.state().borrow_mut().store_glue.secret_access_mode=None;
            env.exec(R88Q_CONTRACT_RECORDS).unwrap();
        }
    }
    fn seed_queue_inputs(env: &WowLuaEnv) {
        use wow_ui_sim::c_api::store_glue_model::VasQueueCharacter;
        let mut sim = env.state().borrow_mut();
        let store = &mut sim.store_glue;
        store.queue_characters = [("Player-1-a1", 21), ("Player-2-b2", 43), ("", 65)]
            .into_iter()
            .map(|(guid, byte)| VasQueueCharacter {
                guid: guid.into(),
                correlation: vec![byte],
            })
            .collect();
        store.connection_present = false;
        store.request_sequence = 0;
        store.submissions.clear();
        store.callbacks.queue_status = true;
    }
    fn assert_queue_counters(env: &WowLuaEnv, sequence: u64, submissions: usize) {
        let sim = env.state().borrow();
        assert_eq!(sim.store_glue.request_sequence, sequence);
        assert_eq!(sim.store_glue.submissions.len(), submissions);
    }
    #[test]
    fn supplemental_queue_matches_advance_while_disconnected_and_submit_on_reconnect() {
        test_timeout! {
            if !active() { return; }
            let env = fixture();
            seed_queue_inputs(&env);
            env.exec("C_StoreGlue.RequestCharacterQueueTime('Player-1-a1'); C_StoreGlue.RequestCharacterQueueTime('Player-1-a1'); C_StoreGlue.RequestCharacterQueueTime('Player-2-b2')").unwrap();
            assert_queue_counters(&env, 3, 0);
            env.exec("C_StoreGlue.RequestCharacterQueueTime('Player-3-c3')").unwrap();
            assert_queue_counters(&env, 3, 0);
            env.state().borrow_mut().store_glue.connection_present = true;
            env.exec("C_StoreGlue.RequestCharacterQueueTime('Player-2-b2'); C_StoreGlue.RequestCharacterQueueTime('Player-1-a1'); C_StoreGlue.RequestCharacterQueueTime('Player-3-c3')").unwrap();
            assert_queue_counters(&env, 5, 2);
            assert_queue_submissions(&env);
            env.exec(R88Q_CONTRACT_RECORDS).unwrap();
        }
    }
    fn assert_queue_submissions(env: &WowLuaEnv) {
        use wow_ui_sim::c_api::store_glue_model::StoreRequest;
        let sim = env.state().borrow();
        for (request, (expected, counter)) in
            sim.store_glue.submissions.iter().zip([(43, 4), (21, 5)])
        {
            match request {
                StoreRequest::Queue {
                    correlation,
                    sequence,
                } => {
                    assert_eq!(correlation, &vec![expected]);
                    assert_eq!(*sequence, counter);
                }
                other => panic!("expected queue submission, got {other:?}"),
            }
        }
    }
    #[test]
    fn supplemental_queue_response_empty_identity_is_nil_and_unmatched_is_silent() {
        test_timeout! {
            if !active() { return; }
            let env = fixture();
            seed_queue_inputs(&env);
            env.exec(R88Q_CONTRACT_QUEUE_LISTENER).unwrap();
            use wow_ui_sim::c_api::store_glue_model::receive_queue_response;
            receive_queue_response(&env, &[99], 12).unwrap();
            env.exec("assert(#R88qSupplementQueue == 0)").unwrap();
            receive_queue_response(&env, &[21], -17).unwrap();
            receive_queue_response(&env, &[43], 29).unwrap();
            receive_queue_response(&env, &[65], -31).unwrap();
            env.exec(R88Q_CONTRACT_QUEUE_RESULTS).unwrap();
            env.exec(R88Q_CONTRACT_RECORDS).unwrap();
        }
    }
    fn service_update(env: &WowLuaEnv, guid: &str, permitted: Option<bool>) {
        use wow_ui_sim::c_api::store_glue_model::receive_service_update;
        let record = VasPurchaseRecord {
            product_id: 189,
            state: 6,
            errors: vec![-37, 0],
        };
        receive_service_update(env, guid.into(), record, permitted).unwrap();
    }
    #[test]
    fn supplemental_service_update_requires_explicit_host_permission() {
        test_timeout! {
            if !active() { return; }
            let env = fixture();
            env.state().borrow_mut().store_glue.callbacks.state_changed = true;
            env.exec(R88Q_CONTRACT_SERVICE_LISTENER).unwrap();
            for permitted in [None, Some(false)] {
                service_update(&env, "Player-1-a1", permitted);
                service_update(&env, "Player-2-b2", permitted);
                service_update(&env, "Player-3-c3", permitted);
                env.exec(R88Q_CONTRACT_RECORDS).unwrap();
                env.exec("assert(#R88qSupplementService == 0)").unwrap();
            }
            service_update(&env, "Player-1-a1", Some(true));
            service_update(&env, "Player-2-b2", Some(true));
            env.exec(R88Q_CONTRACT_SERVICE_RESULTS).unwrap();
        }
    }
    fn publish_secret(env: &WowLuaEnv) {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let value = rilua::table_security::wrap_host_secret_string(lua.state_mut(), "Player-1-a1");
        lua.state_mut().push(value);
        lua.set_global_val("R88qSecret", value).unwrap();
        lua.state_mut().pop();
    }
}
#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_RECORDS: &str = r#"
    local a, ae, ap = CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1')
    local b, be, bp = CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2')
    local n, ne = CharacterSelectListUtil.GetVASInfoForGUID('Player-3-c3')
    assert(a==3 and ap.productID==2003 and ae[1]==17 and ae[2]==0 and ae[3]==-4)
    assert(b==4 and bp.productID==189 and type(be)=='table' and #be==0)
    assert(n==0 and type(ne)=='table' and #ne==0)
"#;
#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_ARGUMENTS: &str = r#"
    local function reject(...)
        for _, api in ipairs({
            {C_StoreGlue.GetVASPurchaseStateInfo,'Usage: local productID, state, errors = C_StoreGlue.GetVASPurchaseStateInfo(guid)'},
            {C_StoreGlue.RequestCharacterQueueTime,'Usage: C_StoreGlue.RequestCharacterQueueTime(guid)'},
        }) do
            local ok,err=pcall(api[1],...)
            assert(not ok and type(err)=='string' and err:find(api[2],1,true))
        end
    end
    reject();reject(nil);reject(false);reject({});reject(function()end)
    reject(coroutine.create(function()end))
    if newproxy then reject(newproxy()) end -- Actual userdata only where the VM exposes it.
    for _, args in ipairs({{17,0,0},{29,0,0}}) do
        local product,state,errors=C_StoreGlue.GetVASPurchaseStateInfo(args[1],false,{})
        assert(product==args[2] and state==args[3] and type(errors)=='table' and #errors==0)
        assert(select('#',C_StoreGlue.RequestCharacterQueueTime(args[1],false,{}))==0)
    end
    for _, setter in ipairs({C_StoreSecure.SetDisconnectOnLogout,C_StoreSecure.SetVASProductReady}) do
        assert(not pcall(setter));assert(not pcall(setter,nil))
        for _, truthy in ipairs({0,'',{},true}) do assert(select('#',setter(truthy,'ignored'))==0) end
        assert(select('#',setter(false))==0)
    end
"#;
#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_SIGNED_PUBLICATION: &str = r#"
    assert(type(__secureenv)=='table')
    assert(type(rawget(__secureenv,'C_StoreGlue'))=='table')
    assert(type(rawget(__secureenv,'C_StoreSecure'))=='table')
    for _, ns in ipairs({C_StoreGlue,__secureenv.C_StoreGlue}) do
        local p,s,e=ns.GetVASPurchaseStateInfo('Player-4-d4')
        assert(p==-27 and s==-9 and #e==3 and e[1]==-8 and e[2]==0 and e[3]==9)
        local pa,sa=ns.GetVASPurchaseStateInfo('Player-1-a1')
        local pb,sb=ns.GetVASPurchaseStateInfo('Player-2-b2')
        local pn,sn,en=ns.GetVASPurchaseStateInfo('Player-3-c3')
        assert(pa==2003 and sa==3 and pb==189 and sb==4 and pn==0 and sn==0 and #en==0)
        assert(type(ns.GetDisconnectOnLogout)=='function' and type(ns.GetVASProductReady)=='function')
        assert(type(ns.ClearVASProductReady)=='function' and type(ns.RequestCharacterQueueTime)=='function')
        assert(type(ns.UpdateVASPurchaseStates)=='function')
        for _, name in ipairs({'GetVASPurchaseStateInfo','GetDisconnectOnLogout','GetVASProductReady',
                'ClearVASProductReady','RequestCharacterQueueTime','UpdateVASPurchaseStates'}) do
            assert(type(rawget(ns,name))=='function')
        end
    end
    __secureenv.C_StoreSecure.SetVASProductReady(true)
    assert(C_StoreGlue.GetVASProductReady(nil,{},false))
    C_StoreGlue.ClearVASProductReady()
    assert(not __secureenv.C_StoreGlue.GetVASProductReady())
    __secureenv.C_StoreSecure.SetDisconnectOnLogout(true)
    assert(C_StoreGlue.GetDisconnectOnLogout(false))
    C_StoreSecure.SetDisconnectOnLogout(false)
    assert(not __secureenv.C_StoreGlue.GetDisconnectOnLogout())
"#;
#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_RESTRICTED: &str = r#"
    -- Explicit restriction metadata, not Lua taint, must precede missing-argument errors.
    assert(select('#',C_StoreSecure.SetVASProductReady())==0)
    assert(select('#',C_StoreSecure.SetVASProductReady(true))==0)
    assert(select('#',C_StoreSecure.SetDisconnectOnLogout(nil))==0)
    assert(select('#',C_StoreSecure.SetDisconnectOnLogout(false))==0)
"#;
#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_SECRET_MODE_ONE: &str = r#"
    local p,s,e=C_StoreGlue.GetVASPurchaseStateInfo(R88qSecret)
    assert(p==2003 and s==3 and e[1]==17)
    local function denied()
        local ok,err=pcall(C_StoreGlue.GetVASPurchaseStateInfo,R88qSecret)
        assert(not ok and err:find('Secret values are only allowed during untainted execution',1,true))
    end
    debug.setobjecttaint(denied,'R88qContractFixture');denied()
"#;

#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_QUEUE_LISTENER: &str = r#"
    R88qSupplementQueue = {}
    local frame = CreateFrame('Frame')
    frame:RegisterEvent('VAS_CHARACTER_QUEUE_STATUS_UPDATE')
    frame:SetScript('OnEvent', function(self, event, ...)
        local guid, value = ...
        table.insert(R88qSupplementQueue, {argc=select('#',...), guid=guid, value=value})
    end)
    R88qSupplementQueueFrame = frame
"#;
#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_QUEUE_RESULTS: &str = r#"
    assert(#R88qSupplementQueue == 3)
    local a,b,empty = unpack(R88qSupplementQueue)
    assert(a.argc==2 and a.guid=='Player-1-a1' and a.value==-17)
    assert(b.argc==2 and b.guid=='Player-2-b2' and b.value==29)
    assert(empty.argc==2 and empty.guid==nil and empty.value==-31)
"#;
#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_SERVICE_LISTENER: &str = r#"
    R88qSupplementService = {}
    local frame = CreateFrame('Frame')
    frame:RegisterEvent('VAS_CHARACTER_STATE_CHANGED')
    frame:SetScript('OnEvent', function(self, event)
        local a = CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1')
        local b = CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2')
        local absent = CharacterSelectListUtil.GetVASInfoForGUID('Player-3-c3')
        table.insert(R88qSupplementService, {event=event, a=a, b=b, absent=absent})
    end)
    R88qSupplementServiceFrame = frame
"#;
#[cfg(feature = "optional-tests")]
const R88Q_CONTRACT_SERVICE_RESULTS: &str = r#"
    assert(#R88qSupplementService==2)
    local first,second = unpack(R88qSupplementService)
    assert(first.event=='VAS_CHARACTER_STATE_CHANGED' and first.a==6 and first.b==4 and first.absent==0)
    assert(second.event=='VAS_CHARACTER_STATE_CHANGED' and second.a==6 and second.b==6 and second.absent==0)
    for _,guid in ipairs({'Player-1-a1','Player-2-b2'}) do
        local state,errors,product = CharacterSelectListUtil.GetVASInfoForGUID(guid)
        assert(state==6 and product.productID==189 and #errors==2 and errors[1]==-37 and errors[2]==0)
    end
"#;

// R88w native contracts exercised through selected original Glue consumers.
// R88w consumer/assertion bodies stay identical across the thin host-route adapters.
#[cfg(feature = "optional-tests")]
mod storeglue_r88w {
    use super::*;
    use wow_ui_sim::c_api::store_glue_model::VasPurchaseRecord;
    const A: &str = "Player-1-a1";
    const B: &str = "Player-2-b2";
    const N: &str = "Player-3-c3";
    fn active() -> bool {
        // VERIFIED Retail; INFERRED PTR/Forever applicability (#1283/#1306).
        r88q_profile_active()
    }
    fn record(product_id: i32, state: i32, errors: &[i32]) -> VasPurchaseRecord {
        VasPurchaseRecord {
            product_id,
            state,
            errors: errors.to_vec(),
        }
    }
    fn fixture() -> common::LockedEnv {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        seed_r88q_product_inputs(&env);
        {
            let mut sim = env.state().borrow_mut();
            let s = &mut sim.store_glue;
            s.purchase_states = [
                (A.into(), record(2003, 3, &[17, 0, -4])),
                (B.into(), record(189, 4, &[0, 29])),
            ]
            .into_iter()
            .collect();
            s.purchase_errors = vec![91];
        }
        r88w_input::seed_guid_aliases(&env);
        r88w_input::install_callbacks(&env);
        env.exec(R88W_RECORDS).unwrap(); // Original consumer before assertions/routes.
        env.exec(R88W_LISTENER).unwrap();
        env
    }
    fn copied(env: &WowLuaEnv) -> Vec<i32> {
        env.state().borrow().store_glue.purchase_errors.clone()
    }
    #[test]
    fn error_notification_is_zero_payload_before_record_removal_including_lookup_miss() {
        test_timeout! {
            if !active() {return;}
            let env=fixture();
            r88w_input::remove(&env,A,0,vec![51,0,-4]);
            env.exec("R88wCountAfterA=#R88wEvents").unwrap();
            r88w_input::remove(&env,B,1,vec![-37,0]);
            env.exec("R88wCountAfterB=#R88wEvents").unwrap();
            r88w_input::remove(&env,N,0,vec![81]);
            env.exec("assert(R88wCountAfterA==2); assert(R88wEvents[1].event=='STORE_VAS_PURCHASE_ERROR' and R88wEvents[1].argc==0 and R88wEvents[1].a==3 and R88wEvents[1].b==4); assert(R88wEvents[2].event=='VAS_CHARACTER_STATE_CHANGED' and R88wEvents[2].a==0 and R88wEvents[2].b==4)").unwrap();
            env.exec("assert(R88wCountAfterB==4); assert(R88wEvents[3].event=='STORE_VAS_PURCHASE_ERROR' and R88wEvents[3].argc==0 and R88wEvents[3].b==4); assert(R88wEvents[4].event=='VAS_CHARACTER_STATE_CHANGED' and R88wEvents[4].b==0)").unwrap();
            env.exec("assert(#R88wEvents==5); assert(R88wEvents[5].event=='STORE_VAS_PURCHASE_ERROR' and R88wEvents[5].argc==0 and R88wEvents[5].n==0)").unwrap();
        }
    }
    #[test]
    fn native_shared_error_copy_gate_replaces_and_clears_independently_of_record_match() {
        test_timeout! {
            if !active() {return;}
            let env=fixture();
            let mut snapshots=Vec::new();
            for (state,errors) in [(0,vec![0,29]),(1,vec![51,0,-4]),(0,vec![-37]),(1,vec![]),(0,vec![19])] {
                r88w_input::remove(&env,N,state,errors);
                snapshots.push(copied(&env));
                env.exec("R88wGateCounts=R88wGateCounts or {}; table.insert(R88wGateCounts,#R88wEvents)").unwrap();
            }
            r88w_input::service(&env,N,record(189,4,&[23]),None);
            let cleared=copied(&env);
            env.exec("table.insert(R88wGateCounts,#R88wEvents)").unwrap();
            assert!(snapshots[0].is_empty());
            env.exec("assert(R88wGateCounts[1]==0)").unwrap();
            assert_eq!(snapshots[1],vec![51,0,-4]);
            assert_eq!(snapshots[2],vec![-37]);
            assert!(snapshots[3].is_empty());
            assert!(cleared.is_empty());
            env.exec(R88W_RECORDS).unwrap();
        }
    }
    #[test]
    fn bulk_state_changed_is_single_zero_payload_after_full_replacement_and_empty_list() {
        test_timeout! {
            if !active() {return;}
            let env=fixture();
            r88w_input::bulk(&env,vec![(A.into(),record(189,6,&[31])),(B.into(),record(2003,2,&[-23]))]);
            env.exec("R88wBulkA=CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1'); R88wBulkB=CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2'); R88wBulkCount=#R88wEvents").unwrap();
            r88w_input::bulk(&env,vec![]);
            env.exec("assert(R88wBulkA==6 and R88wBulkB==2); assert(R88wBulkCount==1 and R88wEvents[1].argc==0 and R88wEvents[1].a==6 and R88wEvents[1].b==2 and R88wEvents[1].n==0)").unwrap();
            env.exec("assert(#R88wEvents==2 and R88wEvents[2].argc==0 and R88wEvents[2].a==0 and R88wEvents[2].b==0)").unwrap();
        }
    }
    #[test]
    fn removal_state_changed_is_zero_payload_only_after_matching_removal() {
        test_timeout! {
            if !active() {return;}
            let env=fixture();
            r88w_input::remove(&env,N,0,vec![]);
            r88w_input::remove(&env,A,0,vec![]);
            r88w_input::remove(&env,B,1,vec![]);
            env.exec("assert(#R88wEvents==2); for _,event in ipairs(R88wEvents) do assert(event.event=='VAS_CHARACTER_STATE_CHANGED' and event.argc==0 and event.n==0) end; assert(R88wEvents[1].a==0 and R88wEvents[1].b==4); assert(R88wEvents[2].a==0 and R88wEvents[2].b==0)").unwrap();
        }
    }
    #[test]
    fn service_state_changed_is_zero_payload_after_permitted_update_only() {
        test_timeout! {
            if !active() {return;}
            let env=fixture();
            r88w_input::service(&env,N,record(189,6,&[31]),None);
            r88w_input::service(&env,A,record(189,6,&[31]),Some(true));
            r88w_input::service(&env,B,record(2003,2,&[-23]),Some(true));
            env.exec("assert(#R88wEvents==2); for _,event in ipairs(R88wEvents) do assert(event.event=='VAS_CHARACTER_STATE_CHANGED' and event.argc==0 and event.n==0) end; assert(R88wEvents[1].a==6 and R88wEvents[1].b==4); assert(R88wEvents[2].a==6 and R88wEvents[2].b==2)").unwrap();
        }
    }
    fn probe_reader(env: &WowLuaEnv, label: &str, guid: &str, input: Vec<i32>) -> Vec<i32> {
        env.state().borrow_mut().store_glue.purchase_errors = input;
        env.exec(&format!("R88wProbeReader({label:?},{guid:?})"))
            .unwrap();
        copied(env)
    }
    #[test]
    fn shared_error_reader_original_message_consumer_sees_sorted_unique_snapshots() {
        test_timeout! {
            if !active() {return;}
            let env=fixture();
            env.exec(R88W_READER_CAPTURE).unwrap();
            let a=probe_reader(&env,"a",A,vec![-3,-7,-3]);
            let b=probe_reader(&env,"b",B,vec![-2,-11,-2,-11]);
            let absent=probe_reader(&env,"n",N,vec![]);
            // Every original-consumer invocation and both direct reads ran before assertions.
            env.exec(R88W_READER_ASSERT).unwrap();
            assert_eq!(a.len(),3); assert_eq!(&a[..2],&[-7,-3]);
            assert_eq!(b.len(),4); assert_eq!(&b[..2],&[-11,-2]);
            assert!(absent.is_empty()); // Compacted tail values deliberately unspecified.
            env.exec(R88W_RECORDS).unwrap();
        }
    }
    #[test]
    fn shared_error_reader_entry_rejection_returns_no_values_without_reading_source() {
        test_timeout! {
            if !active() {return;}
            let env=fixture();
            r88w_input::set_entry_rejected(&env,true);
            let a=denied_probe(&env,A,vec![-3,-7,-3]);
            let b=denied_probe(&env,B,vec![-2,-11,-2,-11]);
            let n=denied_probe(&env,N,vec![]);
            r88w_input::set_entry_rejected(&env,false);
            env.exec("assert(#R88wDeniedResults==3); for _,r in ipairs(R88wDeniedResults) do assert(not r.original[1] and r.count[1] and r.count[2]==0) end").unwrap();
            assert_eq!(a,vec![-3,-7,-3]);
            assert_eq!(b,vec![-2,-11,-2,-11]);
            assert!(n.is_empty());
            env.exec(R88W_RECORDS).unwrap();
        }
    }
    fn denied_probe(env: &WowLuaEnv, guid: &str, input: Vec<i32>) -> Vec<i32> {
        env.state().borrow_mut().store_glue.purchase_errors = input;
        env.exec(&format!("R88wDeniedResults=R88wDeniedResults or {{}}; table.insert(R88wDeniedResults,{{original={{pcall(VASErrorData_GetCombinedMessage,{guid:?})}},count={{pcall(function() return select('#',C_StoreSecure.GetVASErrors()) end)}}}})")).unwrap();
        copied(env)
    }
    #[test]
    fn shared_error_reader_sorts_source_before_pending_error_without_reducing_count() {
        test_timeout! {
            if !active() {return;}
            let env=fixture();
            let a=pending_probe(&env,A,vec![-3,-7,-3]);
            let b=pending_probe(&env,B,vec![-2,-11,-2,-11]);
            let n=pending_probe(&env,N,vec![]);
            env.state().borrow_mut().store_glue.pending_error=None;
            env.exec("for _,v in ipairs(R88wPendingResults) do assert(not v[1] and tostring(v[2]):find('host-reader-pending',1,true)) end; assert(#R88wPendingResults==3)").unwrap();
            assert_eq!(a.len(),3); assert_eq!(&a[..2],&[-7,-3]);
            assert_eq!(b.len(),4); assert_eq!(&b[..2],&[-11,-2]);
            assert!(n.is_empty());
            env.exec(R88W_RECORDS).unwrap();
        }
    }
    fn pending_probe(env: &WowLuaEnv, guid: &str, input: Vec<i32>) -> Vec<i32> {
        {
            let mut sim = env.state().borrow_mut();
            sim.store_glue.purchase_errors = input;
            sim.store_glue.pending_error = Some("host-reader-pending".into());
        }
        env.exec(&format!("R88wPendingResults=R88wPendingResults or {{}}; table.insert(R88wPendingResults,{{pcall(VASErrorData_GetCombinedMessage,{guid:?})}})")).unwrap();
        copied(env)
    }

    const R88W_LISTENER: &str = r#"
    R88wEvents={}
    local frame=CreateFrame('Frame')
    frame:RegisterEvent('STORE_VAS_PURCHASE_ERROR')
    frame:RegisterEvent('VAS_CHARACTER_STATE_CHANGED')
    frame:SetScript('OnEvent',function(self,event,...)
        local a=CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1')
        local b=CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2')
        local n=CharacterSelectListUtil.GetVASInfoForGUID('Player-3-c3')
        table.insert(R88wEvents,{event=event,argc=select('#',...),a=a,b=b,n=n})
    end)
    R88wFrame=frame
"#;
    const R88W_READER_CAPTURE: &str = r#"
    R88wReaderResults={}
    function R88wProbeReader(label,guid)
        local result={original={pcall(VASErrorData_GetCombinedMessage,guid)}}
        local function read() return select('#',C_StoreSecure.GetVASErrors()), C_StoreSecure.GetVASErrors() end
        result.first={pcall(read)}
        result.second={pcall(read)}
        if result.first[1] and type(result.first[3])=='table' then result.first[3][1]=999 end
        result.independent={pcall(read)}
        R88wReaderResults[label]=result
    end
"#;
    const R88W_READER_ASSERT: &str = r#"
    local source=debug.getinfo(VASErrorData_GetCombinedMessage,'S')
    assert(source.source:find('VASErrorLookup.lua',1,true) and source.linedefined==365)
    for _,case in ipairs({{'a',{-7,-3}},{'b',{-11,-2}},{'n',{}}}) do
        local result=R88wReaderResults[case[1]]
        local expected=case[2]
        local messages={}
        for _,value in ipairs(expected) do table.insert(messages,VASErrorData_GetMessage(value,nil)) end
        local message=table.concat(messages,'\n')
        if message=='' then message=BLIZZARD_STORE_VAS_ERROR_OTHER end
        assert(result.original[1] and result.original[2]==message)
        for _,snapshot in ipairs({result.second,result.independent}) do
            assert(snapshot[1] and snapshot[2]==1 and type(snapshot[3])=='table')
            assert(#snapshot[3]==#expected)
            for i,value in ipairs(expected) do assert(snapshot[3][i]==value) end
        end
        assert(result.first[1] and result.first[2]==1 and type(result.first[3])=='table')
    end
"#;

    const R88W_RECORDS: &str = r#"
    local a, ae, ap = CharacterSelectListUtil.GetVASInfoForGUID('Player-1-a1')
    local b, be, bp = CharacterSelectListUtil.GetVASInfoForGUID('Player-2-b2')
    local n, ne = CharacterSelectListUtil.GetVASInfoForGUID('Player-3-c3')
    assert(a==3 and ap.productID==2003 and ae[1]==17 and ae[2]==0 and ae[3]==-4)
    assert(b==4 and bp.productID==189 and type(be)=='table' and #be==0)
    assert(n==0 and type(ne)=='table' and #ne==0)
"#;

    mod r88w_input {
        use super::*;
        use wow_ui_sim::c_api::store_glue_model::{self as model, VasPurchaseRecord};
        pub fn install_callbacks(env: &WowLuaEnv) {
            env.state().borrow_mut().store_glue.callbacks.state_changed = true;
            // #1310: actual callback-present seam, not a replacement event producer.
            env.state().borrow_mut().store_glue.callbacks.purchase_error = true;
        }
        pub fn set_entry_rejected(env: &WowLuaEnv, rejected: bool) {
            // #1310: actual neutral entry-check result, not a replacement getter.
            env.state()
                .borrow_mut()
                .store_glue
                .vas_errors_entry_rejected = rejected;
        }
        pub fn seed_guid_aliases(env: &WowLuaEnv) {
            // Ordinary host keys are parsed by production; no fixture aliases.
            env.state()
                .borrow_mut()
                .store_glue
                .opaque_guid_aliases
                .clear();
        }
        pub fn remove(env: &WowLuaEnv, guid: &str, state: i32, errors: Vec<i32>) {
            assert!(matches!(state, 0 | 1));
            model::receive_removal(env, guid, errors).unwrap();
        }
        pub fn bulk(env: &WowLuaEnv, records: Vec<(String, VasPurchaseRecord)>) {
            model::receive_bulk(env, records).unwrap();
        }
        pub fn service(
            env: &WowLuaEnv,
            guid: &str,
            record: VasPurchaseRecord,
            permitted: Option<bool>,
        ) {
            model::receive_service_update(env, guid.into(), record, permitted).unwrap();
        }
    }
}

// Original-consumer integration, R88w.w2 / disposition #1310. Draft, uncompiled.
#[cfg(feature = "optional-tests")]
mod storeglue_r88w_guid {
    use super::*;
    fn fixture(a: &str, b: &str) -> common::LockedEnv {
        let env = load_blizzard_screen(ScreenKind::CharacterSelect);
        seed_r88q_product_inputs(&env);
        input::seed(&env, a, b);
        env.exec(CAPTURE).unwrap();
        env
    }
    fn capture(env: &WowLuaEnv, text: &str, expected: i32) {
        env.exec(&format!("R88wGUIDCapture({text:?},{expected})"))
            .unwrap();
    }
    fn check(env: &WowLuaEnv) {
        env.exec(ASSERT).unwrap();
    }
    #[test]
    fn ordinary_ascii_case_and_numeric_spelling_reach_two_original_records() {
        test_timeout! {
            if !r88q_profile_active() {return;}
            let env=fixture("Player-1-a1","Player-2-b2");
            capture(&env,"pLaYeR-0001-00A1",1);
            capture(&env,"PLAYER-0002-0xB2",2);
            capture(&env,"Player-3-c3",0);
            check(&env);
        }
    }
    #[test]
    fn player_native_trailing_and_hex_prefix_advance_examples_reach_original_records() {
        test_timeout! {
            if !r88q_profile_active() {return;}
            let env=fixture("Player-1-12","Player-2-0");
            capture(&env,"Player-1-12-FF",1);
            capture(&env,"Player-2-0xZ",2);
            capture(&env,"Player-3-c3",0);
            check(&env);
        }
    }
    #[test]
    fn party_zero_hex_and_verified_lmmparty_kind_alias_reach_original_records() {
        test_timeout! {
            if !r88q_profile_active() {return;}
            let env=fixture("Party-1-2-0","Party-3-4-a1");
            capture(&env,"Party-1-2-Z",1);
            capture(&env,"LMMParty-3-4-A1",2);
            capture(&env,"Party-5-6-c3",0);
            check(&env);
        }
    }
    #[test]
    fn opaque_exact_case_wrapping_and_empty_alias_fallback_reach_original_records() {
        test_timeout! {
            if !r88q_profile_active() {return;}
            let env=fixture("Player-1-a1","Player-2-b2");
            input::opaque_aliases(&env,"Player-1-a1","Player-2-b2");
            capture(&env,"Opaque-4294967297",1);
            capture(&env,"Opaque-4294967298",2);
            capture(&env,"Opaque-1",1);
            capture(&env,"Opaque-2",2);
            for absent in ["Opaque-3","Opaque-4","opaque-1","Opaque-1Z","Opaque-"] {
                capture(&env,absent,0);
            }
            check(&env);
        }
    }
    #[test]
    fn bounded_player_and_party_decimal_limits_do_not_fabricate_matching_records() {
        test_timeout! {
            if !r88q_profile_active() {return;}
            let env=fixture("Player-65535-a1","Party-65535-4294967295-b2");
            capture(&env,"PLAYER-65535-A1",1);
            capture(&env,"party-65535-4294967295-B2",2);
            for absent in ["Player-65536-a1","Party-65536-4294967295-b2",
                    "Party-65535-4294967296-b2","Player-3-c3"] {
                capture(&env,absent,0);
            }
            check(&env);
        }
    }
    const CAPTURE: &str = r#"
        R88wGUIDResults={}
        function R88wGUIDCapture(text,expected)
            local original={pcall(CharacterSelectListUtil.GetVASInfoForGUID,text)}
            local direct={pcall(C_StoreGlue.GetVASPurchaseStateInfo,text)}
            table.insert(R88wGUIDResults,{original=original,direct=direct,expected=expected})
        end
    "#;
    const ASSERT: &str = r#"
        local source=debug.getinfo(CharacterSelectListUtil.GetVASInfoForGUID,'S')
        assert(source.source:find('CharacterSelectListUtil.lua',1,true))
        local observedA,observedB,observedAbsent=false,false,false
        for _,row in ipairs(R88wGUIDResults) do
            local o,d=row.original,row.direct
            assert(o[1] and d[1])
            local product,state,errors=d[2],d[3],d[4]
            assert(type(errors)=='table' and o[2]==state and type(o[3])=='table')
            if row.expected==1 then
                observedA=true
                assert(product==2003 and state==3 and o[4].productID==2003)
                assert(#errors==3 and errors[1]==17 and errors[2]==0 and errors[3]==-4)
                assert(#o[3]==3 and o[3][1]==17 and o[3][2]==0 and o[3][3]==-4)
            elseif row.expected==2 then
                observedB=true
                assert(product==189 and state==4 and o[4].productID==189 and #errors==0 and #o[3]==0)
            else
                observedAbsent=true
                assert(product==0 and state==0 and #errors==0 and #o[3]==0)
            end
        end
        assert(observedA and observedB and observedAbsent)
    "#;
    // RED: exact canonical aliases only. Never seed the alternate spellings being tested.
    // GREEN: replace only this input representation with typed identity/opaque-key setup.
    mod input {
        use super::*;
        use wow_ui_sim::c_api::store_glue_model::VasPurchaseRecord;
        pub fn seed(env: &WowLuaEnv, a: &str, b: &str) {
            let mut sim = env.state().borrow_mut();
            let store = &mut sim.store_glue;
            store.purchase_states.clear();
            store.opaque_guid_aliases.clear();
            for (guid, product_id, state, errors) in
                [(a, 2003, 3, vec![17, 0, -4]), (b, 189, 4, vec![0, 29])]
            {
                store.purchase_states.insert(
                    guid.into(),
                    VasPurchaseRecord {
                        product_id,
                        state,
                        errors,
                    },
                );
            }
        }
        pub fn opaque_aliases(env: &WowLuaEnv, a: &str, b: &str) {
            use wow_ui_sim::c_api::store_guid::{Outcome, parse};
            let mut sim = env.state().borrow_mut();
            for (key, text) in [(1, a), (2, b)] {
                let Outcome::Known(identity) = parse(text.as_bytes()) else {
                    panic!("fixture requires a known ordinary GUID");
                };
                sim.store_glue
                    .opaque_guid_aliases
                    .insert(key, Some(identity));
            }
            sim.store_glue.opaque_guid_aliases.insert(3, None);
        }
    }
}
