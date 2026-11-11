//! Tests for `C_CharacterServices` probes backed by
//! `SimState.character_services`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::CharacterServicesState;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn queued_upgrade_result(env: &WowLuaEnv) -> (bool, i32) {
    env.eval(
        r#"
        local function pack(...)
            return { count = select("#", ...), ... }
        end
        local result = pack(C_CharacterServices.HasQueuedUpgrade())
        return result[1], result.count
        "#,
    )
    .expect("queued-upgrade result is readable")
}

#[test]
fn get_active_character_upgrade_boost_type_returns_nil_by_default() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_CharacterServices.GetActiveCharacterUpgradeBoostType() == nil")
        .unwrap();
    assert!(is_nil, "default SimState should have no active boost type");
}

#[test]
fn get_active_class_trial_boost_type_returns_nil_by_default() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_CharacterServices.GetActiveClassTrialBoostType() == nil")
        .unwrap();
    assert!(is_nil, "default SimState should have no active trial type");
}

#[test]
fn get_active_character_upgrade_boost_type_reflects_seeded_value() {
    let env = env();
    env.state().borrow_mut().character_services = CharacterServicesState {
        active_upgrade_boost_type: Some(5),
        active_class_trial_boost_type: None,
        queued_upgrade_guid_type_nonzero: false,
    };
    let boost_type: f64 = env
        .eval("return C_CharacterServices.GetActiveCharacterUpgradeBoostType()")
        .unwrap();
    assert_eq!(boost_type as i32, 5);
}

#[test]
fn get_active_class_trial_boost_type_reflects_seeded_value() {
    let env = env();
    env.state().borrow_mut().character_services = CharacterServicesState {
        active_upgrade_boost_type: None,
        active_class_trial_boost_type: Some(7),
        queued_upgrade_guid_type_nonzero: false,
    };
    let trial_type: f64 = env
        .eval("return C_CharacterServices.GetActiveClassTrialBoostType()")
        .unwrap();
    assert_eq!(trial_type as i32, 7);
}

#[test]
fn clearing_upgrade_boost_type_returns_nil_again() {
    let env = env();
    env.state()
        .borrow_mut()
        .character_services
        .active_upgrade_boost_type = Some(3);
    env.state()
        .borrow_mut()
        .character_services
        .active_upgrade_boost_type = None;
    let is_nil: bool = env
        .eval("return C_CharacterServices.GetActiveCharacterUpgradeBoostType() == nil")
        .unwrap();
    assert!(is_nil, "cleared boost type should return nil");
}

#[test]
fn has_required_service_for_character_upgrade_returns_false() {
    let env = env();
    let (has_upgrade, has_class_trial): (bool, bool) = env
        .eval(
            r#"
            return
                C_CharacterServices.HasRequiredServiceForCharacterUpgrade(),
                C_CharacterServices.HasRequiredBoostForClassTrial()
            "#,
        )
        .unwrap();
    assert!(
        !has_upgrade,
        "should return false — no active service by default"
    );
    assert!(
        !has_class_trial,
        "should return false — no active trial boost by default"
    );
}

#[test]
fn get_character_service_display_info_returns_empty_table() {
    let env = env();
    let (display_count, vas_count): (i32, i32) = env
        .eval(
            r#"
            return
                #C_CharacterServices.GetCharacterServiceDisplayInfo(),
                #C_CharacterServices.GetVASDistributions()
            "#,
        )
        .unwrap();
    assert_eq!(display_count, 0);
    assert_eq!(vas_count, 0);
}

#[test]
fn get_character_service_display_data_returns_default_shape() {
    let env = env();
    let has_shape: bool = env
        .eval(
            r#"
            local data = C_CharacterServices.GetCharacterServiceDisplayData()
            return data.boostLevel == 80 and
                data.flowTitle == "Character Upgrade" and
                data.popupInfo.textureKit == "characterupdate"
            "#,
        )
        .unwrap();
    assert!(has_shape);
}

#[test]
fn assign_functions_are_callable_without_error() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            C_CharacterServices.AssignUpgradeDistribution()
            C_CharacterServices.AssignPCTDistribution()
            return true
            "#,
        )
        .unwrap();
    assert!(ok);
}

#[test]
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
fn has_queued_upgrade_drives_the_original_character_select_update() {
    test_timeout! {
        let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
        let toc_path = wow_ui_sim::loader::find_toc_file(&ui.join("Blizzard_GlueXML"))
            .expect("GlueXML has a selected profile TOC");
        let toc = wow_ui_sim::toc::TocFile::from_file(&toc_path)
            .expect("selected GlueXML TOC parses");
        assert!(toc.allows_screen(wow_ui_sim::screen::ScreenKind::CharacterSelect));
        assert!(toc.files.iter().any(|file| {
            file.file_name().is_some_and(|name| name == "CharacterSelect.lua")
        }));

        crate::common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape(
            &["Blizzard_GlueXML"],
            &[],
            |env, loaded| {
                assert!(loaded.iter().any(|name| name == "Blizzard_GlueXML"));
                env.exec(
                    r#"
                    local source = debug.getinfo(CharacterSelectFrameMixin.OnUpdate, "S").source
                    assert(source:find("CharacterSelect.lua", 1, true), source)
                    C_StoreGlue = C_StoreGlue or {}
                    C_StoreGlue.GetVASProductReady = function() return false end
                    queuedUpgradeRefreshes = 0
                    CharacterServicesMaster_OnCharacterListUpdate = function()
                        queuedUpgradeRefreshes = queuedUpgradeRefreshes + 1
                    end
                    StoreFrame_WaitingForCharacterListUpdate = function() return false end
                    StoreFrame_OnCharacterListUpdate = function() end
                    StaticPopup_CheckQueuedDialogs = function() end
                    "#,
                )
                .expect("selected original CharacterSelect loads");

                env.state()
                    .borrow_mut()
                    .character_services
                    .queued_upgrade_guid_type_nonzero = false;
                assert_eq!(queued_upgrade_result(env), (false, 1));
                env.exec("CharacterSelectFrameMixin:OnUpdate(0)")
                    .expect("original consumer accepts an empty queued GUID");
                env.state()
                    .borrow_mut()
                    .character_services
                    .queued_upgrade_guid_type_nonzero = true;
                assert_eq!(queued_upgrade_result(env), (true, 1));
                env.exec("CharacterSelectFrameMixin:OnUpdate(0)")
                    .expect("original consumer accepts a nonzero queued GUID type");
                let refreshes: i32 = env
                    .eval("return queuedUpgradeRefreshes")
                    .expect("original refresh count is readable");
                assert_eq!(refreshes, 1, "queued upgrade refreshes the character list");

                env.state()
                    .borrow_mut()
                    .character_services
                    .queued_upgrade_guid_type_nonzero = false;
                env.exec("CharacterSelectFrameMixin:OnUpdate(0)")
                    .expect("original consumer observes the cleared queued GUID");
                let refreshes: i32 = env
                    .eval("return queuedUpgradeRefreshes")
                    .expect("original refresh count is readable after clear");
                assert_eq!(refreshes, 1, "cleared queued GUID does not refresh again");
            },
        );

        let game = WowLuaEnv::new().expect("create Game-screen environment");
        let absent: bool = game
            .eval("return rawget(C_CharacterServices, 'HasQueuedUpgrade') == nil")
            .expect("Game registration state is readable");
        assert!(absent, "native registration is Glue-only");
    }
}
