#![cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
//! VERIFIED R88ap part A §3-4; inferred host inputs supply native state.
use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{GameRuleKey, GameRuleValue};

fn setup() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let roots = ["Blizzard_FriendsFrame", "Blizzard_UnitPopupShared"];
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &roots, &[]);
    assert!(
        roots
            .iter()
            .all(|root| loaded.iter().any(|name| name == root))
    );
    env.exec(r#"
        local role = debug.getinfo(UnitPopupSelectRoleButtonMixin.CanShow, "S").source:gsub("\\", "/")
        assert(role:find("Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua", 1, true), role)
        local friends = debug.getinfo(FriendsListFrame_OnShow, "S").source:gsub("\\", "/")
        assert(friends:find("Blizzard_FriendsFrame/Mainline/FriendsFrame.lua", 1, true), friends)
    "#).unwrap();
    if let Ok(path) = std::env::var("WOW_SIM_SOCIAL_SETTINGS_OVERLAY") {
        env.exec(&std::fs::read_to_string(path).unwrap()).unwrap();
    }
    env
}

fn set_rule(env: &WowLuaEnv, value: Option<i64>, non_integer: bool) {
    let mut sim = env.state().borrow_mut();
    let key = GameRuleKey::Id(93);
    sim.game_rules.rules.remove(&key);
    if let Some(value) = value {
        sim.game_rules.rules.insert(
            key,
            GameRuleValue {
                integer_value: Some(value),
                as_float: value as f64,
                as_int: value,
                as_string: value.to_string(),
            },
        );
    }
    sim.game_rules.non_integer_rule_ids.clear();
    if non_integer {
        sim.game_rules.non_integer_rule_ids.insert(93);
    }
}

fn assert_role(env: &WowLuaEnv, expected: bool) {
    env.exec(&format!("expectedRole = {expected}")).unwrap();
    env.exec(
        r#"
        assert(UnitPopupSelectRoleButtonMixin:CanShow({unit="player"}) == expectedRole,
            "original role entry reflects integer rule low byte")
        assert(select('#', CanShowSetRoleButton()) == 1)
        assert(type(CanShowSetRoleButton()) == 'boolean')
    "#,
    )
    .unwrap();
}

#[test]
fn original_set_role_entry_follows_integer_rule_low_byte() {
    let env = setup();
    env.state().borrow_mut().party_group_active = true;
    env.exec(
        r#"
        assert(IsInGroup(), "fixture group prerequisite")
        assert(not C_Scenario.IsInScenario() and not HasLFGRestrictions())
        assert(not UnitInPartyIsAI("player") and UnitIsUnit("player", "player"))
    "#,
    )
    .unwrap();
    for (value, non_integer, expected) in [
        (None, false, false),
        (Some(0x100), false, false),
        (Some(0x101), false, true),
        (Some(0x101), true, false),
        (Some(0x101), false, true),
    ] {
        set_rule(&env, value, non_integer);
        assert_role(&env, expected);
    }
}

#[test]
fn original_friends_list_follows_settings_byte() {
    let env = setup();
    assert_eq!(
        env.state()
            .borrow()
            .social_restrictions
            .friends_disabled_byte,
        0
    );
    for byte in [0, 1, 0] {
        env.state()
            .borrow_mut()
            .social_restrictions
            .friends_disabled_byte = byte;
        env.exec(&format!("expectedDisabled = {}", byte != 0))
            .unwrap();
        assert_friends(&env);
    }
}

fn assert_friends(env: &WowLuaEnv) {
    env.exec(r#"
            FriendsListFrame_OnShow(FriendsListFrame)
            assert(FriendsListFrame.ScrollBox:IsShown() == not expectedDisabled,
                "original friends scrollbox reflects settings byte")
            assert(FriendsListFrame.ScrollBar:IsShown() == not expectedDisabled)
            assert(FriendsListFrame.FriendsDisabledText:IsShown() == expectedDisabled)
            assert(FriendsFrameAddFriendButton:IsEnabled() == not expectedDisabled)
            if expectedDisabled then
                assert(FriendsListFrame.FriendsDisabledText:GetText() == SOCIAL_TAB_UNAVAILABLE:format(SOCIAL_UI_FRIENDS_TAB_NAME))
            end
            assert(select('#', C_SocialRestrictions.IsFriendsDisabled()) == 1)
            assert(type(C_SocialRestrictions.IsFriendsDisabled()) == 'boolean')
        "#).unwrap();
}
