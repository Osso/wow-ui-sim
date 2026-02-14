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
    apply_edit_mode_layout(env);
    fix_action_bar_nan_size(env);
    fix_action_bar_scale(env);
}

/// Apply preset layout anchors and settings to all EditMode system frames.
fn apply_edit_mode_layout(env: &WowLuaEnv) {
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

        -- Mirror the real UpdateLayoutInfo flow:
        -- 1. layoutApplyInProgress prevents UpdateBottomActionBarPositions
        --    from running inside ApplySystemAnchor (bar has no width yet)
        -- 2. InitSystemAnchors sets up anchors from preset layout
        -- 3. UpdateSystems applies all settings (orientation, grid, etc.)
        -- 4. After clearing the flag, UpdateActionBarPositions computes
        --    the final BOTTOMLEFT anchor using the now-valid bar width
        emm.layoutApplyInProgress = true
        emm:InitSystemAnchors()
        pcall(emm.UpdateSystems, emm)
        emm.layoutApplyInProgress = false
        pcall(emm.UpdateActionBarPositions, emm)

        -- Ensure accountSettings is set so CanEnterEditMode() returns true
        if not emm.accountSettings then
            emm.accountSettings = C_EditMode.GetAccountSettings()
        end
    "#,
    );
}

/// Fix MainActionBar NaN size after UpdateSystems.
///
/// Layout() produces NaN because the bar has no size yet when children try
/// to resolve anchors relative to it (chicken-and-egg). Compute the bar
/// size directly from children's grid positions, then re-run
/// UpdateActionBarPositions to set the correct BOTTOMLEFT anchor.
fn fix_action_bar_nan_size(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not MainActionBar then return end
        local w = MainActionBar:GetWidth()
        if w == w then return end  -- not NaN, nothing to fix
        -- Compute width from button containers only (12 slots, 45px each
        -- with 2px gap = 47px stride). Last container offset + width.
        local lastOx, lastW = 0, 45
        for i = 1, 12 do
            local c = _G["MainActionBarButtonContainer" .. i]
            if c and c:GetNumPoints() > 0 then
                local _, _, _, ox, _ = c:GetPoint(1)
                if ox and ox == ox then lastOx = ox end
            end
        end
        MainActionBar:SetSize(lastOx + lastW, lastW)
        pcall(EditModeManagerFrame.UpdateActionBarPositions,
              EditModeManagerFrame)
    "#,
    );
}

/// Ensure MainActionBar has scale=1 after EditMode initialization.
///
/// Blizzard's EditMode overrides SetScale → SetScaleOverride via
/// `self.SetScale = self.SetScaleOverride` in OnSystemLoad, storing the
/// override in Lua frame_fields. However, mlua's registered metatable
/// methods take priority over frame_fields in __index lookups, so
/// `:SetScale()` calls always hit the Rust method directly, bypassing the
/// Lua override. This means SetScaleOverride (which adjusts anchor offsets
/// for scale changes) never runs, and various code paths that call
/// `:SetScale(0)` during startup leave the bar invisible.
///
/// The real fix would be making __index check frame_fields before metatable
/// methods, but that's a broader architectural change. For now, force
/// scale=1 after init since that's what UIParent_ManageFramePositions
/// would set anyway.
fn fix_action_bar_scale(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if MainActionBar then MainActionBar:SetScale(1) end
    "#,
    );
}

/// Patch EditModeManagerFrame after addon loading.
///
/// Before EDIT_MODE_LAYOUTS_UPDATED fires, layoutInfo is nil. Guard
/// GetActiveLayoutInfo with a fallback. Replace InitSystemAnchors with a
/// custom implementation that reads the active preset layout, applies
/// anchorInfo to all 43 registered system frames, then calls UpdateSystems
/// to apply settings (orientation, num rows, etc.) through the normal
/// Blizzard code path. Per-frame errors are caught by secureexecuterange.
///
/// Also wraps EnterEditMode/ExitEditMode with pcall protection so edit
/// mode can activate even when subsystems crash.
pub fn patch_edit_mode_manager(env: &WowLuaEnv) {
    patch_get_active_layout(env);
    patch_get_setting_value(env);
    patch_init_anchors(env);
    patch_update_systems(env);
    patch_update_layout_info(env);
    patch_default_anchor(env);
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
/// UpdateSystems is called separately from init_edit_mode_layout (after
/// this) to apply EditMode settings through the normal code path.
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
                end
            end
        end
    "#,
    );
}

/// Implement UpdateSystems to call UpdateSystem on each registered frame.
///
/// The real WoW UpdateSystems uses secureexecuterange which swallows
/// per-frame errors. We replicate that: look up each frame's systemInfo
/// from the active layout and call frame:UpdateSystem(sysInfo). Frames
/// that crash (missing subsystems, etc.) are caught individually.
fn patch_update_systems(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        function EditModeManagerFrame:UpdateSystems()
            local function callUpdateSystem(index, systemFrame)
                local sysInfo = self:GetActiveLayoutSystemInfo(
                    systemFrame.system, systemFrame.systemIndex
                )
                if sysInfo then
                    systemFrame:UpdateSystem(sysInfo)
                end
            end
            secureexecuterange(self.registeredSystemFrames, callUpdateSystem)
        end
    "#,
    );
}

/// Override UpdateLayoutInfo to prevent double initialization.
///
/// The EDIT_MODE_LAYOUTS_UPDATED event handler in EditModeManager.lua:178
/// calls UpdateLayoutInfo after init_edit_mode_layout already ran the full
/// InitSystemAnchors + UpdateSystems + UpdateActionBarPositions flow. The
/// second pass produces NaN coordinates because Layout() → GetExtents()
/// fails on children that haven't been sized yet in the live GUI context.
/// Replace with a no-op since init_edit_mode_layout handles everything.
fn patch_update_layout_info(env: &WowLuaEnv) {
    let _ = env.exec(
        r#"
        if not EditModeManagerFrame then return end
        function EditModeManagerFrame:UpdateLayoutInfo() end
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
