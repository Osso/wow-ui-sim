//! Temporary Settings canvas visibility workaround.
//!
//! Preserve the existing visibility compatibility policy on non-Retail profiles.
//! Retail leaves the original Settings registration, selection and startup intact.

use crate::lua_api::WowLuaEnv;

pub(crate) const SETTINGS_CANVAS_LAYOUT_HIDE_LUA: &str = r#"
local function __wow_hide_settings_canvas_frame(frame, layout)
    if type(frame) ~= "table" or type(frame.Hide) ~= "function" then
        return
    end

    local panel = rawget(_G, "SettingsPanel")
    local isCurrentCanvas = false
    if type(panel) == "table"
        and type(panel.IsShown) == "function"
        and panel:IsShown()
        and type(panel.GetCurrentLayout) == "function"
    then
        local ok, currentLayout = pcall(panel.GetCurrentLayout, panel)
        isCurrentCanvas = ok and currentLayout == layout
    end

    if not isCurrentCanvas then
        frame:Hide()
    end
end

local function __wow_hide_registered_settings_canvas_frames()
    local panel = rawget(_G, "SettingsPanel")
    if type(panel) ~= "table"
        or type(panel.GetAllCategories) ~= "function"
        or type(panel.GetLayout) ~= "function"
    then
        return
    end

    local ok, categories = pcall(panel.GetAllCategories, panel)
    if not ok or type(categories) ~= "table" then
        return
    end

    for _, category in ipairs(categories) do
        local layoutOk, layout = pcall(panel.GetLayout, panel, category)
        if layoutOk
            and type(layout) == "table"
            and type(layout.GetFrame) == "function"
            and type(layout.GetLayoutType) == "function"
            and SettingsLayoutMixin
            and layout:GetLayoutType() == SettingsLayoutMixin.LayoutType.Canvas
        then
            local frameOk, frame = pcall(layout.GetFrame, layout)
            if frameOk then
                __wow_hide_settings_canvas_frame(frame, layout)
            end
        end
    end
end

local function __wow_show_current_settings_canvas_frame()
    local panel = rawget(_G, "SettingsPanel")
    if type(panel) ~= "table"
        or type(panel.GetCurrentCategory) ~= "function"
        or type(panel.GetLayout) ~= "function"
    then
        return
    end

    local categoryOk, category = pcall(panel.GetCurrentCategory, panel)
    if not categoryOk or type(category) ~= "table" then
        return
    end

    local layoutOk, layout = pcall(panel.GetLayout, panel, category)
    if not layoutOk
        or type(layout) ~= "table"
        or type(layout.GetFrame) ~= "function"
        or type(layout.GetLayoutType) ~= "function"
        or not SettingsLayoutMixin
        or layout:GetLayoutType() ~= SettingsLayoutMixin.LayoutType.Canvas
    then
        return
    end

    local frameOk, frame = pcall(layout.GetFrame, layout)
    if frameOk and type(frame) == "table" and type(frame.Show) == "function" then
        frame:Show()
    end
end

local function __wow_patch_settings_canvas_registration()
    if type(Settings) ~= "table" or rawget(Settings, "__wow_canvas_layout_hide_patch") then
        return
    end

    if type(Settings.RegisterCanvasLayoutCategory) == "function" then
        local original = Settings.RegisterCanvasLayoutCategory
        Settings.RegisterCanvasLayoutCategory = function(frame, ...)
            local category, layout = original(frame, ...)
            __wow_hide_settings_canvas_frame(frame, layout)
            return category, layout
        end
    end

    if type(Settings.RegisterCanvasLayoutSubcategory) == "function" then
        local original = Settings.RegisterCanvasLayoutSubcategory
        Settings.RegisterCanvasLayoutSubcategory = function(parentCategory, frame, ...)
            local category, layout = original(parentCategory, frame, ...)
            __wow_hide_settings_canvas_frame(frame, layout)
            return category, layout
        end
    end

    if type(Settings.OpenToCategory) == "function" then
        local original = Settings.OpenToCategory
        Settings.OpenToCategory = function(...)
            local result = original(...)
            __wow_hide_registered_settings_canvas_frames()
            __wow_show_current_settings_canvas_frame()
            return result
        end
    end

    rawset(Settings, "__wow_canvas_layout_hide_patch", true)
end

__wow_patch_settings_canvas_registration()
__wow_hide_registered_settings_canvas_frames()
"#;

pub(crate) fn patch(env: &WowLuaEnv) {
    use crate::client_profile::{ACTIVE, ClientProfile};

    if ACTIVE != ClientProfile::Retail {
        let _ = env.exec(SETTINGS_CANVAS_LAYOUT_HIDE_LUA);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_hides_frame_until_displayed() {
        let env = WowLuaEnv::new().expect("env should initialize");
        env.exec(
            r#"
            SettingsLayoutMixin = { LayoutType = { Canvas = "Canvas" } }

            local categories = {}
            local layouts = {}

            SettingsPanel = {
                shown = false,
                currentLayout = nil,
                currentCategory = nil,
                GetAllCategories = function()
                    return categories
                end,
                GetLayout = function(_, category)
                    return layouts[category]
                end,
                IsShown = function(self)
                    return self.shown
                end,
                GetCurrentLayout = function(self)
                    return self.currentLayout
                end,
                GetCurrentCategory = function(self)
                    return self.currentCategory
                end,
            }

            Settings = {
                RegisterCanvasLayoutCategory = function(frame, name)
                    local category = { name = name }
                    local layout = {
                        frame = frame,
                        GetFrame = function(self)
                            return self.frame
                        end,
                        GetLayoutType = function()
                            return SettingsLayoutMixin.LayoutType.Canvas
                        end,
                    }
                    table.insert(categories, category)
                    layouts[category] = layout
                    return category, layout
                end,
                OpenToCategory = function(category)
                    SettingsPanel.shown = true
                    SettingsPanel.currentCategory = category
                    SettingsPanel.currentLayout = layouts[category]
                    return category
                end,
            }
            "#,
        )
        .expect("fake settings surface should install");

        env.exec(SETTINGS_CANVAS_LAYOUT_HIDE_LUA)
            .expect("legacy visibility policy should install");

        let hidden_after_register: bool = env
            .eval(
                r#"
                local frame = CreateFrame("Frame", "SettingsCanvasLeakProbe")
                frame:Show()
                local category, layout = Settings.RegisterCanvasLayoutCategory(frame, "Probe")
                return not frame:IsShown()
                "#,
            )
            .expect("registration probe should run");

        assert!(
            hidden_after_register,
            "settings canvas frame should be hidden after registration"
        );

        let opened_canvas_visible_others_hidden: bool = env
            .eval(
                r#"
                local first = SettingsCanvasLeakProbe
                local firstCategory = SettingsPanel:GetAllCategories()[1]
                local second = CreateFrame("Frame", "SettingsSecondCanvasLeakProbe")
                second:Show()
                local secondCategory = Settings.RegisterCanvasLayoutCategory(second, "Second")

                Settings.OpenToCategory(firstCategory)
                local firstOpened = first:IsShown() and not second:IsShown()

                Settings.OpenToCategory(secondCategory)
                return firstOpened and (not first:IsShown()) and second:IsShown()
                "#,
            )
            .expect("open category probe should run");

        assert!(
            opened_canvas_visible_others_hidden,
            "opening a settings category should show only that category's canvas"
        );
    }
}
