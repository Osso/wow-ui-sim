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
fn mists_game_menu_options_drives_settings_help_and_addons() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ToggleGameMenu()
            if GameMenuFrame == nil or GameMenuFrame:IsShown() ~= true then
                error("GameMenuFrame did not open")
            end

            local function findGameMenuButton(label, optional)
                if GameMenuFrame.buttonPool == nil or type(GameMenuFrame.buttonPool.EnumerateActive) ~= "function" then
                    error("GameMenuFrame button pool is missing")
                end
                for button in GameMenuFrame.buttonPool:EnumerateActive() do
                    if button:GetText() == label then
                        return button
                    end
                end
                if optional then
                    return nil
                end
                error("Game menu button is missing: " .. tostring(label))
            end

            findGameMenuButton(GAMEMENU_OPTIONS):Click()
            if SettingsPanel == nil or SettingsPanel:IsShown() ~= true then
                error("SettingsPanel did not open from game-menu options")
            end
            if GameMenuFrame:IsShown() then
                error("GameMenuFrame stayed open after opening options")
            end

            local currentCategory = SettingsPanel.GetCurrentCategory and SettingsPanel:GetCurrentCategory()
            if currentCategory == nil or currentCategory:GetID() == nil then
                error("settings category did not focus")
            end

            if EditModeManagerFrame ~= nil
                and type(EditModeManagerFrame.IsShown) == "function"
                and EditModeManagerFrame:IsShown() then
                error("Mists game-menu options flow should leave EditMode hidden")
            end

            SettingsPanel.CloseButton:Click()
            if SettingsPanel:IsShown() then
                error("SettingsPanel did not close")
            end
            if GameMenuFrame == nil or GameMenuFrame:IsShown() ~= true then
                error("GameMenuFrame did not reopen after closing options")
            end

            findGameMenuButton(GAMEMENU_OPTIONS):Click()
            if SettingsPanel == nil or SettingsPanel:IsShown() ~= true then
                error("SettingsPanel did not reopen from game-menu options")
            end
            SettingsPanel.ApplyButton:Click()
            if not SettingsPanel:IsShown() then
                error("Apply button should not close SettingsPanel")
            end

            SettingsPanel.CloseButton:Click()
            findGameMenuButton(GAMEMENU_SUPPORT):Click()
            if HelpFrame == nil or HelpFrame:IsShown() ~= true then
                error("HelpFrame did not open")
            end
            HideUIPanel(HelpFrame)

            ToggleGameMenu()
            local addonsButton = findGameMenuButton(ADDONS, true)
            if addonsButton and addonsButton:IsShown() then
                addonsButton:Click()
                if AddonList == nil or AddonList:IsShown() ~= true then
                    error("AddonList did not open")
                end
                HideUIPanel(AddonList)
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
    lua_error_output::assert_no_lua_errors("Interface options", &stdout, &stderr);
}
