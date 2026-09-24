#![cfg(feature = "client-wowforever")]

#[test]
fn character_panel_movement_speed_survives_open_close_and_updates() {
    let root = tempfile::tempdir().unwrap();
    let output = std::process::Command::new("timeout")
        .args(["90", env!("CARGO_BIN_EXE_wow-sim")])
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            for cycle = 1, 2 do
                ToggleCharacter("PaperDollFrame")
                assert(CharacterFrame:IsShown(), "character panel did not open")
                assert(PaperDollFrame:IsShown(), "paper doll did not open")
                PaperDollFrame_UpdateStats()
                local pane = CharacterFrame:GetStatsPane()
                local found = false
                for _, entry in ipairs(pane.elementData) do
                    if entry.name == "MOVESPEED" then found = true end
                end
                assert(found, "movement speed missing from character stats")
                local row = CharacterStatsPane.statsFramePool:Acquire()
                PaperDollFrame_SetMovementSpeed(row, "player")
                assert(math.abs(row.runSpeed - 100) < 0.001)
                assert(math.abs(row.speed - row.runSpeed) < 0.001)
                A_Admin.SetSwimming(true)
                MovementSpeed_OnUpdate(row, 0.016)
                assert(math.abs(row.speed - row.swimSpeed) < 0.001)
                A_Admin.SetSwimming(false)
                MovementSpeed_OnUpdate(row, 0.016)
                assert(math.abs(row.speed - row.runSpeed) < 0.001)
                CharacterStatsPane.statsFramePool:Release(row)
                ToggleCharacter("PaperDollFrame")
                assert(not CharacterFrame:IsShown(), "character panel did not close")
            end
            print("CHARACTER_MOVEMENT_SPEED_DONE")
            "#,
            "lua-errors",
        ])
        .env("XDG_DATA_HOME", root.path().join("data"))
        .env("WOW_SIM_WTF_PATH", root.path().join("wtf"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(
        stdout.contains("CHARACTER_MOVEMENT_SPEED_DONE"),
        "{stdout}\n{stderr}"
    );
    assert!(stdout.trim_end().ends_with("[]"), "{stdout}\n{stderr}");
}
