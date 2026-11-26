//! Quest-log item tooltip source rows supplied through the simulator admin API.
#![cfg(any(feature = "profile-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn quest_log_item_uses_supplied_rows_and_returns_no_data_for_misses() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.exec(
        r##"
        local questID = 81234
        A_Admin.SetQuestLogEntry(questID, "Tooltip source test", false)
        A_Admin.SetQuestLogRewardItems(questID, {
            { name = "Hearthstone", texture = "Interface\\Icons\\INV_Misc_Rune_01", count = 1, quality = 1, isUsable = false, itemID = 6948 },
            { name = "Shield", texture = "Interface\\Icons\\INV_Shield_06", count = 1, quality = 1, isUsable = false, itemID = 229181 },
        })
        local first = C_TooltipInfo.GetQuestLogItem("reward", 1, questID)
        local second = C_TooltipInfo.GetQuestLogItem("reward", 2, questID)
        assert(first and first.id == 6948 and first.lines[1].leftText ~= "")
        assert(second and second.id == 229181 and second.lines[1].leftText ~= first.lines[1].leftText,
            "distinct supplied rows must produce distinct item tooltips")
        assert(select("#", C_TooltipInfo.GetQuestLogItem("reward", 3, questID)) == 0,
            "an index past the supplied rows must not produce item data")

        A_Admin.SetQuestLogEntry(questID + 1, "No reward rows", false)
        assert(select("#", C_TooltipInfo.GetQuestLogItem("reward", 1, questID + 1)) == 0,
            "a quest with no supplied reward rows must not produce item data")
        assert(not pcall(A_Admin.SetQuestLogRewardItems, questID + 2, {}),
            "the setter must reject an unknown quest")
        assert(not pcall(A_Admin.SetQuestLogRewardItems, questID, {
            { name = "Invalid", texture = "icon", count = 0, quality = 1, isUsable = false, itemID = 6948 },
        }), "invalid reward rows must be rejected")
        assert(C_TooltipInfo.GetQuestLogItem("reward", 1, questID).id == 6948,
            "invalid input must not replace the existing reward rows")
        "##,
    )
    .expect("quest-log tooltip source behavior matches the supplied reward state");
}
