//! Temporary UIParent panel toggle helpers.
//!
//! Several Blizzard panel entry points are normally installed by lazy-loaded
//! addons. The simulator still needs these compatibility toggles during
//! startup and runtime addon loading until panel ownership is modeled more
//! directly.

use crate::lua_api::{LoaderEnv, WowLuaEnv};

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

const GETGLOBAL_HELPER_LUA: &str = r#"
local function __wow_getglobal(name)
    return getglobal(name)
end
_G.__wow_panel_getglobal = __wow_getglobal
"#;

const TOGGLE_ACHIEVEMENT_FRAME_LUA: &str = r#"
if __wow_panel_getglobal ~= nil then
    local __wow_getglobal = __wow_panel_getglobal

    local function __wow_patch_summary_empty_text_overlap()
        if rawget(_G, "__wow_achievement_summary_empty_text_patched") then
            return
        end
        if type(AchievementFrameSummary_UpdateAchievements) ~= "function" then
            return
        end

        local original = AchievementFrameSummary_UpdateAchievements
        AchievementFrameSummary_UpdateAchievements = function(...)
            local numAchievements = select('#', ...)
            local results = { original(...) }
            local emptyText = __wow_getglobal("AchievementFrameSummaryAchievementsEmptyText")
            local summary = __wow_getglobal("AchievementFrameSummaryAchievements")
            local buttons = summary and summary.buttons
            local hasVisibleSummaryButton = false

            if type(buttons) == "table" then
                for _, button in ipairs(buttons) do
                    if (type(button) == "table" or type(button) == "userdata")
                        and type(button.IsShown) == "function"
                        and button:IsShown()
                    then
                        hasVisibleSummaryButton = true
                        break
                    end
                end
            end

            if (type(emptyText) == "table" or type(emptyText) == "userdata")
                and type(emptyText.SetShown) == "function"
            then
                emptyText:SetShown(numAchievements == 0 and not hasVisibleSummaryButton)
            end

            return unpack(results)
        end

        rawset(_G, "__wow_achievement_summary_empty_text_patched", true)
    end

    function ToggleAchievementFrame(stats, toggleGuildView)
        local kiosk = __wow_getglobal("Kiosk")
        if ( (kiosk and kiosk.IsEnabled and kiosk.IsEnabled()) or __wow_getglobal("DISALLOW_FRAME_TOGGLING") ) then
            return;
        end

        local cAddOns = __wow_getglobal("C_AddOns")
        if cAddOns and cAddOns.LoadAddOn and cAddOns.IsAddOnLoaded and not cAddOns.IsAddOnLoaded("Blizzard_AchievementUI") then
            cAddOns.LoadAddOn("Blizzard_AchievementUI");
        end
        __wow_patch_summary_empty_text_overlap()

        local achievementFrame = __wow_getglobal("AchievementFrame")
        if not achievementFrame then
            return;
        end

        local achievementToggle = __wow_getglobal("AchievementFrame_ToggleAchievementFrame")
        if type(achievementToggle) == "function" then
            return achievementToggle(stats, toggleGuildView)
        end

        local requestedTab = stats and 3 or 1
        if achievementFrame:IsShown() and achievementFrame.selectedTab == requestedTab then
            local hideUIPanel = __wow_getglobal("HideUIPanel")
            if type(hideUIPanel) == "function" then
                hideUIPanel(achievementFrame)
            else
                achievementFrame:Hide();
            end
        else
            achievementFrame.selectedTab = requestedTab
            local showUIPanel = __wow_getglobal("ShowUIPanel")
            if type(showUIPanel) == "function" then
                showUIPanel(achievementFrame)
            else
                achievementFrame:Show();
            end
        end
    end
end
"#;

const TOGGLE_ENCOUNTER_JOURNAL_LUA: &str = r#"
function ToggleEncounterJournal()
    if DISALLOW_FRAME_TOGGLING then
        return
    end
    if not EncounterJournal and type(EncounterJournal_LoadUI) == "function" then
        EncounterJournal_LoadUI()
    end
    if not EncounterJournal and type(C_AddOns) == "table" and type(C_AddOns.LoadAddOn) == "function" then
        C_AddOns.LoadAddOn("Blizzard_EncounterJournal")
    end
    if EncounterJournal then
        if EncounterJournal:IsShown() then
            if type(HideUIPanel) == "function" then
                HideUIPanel(EncounterJournal)
            else
                EncounterJournal:Hide()
            end
        else
            if type(ShowUIPanel) == "function" then
                ShowUIPanel(EncounterJournal)
            else
                EncounterJournal:Show()
            end
        end
        return true;
    end
    return false;
end
"#;

const TOGGLE_COLLECTIONS_JOURNAL_LUA: &str = r#"
function ToggleCollectionsJournal(tabIndex)
    if DISALLOW_FRAME_TOGGLING then
        return
    end
    if not CollectionsJournal and type(CollectionsJournal_LoadUI) == "function" then
        CollectionsJournal_LoadUI()
    end
    if not CollectionsJournal and type(C_AddOns) == "table" and type(C_AddOns.LoadAddOn) == "function" then
        C_AddOns.LoadAddOn("Blizzard_Collections")
    end
    if not CollectionsJournal then
        return
    end

    if type(SetCollectionsJournalShown) == "function" then
        local tabMatches = not tabIndex or tabIndex == PanelTemplates_GetSelectedTab(CollectionsJournal)
        local isShown = CollectionsJournal:IsShown() and tabMatches
        SetCollectionsJournalShown(not isShown, tabIndex)
    elseif CollectionsJournal:IsShown() then
        if type(HideUIPanel) == "function" then
            HideUIPanel(CollectionsJournal)
        else
            CollectionsJournal:Hide()
        end
    else
        if type(ShowUIPanel) == "function" then
            ShowUIPanel(CollectionsJournal)
        else
            CollectionsJournal:Show()
        end
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PANEL_REGISTRATION_DEFAULTS_LUA)?;
    lua.exec(STARTUP_NAVIGATION_DEFAULTS_LUA)?;
    Ok(())
}

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(GETGLOBAL_HELPER_LUA);
    let _ = env.exec(TOGGLE_ACHIEVEMENT_FRAME_LUA);
    let _ = env.exec(TOGGLE_ENCOUNTER_JOURNAL_LUA);
    let _ = env.exec(TOGGLE_COLLECTIONS_JOURNAL_LUA);
}

pub(crate) fn patch_collections_journal_loader(env: &LoaderEnv<'_>) {
    let _ = env.exec(TOGGLE_COLLECTIONS_JOURNAL_LUA);
}

pub(crate) fn patch_encounter_journal_loader(env: &LoaderEnv<'_>) {
    let _ = env.exec(TOGGLE_ENCOUNTER_JOURNAL_LUA);
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

    #[test]
    fn patch_preserves_registered_panel_manager_behavior() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            registeredPanel = nil
            RegisterUIPanel = function(frame)
                registeredPanel = frame
            end
            local panel = CreateFrame("Frame", "DirectTogglePanel", UIParent)
            showCalled = false
            hideCalled = false
            ShowUIPanel = function()
                showCalled = true
            end
            HideUIPanel = function()
                hideCalled = true
            end
            "#,
        )
        .expect("fixture should install real-ish panel globals");

        patch(&env);

        let result: String = env
            .eval(
                r#"
                RegisterUIPanel(DirectTogglePanel, {})
                if registeredPanel ~= DirectTogglePanel then return "original" end
                if DirectTogglePanel.editModeManuallyShown ~= nil then return "flag" end
                ShowUIPanel(DirectTogglePanel)
                if showCalled ~= true then return "show" end
                HideUIPanel(DirectTogglePanel)
                if hideCalled ~= true then return "hide" end
                return "ok"
                "#,
            )
            .expect("panel manager registration probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn installs_getglobal_helper_and_toggle_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        patch(&env);

        let helper_and_toggles_installed: bool = env
            .eval(
                r#"
                ShowUIPanel = function() end
                return __wow_panel_getglobal("ShowUIPanel") == ShowUIPanel
                    and type(ToggleAchievementFrame) == "function"
                    and type(ToggleEncounterJournal) == "function"
                    and type(ToggleCollectionsJournal) == "function"
                "#,
            )
            .expect("panel toggle helpers should be readable");

        assert!(helper_and_toggles_installed);
    }

    #[test]
    fn toggle_achievement_frame_loads_and_shows_frame() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            loaded_addon = nil
            shown_frame = nil
            C_AddOns = {
                IsAddOnLoaded = function()
                    return false
                end,
                LoadAddOn = function(addonName)
                    loaded_addon = addonName
                    AchievementFrame = {
                        selectedTab = nil,
                        shown = false,
                        IsShown = function(self)
                            return self.shown
                        end,
                        Show = function(self)
                            self.shown = true
                        end,
                        Hide = function(self)
                            self.shown = false
                        end,
                    }
                end,
            }
            ShowUIPanel = function(frame)
                shown_frame = frame
                frame:Show()
            end
            "#,
        )
        .expect("achievement-frame fixture should install");

        patch(&env);

        let (loaded_addon, selected_tab, was_shown): (String, i64, bool) = env
            .eval(
                r#"
                ToggleAchievementFrame(true, false)
                return loaded_addon, AchievementFrame.selectedTab, shown_frame == AchievementFrame
                "#,
            )
            .expect("achievement toggle should load and show frame");

        assert_eq!(loaded_addon, "Blizzard_AchievementUI");
        assert_eq!(selected_tab, 3);
        assert!(was_shown);
    }

    #[test]
    fn toggle_encounter_journal_uses_panel_show_and_hide() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            EncounterJournal = {
                shown = false,
                IsShown = function(self)
                    return self.shown
                end,
                Show = function(self)
                    self.shown = true
                end,
                Hide = function(self)
                    self.shown = false
                end,
            }
            ShowUIPanel = function(frame)
                frame:Show()
            end
            HideUIPanel = function(frame)
                frame:Hide()
            end
            "#,
        )
        .expect("encounter-journal fixture should install");

        patch(&env);

        let (first_result, shown_after_first, second_result, shown_after_second): (
            bool,
            bool,
            bool,
            bool,
        ) = env
            .eval(
                r#"
                local firstResult = ToggleEncounterJournal()
                local shownAfterFirst = EncounterJournal:IsShown()
                local secondResult = ToggleEncounterJournal()
                return firstResult, shownAfterFirst, secondResult, EncounterJournal:IsShown()
                "#,
            )
            .expect("encounter toggle should show and hide frame");

        assert!(first_result);
        assert!(shown_after_first);
        assert!(second_result);
        assert!(!shown_after_second);
    }

    #[test]
    fn toggle_collections_journal_uses_collection_panel_state_helper() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            CollectionsJournal = {
                shown = false,
                IsShown = function(self)
                    return self.shown
                end,
            }
            PanelTemplates_GetSelectedTab = function()
                return 2
            end
            requested_shown = nil
            requested_tab = nil
            SetCollectionsJournalShown = function(shown, tabIndex)
                requested_shown = shown
                requested_tab = tabIndex
            end
            "#,
        )
        .expect("collections-journal fixture should install");

        patch(&env);

        let (requested_shown, requested_tab): (bool, i64) = env
            .eval(
                r#"
                ToggleCollectionsJournal(3)
                return requested_shown, requested_tab
                "#,
            )
            .expect("collections toggle should use shown helper");

        assert!(requested_shown);
        assert_eq!(requested_tab, 3);
    }
}
