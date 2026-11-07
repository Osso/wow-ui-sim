#![cfg(feature = "client-mists")]

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[test]
fn mists_quest_log_selects_entries_and_refreshes_tracker() {
    let output = bounded_process::output(
        90,
        Command::new(env!("CARGO_BIN_EXE_wow-sim")).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleQuestLog()
            if QuestLogFrame_Update then
                QuestLogFrame_Update()
            end

            local entries = GetNumQuestLogEntries()
            if entries < 2 then
                error("quest log has no quest entries")
            end

            local headerTitle, _, _, isHeader = GetQuestLogTitle(1)
            if headerTitle == nil or not isHeader then
                error("quest log first row is not a header")
            end

            local questTitle, _, _, questIsHeader, _, _, _, questID = GetQuestLogTitle(2)
            if questTitle == nil or questTitle == "" or questIsHeader or not questID then
                error("quest log second row is not a quest")
            end

            SelectQuestLogEntry(2)
            if GetQuestLogSelection() ~= 2 then
                error("SelectQuestLogEntry did not update the selected log index")
            end

            if GetAbandonQuestName() ~= questTitle then
                error("GetAbandonQuestName did not return the selected quest title")
            end

            if not CanAbandonQuest(questID) then
                error("selected quest should be abandonable")
            end

            if GetNumQuestWatches() < 1 then
                error("objective tracker has no watched quests")
            end

            if GetQuestIndexForWatch(1) ~= 2 then
                error("first watched quest does not resolve to the first quest row")
            end

            if not IsQuestWatched(2) then
                error("first quest row should be tracked")
            end

            WatchFrame_Update()
            if not WatchFrameHeader:IsShown() then
                error("watch frame header did not show tracked objectives")
            end

            local trackerTitle = WatchFrameTitle:GetText()
            if not trackerTitle or not trackerTitle:find("Objectives", 1, true) then
                error("watch frame title did not render objectives text")
            end

            local probeLine = WatchFrame.linePool:Acquire()
            local fontPath, fontSize = probeLine.text:GetFont()
            WatchFrame.linePool:Release(probeLine)
            if not fontPath or not fontPath:find("FRIZQT", 1, true) or fontSize ~= 12 then
                error("watch frame objective lines did not inherit WatchFontTemplate")
            end

            local collapseTexture = WatchFrameCollapseExpandButton:GetNormalTexture()
            local collapsePath = collapseTexture and collapseTexture:GetTexture()
            if not collapsePath or not collapsePath:find("UI-Panel-QuestHideButton", 1, true) then
                error("watch frame collapse button texture was missing")
            end
            if not WatchFrameCollapseExpandButton:IsVisible()
                or WatchFrameCollapseExpandButton:GetWidth() <= 0
                or WatchFrameCollapseExpandButton:GetHeight() <= 0 then
                error("watch frame collapse button chrome is not renderable")
            end

            local poiButton = WatchFrameLines.poiTable and WatchFrameLines.poiTable.numeric[1]
            local poiTexture = poiButton and poiButton.Display and poiButton.Display.Icon
            local poiPath = poiTexture and poiTexture:GetTexture()
            if not poiPath or not poiPath:find("UI-QuestPoi-NumberIcons", 1, true) then
                error("watch frame quest POI texture was missing")
            end
            if not poiButton:IsVisible()
                or not poiTexture:IsVisible()
                or poiTexture:GetWidth() <= 0
                or poiTexture:GetHeight() <= 0 then
                error("watch frame quest POI chrome is not renderable")
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
        "quest log opened with Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
