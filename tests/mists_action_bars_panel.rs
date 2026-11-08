#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_action_bag_micro_and_status_bars_update_slots_and_hover_scripts() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
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
                "MainMenuBar",
                "ActionButton1",
                "ActionButton12",
                "MultiBarBottomLeftButton1",
                "MainMenuBarBackpackButton",
                "CharacterBag0Slot",
                "CharacterBag1Slot",
                "CharacterBag2Slot",
                "CharacterBag3Slot",
                "MainMenuExpBar",
                "MainMenuBarPerformanceBarFrame",
            }) do
                mustFrame(name)
            end

            for _, name in ipairs({
                "HasVehicleActionBar",
                "HasOverrideActionBar",
                "HasBonusActionBar",
                "HasTempShapeshiftActionBar",
                "GetWatchedFactionInfo",
            }) do
                if type(_G[name]) ~= "function" then
                    error(name .. " missing")
                end
            end

            if HasVehicleActionBar() ~= false or HasOverrideActionBar() ~= false then
                error("vehicle/override action bar defaults should be false")
            end

            local microButtons = {
                "CharacterMicroButton",
                "SpellbookMicroButton",
                "TalentMicroButton",
                "AchievementMicroButton",
                "QuestLogMicroButton",
                "MainMenuMicroButton",
                "HelpMicroButton",
            }
            for _, name in ipairs(microButtons) do
                local button = mustFrame(name)
                if button:GetNormalTexture() == nil then
                    error(name .. " normal texture missing")
                end
                if button:GetHighlightTexture() == nil then
                    error(name .. " highlight texture missing")
                end
                local onEnter = button:GetScript("OnEnter")
                local onLeave = button:GetScript("OnLeave")
                if type(onEnter) == "function" then
                    onEnter(button)
                end
                if type(onLeave) == "function" then
                    onLeave(button)
                end
            end

            A_Admin.ClearActionBars()
            A_Admin.SetActionSlot(1, 19750)
            ActionButton_Update(ActionButton1)
            local actionType, actionID = GetActionInfo(1)
            if actionType ~= "spell" or actionID ~= 19750 then
                error("action button slot mismatch")
            end
            if ActionButton1Icon:GetTexture() == nil then
                error("action button icon did not refresh")
            end

            for _, name in ipairs({
                "ActionButton1",
                "MainMenuBarBackpackButton",
                "CharacterBag0Slot",
                "CharacterBag1Slot",
            }) do
                local frame = mustFrame(name)
                local onEnter = frame:GetScript("OnEnter")
                local onLeave = frame:GetScript("OnLeave")
                if type(onEnter) == "function" then
                    onEnter(frame)
                end
                if type(onLeave) == "function" then
                    onLeave(frame)
                end
            end
            "#,
            "lua-errors",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("Action/bag/micro/status bar flow", &stdout, &stderr);
}

