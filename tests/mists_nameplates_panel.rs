#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

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
fn mists_nameplate_driver_acquires_renderable_unit_frame() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            if type(NamePlateDriverFrame) ~= "table" then
                error("NamePlateDriverFrame missing")
            end
            if tonumber(GetCVar("NamePlateVerticalScale")) == nil then
                error("NamePlateVerticalScale default is not numeric")
            end

            local plate = CreateFrame("Frame", "MistsNamePlateRenderProbe", UIParent)
            plate:SetSize(128, 32)
            NamePlateDriverFrame:OnNamePlateCreated(plate)
            NamePlateDriverFrame:AcquireUnitFrame(plate)
            CompactUnitFrame_SetUpFrame(plate.UnitFrame, DefaultCompactNamePlateEnemyFrameSetup)

            local unitFrame = plate.UnitFrame
            if unitFrame == nil then
                error("driver did not acquire UnitFrame")
            end
            if unitFrame:GetWidth() ~= 128 or unitFrame:GetHeight() ~= 32 then
                error("UnitFrame does not fill nameplate: " .. unitFrame:GetWidth() .. "x" .. unitFrame:GetHeight())
            end

            local healthBar = unitFrame.healthBar
            if healthBar == nil or healthBar:GetWidth() <= 0 or healthBar:GetHeight() <= 0 then
                error("health bar is not renderable")
            end
            if healthBar:GetStatusBarTexture() == nil or healthBar:GetStatusBarTexture():GetTexture() == nil then
                error("health bar texture missing")
            end

            if unitFrame.CastBar == nil or unitFrame.CastBar:GetWidth() <= 0 or unitFrame.CastBar:GetHeight() <= 0 then
                error("cast bar is not renderable")
            end
            "#,
            "lua-errors",
        ])
        )
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("Nameplate rendering probe", &stdout, &stderr);
}

