#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::{Command, Output};

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_craft_frame_load_ui_creates_renderable_craft_frame() {
    assert_panel_loads_cleanly(
        "CraftFrame",
        "CraftFrame",
        r#"
            CraftFrame_LoadUI()
            if not CraftFrame then
                error("CraftFrame was not created")
            end
            ShowUIPanel(CraftFrame)
            if not CraftFrame:IsShown() then
                error("CraftFrame did not show")
            end
            if (CraftFrame:GetWidth() or 0) <= 0 or (CraftFrame:GetHeight() or 0) <= 0 then
                error("CraftFrame has no renderable bounds")
            end
            "#,
    );
}

#[test]
fn mists_trade_skill_frame_load_ui_creates_renderable_trade_skill_frame() {
    assert_panel_loads_cleanly(
        "TradeSkillFrame",
        "TradeSkillFrame",
        r#"
            TradeSkillFrame_LoadUI()
            if not TradeSkillFrame then
                error("TradeSkillFrame was not created")
            end
            ShowUIPanel(TradeSkillFrame)
            if not TradeSkillFrame:IsShown() then
                error("TradeSkillFrame did not show")
            end
            if (TradeSkillFrame:GetWidth() or 0) <= 0 or (TradeSkillFrame:GetHeight() or 0) <= 0 then
                error("TradeSkillFrame has no renderable bounds")
            end
            "#,
    );
}

#[test]
fn mists_class_trainer_frame_loads_with_named_textures() {
    assert_panel_loads_cleanly(
        "ClassTrainerFrame",
        "ClassTrainerFrame",
        r#"
            ClassTrainerFrame_LoadUI()
            if not ClassTrainerFrame then
                error("ClassTrainerFrame was not created")
            end
            if not ClassTrainerExpandTabMiddle then
                error("ClassTrainerExpandTabMiddle was not created")
            end
            ClassTrainerFrame_Show()
            if not ClassTrainerFrame:IsShown() then
                error("ClassTrainerFrame did not show")
            end
            if (ClassTrainerFrame:GetWidth() or 0) <= 0 or (ClassTrainerFrame:GetHeight() or 0) <= 0 then
                error("ClassTrainerFrame has no renderable bounds")
            end
            "#,
    );
}

fn assert_panel_loads_cleanly(panel_name: &str, filter_name: &str, exec_lua: &str) {
    let output_path = format!("/tmp/mists-{}-panel", panel_name.to_ascii_lowercase());
    let output = capture_panel_screenshot(filter_name, exec_lua, &output_path);
    assert_panel_screenshot_output(panel_name, &output);
}

fn capture_panel_screenshot(filter_name: &str, exec_lua: &str, output_path: &str) -> Output {
    bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            exec_lua,
            "screenshot",
            "--filter",
            filter_name,
            "--output",
            output_path,
        ]),
    )
    .expect("failed to run wow-sim screenshot")
}

fn assert_panel_screenshot_output(panel_name: &str, output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "{panel_name} screenshot failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_sink_errors(panel_name, &stdout, &stderr);
    assert_panel_rendered_textures(panel_name, &stderr);
}

fn assert_panel_rendered_textures(panel_name: &str, stderr: &str) {
    assert!(
        parse_texture_request_count(stderr).is_some_and(|count| count > 1),
        "{panel_name} screenshot did not render panel textures\nstderr:\n{stderr}"
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
