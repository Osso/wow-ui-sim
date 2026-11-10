//! Tests for `C_CharacterServices` probes backed by
//! `SimState.character_services`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::CharacterServicesState;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
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
