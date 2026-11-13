//! Temporary UIParent panel toggle helpers.
//!
//! Several Blizzard panel entry points are normally installed by lazy-loaded
//! addons. The simulator still needs these compatibility toggles during
//! startup and runtime addon loading until panel ownership is modeled more
//! directly.

#[cfg(test)]
use crate::lua_api::WowLuaEnv;

const PANEL_REGISTRATION_DEFAULTS_LUA: &str = r#"
if ShowUIPanel == nil then
    function ShowUIPanel(frame)
        if frame ~= nil and type(frame.Show) == "function" then
            frame:Show()
            return true
        end
        return false
    end
end

if HideUIPanel == nil then
    function HideUIPanel(frame)
        if frame ~= nil and type(frame.Hide) == "function" then
            frame:Hide()
            return true
        end
        return false
    end
end

if CloseAllWindows == nil then
    function CloseAllWindows()
        return false
    end
end

if CloseMenus == nil then
    function CloseMenus()
        local closed = false
        if type(UIMenus) == "table" then
            for _, name in pairs(UIMenus) do
                local menu = _G[name]
                if menu ~= nil and type(menu.IsShown) == "function" and menu:IsShown() then
                    if type(menu.Hide) == "function" then
                        menu:Hide()
                        closed = true
                    end
                end
            end
        end
        return closed
    end
end
"#;

const STARTUP_NAVIGATION_DEFAULTS_LUA: &str = r#"
local uiParent = rawget(_G, "UIParent")

local function ensure_frame(name)
    local frame = rawget(_G, name)
    if frame == nil and type(CreateFrame) == "function" then
        frame = CreateFrame("Frame", name, uiParent)
        rawset(_G, name, frame)
    end
    return frame
end

local function existing_frame(name)
    return rawget(_G, name)
end

local function set_existing_frame_visibility(name, visible)
    local frame = existing_frame(name)
    if frame == nil then
        return nil
    end

    if type(frame.Show) == "function" and visible then
        frame:Show()
    elseif type(frame.Hide) == "function" and not visible then
        frame:Hide()
    else
        rawset(frame, "visible", visible and true or false)
    end
    return frame
end

local function toggle_single_frame(name, extraNames)
    local frame = existing_frame(name)
    if frame == nil then
        return false
    end

    local isShown = type(frame.IsShown) == "function" and frame:IsShown()
    local newVisible = not isShown
    set_existing_frame_visibility(name, newVisible)
    if type(extraNames) == "table" then
        for _, extraName in ipairs(extraNames) do
            set_existing_frame_visibility(extraName, newVisible)
        end
    end
    return newVisible
end

for _, name in ipairs({
    "MainActionBar",
    "MultiBarBottomLeft",
    "MultiBarBottomRight",
    "MultiBarRight",
    "MultiBarLeft",
}) do
    local frame = ensure_frame(name)
    if frame ~= nil and rawget(frame, "MarkAllSettingsDirty") == nil then
        function frame:MarkAllSettingsDirty() end
    end
end

for _, name in ipairs({
    "MailFrame",
    "InboxFrame",
    "PVEFrame",
    "SettingsPanel",
}) do
    local frame = existing_frame(name)
    if frame ~= nil and rawget(frame, "MarkAllSettingsDirty") == nil then
        function frame:MarkAllSettingsDirty() end
    end
end

if rawget(_G, "OpenAllBags") == nil then
    function OpenAllBags()
        if set_existing_frame_visibility("ContainerFrameCombinedBags", true) ~= nil then
            return
        end
        for i = 1, 6 do
            if set_existing_frame_visibility("ContainerFrame" .. i, true) ~= nil then
                return
            end
        end
    end
end

if rawget(_G, "ToggleLFDParentFrame") == nil then
    function ToggleLFDParentFrame()
        return toggle_single_frame("PVEFrame")
    end
end

if rawget(_G, "UpdateRaidAndPartyFrames") == nil then
    function UpdateRaidAndPartyFrames()
        if PartyFrame and type(PartyFrame.UpdatePartyFrames) == "function" then
            pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
        end
    end
end

if rawget(_G, "HelpOpenWebTicketButton_OnUpdate") == nil then
    function HelpOpenWebTicketButton_OnUpdate() end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PANEL_REGISTRATION_DEFAULTS_LUA)?;
    lua.exec(STARTUP_NAVIGATION_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[path = "close_menus.rs"]
    mod close_menus_tests;

    #[test]
    fn does_not_fabricate_original_panel_registry() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("assert(RegisterUIPanel == nil and UIPanelWindows == nil)")
            .expect("original panel owner has not loaded");
    }

    #[test]
    fn preserves_existing_panel_globals() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            RegisterUIPanel = function()
                registeredByExisting = true
            end
            CloseAllWindows = function()
                return "existing"
            end
            "#,
        )
        .expect("fixture should install existing panel globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("panel defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                RegisterUIPanel()
                if registeredByExisting ~= true then return "register" end
                if CloseAllWindows() ~= "existing" then return "close" end
                return "ok"
                "#,
            )
            .expect("panel preservation probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn installs_startup_navigation_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            OpenAllBags = nil
            ToggleLFDParentFrame = nil
            UpdateRaidAndPartyFrames = nil
            HelpOpenWebTicketButton_OnUpdate = nil
            SettingsPanel = nil
            MailFrame = nil
            InboxFrame = nil
            PVEFrame = nil
            ContainerFrameCombinedBags = nil

            SettingsPanel = CreateFrame("Frame", "SettingsPanel")
            PVEFrame = CreateFrame("Frame", "PVEFrame")
            PVEFrame:Hide()
            ContainerFrameCombinedBags = CreateFrame("Frame", "ContainerFrameCombinedBags")
            "#,
        )
        .expect("fixture should clear startup navigation globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("startup navigation defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if type(OpenAllBags) ~= "function" then return "bags_function" end
                if type(ToggleLFDParentFrame) ~= "function" then return "lfd_function" end
                if type(UpdateRaidAndPartyFrames) ~= "function" then return "raid_function" end
                if type(HelpOpenWebTicketButton_OnUpdate) ~= "function" then return "help_function" end

                if MailFrame ~= nil then return "mail_placeholder" end
                if InboxFrame ~= nil then return "inbox_placeholder" end

                if type(SettingsPanel.MarkAllSettingsDirty) ~= "function" then return "settings_method" end
                SettingsPanel:MarkAllSettingsDirty()

                OpenAllBags()
                if not ContainerFrameCombinedBags:IsShown() then return "bags_show" end

                ToggleLFDParentFrame()
                if not PVEFrame:IsShown() then return "lfd_show" end
                ToggleLFDParentFrame()
                if PVEFrame:IsShown() then return "lfd_hide" end

                PartyFrame = {
                    updated = false,
                    UpdatePartyFrames = function(self)
                        self.updated = true
                    end,
                }
                UpdateRaidAndPartyFrames()
                if PartyFrame.updated ~= true then return "raid_update" end

                HelpOpenWebTicketButton_OnUpdate()
                return "ok"
                "#,
            )
            .expect("startup navigation defaults probe should run");

        assert_eq!(result, "ok");
    }
}
