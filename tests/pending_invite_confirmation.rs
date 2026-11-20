#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn pending_invite_confirmation_event_drives_the_dialog_queue() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let roots = [
        "Blizzard_Game",
        "Blizzard_LFGUtil",
        "Blizzard_StaticPopup_Game",
        "Blizzard_UIPanels_Game",
    ];
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &roots, &[]);
    for name in roots {
        assert!(
            loaded.iter().any(|loaded| loaded == name),
            "{name} did not load"
        );
    }

    env.exec(
        r#"
        A_Admin.SetPendingInviteConfirmations([=[
        [
            {"id":"invite-a", "confirmationType":0, "name":"Aster", "guid":"Player-1-A", "invalidQueues":[{"queueType":"pvp", "mapName":"North Battleground"}]},
            {"id":"invite-b", "confirmationType":0, "name":"Bramble", "guid":"Player-1-B", "invalidQueues":[{"queueType":"pvp", "mapName":"South Battleground"}]}
        ]
        ]=])
        "#,
    )
    .expect("host should add two distinct pending invite confirmations");

    env.exec(
        r#"
        local dialog = StaticPopup_FindVisible("GROUP_INVITE_CONFIRMATION")
        assert(dialog and dialog:IsShown(), "GROUP_INVITE_CONFIRMATION should open")
        local text = dialog.Text:GetText()
        assert(text:find("Aster", 1, true), text)
        assert(text:find("North Battleground", 1, true), text)

        dialog.button1:Click()

        dialog = StaticPopup_FindVisible("GROUP_INVITE_CONFIRMATION")
        assert(dialog and dialog:IsShown(), "the next confirmation should open after answering A")
        text = dialog.Text:GetText()
        assert(text:find("Bramble", 1, true), text)
        assert(text:find("South Battleground", 1, true), text)

        dialog.button2:Click()
        assert(StaticPopup_FindVisible("GROUP_INVITE_CONFIRMATION") == nil)
        "#,
    )
    .expect("the native event and dialog callbacks should advance the Rust confirmation queue");
}
