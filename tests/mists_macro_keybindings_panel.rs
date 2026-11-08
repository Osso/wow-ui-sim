#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_macro_selection_and_keybindings_mutate_state() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            ShowMacroFrame()
            if MacroFrame == nil or MacroFrame:IsShown() ~= true then
                error("MacroFrame did not open")
            end
            if MacroFrame.MacroSelector == nil or MacroFrame.MacroSelector.numMacros <= 0 then
                error("Macro selector did not populate")
            end
            if MacroFrameSelectedMacroName:GetText() == nil then
                error("selected macro name did not populate")
            end

            local accountCount, characterCount = GetNumMacros()
            if accountCount <= 0 or characterCount <= 0 then
                error("macro counts are empty")
            end
            local name, icon, body = GetMacroInfo(1)
            if type(name) ~= "string" or type(icon) ~= "string" or type(body) ~= "string" then
                error("macro info did not populate")
            end
            RunMacro(1)

            local bindingIndex = C_KeyBindings.GetBindingIndex("INTERACTTARGET")
            if type(bindingIndex) ~= "number" then
                error("INTERACTTARGET binding index missing")
            end
            local action = GetBinding(bindingIndex)
            if action ~= "INTERACTTARGET" then
                error("binding registry row did not round trip")
            end

            SetBinding("CTRL-M", "TOGGLEWORLDMAP")
            if GetBindingAction("CTRL-M") ~= "TOGGLEWORLDMAP" then
                error("SetBinding did not round trip")
            end
            SetModifiedClick("SELFCAST", "CTRL")
            if GetModifiedClick("SELFCAST") ~= "CTRL" then
                error("modified click did not round trip")
            end

            SettingsPanel:OpenToCategory(KEY_BINDINGS)
            if SettingsPanel == nil or SettingsPanel:IsShown() ~= true then
                error("SettingsPanel keybindings category did not open")
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
    lua_error_output::assert_no_lua_errors("Macro/keybindings panel", &stdout, &stderr);
}
