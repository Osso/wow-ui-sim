//! Quest-giver requests over supplied quest offer and reward state.
#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

fn run_cases(roots: &[&str], cases: &str, what: &str) {
    with_blizzard_addon_closure(roots, &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == roots[0]), "{what}: {loaded:?}");
        env.exec(cases).unwrap_or_else(|err| panic!("{what}: {err}"));
    });
}

// QuestFrame's detail panel needs QuestGetAutoAccept/QuestFlagsPVP (outside
// this lane), so the requests run against the modeled offer and dialog.
const QUEST_CASES: &str = r#"
local seen = {}
for _, event in ipairs({"QUEST_ACCEPTED", "QUEST_FINISHED", "QUEST_TURNED_IN"}) do
    EventRegistry:RegisterFrameEventAndCallback(event, function(_, ...)
        seen[#seen + 1] = table.concat({event, tostring((...))}, ":")
    end)
end
AcceptQuest()
DeclineQuest()
assert(#seen == 0, "no offer: no response")
A_Admin.OpenQuestNpc(80002, "Into the Fray")
C_GossipInfo.SelectAvailableQuest(80002)
AcceptQuest()
assert(seen[1] == "QUEST_ACCEPTED:80002" and seen[2] == "QUEST_FINISHED:nil", table.concat(seen, ","))
assert(C_QuestLog.GetLogIndexForQuestID(80002) ~= nil, "accepted quest is in the log")
A_Admin.OpenQuestNpc(80003, "Second Offer")
C_GossipInfo.SelectAvailableQuest(80003)
DeclineQuest()
assert(#seen == 3 and seen[3] == "QUEST_FINISHED:nil")
assert(C_QuestLog.GetLogIndexForQuestID(80003) == nil, "declined quest is not accepted")
A_Admin.SetQuestLogEntry(80001, "Reward Quest", true)
A_Admin.SetQuestRewardChoices(80001, {
    {name = "Blade", texture = "Interface\\Icons\\INV_Sword_04", count = 1, quality = 3, isUsable = true},
    {name = "Shield", texture = "Interface\\Icons\\INV_Shield_04", count = 1, quality = 3, isUsable = true},
})
A_Admin.OpenMultiQuestNpc()
C_GossipInfo.SelectActiveQuest(80001)
GetQuestReward(nil)
GetQuestReward(3)
assert(#seen == 3 and C_QuestLog.GetLogIndexForQuestID(80001) ~= nil, "a choice is required")
GetQuestReward(2)
assert(seen[4] == "QUEST_TURNED_IN:80001" and seen[5] == "QUEST_FINISHED:nil", table.concat(seen, ","))
assert(C_QuestLog.GetLogIndexForQuestID(80001) == nil and C_QuestLog.IsQuestFlaggedCompleted(80001))
GetQuestReward(2)
assert(#seen == 5, "a claimed quest cannot be claimed again")
"#;

#[test]
#[cfg(feature = "client-retail")]
fn quest_giver_requests_accept_decline_and_claim_rewards() {
    run_cases(&["Blizzard_SharedXMLBase"], QUEST_CASES, "quest requests");
}
