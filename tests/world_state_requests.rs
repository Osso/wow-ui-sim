//! Totem dismissal, world-marker clearing and quest-giver requests driven
//! through their original Blizzard consumers with A_Admin host inputs.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

fn run_cases(roots: &[&str], cases: &str, what: &str) {
    with_blizzard_addon_closure(roots, &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == roots[0]), "{what}: {loaded:?}");
        env.exec(cases).unwrap_or_else(|err| panic!("{what}: {err}"));
    });
}

const TOTEM_CASES: &str = r#"
A_Admin.SetManualClock(100)
assert(not pcall(A_Admin.SetTotem, 9, "Searing Totem", 60, 135825, 3599, false), "slot range")
assert(not pcall(A_Admin.SetTotem, 1, "Searing Totem", 0, 135825, 3599, false), "duration")
assert(GetTotemTimeLeft(1) == 0 and GetTotemCannotDismiss(1) == nil, "empty slot")
A_Admin.SetTotem(1, "Searing Totem", 60, 135825, 3599, false)
A_Admin.SetTotem(2, "Earthen Wall", 30, 136098, 198838, true)
local function buttons()
    local bySlot = {}
    for button in TotemFrame.totemPool:EnumerateActive() do bySlot[button.slot] = button end
    return bySlot
end
assert(TotemFrame:IsShown() and buttons()[1] and buttons()[2], "PLAYER_TOTEM_UPDATE shows both")
assert(GetTotemTimeLeft(1) == 60 and GetTotemTimeLeft(2) == 30)
assert(GetTotemCannotDismiss(1) == false and GetTotemCannotDismiss(2) == true)
A_Admin.AdvanceTime(12.5)
assert(GetTotemTimeLeft(1) == 47.5 and GetTotemTimeLeft(2) == 17.5, tostring(GetTotemTimeLeft(1)))
buttons()[2]:Click("RightButton")
assert(GetTotemInfo(2) == true and buttons()[2], "undismissable totem stays")
buttons()[1]:Click("RightButton")
assert(GetTotemInfo(1) == false and not buttons()[1], "right click destroys the totem")
assert(GetTotemTimeLeft(1) == 0 and GetTotemCannotDismiss(1) == nil)
A_Admin.AdvanceTime(20)
assert(GetTotemTimeLeft(2) == 0 and GetTotemCannotDismiss(2) == nil, "expired")
"#;

#[test]
fn totem_frame_counts_down_and_dismisses_supplied_totems() {
    run_cases(&["Blizzard_UnitFrame"], TOTEM_CASES, "TotemFrame");
}

const WORLD_MARKER_CASES: &str = r#"
assert(not pcall(A_Admin.SetWorldMarker, 9, true), "marker range")
local markers = CompactRaidFrameManager.displayFrame.raidMarkers
markers.raidMarkerGroundTab:Click()
assert(markers.activeTab == markers.raidMarkerGroundTab)
local function button(worldIndex)
    for i = 1, NUM_WORLD_RAID_MARKERS do
        local candidate = markers["raidMarker" .. i]
        if WORLD_RAID_MARKER_ORDER[candidate:GetMarker()] == worldIndex then return candidate end
    end
end
for _, index in ipairs({2, 5, 7}) do A_Admin.SetWorldMarker(index, true) end
CompactRaidFrameManager_UpdateRaidIcons()
assert(IsRaidMarkerActive(2) and IsRaidMarkerActive(5) and not IsRaidMarkerActive(4))
assert(button(2).backgroundTexture:GetAtlas() == "GM-button-marker-applied")
button(2):Click("RightButton")
assert(not IsRaidMarkerActive(2) and IsRaidMarkerActive(5) and IsRaidMarkerActive(7))
assert(button(2).backgroundTexture:GetAtlas() == "GM-button-marker-available")
button(4):Click("RightButton")
assert(IsRaidMarkerActive(5) and IsRaidMarkerActive(7), "inactive marker: nothing to clear")
ClearRaidMarker(5)
assert(not IsRaidMarkerActive(5) and IsRaidMarkerActive(7))
A_Admin.SetWorldMarker(3, true)
markers.raidMarkerReset:Click()
assert(not IsRaidMarkerActive(3) and not IsRaidMarkerActive(7), "reset clears every marker")
"#;

#[test]
fn raid_manager_ground_markers_clear_supplied_world_markers() {
    run_cases(&["Blizzard_CompactRaidFrames"], WORLD_MARKER_CASES, "world markers");
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
fn quest_giver_requests_accept_decline_and_claim_rewards() {
    run_cases(&["Blizzard_SharedXMLBase"], QUEST_CASES, "quest requests");
}
