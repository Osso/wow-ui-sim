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

const QUEST_REWARD_READ_CASES: &str = r#"
A_Admin.SetQuestLogEntry(80001, "Choice Quest", true)
A_Admin.SetQuestLogEntry(80002, "Other Quest", true)
A_Admin.SetQuestRewardChoices(80001, {
    {name = "Sword", texture = "Interface\Icons\INV_Sword_04", count = 2, quality = 3, isUsable = true, itemID = 212405},
    {name = "Bow", texture = "Interface\Icons\INV_Weapon_Bow_01", count = 1, quality = 4, isUsable = false, itemID = 273784},
})
A_Admin.SetQuestRewardChoices(80002, {
    {name = "Sword", texture = "Interface\Icons\INV_Sword_04", count = 1, quality = 1, isUsable = false, itemID = 212405},
})
local i, kind = QuestUtils_GetBestQualityItemRewardIndex(80001)
assert(i == 2 and kind == "choice")
local name, _, count, quality, usable, id = QuestUtils_GetQuestLogRewardInfo(i, 80001, kind)
assert(name == "Bow" and count == 1 and quality == 4 and not usable and id == 273784)
i, kind = QuestUtils_GetBestQualityItemRewardIndex(80002)
assert(i == 1 and kind == "choice")
assert(QuestUtils_GetQuestLogRewardInfo(i, 80002, kind) == "Sword")
assert(GetQuestLogItemLink("choice", 1, 80001):find("item:212405", 1, true))
assert(GetQuestLogItemLink("choice", 2, 80001):find("item:273784", 1, true))
assert(GetQuestLogItemLink("choice", 2, 80002) == nil)
A_Admin.OpenMultiQuestNpc()
C_GossipInfo.SelectActiveQuest(80001)
assert(GetNumQuestChoices() == 2)
assert(GetQuestItemLink("choice", 1):find("item:212405", 1, true))
assert(GetQuestItemLink("choice", 2):find("item:273784", 1, true))
DeclineQuest()
assert(GetNumQuestChoices() == 0 and GetQuestItemLink("choice", 1) == nil)
"#;

#[test]
fn quest_reward_reads_drive_original_choice_selection_and_item_links() {
    run_cases(&["Blizzard_FrameXMLUtil"], QUEST_REWARD_READ_CASES, "quest reward reads");
}
