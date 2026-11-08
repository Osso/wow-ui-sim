#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::path::PathBuf;
use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

fn wow_sim_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_wow-sim")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("wow-sim")
        })
}

#[test]
fn mists_loot_group_and_bonus_roll_actions_record_state() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            local function mustFrame(name)
                local frame = _G[name]
                if frame == nil then
                    error(name .. " missing")
                end
                if frame.GetWidth and (frame:GetWidth() or 0) <= 0 then
                    error(name .. " has no width")
                end
                if frame.GetHeight and (frame:GetHeight() or 0) <= 0 then
                    error(name .. " has no height")
                end
                return frame
            end

            for _, name in ipairs({
                "GetLootSlotInfo",
                "GetLootSlotLink",
                "GetLootSlotType",
                "LootSlotHasItem",
                "LootSlot",
                "IsFishingLoot",
                "RollOnLoot",
                "ConfirmLootRoll",
            }) do
                if type(_G[name]) ~= "function" then
                    error(name .. " missing")
                end
            end

            A_Admin.ClearLoot()
            A_Admin.AddLootItem(6948, 1)
            A_Admin.AddLootItem(19019, 2)

            FireEvent("LOOT_OPENED", false)
            mustFrame("LootFrame")
            if LootFrame:IsShown() ~= true then
                error("LootFrame did not open")
            end
            if LootFrame.numLootItems ~= 2 then
                error("LootFrame item count mismatch: " .. tostring(LootFrame.numLootItems))
            end
            if LootButton1:IsShown() ~= true or LootButton1IconTexture:GetTexture() == nil then
                error("first loot button did not render")
            end
            if LootButton1Text:GetText() == nil or LootButton1Text:GetText() == "" then
                error("first loot button has no item text")
            end
            if GetLootSlotType(1) ~= LOOT_SLOT_ITEM then
                error("first loot slot type is not item")
            end
            if not LootSlotHasItem(1) then
                error("first loot slot should have item")
            end
            if type(GetLootSlotLink(1)) ~= "string" then
                error("first loot slot link missing")
            end
            LootSlot(1)
            if GetNumLootItems() ~= 1 then
                error("LootSlot did not clear the selected slot")
            end
            FireEvent("LOOT_CLOSED")

            A_Admin.StartLootRoll(77, 30, "Group Loot Sword", "Interface\\Icons\\INV_Sword_04", 4, 200, "|cffa335ee|Hitem:19019::::::::|h[Group Loot Sword]|h|r")
            mustFrame("GroupLootContainer")
            mustFrame("GroupLootFrame1")
            if GroupLootFrame1:IsShown() ~= true then
                error("GroupLootFrame1 did not show")
            end
            if GroupLootFrame1.Name:GetText() ~= "Group Loot Sword" then
                error("group loot item name mismatch")
            end
            if GroupLootFrame1.IconFrame.Icon:GetTexture() == nil then
                error("group loot icon missing")
            end
            GroupLootFrame1.NeedButton:GetScript("OnClick")(GroupLootFrame1.NeedButton)
            if A_Admin.GetLastLootRollChoice() ~= 1 then
                error("need roll choice was not recorded")
            end

            BonusRollFrame_StartBonusRoll(1, "Bonus Loot", 30, 2245)
            mustFrame("BonusRollFrame")
            if BonusRollFrame:IsShown() ~= true or BonusRollFrame.state ~= "prompt" then
                error("BonusRollFrame prompt did not show")
            end
            BonusRollFrame_OnEvent(BonusRollFrame, "BONUS_ROLL_STARTED")
            BonusRollFrame_OnEvent(BonusRollFrame, "BONUS_ROLL_RESULT", "item", "|cff0070dd|Hitem:6948::::::::|h[Hearthstone]|h|r", 1, 65)
            BonusRollFrame_FinishedFading(BonusRollFrame)
            if BonusRollLootWonFrame == nil then
                error("bonus-roll loot won frame missing")
            end
            "#,
            "lua-errors",
        ])
        )
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("Loot panel", &stdout, &stderr);
}
