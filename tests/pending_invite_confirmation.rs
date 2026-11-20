#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::Path;

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn pending_invite_confirmation_event_drives_the_dialog_queue() {
    let ui = blizzard_ui_dir();
    let env = load_invite_consumers(&ui);
    supply_invites(&env);
    accept_first_invite(&env);
    env.process_timers()
        .expect("dispatch the next confirmation after A's dialog closes");
    decline_second_invite(&env);
}

fn load_invite_consumers(ui: &Path) -> WowLuaEnv {
    let env = new_blizzard_addon_env(ui);
    let roots = [
        "Blizzard_Game",
        "Blizzard_LFGUtil",
        "Blizzard_StaticPopup_Game",
        "Blizzard_UIPanels_Game",
    ];
    let loaded = load_blizzard_addon_closure_into_env(&env, ui, &roots, &[]);
    for name in roots {
        assert!(
            loaded.iter().any(|loaded| loaded == name),
            "{name} did not load"
        );
    }
    env
}

fn supply_invites(env: &WowLuaEnv) {
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
}

fn accept_first_invite(env: &WowLuaEnv) {
    env.exec(
        r#"
        local dialog = StaticPopup_FindVisible("GROUP_INVITE_CONFIRMATION")
        assert(dialog and dialog:IsShown(), "GROUP_INVITE_CONFIRMATION should open")
        local text = dialog.Text:GetText()
        assert(text:find("Aster", 1, true), text)
        assert(text:find("North Battleground", 1, true), text)
        dialog.button1:Click()
        assert(GetNextPendingInviteConfirmation() == "invite-b", "answering A should remove it")
        "#,
    )
    .expect("the dialog should answer A and remove it from the Rust queue");
}

fn decline_second_invite(env: &WowLuaEnv) {
    env.exec(
        r#"
        local dialog = StaticPopup_FindVisible("GROUP_INVITE_CONFIRMATION")
        assert(dialog and dialog:IsShown(), "the next confirmation should open after answering A")
        local text = dialog.Text:GetText()
        assert(text:find("Bramble", 1, true), text)
        assert(text:find("South Battleground", 1, true), text)
        dialog.button2:Click()
        assert(StaticPopup_FindVisible("GROUP_INVITE_CONFIRMATION") == nil)
        "#,
    )
    .expect("the dialog should decline B and leave no confirmation visible");
}
