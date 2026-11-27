//! Temporary Artifact UI `ShowUIPanel` guard.
//!
//! This keeps hidden Artifact panels from opening when the simulated artifact
//! viewability state says retail would route through the panel failure handler.

use crate::lua_api::LoaderEnv;

const ARTIFACT_UI_SHOW_PANEL_GUARD_WORKAROUND_LUA: &str = r#"
if rawget(_G, "__wow_artifact_ui_show_panel_guard_wrapped") then
    return
end

if type(ShowUIPanel) ~= "function" then
    return
end

local originalShowUIPanel = ShowUIPanel

local function shouldBlockArtifactPanel(frame)
    return frame == ArtifactFrame
        and type(ArtifactUI_CanViewArtifact) == "function"
        and not ArtifactUI_CanViewArtifact()
end

local function callArtifactShowFailedFunc()
    local entry = type(UIPanelWindows) == "table" and UIPanelWindows["ArtifactFrame"] or nil
    local showFailedFunc = type(entry) == "table" and entry.showFailedFunc or nil
    if type(showFailedFunc) == "function" then
        showFailedFunc()
    end
end

ShowUIPanel = function(frame, ...)
    if frame and frame:IsShown() then
        return originalShowUIPanel(frame, ...)
    end

    if shouldBlockArtifactPanel(frame) then
        callArtifactShowFailedFunc()
        return
    end

    return originalShowUIPanel(frame, ...)
end

rawset(_G, "__wow_artifact_ui_show_panel_guard_wrapped", true)
"#;

pub(crate) fn patch(env: &LoaderEnv<'_>) {
    let _ = env.exec(ARTIFACT_UI_SHOW_PANEL_GUARD_WORKAROUND_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;

    fn patch_env(env: &WowLuaEnv) {
        let _ = env.exec(ARTIFACT_UI_SHOW_PANEL_GUARD_WORKAROUND_LUA);
    }

    fn install_show_panel_fixtures(env: &WowLuaEnv) {
        env.exec(
            r#"
            show_calls = 0
            failed_calls = 0
            ArtifactFrame = {
                shown = false,
                IsShown = function(self)
                    return self.shown
                end,
            }
            OtherFrame = {
                shown = false,
                IsShown = function(self)
                    return self.shown
                end,
            }
            UIPanelWindows = {
                ArtifactFrame = {
                    showFailedFunc = function()
                        failed_calls = failed_calls + 1
                    end,
                },
            }
            ShowUIPanel = function(frame)
                show_calls = show_calls + 1
                return frame
            end
            "#,
        )
        .expect("show panel fixtures should install");
    }

    #[test]
    fn blocks_unviewable_hidden_artifact_panel() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_show_panel_fixtures(&env);
        env.exec("ArtifactUI_CanViewArtifact = function() return false end")
            .expect("artifact viewability fixture should install");

        patch_env(&env);
        env.exec("ShowUIPanel(ArtifactFrame)")
            .expect("artifact show call should run");

        let (show_calls, failed_calls): (i64, i64) = env
            .eval("return show_calls, failed_calls")
            .expect("show panel counters should be readable");

        assert_eq!(show_calls, 0);
        assert_eq!(failed_calls, 1);
    }

    #[test]
    fn lets_already_shown_artifact_panel_call_original() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_show_panel_fixtures(&env);
        env.exec(
            r#"
            ArtifactFrame.shown = true
            ArtifactUI_CanViewArtifact = function()
                return false
            end
            "#,
        )
        .expect("shown artifact fixture should install");

        patch_env(&env);
        env.exec("ShowUIPanel(ArtifactFrame)")
            .expect("shown artifact show call should run");

        let (show_calls, failed_calls): (i64, i64) = env
            .eval("return show_calls, failed_calls")
            .expect("show panel counters should be readable");

        assert_eq!(show_calls, 1);
        assert_eq!(failed_calls, 0);
    }

    #[test]
    fn lets_non_artifact_panel_call_original() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_show_panel_fixtures(&env);
        env.exec("ArtifactUI_CanViewArtifact = function() return false end")
            .expect("artifact viewability fixture should install");

        patch_env(&env);
        env.exec("ShowUIPanel(OtherFrame)")
            .expect("ordinary show call should run");

        let (show_calls, failed_calls): (i64, i64) = env
            .eval("return show_calls, failed_calls")
            .expect("show panel counters should be readable");

        assert_eq!(show_calls, 1);
        assert_eq!(failed_calls, 0);
    }
}
