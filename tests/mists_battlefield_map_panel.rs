#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_battlefield_map_screenshot_dispatch_avoids_zero_canvas_scale() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleBattlefieldMap()
            if not (BattlefieldMapFrame and BattlefieldMapFrame:IsShown()) then
                error("BattlefieldMapFrame did not open")
            end
            if not (BattlefieldMapFrame.ScrollContainer and BattlefieldMapFrame.ScrollContainer.Child) then
                error("BattlefieldMapFrame has no scroll canvas")
            end
            local canvasScale = BattlefieldMapFrame.ScrollContainer.Child:GetScale()
            if canvasScale <= 0 then
                error("BattlefieldMapFrame scroll canvas scale is not positive")
            end
            "#,
            "screenshot",
            "--filter",
            "BattlefieldMapFrame",
            "--output",
            "/tmp/mists-battlefield-map-panel",
        ])
        )
        .expect("failed to run wow-sim screenshot");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "BattlefieldMapFrame screenshot failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("SetScale(): Scale must be > 0")
            && !stderr.contains("SetScale(): Scale must be > 0"),
        "BattlefieldMapFrame screenshot emitted SetScale error\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_sink_errors("BattlefieldMapFrame screenshot", &stdout, &stderr);
    assert!(
        parse_texture_request_count(&stderr).is_some_and(|count| count > 1),
        "BattlefieldMapFrame screenshot did not render map textures\nstderr:\n{stderr}"
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
