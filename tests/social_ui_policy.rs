#![cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
//! VERIFIED R88ap part B §6; host input represents policy, not friend-list state.

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const ADDONS: &[&str] = &[
    "Blizzard_FriendsFrame",
    "Blizzard_QuickJoin",
    "Blizzard_RaidFrame",
    "Blizzard_RecruitAFriend",
];

fn setup() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, ADDONS, &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_FriendsFrame"));
    env.exec(
        r#"
        local source = debug.getinfo(FriendsFrame_ShowSubFrame, "S").source:gsub("\\", "/")
        assert(source:find("Blizzard_FriendsFrame/Mainline/FriendsFrame.lua", 1, true), source)
        assert(C_GameRules.GetActiveGameMode() ~= Enum.GameMode.Plunderstorm)
    "#,
    )
    .unwrap();
    if let Ok(path) = std::env::var("WOW_SIM_SOCIAL_POLICY_OVERLAY") {
        env.exec(&std::fs::read_to_string(path).expect("read explicit producer overlay"))
            .unwrap();
    }
    env
}

#[test]
fn original_friends_subframes_follow_social_policy() {
    let env = setup();
    for policy in [Some(true), Some(false), None, Some(true)] {
        env.state().borrow_mut().social_system_policy_enabled = policy;
        env.exec(&format!("expectedPolicy = {}", policy.unwrap_or(false)))
            .unwrap();
        env.exec(r#"
            -- Observe the original selector through its real public consumer.
            FriendsListFrame:Hide()
            FriendsFrame_ShowSubFrame("FriendsListFrame")
            assert(FriendsListFrame:IsShown() == not expectedPolicy, "original FriendsFrame policy selection")
            FriendsFrame_ShowSubFrame("WhoFrame")
            assert(WhoFrame:IsShown(), "WhoFrame remains allowed")
            assert(not FriendsListFrame:IsShown(), "non-selected friends frame is hidden")
        "#).expect("original FriendsFrame must select policy-appropriate subframes");
    }
}

#[test]
fn social_policy_has_one_boolean_and_ignores_arguments() {
    let env = WowLuaEnv::new().unwrap();
    for policy in [None, Some(false), Some(true)] {
        env.state().borrow_mut().social_system_policy_enabled = policy;
        env.exec(&format!("expectedPolicy = {}", policy.unwrap_or(false)))
            .unwrap();
        env.exec(
            r#"
            assert(select('#', C_SocialUI.IsSystemEnabled()) == 1)
            assert(type(C_SocialUI.IsSystemEnabled()) == 'boolean')
            assert(C_SocialUI.IsSystemEnabled() == expectedPolicy)
            assert(C_SocialUI.IsSystemEnabled(99) == expectedPolicy)
            assert(C_SocialUI.IsSystemEnabled(nil, {}, 'unknown') == expectedPolicy)
        "#,
        )
        .unwrap();
    }
}
