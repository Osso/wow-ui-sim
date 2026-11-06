#![cfg(feature = "client-mists")]

use std::path::PathBuf;
use std::process::Command;

/// Cargo sets `CARGO_BIN_EXE_wow-sim` when it builds this test target, so the
/// path is fixed at compile time. Reading the variable at run time falls back
/// to `<manifest>/target/debug/wow-sim`, which does not exist when the test
/// binary is executed from a shared target directory (as `scripts/slot-build.py`
/// does).
fn wow_sim_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_wow-sim"))
}

#[test]
fn mists_pvp_ui_supports_honor_battleground_and_conquest_panels() {
    let output = Command::new("timeout")
        .arg("90")
        .arg(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            local ok, reason = LoadAddOn("Blizzard_PVPUI")
            if ok == false then
                error("Blizzard_PVPUI failed to load: " .. tostring(reason))
            end
            if not PVPQueueFrame then
                error("PVPQueueFrame missing")
            end

            PVPQueueFrame:Show()
            PVPQueueFrame_Update(PVPQueueFrame)

            if GetNumBattlegroundTypes() < 3 then
                error("battleground list is not seeded")
            end

            HonorQueueFrame_SetType("specific")
            HonorQueueFrameSpecificList_Update()
            local specificButton = HonorQueueFrame.SpecificFrame.buttons[1]
            if not specificButton or not specificButton:IsShown() or not specificButton.bgID then
                error("specific battleground row missing")
            end

            HonorQueueFrame_SetType("bonus")
            HonorQueueFrameBonusFrame_Update()
            if not HonorQueueFrame.BonusFrame.selectedButton or not HonorQueueFrame.BonusFrame.selectedButton.bgID then
                error("bonus battleground selection missing")
            end
            HonorQueueFrame_Queue(false, true)
            local status = GetBattlefieldStatus(HonorQueueFrame.BonusFrame.selectedButton.bgID)
            if status ~= "queued" then
                error("bonus battleground queue did not update status")
            end

            ConquestQueueFrame:Show()
            ConquestQueueFrame_Update(ConquestQueueFrame)
            if not ConquestQueueFrame.NoSeason:IsShown() then
                error("conquest no-season state missing")
            end
            "#,
            "lua-errors",
        ])
        .output()
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
fn mists_toggle_pvp_frame_routes_through_group_finder_without_errors() {
    let output = Command::new("timeout")
        .arg("90")
        .arg(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            TogglePVPFrame()
            if not (PVEFrame and PVEFrame:IsShown()) then
                error("PVEFrame did not open")
            end
            if not (PVPQueueFrame and PVPQueueFrame:IsShown()) then
                error("PVPQueueFrame did not open")
            end
            if not (HonorQueueFrame and HonorQueueFrame:IsShown()) then
                error("HonorQueueFrame did not open")
            end
            "#,
            "lua-errors",
        ])
        .output()
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
    // The `lua-errors` subcommand prints the unique error array last on stdout
    // and always writes a final summary to stderr (`=== Final observed Lua error
    // summary ===`, `Lua errors: N unique`). A substring search for "Lua error"
    // therefore matches that summary even when no error was recorded.
    assert_eq!(
        stdout.trim(),
        "[]",
        "PvP panel flow reported Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("[exec-lua] error"),
        "the exec-lua probe failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
