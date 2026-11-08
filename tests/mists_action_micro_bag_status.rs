#![cfg(feature = "client-mists")]

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_action_micro_bag_and_status_bars_are_interactive() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            A_Admin.ClearActionBars()
            A_Admin.SetActionSlot(1, 19750)
            if not HasAction(1) then
                error("seeded action slot 1 is not active")
            end

            local actionType, actionID = GetActionInfo(1)
            if actionType ~= "spell" or actionID ~= 19750 then
                error("slot 1 action mismatch: " .. tostring(actionType) .. "/" .. tostring(actionID))
            end

            if not ActionButton1 then
                error("ActionButton1 missing")
            end
            ActionButton1:UpdateAction(true)
            if ActionButton1.action ~= 1 then
                error("ActionButton1 action mismatch: " .. tostring(ActionButton1.action))
            end
            if not ActionButton1Icon or not ActionButton1Icon:GetTexture() then
                error("ActionButton1 icon did not update from seeded action")
            end
            local actionButtonBottom = ActionButton1:GetBottom()
            if actionButtonBottom == nil or actionButtonBottom > 80 then
                error("ActionButton1 was not positioned on the bottom action bar: " .. tostring(actionButtonBottom))
            end

            local microButtons = {
                "CharacterMicroButton",
                "SpellbookMicroButton",
                "TalentMicroButton",
                "QuestLogMicroButton",
                "MainMenuMicroButton",
            }
            for _, buttonName in ipairs(microButtons) do
                local button = _G[buttonName]
                if not button or not button:IsShown() then
                    error(buttonName .. " is missing or hidden")
                end
            end
            UpdateMicroButtons()
            MicroButtonTooltipText(nil, nil)

            A_Admin.SetMouseOverFrame(MainMenuMicroButton)
            MainMenuMicroButton:GetScript("OnMouseDown")(MainMenuMicroButton, "LeftButton")
            local onClick = MainMenuMicroButton:GetScript("OnClick")
            if onClick then
                onClick(MainMenuMicroButton, "LeftButton", false)
            end
            MainMenuMicroButton:GetScript("OnMouseUp")(MainMenuMicroButton, "LeftButton")
            A_Admin.SetMouseOverFrame(nil)
            if not (GameMenuFrame and GameMenuFrame:IsShown()) then
                error("MainMenuMicroButton live click sequence did not show GameMenuFrame")
            end
            MainMenuMicroButton:Click()
            if GameMenuFrame:IsShown() then
                error("MainMenuMicroButton second click did not hide GameMenuFrame")
            end

            A_Admin.SetMouseOverFrame(MainMenuMicroButton)
            MainMenuMicroButton:GetScript("OnMouseDown")(MainMenuMicroButton, "LeftButton")
            MainMenuMicroButton:GetScript("OnMouseUp")(MainMenuMicroButton, "LeftButton")
            local clickAfterMouseUp = MainMenuMicroButton:GetScript("OnClick")
            if clickAfterMouseUp then
                clickAfterMouseUp(MainMenuMicroButton, "LeftButton", false)
            end
            A_Admin.SetMouseOverFrame(nil)
            if not GameMenuFrame:IsShown() then
                error("MainMenuMicroButton OnMouseUp/OnClick order closed GameMenuFrame")
            end
            HideUIPanel(GameMenuFrame)

            StoreMicroButton:Click()
            if not (CatalogShopFrame and CatalogShopFrame:IsShown()) then
                error("StoreMicroButton click did not show CatalogShopFrame")
            end
            if not StoreFrame_IsShown() then
                error("StoreMicroButton click did not update store shown state")
            end

            if not MainMenuBarBackpackButton or not CharacterBag0Slot then
                error("Mists bag bar buttons are missing")
            end
            A_Admin.ClearBags()
            A_Admin.AddBagItem(0, 1, 6948, 1)
            MainMenuBarBackpackButton_UpdateFreeSlots()
            if C_Container.CalculateTotalNumberOfFreeBagSlots() ~= 79 then
                error("total free bag slots mismatch: " .. tostring(C_Container.CalculateTotalNumberOfFreeBagSlots()))
            end
            if MainMenuBarBackpackButton.freeSlots ~= 79 then
                error("backpack free slots mismatch: " .. tostring(MainMenuBarBackpackButton.freeSlots))
            end
            if C_Container.GetContainerNumSlots(0) ~= 16 then
                error("backpack slot count changed")
            end

            if MainMenuExpBar then
                if not MainMenuExpBar.TextString then
                    error("MainMenuExpBar TextStatusBar surface missing")
                end
                MainMenuExpBar:SetMinMaxValues(0, 100)
                MainMenuExpBar:SetValue(25)
                ShowTextStatusBarText(MainMenuExpBar)
                local text = MainMenuExpBar.TextString:GetText()
                if not text or not text:find("25") then
                    error("MainMenuExpBar text did not include current value: " .. tostring(text))
                end
            elseif not (
                StatusTrackingBarManager
                and StatusTrackingBarManager.MainStatusTrackingBarContainer
                and StatusTrackingBarManager.MainStatusTrackingBarContainer:IsShown()
            ) then
                error("Mists status tracking bar surface missing")
            end
            "#,
            "dump-tree",
            "--filter-key",
            "MainMenuBar",
        ])
        )
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_no_lua_errors(&stdout, &stderr);
}

#[test]
fn mists_objective_tracker_hides_legacy_watch_frame() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-saved-vars",
            "--exec-lua",
            r#"
            if ObjectiveTrackerFrame == nil or not ObjectiveTrackerFrame:IsVisible() then
                error("ObjectiveTrackerFrame is missing or hidden")
            end
            if WatchFrame ~= nil and WatchFrame:IsVisible() then
                error("legacy WatchFrame is visible alongside ObjectiveTrackerFrame")
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
    assert_no_lua_errors(&stdout, &stderr);
}

fn assert_no_lua_errors(stdout: &str, stderr: &str) {
    assert!(
        !stdout.contains("Lua error") && !stderr.contains("Lua error"),
        "Mists HUD parity probe emitted Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
