#![cfg(feature = "client-mists")]

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
fn mists_utility_and_dialog_panels_support_state_changing_interactions() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            UTILITY_DIALOG_INTERACTIONS_LUA,
            "lua-errors",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "utility/dialog interaction flow failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_no_lua_errors(&stdout, &stderr);
}

fn assert_no_lua_errors(stdout: &str, stderr: &str) {
    assert!(
        !stdout.contains("Lua error")
            && !stderr.contains("Lua error")
            && !stdout.contains("[exec-lua] error")
            && !stderr.contains("[exec-lua] error"),
        "utility/dialog interaction flow emitted Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

const UTILITY_DIALOG_INTERACTIONS_LUA: &str = r#"
    local function fail(message)
        error(message, 0)
    end

    local function assertShown(frame, label)
        if not frame or not frame:IsShown() then
            fail(label .. " did not show")
        end
    end

    LoadAddOn("Blizzard_ChallengesUI")
    LoadAddOn("Blizzard_PVEUI")
    PVEFrame:Show()
    ChallengesFrame:Show()
    assertShown(ChallengesFrame, "ChallengesFrame")
    local challengeButton = ChallengesFrameDungeonButton1
    if not challengeButton or not challengeButton.mapID then
        fail("ChallengesFrame did not seed a dungeon button")
    end
    if ChallengesFrame.selectedMapID ~= nil then
        fail("ChallengesFrame unexpectedly started with a selected map")
    end
    challengeButton:Click()
    if ChallengesFrame.selectedMapID ~= challengeButton.mapID then
        fail("ChallengesFrame dungeon button click did not select its map")
    end
    if not challengeButton.selectedTex:IsShown() then
        fail("ChallengesFrame selected dungeon did not show selected texture")
    end
    if ChallengesFrame.details.MapName:GetText() ~= challengeButton.text:GetText() then
        fail("ChallengesFrame selected dungeon did not update visible details")
    end

    QuestChoice_LoadUI()
    ShowUIPanel(QuestChoiceFrame)
    assertShown(QuestChoiceFrame, "QuestChoiceFrame")
    if QuestChoiceFrame.choiceID ~= 9001 then
        fail("QuestChoiceFrame did not retain the seeded choice id")
    end
    local firstChoice = QuestChoiceFrame.Option1
    if not firstChoice.optID or firstChoice.OptionButton:GetText() == "" then
        fail("QuestChoiceFrame did not populate the first option")
    end
    firstChoice.OptionButton:Click()
    if QuestChoiceFrame:IsShown() then
        fail("QuestChoiceFrame option button did not hide the dialog")
    end

    TimeManager_LoadUI()
    ShowUIPanel(TimeManagerFrame)
    assertShown(TimeManagerFrame, "TimeManagerFrame")
    if StopwatchFrame:IsShown() then
        Stopwatch_Toggle()
    end
    TimeManagerStopwatchCheck:Click()
    if not StopwatchFrame:IsShown() or not TimeManagerStopwatchCheck:GetChecked() then
        fail("TimeManager stopwatch checkbox did not show the stopwatch")
    end
    TimeManagerAlarmMessageEditBox:SetText("Mists parity alarm")
    TimeManagerAlarmMessageEditBox_OnEditFocusLost(TimeManagerAlarmMessageEditBox)
    if GetCVar("timeMgrAlarmMessage") ~= "Mists parity alarm" then
        fail("TimeManager alarm message did not persist through its editbox handler")
    end
    local militaryBefore = GetCVar("timeMgrUseMilitaryTime")
    TimeManagerMilitaryTimeCheck:Click()
    if GetCVar("timeMgrUseMilitaryTime") == militaryBefore then
        fail("TimeManager military-time checkbox did not update its CVar")
    end

    LoadAddOn("Blizzard_MovePad")
    MovePadFrame:Show()
    assertShown(MovePadFrame, "MovePadFrame")
    MovePadFrame:SetPressAndHoldMode(false)
    local forwardStart = 0
    local forwardStop = 0
    local backwardStart = 0
    MovePadForward.startAction = function() forwardStart = forwardStart + 1 end
    MovePadForward.stopAction = function() forwardStop = forwardStop + 1 end
    MovePadBackward.startAction = function() backwardStart = backwardStart + 1 end
    MovePadBackward.stopAction = function() end

    MovePadForward:Click()
    if not MovePadForward:GetChecked() or forwardStart ~= 1 then
        fail("MovePad forward click did not check the button and start movement")
    end
    MovePadBackward:Click()
    if MovePadForward:GetChecked() or not MovePadBackward:GetChecked() then
        fail("MovePad opposing button click did not swap checked state")
    end
    if forwardStop == 0 or backwardStart ~= 1 then
        fail("MovePad opposing button click did not stop forward and start backward")
    end
"#;
