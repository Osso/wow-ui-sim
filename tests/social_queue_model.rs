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

const GROUP_INPUT: &str = r#"
A_Admin.SetSocialQueueGroups([[ [
    {"guid":"group-alpha","info":{"canJoin":true,"numQueues":1,"needTank":false,"needHealer":false,"needDamage":true,"isSoloQueueParty":false,"questSessionActive":false,"leaderGUID":"leader-alpha"},"members":[],"queues":[{"clientID":101,"eligible":false,"needTank":false,"needHealer":false,"needDamage":true,"isAutoAccept":false,"queueData":{"queueType":"pvp"}}]},
    {"guid":"group-beta","info":{"canJoin":true,"numQueues":2,"needTank":true,"needHealer":false,"needDamage":false,"isSoloQueueParty":true,"questSessionActive":false,"leaderGUID":"leader-beta"},"members":[],"queues":[{"clientID":202,"eligible":false,"needTank":true,"needHealer":false,"needDamage":false,"isAutoAccept":false,"queueData":{"queueType":"pvp"}}]}
] ]])
"#;
const CONFIG_INPUT: &str =
    r#"A_Admin.SetSocialQueueConfig([[{"QUEUE_MULTIPLIER":1,"PLAYER_MULTIPLIER":1}]])"#;
const QUICK_JOIN_ASSERTIONS: &str = r#"
local groups = C_SocialQueue.GetAllGroups()
assert(#groups == 2 and groups[1] == "group-alpha" and groups[2] == "group-beta", "group query lost supplied IDs")
local canJoinA, queueCountA = C_SocialQueue.GetGroupInfo(groups[1])
local canJoinB, queueCountB, needTankB = C_SocialQueue.GetGroupInfo(groups[2])
assert(canJoinA and queueCountA == 1 and canJoinB and queueCountB == 2 and needTankB, "group info was not projected per ID")
assert(#C_SocialQueue.GetGroupMembers(groups[1]) == 0, "group member query should return the supplied empty list")
assert(C_SocialQueue.GetGroupQueues(groups[2])[1].clientID == 202, "queue query returned the wrong group data")
FriendsFrame_UpdateQuickJoinTab(#groups)
assert(FriendsFrameTab4:GetText() == QUICK_JOIN .. " " .. string.format(NUMBER_IN_PARENTHESES, 2), "FriendsFrame tab did not count both groups")
assert(QuickJoinToastButton.groups[groups[1]].guid == "group-alpha", "toast consumer lost alpha")
assert(QuickJoinToastButton.groups[groups[2]].guid == "group-beta", "toast consumer lost beta")
"#;

fn setup() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    env.exec(CONFIG_INPUT)
        .expect("host config should be supplied");
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
fn quick_join_consumer_reads_distinct_host_groups() {
    let env = setup();
    env.exec(GROUP_INPUT)
        .expect("host groups should trigger the Quick Join update");
    env.exec(QUICK_JOIN_ASSERTIONS)
        .expect("Quick Join should display supplied social queue groups");
}
