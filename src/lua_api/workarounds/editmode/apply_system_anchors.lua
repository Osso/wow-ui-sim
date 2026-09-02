
        if not EditModeManagerFrame then return end
        local emm = EditModeManagerFrame
        if not emm.layoutInfo then return end
        emm.layoutApplyInProgress = true

        local function system_frame_name(systemFrame)
            if not systemFrame then
                return "nil"
            end
            if type(systemFrame.GetName) == "function" then
                local name = systemFrame:GetName()
                if name ~= nil then
                    return name
                end
            end
            return tostring(systemFrame.system) .. ":" .. tostring(systemFrame.systemIndex)
        end

        local function is_bootstrap_action_bar(systemFrame)
            local frameName = system_frame_name(systemFrame)
            if string.sub(frameName, 1, 8) == "MultiBar" then
                return true
            end
            if not systemFrame or not EditModeUtil then
                return false
            end
            if Enum and Enum.EditModeSystem and systemFrame.system == Enum.EditModeSystem.ActionBar then
                return true
            end
            return EditModeUtil:IsBottomAnchoredActionBar(systemFrame)
                or EditModeUtil:IsRightAnchoredActionBar(systemFrame)
        end

        local function skips_expensive_startup_system_update(systemFrame)
            local frameName = system_frame_name(systemFrame)
            -- These frames run full roster/unit layout work during UpdateSystem.
            -- Their default startup positions only need systemInfo + anchors.
            return frameName == "PartyFrame"
                or frameName == "CompactArenaFrame"
                or frameName == "CompactRaidFrameContainer"
        end

        local function skips_full_startup_scale_update(systemFrame)
            local frameName = system_frame_name(systemFrame)
            -- Saved frame-size values are raw EditMode slider values. Running
            -- full size updates during startup can transiently feed those raw
            -- defaults into SetScale before the frame is fully ready.
            return frameName == "PlayerFrame"
                or frameName == "TargetFrame"
                or frameName == "FocusFrame"
                or frameName == "PlayerCastingBarFrame"
        end

        local function ensure_setting_display_info_map(systemFrame)
            if not EditModeSettingDisplayInfoManager
                or not EditModeSettingDisplayInfoManager.GetSystemSettingDisplayInfoMap then
                return
            end

            local displayInfoMap = systemFrame.settingDisplayInfoMap
            local needsMap = not displayInfoMap
            if displayInfoMap and systemFrame.systemInfo then
                for _, settingInfo in ipairs(systemFrame.systemInfo.settings or {}) do
                    if displayInfoMap[settingInfo.setting] == nil then
                        needsMap = true
                        break
                    end
                end
            end
            if not needsMap then
                return
            end

            local ok, resolvedMap = pcall(
                EditModeSettingDisplayInfoManager.GetSystemSettingDisplayInfoMap,
                EditModeSettingDisplayInfoManager,
                systemFrame.system
            )
            if ok and resolvedMap then
                systemFrame.settingDisplayInfoMap = resolvedMap
            end
        end

        for _, systemFrame in ipairs(emm.registeredSystemFrames or {}) do
            ensure_setting_display_info_map(systemFrame)
        end

        -- Read-only lookup of the active layout's systemInfo for branch
        -- decisions. Must not mutate the frame: pre-building the settingMap
        -- before the real UpdateSystem runs turns Blizzard's first apply into
        -- a no-change diff and silently skips every dirty-gated per-setting
        -- handler (e.g. DebuffFrame ShowDispelType).
        local function lookup_system_info(systemFrame)
            local systemIndex = systemFrame.systemIndex
            local systemInfo = emm:GetActiveLayoutSystemInfo(systemFrame.system, systemIndex)
            if not systemInfo and systemIndex == nil then
                systemInfo = emm:GetActiveLayoutSystemInfo(systemFrame.system, -1)
            end
            return systemInfo
        end

        local function seed_system_frame(systemFrame)
            local systemInfo = lookup_system_info(systemFrame)
            if not systemInfo then
                return
            end

            systemFrame.savedSystemInfo = CopyTable(systemInfo)
            systemFrame.systemInfo = systemInfo
            systemFrame:SetHasActiveChanges(false)
            ensure_setting_display_info_map(systemFrame)
            systemFrame:UpdateSettingMap(true)
            return true
        end

        local function anchor_targets_system_frame(systemFrame, anchorInfo)
            if not systemFrame or type(anchorInfo) ~= "table" then
                return false
            end

            local relativeTo = anchorInfo.relativeTo
            if relativeTo == systemFrame then
                return true
            end

            if type(relativeTo) == "string" and type(systemFrame.GetName) == "function" then
                return relativeTo == systemFrame:GetName()
            end

            return false
        end

        local function apply_system_anchor_if_safe(systemFrame)
            if not systemFrame or not systemFrame.ApplySystemAnchor then
                return
            end

            local frameName = system_frame_name(systemFrame)
            if frameName == "PlayerFrame" and PlayerCastingBarFrame then
                return
            end

            local systemInfo = systemFrame.systemInfo
            if anchor_targets_system_frame(systemFrame, systemInfo and systemInfo.anchorInfo) then
                return
            end

            pcall(systemFrame.ApplySystemAnchor, systemFrame)
        end

        local function is_unlocked_cast_bar(systemFrame)
            if system_frame_name(systemFrame) ~= "PlayerCastingBarFrame" then
                return false
            end
            local castBarSettings = Enum and Enum.EditModeCastBarSetting
            if not castBarSettings then
                return true
            end
            local lockSetting = castBarSettings.LockToPlayerFrame
            if lockSetting == nil then
                return true
            end
            if systemFrame.GetSettingValueBool then
                local ok, locked = pcall(systemFrame.GetSettingValueBool, systemFrame, lockSetting)
                if ok then
                    return not locked
                end
            end
            local systemInfo = systemFrame.systemInfo
            for _, settingInfo in ipairs(systemInfo and systemInfo.settings or {}) do
                if settingInfo.setting == lockSetting then
                    return not (settingInfo.value == 1 or settingInfo.value == true)
                end
            end
            return false
        end

        local function apply_anchor_info_directly(systemFrame)
            local anchorInfo = systemFrame and systemFrame.systemInfo and systemFrame.systemInfo.anchorInfo
            if not anchorInfo then
                return false
            end
            local fields = debug and debug.getfenv and debug.getfenv(systemFrame)
            fields = fields and fields[1]
            local clearAllPoints = fields and rawget(fields, "ClearAllPointsBase")
            clearAllPoints = clearAllPoints or systemFrame.ClearAllPointsBase
            local setPoint = fields and rawget(fields, "SetPointBase")
            setPoint = setPoint or systemFrame.SetPointBase
            if not setPoint then
                return false
            end
            local relativeTo = anchorInfo.relativeTo
            if type(relativeTo) == "string" then
                relativeTo = _G[relativeTo]
            end
            if not relativeTo then
                relativeTo = UIParent
            end
            if clearAllPoints then
                clearAllPoints(systemFrame)
            end
            setPoint(
                systemFrame,
                anchorInfo.point or "CENTER",
                relativeTo,
                anchorInfo.relativePoint or anchorInfo.point or "CENTER",
                anchorInfo.offsetX or 0,
                anchorInfo.offsetY or 0
            )
            return true
        end

        local function apply_hidden_visibility(systemFrame)
            if not systemFrame then
                return
            end
            if systemFrame.SetShownBase then
                pcall(systemFrame.SetShownBase, systemFrame, false)
            elseif systemFrame.Hide then
                pcall(systemFrame.Hide, systemFrame)
            elseif systemFrame.SetShown then
                pcall(systemFrame.SetShown, systemFrame, false)
            end
        end

        local function profile_hides_system(systemFrame)
            local systemInfo = systemFrame and systemFrame.systemInfo
            return type(systemInfo) == "table" and systemInfo.hidden == true
        end

        local function install_profile_hidden_shown_state_guard(systemFrame)
            if not profile_hides_system(systemFrame)
                or type(systemFrame.UpdateShownState) ~= "function"
                or systemFrame.__wow_profile_hidden_update_shown_state_guard then
                return
            end

            systemFrame.__wow_profile_hidden = true
            systemFrame.__wow_profile_hidden_update_shown_state_guard = true
            systemFrame.UpdateShownState = function(self)
                local noneIndex = -1
                if StatusTrackingBarInfo and StatusTrackingBarInfo.BarsEnum then
                    noneIndex = StatusTrackingBarInfo.BarsEnum.None or noneIndex
                end

                local shouldShow = self.shownBarIndex ~= noneIndex or self.isInEditMode
                if self.__wow_profile_hidden and not self.isInEditMode then
                    shouldShow = false
                end

                if self.SetShownBase then
                    pcall(self.SetShownBase, self, shouldShow)
                elseif self.SetShown then
                    pcall(self.SetShown, self, shouldShow)
                elseif shouldShow and self.Show then
                    pcall(self.Show, self)
                elseif not shouldShow and self.Hide then
                    pcall(self.Hide, self)
                end

                if UIParent_ManageFramePositions then
                    pcall(UIParent_ManageFramePositions)
                end
            end
        end

        local function should_hide_after_system_update(systemFrame, wasShown)
            install_profile_hidden_shown_state_guard(systemFrame)

            if not wasShown then
                return true
            end

            return profile_hides_system(systemFrame)
        end

        local function preserve_hidden_state(systemFrame, update)
            if not systemFrame or type(update) ~= "function" then
                return
            end

            local wasShown = true
            if systemFrame.IsShown then
                local ok, shown = pcall(systemFrame.IsShown, systemFrame)
                if ok then
                    wasShown = shown
                end
            end

            update()

            if should_hide_after_system_update(systemFrame, wasShown) then
                apply_hidden_visibility(systemFrame)
            end
        end

        local function update_system_setting_with_display_value(systemFrame, setting, displayValue)
            if not systemFrame or not systemFrame.UpdateSystemSetting then
                return
            end

            local oldGetSettingValue = systemFrame.GetSettingValue
            if oldGetSettingValue then
                systemFrame.GetSettingValue = function(self, requestedSetting, ...)
                    if requestedSetting == setting then
                        return displayValue
                    end
                    return oldGetSettingValue(self, requestedSetting, ...)
                end
            end

            pcall(systemFrame.UpdateSystemSetting, systemFrame, setting, true)

            if oldGetSettingValue then
                systemFrame.GetSettingValue = oldGetSettingValue
            end
        end

        local function positive_display_value_for_setting(systemFrame, setting, rawValue)
            local displayValue
            if systemFrame.GetSettingValue then
                local ok, value = pcall(systemFrame.GetSettingValue, systemFrame, setting)
                if ok then
                    displayValue = value
                end
            end
            if not displayValue or displayValue <= 0 then
                local displayInfo = systemFrame.settingDisplayInfoMap
                    and systemFrame.settingDisplayInfoMap[setting]
                if displayInfo and displayInfo.ConvertValueForDisplay then
                    local ok, value = pcall(
                        displayInfo.ConvertValueForDisplay,
                        displayInfo,
                        rawValue
                    )
                    if ok then
                        displayValue = value
                    end
                end
            end
            if not displayValue or displayValue <= 0 then
                displayValue = 100
            end
            return displayValue
        end

        local function replay_system_settings(systemFrame)
            local systemInfo = systemFrame and systemFrame.systemInfo
            if not systemInfo or not systemFrame.UpdateSystemSetting then
                return
            end
            local frameName = system_frame_name(systemFrame)

            for _, settingInfo in ipairs(systemInfo.settings or {}) do
                local setting = settingInfo.setting
                local unitFrameSettings = Enum and Enum.EditModeUnitFrameSetting
                local castBarSettings = Enum and Enum.EditModeCastBarSetting
                local needsAuraUpdate = unitFrameSettings
                    and setting == unitFrameSettings.BuffsOnTop
                    and not systemFrame.UpdateAuras
                local needsDirectFrameSizeReplay = (frameName == "PlayerFrame"
                    or frameName == "TargetFrame"
                    or frameName == "FocusFrame")
                    and unitFrameSettings
                    and setting == unitFrameSettings.FrameSize
                local needsCastBarScaleReplay = frameName == "PlayerCastingBarFrame"
                    and castBarSettings
                    and setting == castBarSettings.BarSize
                local needsCastBarLockReplay = frameName == "PlayerCastingBarFrame"
                    and castBarSettings
                    and setting == castBarSettings.LockToPlayerFrame
                if needsAuraUpdate then
                    systemFrame.buffsOnTop = settingInfo.value == 1
                        or settingInfo.value == true
                elseif needsDirectFrameSizeReplay then
                    local frameSize = positive_display_value_for_setting(
                        systemFrame,
                        setting,
                        settingInfo.value
                    )
                    if systemFrame.SetScale then
                        pcall(systemFrame.SetScale, systemFrame, frameSize / 100)
                    end
                    update_system_setting_with_display_value(systemFrame, setting, frameSize)
                elseif needsCastBarScaleReplay then
                    local barSize = positive_display_value_for_setting(
                        systemFrame,
                        setting,
                        settingInfo.value
                    )
                    if systemFrame.SetScale then
                        pcall(systemFrame.SetScale, systemFrame, barSize / 100)
                    end
                    update_system_setting_with_display_value(systemFrame, setting, barSize)
                elseif needsCastBarLockReplay then
                    local locked = false
                    if systemFrame.GetSettingValueBool then
                        local ok, value = pcall(systemFrame.GetSettingValueBool, systemFrame, setting)
                        locked = ok and value
                    else
                        locked = settingInfo.value == 1 or settingInfo.value == true
                    end
                    if locked then
                        if PlayerFrame_AttachCastBar then
                            pcall(PlayerFrame_AttachCastBar)
                        end
                    elseif systemFrame.attachedToPlayerFrame and PlayerFrame_DetachCastBar then
                        pcall(PlayerFrame_DetachCastBar)
                    end
                else
                    pcall(systemFrame.UpdateSystemSetting, systemFrame, setting, true)
                end
            end
        end

        local function refresh_system_layout_after_setting_replay(systemFrame)
            if systemFrame and systemFrame.UpdateGridLayout then
                pcall(systemFrame.UpdateGridLayout, systemFrame)
            end
        end

        local function mirror_and_clear_compact_unit_frame_settings(systemFrame)
            local systemInfo = systemFrame and systemFrame.systemInfo
            if not systemInfo then
                return
            end

            for _, settingInfo in ipairs(systemInfo.settings or {}) do
                if EditModeSystemMixin and EditModeSystemMixin.UpdateSystemSetting then
                    pcall(EditModeSystemMixin.UpdateSystemSetting, systemFrame, settingInfo.setting, true)
                end
                if systemFrame.ClearDirtySetting then
                    pcall(systemFrame.ClearDirtySetting, systemFrame, settingInfo.setting)
                end
            end
        end

        local function refresh_compact_frame_art()
            if CompactRaidFrameContainer and CompactRaidFrameContainer.ApplyMultipleToFrames then
                pcall(
                    CompactRaidFrameContainer.ApplyMultipleToFrames,
                    CompactRaidFrameContainer,
                    "mini",
                    DefaultCompactMiniFrameSetup,
                    "normal",
                    CompactUnitFrame_UpdateAllFromEditMode,
                    "group",
                    CompactRaidGroup_UpdateBorder
                )
            end
        end

        local function replay_compact_unit_frame_settings(systemFrame)
            local frameName = system_frame_name(systemFrame)
            if frameName ~= "PartyFrame"
                and frameName ~= "CompactArenaFrame"
                and frameName ~= "CompactRaidFrameContainer" then
                return false
            end
            if not EditModeSystemMixin or not EditModeSystemMixin.UpdateSystemSetting then
                return false
            end

            mirror_and_clear_compact_unit_frame_settings(systemFrame)

            if frameName == "PartyFrame" then
                if CompactPartyFrame and CompactRaidGroup_UpdateBorder then
                    pcall(CompactRaidGroup_UpdateBorder, CompactPartyFrame)
                end
                if UpdateRaidAndPartyFrames then
                    pcall(UpdateRaidAndPartyFrames)
                end
                if CompactPartyFrame and CompactPartyFrame.RefreshMembers then
                    pcall(CompactPartyFrame.RefreshMembers, CompactPartyFrame)
                end
                if PartyFrame and PartyFrame.UpdatePartyMemberBackground then
                    pcall(PartyFrame.UpdatePartyMemberBackground, PartyFrame)
                end
                if PartyFrame and PartyFrame.UpdatePaddingAndLayout then
                    pcall(PartyFrame.UpdatePaddingAndLayout, PartyFrame)
                end
                if systemFrame.UpdateSelectionVerticalState then
                    pcall(systemFrame.UpdateSelectionVerticalState, systemFrame)
                end
                if systemFrame.UpdateSystemSettingFrameSize then
                    pcall(systemFrame.UpdateSystemSettingFrameSize, systemFrame)
                end
                return true
            end

            if frameName == "CompactRaidFrameContainer" then
                if CompactRaidFrameContainer and CompactRaidFrameContainer.TryUpdate then
                    pcall(CompactRaidFrameContainer.TryUpdate, CompactRaidFrameContainer)
                end
                if CompactRaidFrameContainer and CompactRaidFrameContainer.Layout then
                    pcall(CompactRaidFrameContainer.Layout, CompactRaidFrameContainer)
                end
                refresh_compact_frame_art()
                if EditModeManagerFrame and EditModeManagerFrame.UpdateRaidContainerFlow then
                    pcall(EditModeManagerFrame.UpdateRaidContainerFlow, EditModeManagerFrame)
                end
                return true
            end

            if frameName == "CompactArenaFrame" then
                refresh_compact_frame_art()
                if systemFrame.RefreshMembers then
                    pcall(systemFrame.RefreshMembers, systemFrame)
                end
                if PartyFrame and PartyFrame.UpdatePaddingAndLayout then
                    pcall(PartyFrame.UpdatePaddingAndLayout, PartyFrame)
                end
                return true
            end

            return false
        end

        local function refresh_action_bar_system(systemFrame)
            local systemInfo = systemFrame.systemInfo
            local actionButtons = systemFrame.actionButtons
            local hasActionButtons = actionButtons
                and actionButtons[1] ~= nil
                and actionButtons.GetObjectType == nil

            local function mark_action_bar_layout_dirty()
                if systemFrame.MarkGridLayoutDirty then
                    systemFrame:MarkGridLayoutDirty()
                end
                if systemFrame.MarkDividersDirty then
                    systemFrame:MarkDividersDirty()
                end
                if systemFrame.MarkBarArtDirty then
                    systemFrame:MarkBarArtDirty()
                end
            end

            local function apply_action_bar_setting(settingInfo)
                local actionBarSettings = Enum and Enum.EditModeActionBarSetting
                if not actionBarSettings then
                    return
                end

                local setting = settingInfo.setting
                local function get_setting_value(useRawValue)
                    if systemFrame.GetSettingValue then
                        local ok, convertedValue = pcall(
                            systemFrame.GetSettingValue,
                            systemFrame,
                            setting,
                            useRawValue
                        )
                        if ok and convertedValue ~= nil then
                            return convertedValue
                        end
                    end
                    return settingInfo.value
                end

                local rawValue = get_setting_value(true)
                local value = get_setting_value(false)
                local valueBool = value == 1 or value == true

                if setting == actionBarSettings.Orientation then
                    systemFrame.isHorizontal = rawValue == Enum.ActionBarOrientation.Horizontal
                    if systemFrame.Selection and systemFrame.Selection.SetVerticalState then
                        pcall(systemFrame.Selection.SetVerticalState, systemFrame.Selection, not systemFrame.isHorizontal)
                    end
                    systemFrame.addButtonsToRight = true
                    systemFrame.addButtonsToTop = systemFrame.isHorizontal
                    mark_action_bar_layout_dirty()
                elseif setting == actionBarSettings.NumRows then
                    systemFrame.numRows = value
                    mark_action_bar_layout_dirty()
                elseif setting == actionBarSettings.NumIcons then
                    local useRawValue = false
                    if GameRulesUtil
                        and GameRulesUtil.AllowBelowMinimumActionBarIcons then
                        local ok, allowBelowMinimum = pcall(
                            GameRulesUtil.AllowBelowMinimumActionBarIcons,
                            GameRulesUtil
                        )
                        useRawValue = ok and allowBelowMinimum
                    end
                    systemFrame.numButtonsShowable = get_setting_value(useRawValue)
                    if systemFrame.UpdateShownButtons then
                        pcall(systemFrame.UpdateShownButtons, systemFrame)
                    end
                    mark_action_bar_layout_dirty()
                elseif setting == actionBarSettings.IconSize then
                    systemFrame.iconSize = value
                    local iconScale = value / 100
                    if iconScale <= 0 then
                        return
                    end
                    if systemFrame.EditModeSetScale then
                        pcall(systemFrame.EditModeSetScale, systemFrame, iconScale)
                    end
                    if hasActionButtons then
                        for _, actionButton in pairs(actionButtons) do
                            local container = actionButton and actionButton.container
                            if container and container.SetScale then
                                pcall(container.SetScale, container, iconScale)
                            end
                        end
                    end
                    if systemFrame.Layout then
                        pcall(systemFrame.Layout, systemFrame)
                    end
                    if emm.UpdateActionBarLayout then
                        pcall(emm.UpdateActionBarLayout, emm, systemFrame)
                    end
                    mark_action_bar_layout_dirty()
                elseif setting == actionBarSettings.IconPadding then
                    systemFrame.buttonPadding = value
                    mark_action_bar_layout_dirty()
                elseif setting == actionBarSettings.HideBarArt then
                    systemFrame.hideBarArt = valueBool
                    if systemFrame.UpdateEndCaps then
                        pcall(systemFrame.UpdateEndCaps, systemFrame, systemFrame.hideBarArt)
                    end
                    if systemFrame.BorderArt then
                        systemFrame.BorderArt:SetShown(not systemFrame.hideBarArt)
                    end
                    if hasActionButtons then
                        for _, actionButton in pairs(actionButtons) do
                            if actionButton and actionButton.UpdateButtonArt then
                                pcall(actionButton.UpdateButtonArt, actionButton)
                            end
                        end
                    end
                    mark_action_bar_layout_dirty()
                elseif setting == actionBarSettings.HideBarScrolling then
                    if systemFrame.ActionBarPageNumber then
                        systemFrame.ActionBarPageNumber:SetShown(not valueBool)
                    end
                    if systemFrame.MarkBarArtDirty then
                        systemFrame:MarkBarArtDirty()
                    end
                elseif setting == actionBarSettings.VisibleSetting
                    and Enum.ActionBarVisibleSetting then
                    if rawValue == Enum.ActionBarVisibleSetting.InCombat then
                        systemFrame.visibility = "InCombat"
                    elseif rawValue == Enum.ActionBarVisibleSetting.OutOfCombat then
                        systemFrame.visibility = "OutOfCombat"
                    elseif rawValue == Enum.ActionBarVisibleSetting.Hidden then
                        systemFrame.visibility = "Hidden"
                    else
                        systemFrame.visibility = "Always"
                    end
                    if systemFrame.UpdateVisibility then
                        pcall(systemFrame.UpdateVisibility, systemFrame)
                    end
                elseif setting == actionBarSettings.AlwaysShowButtons then
                    systemFrame.alwaysShowButtons = valueBool
                    if hasActionButtons
                        and systemFrame.SetShowGrid
                        and ACTION_BUTTON_SHOW_GRID_REASON_CVAR then
                        pcall(
                            systemFrame.SetShowGrid,
                            systemFrame,
                            valueBool,
                            ACTION_BUTTON_SHOW_GRID_REASON_CVAR
                        )
                    end
                    if systemFrame.MarkDividersDirty then
                        systemFrame:MarkDividersDirty()
                    end
                end
            end

            -- Replay the action-bar setting handlers without the full
            -- EditMode frame update path. Some bars expose FrameRef userdata
            -- through actionButtons during bootstrap; Blizzard's handlers call
            -- pairs(self.actionButtons), which is not safe until the real Lua
            -- array exists.
            for _, settingInfo in ipairs(systemInfo and systemInfo.settings or {}) do
                apply_action_bar_setting(settingInfo)
            end
            if hasActionButtons and systemFrame.RefreshGridLayout then
                systemFrame:RefreshGridLayout()
            end
            if hasActionButtons and systemFrame.RefreshDividers then
                systemFrame:RefreshDividers()
            end
            if systemFrame.RefreshBarArt then
                systemFrame:RefreshBarArt()
            end
        end

        for _, systemFrame in ipairs(emm.registeredSystemFrames or {}) do
            if is_bootstrap_action_bar(systemFrame) then
                -- Full EditMode action-bar updates are expensive on the live
                -- path and can stall startup. Seed just enough state for the
                -- layout pass to run, then apply the saved anchor directly.
                if seed_system_frame(systemFrame) then
                    apply_system_anchor_if_safe(systemFrame)
                    refresh_action_bar_system(systemFrame)
                end
            elseif skips_expensive_startup_system_update(systemFrame) then
                if seed_system_frame(systemFrame) then
                    apply_system_anchor_if_safe(systemFrame)
                    if not replay_compact_unit_frame_settings(systemFrame) then
                        replay_system_settings(systemFrame)
                    end
                end
            elseif skips_full_startup_scale_update(systemFrame) then
                if seed_system_frame(systemFrame) then
                    replay_system_settings(systemFrame)
                    if system_frame_name(systemFrame) == "PlayerFrame" then
                        apply_anchor_info_directly(systemFrame)
                    elseif is_unlocked_cast_bar(systemFrame) then
                        apply_anchor_info_directly(systemFrame)
                    else
                        apply_system_anchor_if_safe(systemFrame)
                    end
                end
            else
                local systemInfo = lookup_system_info(systemFrame)
                if systemInfo
                    and anchor_targets_system_frame(systemFrame, systemInfo.anchorInfo) then
                    -- Saved layouts can contain self-relative anchors for dependent
                    -- systems such as BuffFrame. Seed their layout state, but do
                    -- not hand that impossible anchor to SetPoint during startup.
                    seed_system_frame(systemFrame)
                    replay_system_settings(systemFrame)
                    refresh_system_layout_after_setting_replay(systemFrame)
                elseif systemInfo then
                    preserve_hidden_state(systemFrame, function()
                        -- Mirrors EditModeManagerFrameMixin:UpdateSystem with
                        -- the systemInfo resolved above (covering the saved
                        -- -1 index quirk for singleton rows the manager's own
                        -- nil-index lookup would miss). Deliberately NOT
                        -- seeded first: UpdateSystem must see the frame's real
                        -- pre-apply settingMap (usually absent on startup) so
                        -- its UpdateSettingMap diff marks settings dirty and
                        -- the per-setting handlers actually run.
                        pcall(systemFrame.UpdateSystem, systemFrame, systemInfo)
                        apply_system_anchor_if_safe(systemFrame)
                    end)
                end
            end
        end

        emm.layoutApplyInProgress = false
