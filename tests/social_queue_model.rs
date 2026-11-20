#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

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

const QUICK_JOIN_GROUPS: &str = r#"
A_Admin.SetSocialQueueGroups([[ [
    {"guid":"group-alpha","info":{"canJoin":true,"numQueues":1,"needTank":false,"needHealer":false,"needDamage":true,"isSoloQueueParty":false,"questSessionActive":false,"leaderGUID":"leader-alpha"},"members":[],"queues":[]},
    {"guid":"group-beta","info":{"canJoin":true,"numQueues":2,"needTank":true,"needHealer":false,"needDamage":false,"isSoloQueueParty":false,"questSessionActive":false,"leaderGUID":"leader-beta"},"members":[],"queues":[]}
] ]])
QuickJoinFrame.entries:UpdateAll()
local entries = QuickJoinFrame.entries:GetEntries()
assert(#entries == 2, "Quick Join displays both supplied groups")
assert(entries[1]:GetGUID() == "group-alpha")
assert(entries[2]:GetGUID() == "group-beta")
assert(entries[1]:CanJoin() == true)
assert(entries[2]:CanJoin() == true)
"#;

fn setup() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, ADDONS, &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_QuickJoin"));
    env
}

#[test]
fn social_queue_config_returns_host_supplied_fields() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"A_Admin.SetSocialQueueConfig([[{"QUEUE_MULTIPLIER":7,"TOASTS_DISABLED":false}]])
        assert(C_SocialQueue.GetConfig().QUEUE_MULTIPLIER == 7)"#,
    )
    .expect("Social Queue config should return supplied fields");
}

#[test]
fn quick_join_entries_show_distinct_host_supplied_groups() {
    let env = setup();
    env.exec(QUICK_JOIN_GROUPS)
        .expect("Quick Join should display supplied social queue groups");
}
