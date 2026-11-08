#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::path::PathBuf;
use std::process::Command;

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
fn mists_raid_unit_frames_emit_filtered_textures() {
    let output = run_filtered_screenshot(
        "RaidFrame",
        r#"
        A_Admin.SetPartySize(6)
        A_Admin.SetInstanceInfo("Vault of Archavon", "raid", 16, 20)
        for i = 1, 6 do
            A_Admin.SetPartyMember(i, "Raider" .. i, ((i - 1) % 11) + 1, 90)
        end
        LoadAddOn("Blizzard_RaidUI")
        RaidParentFrame:Show()
        RaidParentFrame_SetView(1)
        RaidFrame:Show()
        RaidGroupFrame_Update()
        for i = 1, 8 do
            local group = _G["RaidGroup" .. i]
            if group then group:Show() end
        end
        "#,
    );

    assert_filtered_screenshot_has_unit_frame_textures(&output, "RaidFrame");
}

#[test]
fn mists_arena_enemy_frames_emit_filtered_textures() {
    let output = run_filtered_screenshot(
        "ArenaEnemyFrame1",
        r#"
        A_Admin.SetInstanceInfo("Nagrand Arena", "arena", 0, 5)
        LoadAddOn("Blizzard_ArenaUI")
        ArenaEnemyFrames_Enable(ArenaEnemyFrames)
        for i = 1, 5 do
            local frame = _G["ArenaEnemyFrame" .. i]
            if frame then
                ArenaEnemyFrame_SetMysteryPlayer(frame)
                frame:Show()
            end
        end
        "#,
    );

    assert_filtered_screenshot_has_unit_frame_textures(&output, "ArenaEnemyFrame1");
}

fn run_filtered_screenshot(root: &str, lua: &str) -> std::process::Output {
    Command::new("timeout")
        .arg("90")
        .arg(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            lua,
            "screenshot",
            "--filter",
            root,
            "--output",
            "/tmp/mists-unit-frame-test",
        ])
        .output()
        .expect("failed to run wow-sim screenshot")
}

fn assert_filtered_screenshot_has_unit_frame_textures(output: &std::process::Output, root: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "{root} screenshot failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_sink_errors(root, &stdout, &stderr);

    let texture_requests = parse_texture_request_count(&stderr)
        .unwrap_or_else(|| panic!("{root} screenshot did not report QuadBatch\nstderr:\n{stderr}"));
    assert!(
        texture_requests > 1,
        "{root} filtered screenshot only emitted background texture request\nstderr:\n{stderr}"
    );
}

fn parse_texture_request_count(stderr: &str) -> Option<u32> {
    stderr.lines().rev().find_map(|line| {
        let (_, suffix) = line.split_once("QuadBatch: ")?;
        let (_, texture_suffix) = suffix.split_once(" quads, ")?;
        let (count, _) = texture_suffix.split_once(" texture requests")?;
        count.parse().ok()
    })
}
