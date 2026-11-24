#![cfg(feature = "client-wowforever")]

#[path = "common/blizzard_addon_closure.rs"]
mod closure;

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn friends_frame_displays_two_supplied_battle_tags() {
    let ui = closure::blizzard_ui_dir();
    let (env, loaded) = closure::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_FriendsFrame"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_FriendsFrame"));
    assert_two_battle_tags(&env);
}

fn assert_two_battle_tags(env: &WowLuaEnv) {
    let tags: (String, String) = env
        .eval(
            r#"
            A_Admin.SetPlayerBattleNetInfo("Malfurion#1234")
            FriendsFrame_CheckBattlenetStatus()
            local first = FriendsFrameBattlenetFrame.Tag:GetText()
            A_Admin.SetPlayerBattleNetInfo("Tyrande#5678")
            FriendsFrame_CheckBattlenetStatus()
            return first, FriendsFrameBattlenetFrame.Tag:GetText()
            "#,
        )
        .expect("FriendsFrame should display supplied Battle.net account tags");
    assert_eq!(
        tags,
        (
            "Malfurion|cff416380#1234|r".into(),
            "Tyrande|cff416380#5678|r".into(),
        )
    );
}
