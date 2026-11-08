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
fn mists_hud_keeps_unit_frames_and_bottom_bar_textured() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            HUD_LAYOUT_REGRESSION_LUA,
            "lua-errors",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "HUD layout regression probe failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("HUD layout regression probe", &stdout, &stderr);
}

const HUD_LAYOUT_REGRESSION_LUA: &str = r#"
    local function fail(message)
        error(message, 0)
    end

    local function requireFrame(name)
        local frame = _G[name]
        if not frame then
            fail(name .. " missing")
        end
        return frame
    end

    local function requireShown(name)
        local frame = requireFrame(name)
        if frame.IsShown and not frame:IsShown() then
            fail(name .. " hidden")
        end
        return frame
    end

    local function requirePositiveSize(frame, label, minWidth, minHeight)
        local width = frame.GetWidth and frame:GetWidth() or 0
        local height = frame.GetHeight and frame:GetHeight() or 0
        if width < minWidth or height < minHeight then
            fail(label .. " too small: " .. tostring(width) .. "x" .. tostring(height))
        end
    end

    local function requireInside(child, parent, label)
        local childLeft, childRight = child:GetLeft(), child:GetRight()
        local parentLeft, parentRight = parent:GetLeft(), parent:GetRight()
        if not childLeft or not childRight or not parentLeft or not parentRight then
            fail(label .. " has unresolved horizontal geometry")
        end
        if childLeft < parentLeft or childRight > parentRight then
            fail(label .. " leaks outside its parent horizontally")
        end
    end

    local function requireTexture(texture, label)
        if not texture or not texture.GetTexture or not texture:GetTexture() then
            fail(label .. " has no resolved texture")
        end
    end

    A_Admin.SetTarget("Khadgar", 70, 8, false)
    A_Admin.SetTargetHealth(620000, 620000)
    A_Admin.SetTargetPower(180000, 180000, 0)
    if TargetFrame_Update then
        TargetFrame_Update(TargetFrame)
    end
    TargetFrame:Show()

    local targetFrame = requireShown("TargetFrame")
    local targetName = requireFrame("TargetFrameTextureFrameName")
    requirePositiveSize(targetFrame, "TargetFrame", 220, 90)
    requirePositiveSize(targetName, "TargetFrame name", 80, 8)
    requireInside(targetName, targetFrame, "TargetFrame name")
    if targetName:GetText() ~= "Khadgar" then
        fail("TargetFrame did not render the seeded target name")
    end
    requireInside(requireFrame("TargetFrameHealthBar"), targetFrame, "TargetFrame health bar")
    requireInside(requireFrame("TargetFrameManaBar"), targetFrame, "TargetFrame mana bar")
    requireTexture(TargetFrameTextureFrameTexture, "TargetFrame portrait/background art")

    local playerFrame = requireShown("PlayerFrame")
    local playerName = requireFrame("PlayerName")
    requirePositiveSize(playerFrame, "PlayerFrame", 220, 90)
    requirePositiveSize(playerName, "PlayerFrame name", 80, 8)
    requireInside(playerName, playerFrame, "PlayerFrame name")
    requireInside(requireFrame("PlayerFrameHealthBar"), playerFrame, "PlayerFrame health bar")
    requireInside(requireFrame("PlayerFrameManaBar"), playerFrame, "PlayerFrame mana bar")
    requireTexture(PlayerFrameTexture, "PlayerFrame art")

    A_Admin.ClearActionBars()
    for slot = 1, 12 do
        A_Admin.SetActionSlot(slot, 19750)
        local button = requireShown("ActionButton" .. slot)
        ActionButton_Update(button)
        requirePositiveSize(button, "ActionButton" .. slot, 30, 30)
        requireTexture(_G["ActionButton" .. slot .. "Icon"], "ActionButton" .. slot .. " icon")
    end

    for _, name in ipairs({
        "MainMenuBarBackpackButton",
        "CharacterBag0Slot",
        "CharacterBag1Slot",
        "CharacterBag2Slot",
        "CharacterBag3Slot",
    }) do
        local button = requireShown(name)
        requirePositiveSize(button, name, 28, 28)
        requireTexture(button:GetNormalTexture(), name .. " normal texture")
        requireTexture(button:GetHighlightTexture(), name .. " highlight texture")
    end
"#;
