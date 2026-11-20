#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn pending_invite_confirmation_warnings_use_each_supplied_invite() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    #[cfg(feature = "client-retail")]
    let roots = ["Blizzard_LFGUtil", "Blizzard_UIPanels_Game"];
    #[cfg(feature = "client-wowforever")]
    let roots = ["Blizzard_StaticPopup_Game"];
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &roots, &[]);
    #[cfg(feature = "client-retail")]
    assert!(loaded.iter().any(|name| name == "Blizzard_LFGUtil"));
    #[cfg(feature = "client-wowforever")]
    assert!(loaded.iter().any(|name| name == "Blizzard_StaticPopup_Game"));

    env.exec(
        r#"
        A_Admin.SetPendingInviteConfirmations([=[
        [
            {"id":"invite-a", "name":"Aster", "guid":"Player-1-A", "invalidQueues":[{"queueType":"pvp", "mapName":"North Battleground"}]},
            {"id":"invite-b", "name":"Bramble", "guid":"Player-1-B", "invalidQueues":[{"queueType":"pvp", "mapName":"South Battleground"}]}
        ]
        ]=])
        "#,
    )
    .expect("host should supply two distinct pending invitations");

    #[cfg(feature = "client-retail")]
    assert_retail_lfg_consumer(&env);
    #[cfg(feature = "client-wowforever")]
    assert_forever_party_invite_consumer(&env);
}

#[cfg(feature = "client-retail")]
fn assert_retail_lfg_consumer(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let warnings: (String, String) = env
        .eval(
            r#"
            return CreatePendingInviteConfirmationText_GetWarnings("invite-a", "Aster", "Player-1-A", true, false),
                   CreatePendingInviteConfirmationText_GetWarnings("invite-b", "Bramble", "Player-1-B", true, false)
            "#,
        )
        .expect("Retail LFGUtil should format each supplied invite");
    assert!(warnings.0.contains("North Battleground"), "{}", warnings.0);
    assert!(warnings.1.contains("South Battleground"), "{}", warnings.1);
    assert_ne!(warnings.0, warnings.1);
}

#[cfg(feature = "client-wowforever")]
fn assert_forever_party_invite_consumer(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        local first = GetNextPendingInviteConfirmation()
        local _, firstName = GetInviteConfirmationInfo(first)
        local firstPopup = StaticPopup_Show("PARTY_INVITE", firstName)
        assert(firstPopup.Text:GetText():find(firstName, 1, true))
        C_PartyInfo.ConfirmInviteUnit(firstName)
        StaticPopup_Hide("PARTY_INVITE")

        local second = GetNextPendingInviteConfirmation()
        local _, secondName = GetInviteConfirmationInfo(second)
        local secondPopup = StaticPopup_Show("PARTY_INVITE", secondName)
        assert(secondPopup.Text:GetText():find(secondName, 1, true))
        assert(firstName ~= secondName)
        "#,
    )
    .expect("Forever PARTY_INVITE should display each supplied invite name");
}
