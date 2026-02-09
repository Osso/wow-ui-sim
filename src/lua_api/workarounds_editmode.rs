//! EditMode layout workarounds.
//!
//! Patches EditModeManagerFrame to apply preset layout anchors to all 43
//! registered system frames. The real UpdateLayoutInfo crashes partway through
//! due to cascading dependencies, so we manually set up layoutInfo and call
//! our custom InitSystemAnchors.

use super::WowLuaEnv;

/// Initialize EditMode layout info and apply system anchors.
///
/// EDIT_MODE_LAYOUTS_UPDATED fires during startup but UpdateLayoutInfo
/// crashes partway through (cascading dependencies). This leaves
/// layoutInfo nil. Manually set it up from C_EditMode.GetLayouts() +
/// preset layouts, then call our custom InitSystemAnchors. Also ensures
/// accountSettings is initialized so CanEnterEditMode() returns true.
pub fn init_edit_mode_layout(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        local emm = EditModeManagerFrame
        if emm.layoutInfo then return end  -- already initialized

        local layoutInfo = C_EditMode.GetLayouts()
        emm.layoutInfo = layoutInfo
        local savedLayouts = emm.layoutInfo.layouts
        emm.layoutInfo.layouts = EditModePresetLayoutManager:GetCopyOfPresetLayouts()
        tAppendAll(emm.layoutInfo.layouts, savedLayouts)
        emm:InitSystemAnchors()

        -- Ensure accountSettings is set so CanEnterEditMode() returns true
        if not emm.accountSettings then
            emm.accountSettings = C_EditMode.GetAccountSettings()
        end
    "#,
    );
}

/// Patch EditModeManagerFrame after addon loading.
///
/// Before EDIT_MODE_LAYOUTS_UPDATED fires, layoutInfo is nil. Guard
/// GetActiveLayoutInfo with a fallback. Replace InitSystemAnchors with a
/// custom implementation that reads the active preset layout and applies
/// anchorInfo to all 43 registered system frames. Stub UpdateSystems as
/// a no-op since our InitSystemAnchors already handles positioning and
/// the full UpdateSystem chain has too many dependencies.
///
/// Also patches ShowUIPanel/HideUIPanel to bypass FramePositionDelegate,
/// and wraps EnterEditMode/ExitEditMode with pcall protection so edit
/// mode can activate even when subsystems crash.
pub fn patch_edit_mode_manager(env: &WowLuaEnv) {
    patch_get_active_layout(env);
    patch_get_setting_value(env);
    patch_init_anchors(env);
    patch_update_systems(env);
    patch_default_anchor(env);
    patch_show_hide_ui_panel(env);
    patch_enter_exit_edit_mode(env);
}

/// Guard GetActiveLayoutInfo against nil layoutInfo (pre-event calls).
fn patch_get_active_layout(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        local emm = EditModeManagerFrame
        local origGetActiveLayoutInfo = emm.GetActiveLayoutInfo
        function emm:GetActiveLayoutInfo()
            if not self.layoutInfo then
                return { layoutType = 0, layoutIndex = 1, systems = {} }
            end
            return origGetActiveLayoutInfo(self)
        end
    "#,
    );
}

/// Guard GetSettingValue against nil settingMap.
///
/// During startup events (VARIABLES_LOADED, PLAYER_ENTERING_WORLD), frames
/// may have systemInfo set (IsInitialized() → true) but settingMap not yet
/// populated (init_edit_mode_layout runs post-events). Return 0 when
/// settingMap is nil or missing the requested setting, matching the
/// IsInitialized() fallback behavior.
fn patch_get_setting_value(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeSystemMixin then return end
        function EditModeSystemMixin:GetSettingValue(setting, useRawValue)
            if not self:IsInitialized() then
                return 0
            end
            local compositeNumberValue = self:GetCompositeNumberSettingValue(setting, useRawValue)
            if compositeNumberValue ~= nil then
                return compositeNumberValue
            end
            if not self.settingMap or not self.settingMap[setting] then
                return 0
            end
            if useRawValue then
                return self.settingMap[setting].value
            else
                return self.settingMap[setting].displayValue or self.settingMap[setting].value
            end
        end
    "#,
    );
}

/// Custom InitSystemAnchors: apply preset layout anchors directly.
///
/// Builds a lookup from (system, systemIndex) → sysInfo, then iterates
/// registeredSystemFrames and calls ClearAllPoints + SetPoint for each.
/// Sets systemInfo on each frame so IsInitialized() returns true.
fn patch_init_anchors(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        local emm = EditModeManagerFrame
        function emm:InitSystemAnchors()
            local activeLayout = self:GetActiveLayoutInfo()
            if not activeLayout or not activeLayout.systems then return end

            -- Build lookup: "system:systemIndex" -> sysInfo
            local lookup = {}
            for _, sysInfo in ipairs(activeLayout.systems) do
                local idx = sysInfo.systemIndex or 0
                local key = tostring(sysInfo.system) .. ":" .. tostring(idx)
                lookup[key] = sysInfo
            end

            -- Apply anchors to all registered frames
            for _, frame in ipairs(self.registeredSystemFrames) do
                local idx = frame.systemIndex or 0
                local key = tostring(frame.system) .. ":" .. tostring(idx)
                local sysInfo = lookup[key]
                if sysInfo and sysInfo.anchorInfo then
                    frame:ClearAllPoints()
                    local a = sysInfo.anchorInfo
                    local rel = a.relativeTo
                    if type(rel) == "string" then
                        rel = _G[rel] or rel
                    end
                    frame:SetPoint(
                        a.point, rel, a.relativePoint,
                        a.offsetX, a.offsetY
                    )
                    -- Set systemInfo so IsInitialized() returns true,
                    -- and build settingMap so GetSettingValue() works.
                    frame.systemInfo = sysInfo
                    if EditModeUtil and EditModeUtil.GetSettingMapFromSettings then
                        frame.settingMap = EditModeUtil:GetSettingMapFromSettings(
                            sysInfo.settings, frame.settingDisplayInfoMap
                        )
                    end
                end
            end
        end
    "#,
    );
}

/// Stub UpdateSystems — InitSystemAnchors handles positioning and the
/// full UpdateSystem chain (settings, dialogs, etc.) isn't needed.
fn patch_update_systems(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        function EditModeManagerFrame:UpdateSystems() end
    "#,
    );
}

/// Provide a fallback GetDefaultAnchor for frames that query it.
fn patch_default_anchor(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        function EditModeManagerFrame:GetDefaultAnchor(frame)
            return {
                point = "TOPRIGHT",
                relativeTo = UIParent,
                relativePoint = "TOPRIGHT",
                offsetX = -205,
                offsetY = -13,
            }
        end
    "#,
    );
}

/// Override ShowUIPanel/HideUIPanel to bypass FramePositionDelegate.
///
/// The real ShowUIPanel dispatches through FramePositionDelegate secure
/// attributes which the simulator doesn't support, so panels never become
/// visible. Replace with simple Show()/Hide() wrappers.
fn patch_show_hide_ui_panel(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        ShowUIPanel = function(frame, force)
            if not frame or frame:IsShown() then return end
            frame:Show()
        end

        HideUIPanel = function(frame, skipSetPoint)
            if not frame or not frame:IsShown() then return end
            frame:Hide()
        end
    "#,
    );
}

/// Wrap EnterEditMode/ExitEditMode with pcall protection.
///
/// EnterEditMode calls crash-prone functions: ShowSystemSelections
/// iterates 43 frames, AccountSettings does 30+ Setup/Refresh calls.
/// Wrapping each step with pcall lets edit mode activate even when
/// individual subsystems fail.
fn patch_enter_exit_edit_mode(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        local emm = EditModeManagerFrame

        function emm:EnterEditMode()
            self.editModeActive = true
            pcall(self.ClearActiveChangesFlags, self)
            pcall(self.UpdateDropdownOptions, self)
            pcall(self.ShowSystemSelections, self)
            if self.AccountSettings
                and self.AccountSettings.OnEditModeEnter then
                pcall(
                    self.AccountSettings.OnEditModeEnter,
                    self.AccountSettings
                )
            end
            pcall(EventRegistry.TriggerEvent,
                EventRegistry, "EditMode.Enter")
        end

        function emm:ExitEditMode()
            self.editModeActive = false
            pcall(self.ClearSelectedSystem, self)
            pcall(function()
                secureexecuterange(
                    self.registeredSystemFrames,
                    function(_, f)
                        if f.OnEditModeExit then
                            pcall(f.OnEditModeExit, f)
                        end
                    end
                )
            end)
            if self.AccountSettings
                and self.AccountSettings.OnEditModeExit then
                pcall(
                    self.AccountSettings.OnEditModeExit,
                    self.AccountSettings
                )
            end
            pcall(EventRegistry.TriggerEvent,
                EventRegistry, "EditMode.Exit")
        end
    "#,
    );
}
