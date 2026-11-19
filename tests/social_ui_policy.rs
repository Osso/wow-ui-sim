#![cfg(any(
    all(feature = "profile-retail", feature = "retail-12-1-5"),
    feature = "client-wowforever"
))]

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
    let expected_consumer = match wow_ui_sim::client_profile::ACTIVE {
        wow_ui_sim::client_profile::ClientProfile::WowForever => {
            "Blizzard_FriendsFrame/Camelot/FriendsFrame.lua"
        }
        _ => "Blizzard_FriendsFrame/Mainline/FriendsFrame.lua",
    };
    env.exec(&format!(
        r#"
        local source = debug.getinfo(FriendsFrame_ShowSubFrame, "S").source:gsub("\\", "/")
        assert(source:find("{expected_consumer}", 1, true), source)
        assert(C_GameRules.GetActiveGameMode() ~= Enum.GameMode.Plunderstorm)
    "#
    ))
    .unwrap();
    if let Ok(path) = std::env::var("WOW_SIM_SOCIAL_POLICY_OVERLAY") {
        env.exec(&std::fs::read_to_string(path).expect("read Social UI test overlay"))
            .unwrap();
    }
    env
}

fn install_social_status_listener(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(GetCVarDefault("socialUIEnabledClient") == "0")
        assert(GetCVar("socialUIEnabledClient") == "0")
        assert(SetCVar("socialUIEnabledClient", "1") == false)
        assert(C_CVar.SetCVar("socialUIEnabledClient", "1") == false)
        assert(GetCVar("socialUIEnabledClient") == "0")
        socialStatusUpdates = 0
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("SOCIAL_UI_SYSTEM_STATUS_UPDATED")
        listener:SetScript("OnEvent", function()
            socialStatusUpdates = socialStatusUpdates + 1
            socialEnabledOnUpdate = C_SocialUI.IsSystemEnabled()
        end)
        "#,
    )
    .unwrap();
}

fn assert_retail_enabled_friends_consumer(env: &WowLuaEnv) {
    env.exec(
        r#"
        FriendsListFrame:Hide()
        FriendsFrame_ShowSubFrame("FriendsListFrame")
        assert(not FriendsListFrame:IsShown(), "friends list is unavailable when enabled")
        WhoFrame:Hide()
        FriendsFrame_ShowSubFrame("WhoFrame")
        assert(WhoFrame:IsShown(), "Who remains allowed when enabled")
        assert(not FriendsListFrame:IsShown(), "non-selected friends frame is hidden")
        assert(C_SocialUI.IsSystemEnabled() == true)
        assert(socialStatusUpdates == 1 and socialEnabledOnUpdate == true)
        "#,
    )
    .unwrap();
}

// Camelot has no WhoFrame entry; its enabled FriendsFrame option is FriendsListFrame.
fn assert_forever_enabled_friends_consumer(env: &WowLuaEnv) {
    env.exec(
        r#"
        FriendsListFrame:Hide()
        FriendsFrame_ShowSubFrame("FriendsListFrame")
        assert(FriendsListFrame:IsShown(), "Camelot keeps FriendsListFrame allowed")
        QuickJoinFrame:Hide()
        FriendsFrame_ShowSubFrame("QuickJoinFrame")
        assert(not QuickJoinFrame:IsShown(), "Camelot blocks Quick Join when enabled")
        assert(not FriendsListFrame:IsShown(), "Camelot hides the unselected friends frame")
        assert(C_SocialUI.IsSystemEnabled() == true)
        assert(socialStatusUpdates == 1 and socialEnabledOnUpdate == true)
        "#,
    )
    .unwrap();
}

fn assert_enabled_friends_consumer(env: &WowLuaEnv) {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Retail {
        assert_retail_enabled_friends_consumer(env);
    } else {
        assert_forever_enabled_friends_consumer(env);
    }
}

fn assert_retail_disabled_friends_consumer(env: &WowLuaEnv) {
    env.exec(
        r#"
        FriendsListFrame:Hide()
        FriendsFrame_ShowSubFrame("FriendsListFrame")
        assert(FriendsListFrame:IsShown(), "friends list is shown when disabled")
        WhoFrame:Hide()
        FriendsFrame_ShowSubFrame("WhoFrame")
        assert(WhoFrame:IsShown(), "Who remains allowed when disabled")
        assert(not FriendsListFrame:IsShown(), "non-selected friends frame is hidden")
        assert(C_SocialUI.IsSystemEnabled() == false)
        assert(socialStatusUpdates == 2 and socialEnabledOnUpdate == false)
        "#,
    )
    .unwrap();
}

fn assert_forever_disabled_friends_consumer(env: &WowLuaEnv) {
    env.exec(
        r#"
        FriendsListFrame:Hide()
        FriendsFrame_ShowSubFrame("FriendsListFrame")
        assert(FriendsListFrame:IsShown(), "Camelot shows FriendsListFrame when disabled")
        QuickJoinFrame:Hide()
        FriendsFrame_ShowSubFrame("QuickJoinFrame")
        assert(QuickJoinFrame:IsShown(), "Camelot allows Quick Join when disabled")
        assert(not FriendsListFrame:IsShown(), "non-selected Camelot friends frame is hidden")
        assert(C_SocialUI.IsSystemEnabled() == false)
        assert(socialStatusUpdates == 2 and socialEnabledOnUpdate == false)
        "#,
    )
    .unwrap();
}

fn assert_disabled_friends_consumer(env: &WowLuaEnv) {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Retail {
        assert_retail_disabled_friends_consumer(env);
    } else {
        assert_forever_disabled_friends_consumer(env);
    }
}

#[test]
fn original_friends_subframes_follow_native_social_cvar() {
    let env = setup();
    install_social_status_listener(&env);
    assert!(env.set_native_cvar("socialUIEnabledClient", "1").unwrap());
    assert_enabled_friends_consumer(&env);
    assert!(env.set_native_cvar("socialUIEnabledClient", "0").unwrap());
    assert_disabled_friends_consumer(&env);
}

#[test]
fn social_system_enabled_has_one_boolean_and_ignores_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        assert(select("#", C_SocialUI.IsSystemEnabled()) == 1)
        assert(type(C_SocialUI.IsSystemEnabled()) == "boolean")
        assert(C_SocialUI.IsSystemEnabled() == false)
        assert(C_SocialUI.IsSystemEnabled(99) == false)
        assert(C_SocialUI.IsSystemEnabled(nil, {}, "unknown") == false)
        "##,
    )
    .unwrap();
    assert!(env.set_native_cvar("socialUIEnabledClient", "2").unwrap());
    env.exec(
        r##"
        assert(C_SocialUI.IsSystemEnabled() == true)
        assert(C_SocialUI.IsSystemEnabled(99) == true)
        assert(C_SocialUI.IsSystemEnabled(nil, {}, "unknown") == true)
        "##,
    )
    .unwrap();
}
