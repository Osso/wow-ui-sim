#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_lfg_queue_shows_minimap_eye_icon() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            SetLFGDungeon(LE_LFG_CATEGORY_LFD, 1203)
            JoinLFG(LE_LFG_CATEGORY_LFD)

            if not MiniMapLFGFrame or not MiniMapLFGFrame:IsShown() then
                error("MiniMapLFGFrame did not show after joining LFD")
            end
            if not MiniMapLFGFrame.eye or not MiniMapLFGFrame.eye:IsShown() then
                error("MiniMapLFGFrame eye child did not show after joining LFD")
            end
            if not MiniMapLFGFrame.eye.Texture or not MiniMapLFGFrame.eye.Texture:IsShown() then
                error("MiniMapLFGFrame eye texture did not show after joining LFD")
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
    lua_error_output::assert_no_lua_errors("Mists LFG eye probe", &stdout, &stderr);
}
