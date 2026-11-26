//! Temporary DamageMeter scrollbox extent workaround.
//!
//! The simulator does not yet model enough of DamageMeter's live scrollbox
//! layout timing, so newly initialized session/source windows can keep a zero
//! element extent. Seed the extent from the addon's bar height until the
//! underlying scrollbox/layout path is complete.

use crate::lua_api::LoaderEnv;

const DAMAGE_METER_INITIAL_SCROLLBOX_EXTENT_LUA: &str = r#"
local function patch_damage_meter_window_initialize_scrollbox(mixinName)
    local mixin = rawget(_G, mixinName)
    if type(mixin) ~= "table" or type(mixin.InitializeScrollBox) ~= "function" or mixin.__wow_initial_extent_patch then
        return
    end

    mixin.__wow_initial_extent_patch = true
    local original = mixin.InitializeScrollBox
    mixin.InitializeScrollBox = function(self, ...)
        local result = original(self, ...)
        local scrollBox = type(self.GetScrollBox) == "function" and self:GetScrollBox() or nil
        local view = scrollBox and type(scrollBox.GetView) == "function" and scrollBox:GetView() or nil
        if view and type(view.SetElementExtent) == "function" then
            view:SetElementExtent(self:GetBarHeight())
        end
        return result
    end
end

patch_damage_meter_window_initialize_scrollbox("DamageMeterSessionWindowMixin")
patch_damage_meter_window_initialize_scrollbox("DamageMeterSourceWindowMixin")

local function apply_damage_meter_scrollbox_extent(window)
    if type(window) ~= "table" or type(window.GetScrollBox) ~= "function" or type(window.GetBarHeight) ~= "function" then
        return
    end
    local scrollBox = window:GetScrollBox()
    local view = scrollBox and type(scrollBox.GetView) == "function" and scrollBox:GetView() or nil
    if view and type(view.SetElementExtent) == "function" then
        view:SetElementExtent(window:GetBarHeight())
        if type(scrollBox.FullUpdate) == "function" and ScrollBoxConstants then
            scrollBox:FullUpdate(ScrollBoxConstants.UpdateImmediately)
        end
    end
end

if type(DamageMeter) == "table" and type(DamageMeter.ForEachSessionWindow) == "function" then
    DamageMeter:ForEachSessionWindow(function(sessionWindow)
        apply_damage_meter_scrollbox_extent(sessionWindow)
        if type(sessionWindow.GetSourceWindow) == "function" then
            apply_damage_meter_scrollbox_extent(sessionWindow:GetSourceWindow())
        end
    end)
end
"#;

pub(crate) fn patch(env: &LoaderEnv<'_>) {
    let _ = env.exec(DAMAGE_METER_INITIAL_SCROLLBOX_EXTENT_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn initializes_and_refreshes_window_extents() {
        let env = WowLuaEnv::new().expect("env should initialize");
        env.exec(
            r#"
            ScrollBoxConstants = { UpdateImmediately = "now" }

            local function makeWindow(height)
                local view = {
                    extent = 0,
                    SetElementExtent = function(self, extent)
                        self.extent = extent
                    end,
                }
                local scrollBox = {
                    view = view,
                    fullUpdates = 0,
                    GetView = function(self)
                        return self.view
                    end,
                    FullUpdate = function(self, mode)
                        self.fullUpdates = self.fullUpdates + 1
                        self.lastMode = mode
                    end,
                }
                return {
                    scrollBox = scrollBox,
                    GetScrollBox = function(self)
                        return self.scrollBox
                    end,
                    GetBarHeight = function()
                        return height
                    end,
                }
            end

            sessionWindow = makeWindow(19)
            sourceWindow = makeWindow(23)
            function sessionWindow:GetSourceWindow()
                return sourceWindow
            end

            DamageMeterSessionWindowMixin = {
                InitializeScrollBox = function(self)
                    self.initialized = true
                    return "session-result"
                end,
            }
            DamageMeterSourceWindowMixin = {
                InitializeScrollBox = function(self)
                    self.initialized = true
                    return "source-result"
                end,
            }
            DamageMeter = {
                ForEachSessionWindow = function(_, callback)
                    callback(sessionWindow)
                end,
            }
            "#,
        )
        .expect("fake DamageMeter surface should install");

        patch(&env.loader_env());

        let (session_extent, source_extent, session_updates, source_updates): (i64, i64, i64, i64) =
            env.eval(
                r#"
                return sessionWindow.scrollBox.view.extent,
                    sourceWindow.scrollBox.view.extent,
                    sessionWindow.scrollBox.fullUpdates,
                    sourceWindow.scrollBox.fullUpdates
                "#,
            )
            .expect("patched existing windows should be readable");

        assert_eq!(session_extent, 19);
        assert_eq!(source_extent, 23);
        assert_eq!(session_updates, 1);
        assert_eq!(source_updates, 1);

        let initialized_extent: i64 = env
            .eval(
                r#"
                local newWindow = {
                    scrollBox = {
                        view = {
                            extent = 0,
                            SetElementExtent = function(self, extent)
                                self.extent = extent
                            end,
                        },
                        GetView = function(self)
                            return self.view
                        end,
                    },
                    GetScrollBox = function(self)
                        return self.scrollBox
                    end,
                    GetBarHeight = function()
                        return 31
                    end,
                }
                DamageMeterSessionWindowMixin.InitializeScrollBox(newWindow)
                return newWindow.scrollBox.view.extent
                "#,
            )
            .expect("patched mixin should seed new window extent");

        assert_eq!(initialized_extent, 31);
    }
}
