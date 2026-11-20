#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn pending_invite_confirmation_warnings_use_each_supplied_invite() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_LFGUtil"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_LFGUtil"));

    env.exec(
        r#"
        A_Admin.SetPendingInviteConfirmations([[
            {"id":"invite-a", "name":"Aster", "guid":"Player-1-A", "invalidQueues":[{"queueType":"pvp", "mapName":"North Battleground"}]},
            {"id":"invite-b", "name":"Bramble", "guid":"Player-1-B", "invalidQueues":[{"queueType":"pvp", "mapName":"South Battleground"}]}
        ]])
        "#,
    )
    .expect("host should supply two distinct pending invitations");

    let warnings: (String, String) = env
        .eval(
            r#"
            return CreatePendingInviteConfirmationText_GetWarnings("invite-a", "Aster", "Player-1-A", true, false),
                   CreatePendingInviteConfirmationText_GetWarnings("invite-b", "Bramble", "Player-1-B", true, false)
            "#,
        )
        .expect("the selected Blizzard invite consumer should format each supplied confirmation");

    assert!(warnings.0.contains("North Battleground"), "{}", warnings.0);
    assert!(warnings.1.contains("South Battleground"), "{}", warnings.1);
    assert_ne!(warnings.0, warnings.1);
}
