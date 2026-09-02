
    local function setSystemSetting(systemInfo, setting, value)
        if not systemInfo or not systemInfo.settings then return end
        for _, settingInfo in ipairs(systemInfo.settings) do
            if settingInfo.setting == setting then
                settingInfo.value = value
                return
            end
        end
        table.insert(systemInfo.settings, { setting = setting, value = value })
    end

    local function hasSystemSetting(systemInfo, setting)
        if not systemInfo or not systemInfo.settings then return false end
        for _, settingInfo in ipairs(systemInfo.settings) do
            if settingInfo.setting == setting then
                return true
            end
        end
        return false
    end

    local function copyAnchorInfo(anchorInfo)
        if type(anchorInfo) ~= "table" then
            return anchorInfo
        end
        local copy = {}
        for key, value in pairs(anchorInfo) do
            copy[key] = value
        end
        return copy
    end

    local function copySettings(settings)
        local copy = {}
        if type(settings) ~= "table" then
            return copy
        end
        for i, settingInfo in ipairs(settings) do
            copy[i] = {
                setting = settingInfo.setting,
                value = settingInfo.value,
            }
        end
        return copy
    end

    local function copySystems(systems)
        local copy = {}
        if type(systems) ~= "table" then
            return copy
        end
        for i, systemInfo in ipairs(systems) do
            copy[i] = {
                system = systemInfo.system,
                systemIndex = systemInfo.systemIndex,
                hidden = systemInfo.hidden,
                isInDefaultPosition = systemInfo.isInDefaultPosition,
                anchorInfo = copyAnchorInfo(systemInfo.anchorInfo),
                settings = copySettings(systemInfo.settings),
            }
        end
        return copy
    end

    local function copyLayouts(layouts)
        local copy = {}
        if type(layouts) ~= "table" then
            return copy
        end
        for i, layoutInfo in ipairs(layouts) do
            copy[i] = {
                layoutIndex = layoutInfo.layoutIndex,
                layoutName = layoutInfo.layoutName,
                layoutType = layoutInfo.layoutType,
                systems = copySystems(layoutInfo.systems),
            }
        end
        return copy
    end

    local function defaultSettingsFromModernMap(systemInfo)
        if not EditModePresetLayoutManager
            or not EditModePresetLayoutManager.GetModernSystemMap then
            return nil
        end

        local modernMap = EditModePresetLayoutManager:GetModernSystemMap()
        local systemDefaults = modernMap and modernMap[systemInfo.system]
        if type(systemDefaults) ~= "table" then
            return nil
        end

        if systemInfo.systemIndex == nil or systemInfo.systemIndex == -1 then
            if type(systemDefaults.settings) == "table" then
                return systemDefaults.settings
            end
        end

        local indexedDefaults = systemDefaults[systemInfo.systemIndex]
        if type(indexedDefaults) == "table" then
            return indexedDefaults.settings
        end
        return nil
    end

    local function defaultSettingsFromManager(systemInfo)
        if not EditModePresetLayoutManager
            or not EditModePresetLayoutManager.GetAllDefaultSettingsForSystem then
            return nil
        end

        local ok, defaults = pcall(
            EditModePresetLayoutManager.GetAllDefaultSettingsForSystem,
            EditModePresetLayoutManager,
            systemInfo.system,
            systemInfo.systemIndex
        )
        if ok then
            return defaults
        end
        return nil
    end

    local function mergeDefaultSystemSettings(systemInfo)
        if not systemInfo
            or not EditModePresetLayoutManager then
            return
        end

        local defaults = defaultSettingsFromModernMap(systemInfo)
        if not defaults and not EditModePresetLayoutManager.GetModernSystemMap then
            defaults = defaultSettingsFromManager(systemInfo)
        end
        if type(defaults) ~= "table" then
            return
        end

        for setting, value in pairs(defaults) do
            if not hasSystemSetting(systemInfo, setting) then
                setSystemSetting(systemInfo, setting, value)
            end
        end
    end

    local function mergeDefaultSettings(layoutInfo)
        if not layoutInfo or not layoutInfo.layouts then return end
        for _, layout in ipairs(layoutInfo.layouts) do
            if layout.layoutType ~= Enum.EditModeLayoutType.Preset and layout.systems then
                for _, systemInfo in ipairs(layout.systems) do
                    mergeDefaultSystemSettings(systemInfo)
                end
            end
        end
    end

    local function forceStandardPartyFrames(layoutInfo)
        if not layoutInfo or not layoutInfo.layouts then return end
        for _, preset in ipairs(layoutInfo.layouts) do
            if type(preset) == "table"
                and preset.layoutType == Enum.EditModeLayoutType.Preset
                and preset.systems then
                for _, systemInfo in ipairs(preset.systems) do
                    if systemInfo.system == Enum.EditModeSystem.UnitFrame
                        and systemInfo.systemIndex == Enum.EditModeUnitFrameSystemIndices.Party then
                        setSystemSetting(systemInfo, Enum.EditModeUnitFrameSetting.UseRaidStylePartyFrames, 0)
                    end
                end
            end
        end
    end

    local function remapActiveLayoutAfterPresetPrepend(layoutInfo, savedLayouts, presetCount)
        if not layoutInfo or type(layoutInfo.activeLayout) ~= "number" then
            return 1
        end
        if type((savedLayouts or {})[layoutInfo.activeLayout]) == "table" then
            return presetCount + layoutInfo.activeLayout
        end
        for savedIndex, savedLayout in ipairs(savedLayouts or {}) do
            if type(savedLayout) == "table" and savedLayout.layoutIndex == layoutInfo.activeLayout then
                return presetCount + savedIndex
            end
        end
        if layoutInfo.activeLayout >= 1 and layoutInfo.activeLayout <= #(layoutInfo.layouts or {}) then
            return layoutInfo.activeLayout
        end
        return 1
    end

    if not EditModeManagerFrame then return end
    local emm = EditModeManagerFrame

    local function savedLayoutsFromLayoutInfo(layoutInfo)
        local savedLayouts = {}
        local layoutType = Enum and Enum.EditModeLayoutType
        if not layoutInfo or type(layoutInfo.layouts) ~= "table" then
            return savedLayouts
        end
        for _, layout in ipairs(layoutInfo.layouts) do
            if type(layout) == "table"
                and (not layoutType or layout.layoutType ~= layoutType.Preset) then
                table.insert(savedLayouts, layout)
            end
        end
        return savedLayouts
    end

    local function updateLayoutCounts(savedLayouts)
        if type(emm.UpdateLayoutCounts) == "function" then
            local ok = pcall(emm.UpdateLayoutCounts, emm, savedLayouts or {})
            if ok then
                return
            end
        end

        local layoutType = Enum and Enum.EditModeLayoutType
        if type(layoutType) ~= "table" then
            return
        end

        emm.numLayouts = {}
        if layoutType.Account ~= nil then
            emm.numLayouts[layoutType.Account] = 0
        end
        if layoutType.Character ~= nil then
            emm.numLayouts[layoutType.Character] = 0
        end

        for _, layout in ipairs(savedLayouts or {}) do
            if type(layout) == "table"
                and layout.layoutType ~= nil
                and layout.layoutType ~= layoutType.Preset then
                emm.numLayouts[layout.layoutType] = (emm.numLayouts[layout.layoutType] or 0) + 1
            end
        end
    end

    local savedLayoutsForCounts
    if not emm.layoutInfo then
        local layoutInfo = C_EditMode.GetLayouts()
        emm.layoutInfo = layoutInfo
        local savedLayouts = copyLayouts(emm.layoutInfo.layouts)
        savedLayoutsForCounts = copyLayouts(savedLayouts)
        emm.layoutInfo.layouts = copyLayouts(EditModePresetLayoutManager.presetLayoutInfo)
        local presetCount = #emm.layoutInfo.layouts
        tAppendAll(emm.layoutInfo.layouts, savedLayouts)
        emm.layoutInfo.activeLayout = remapActiveLayoutAfterPresetPrepend(emm.layoutInfo, savedLayouts, presetCount)
    else
        savedLayoutsForCounts = copyLayouts(savedLayoutsFromLayoutInfo(emm.layoutInfo))
    end
    updateLayoutCounts(savedLayoutsForCounts)
    mergeDefaultSettings(emm.layoutInfo)
    forceStandardPartyFrames(emm.layoutInfo)
    local function applyAccountSettingOverrides()
        local accountSettings = emm.AccountSettings
        local accountEnum = Enum and Enum.EditModeAccountSetting
        if not accountSettings or not accountEnum then
            return
        end

        local function getAccountSettingValue(setting)
            local settingValue = nil
            if emm.GetAccountSettingValue then
                settingValue = emm:GetAccountSettingValue(setting)
            else
                for _, settingInfo in ipairs(emm.accountSettings or {}) do
                    if settingInfo.setting == setting then
                        settingValue = settingInfo.value
                        break
                    end
                end
            end
            return settingValue
        end

        local function getAccountSettingBool(setting)
            local settingValue = getAccountSettingValue(setting)
            if settingValue == nil then
                return nil
            end
            return settingValue == 1 or settingValue == true
        end

        local function applyFrameSetting(setting, frame, setter, isBool)
            if setting == nil or type(frame) ~= "table" or type(frame[setter]) ~= "function" then
                return
            end
            local settingValue
            if isBool then
                settingValue = getAccountSettingBool(setting)
            else
                settingValue = getAccountSettingValue(setting)
            end
            if settingValue ~= nil then
                pcall(frame[setter], frame, settingValue)
            end
        end

        local managerSettings = {
            { setting = accountEnum.ShowGrid, setter = "SetGridShown", isBool = true },
            { setting = accountEnum.GridSpacing, setter = "SetGridSpacing" },
            { setting = accountEnum.EnableSnap, setter = "SetEnableSnap", isBool = true },
            { setting = accountEnum.EnableAdvancedOptions, setter = "SetEnableAdvancedOptions", isBool = true },
        }
        for _, settingInfo in ipairs(managerSettings) do
            applyFrameSetting(settingInfo.setting, emm, settingInfo.setter, settingInfo.isBool)
        end

        local accountSettingSetters = {
            { setting = accountEnum.SettingsExpanded, setter = "SetExpandedState" },
            { setting = accountEnum.ShowTargetAndFocus, setter = "SetTargetAndFocusShown" },
            { setting = accountEnum.ShowPartyFrames, setter = "SetPartyFramesShown" },
            { setting = accountEnum.ShowRaidFrames, setter = "SetRaidFramesShown" },
            { setting = accountEnum.ShowStanceBar, setter = "SetStanceBarShown" },
            { setting = accountEnum.ShowPetActionBar, setter = "SetPetActionBarShown" },
            { setting = accountEnum.ShowPossessActionBar, setter = "SetPossessActionBarShown" },
            { setting = accountEnum.ShowCastBar, setter = "SetCastBarShown" },
            { setting = accountEnum.ShowEncounterBar, setter = "SetEncounterBarShown" },
            { setting = accountEnum.ShowExtraAbilities, setter = "SetExtraAbilitiesShown" },
            { setting = accountEnum.ShowBuffsAndDebuffs, setter = "SetBuffsAndDebuffsShown" },
            { setting = accountEnum.ShowExternalDefensives, setter = "SetExternalDefensivesShown" },
            { setting = accountEnum.ShowTalkingHeadFrame, setter = "SetTalkingHeadFrameShown" },
            { setting = accountEnum.ShowVehicleLeaveButton, setter = "SetVehicleLeaveButtonShown" },
            { setting = accountEnum.ShowBossFrames, setter = "SetBossFramesShown" },
            { setting = accountEnum.ShowArenaFrames, setter = "SetArenaFramesShown" },
            { setting = accountEnum.ShowLootFrame, setter = "SetLootFrameShown" },
            { setting = accountEnum.ShowHudTooltip, setter = "SetHudTooltipShown" },
            { setting = accountEnum.ShowStatusTrackingBar2, setter = "SetStatusTrackingBar2Shown" },
            { setting = accountEnum.ShowDurabilityFrame, setter = "SetDurabilityFrameShown" },
            { setting = accountEnum.ShowPetFrame, setter = "SetPetFrameShown" },
            { setting = accountEnum.ShowTimerBars, setter = "SetTimerBarsShown" },
            { setting = accountEnum.ShowVehicleSeatIndicator, setter = "SetVehicleSeatIndicatorShown" },
            { setting = accountEnum.ShowArchaeologyBar, setter = "SetArchaeologyBarShown" },
            { setting = accountEnum.ShowCooldownViewer, setter = "SetCooldownViewerShown" },
            { setting = accountEnum.ShowPersonalResourceDisplay, setter = "SetPersonalResourceDisplayShown" },
            { setting = accountEnum.ShowEncounterEvents, setter = "SetEncounterEventsShown" },
            { setting = accountEnum.ShowDamageMeter, setter = "SetDamageMeterShown" },
            { setting = accountEnum.ShowTotemActionBar, setter = "SetTotemActionBarShown" },
        }
        for _, settingInfo in ipairs(accountSettingSetters) do
            applyFrameSetting(settingInfo.setting, accountSettings, settingInfo.setter, true)
        end

        -- RefreshStatusTrackingBar2 is edit-mode-only (EditModeManager.lua:2489,
        -- reached from OnEditModeEnter): it sets isInEditMode and highlights
        -- the secondary container, which then rendered its selection brackets
        -- outside edit mode. syncStatusTrackingBars below applies the setting.

        local function getActiveLayoutInfo()
            local layoutInfo = emm.layoutInfo
            if type(layoutInfo) ~= "table" or type(layoutInfo.layouts) ~= "table" then
                return nil
            end
            return layoutInfo.layouts[layoutInfo.activeLayout]
        end

        local function getStatusTrackingSystemInfo(systemIndex)
            local editModeSystem = Enum and Enum.EditModeSystem
            if not editModeSystem or editModeSystem.StatusTrackingBar == nil then
                return nil
            end
            local activeLayout = getActiveLayoutInfo()
            for _, systemInfo in ipairs(activeLayout and activeLayout.systems or {}) do
                if systemInfo.system == editModeSystem.StatusTrackingBar
                    and systemInfo.systemIndex == systemIndex then
                    return systemInfo
                end
            end
            return nil
        end

        local function setStatusTrackingContainerAvailable(container, available)
            local manager = StatusTrackingBarManager
            if type(manager) ~= "table" or type(manager.barContainers) ~= "table"
                or type(container) ~= "table" then
                return false
            end

            local containerIndex = nil
            for index, barContainer in ipairs(manager.barContainers) do
                if barContainer == container then
                    containerIndex = index
                    break
                end
            end

            if available then
                if not containerIndex then
                    table.insert(manager.barContainers, container)
                end
                return true
            end

            if containerIndex then
                table.remove(manager.barContainers, containerIndex)
            end
            if type(container.SetShownBar) == "function" and StatusTrackingBarInfo
                and StatusTrackingBarInfo.BarsEnum then
                pcall(container.SetShownBar, container, StatusTrackingBarInfo.BarsEnum.None)
            end
            if type(container.Hide) == "function" then
                pcall(container.Hide, container)
            end
            return true
        end

        local function syncStatusTrackingBars()
            local statusTrackingIndices = Enum and Enum.EditModeStatusTrackingBarSystemIndices
            if statusTrackingIndices then
                local primaryInfo = getStatusTrackingSystemInfo(statusTrackingIndices.StatusTrackingBar1)
                if primaryInfo and primaryInfo.hidden then
                    setStatusTrackingContainerAvailable(MainStatusTrackingBarContainer, false)
                end
            end

            local shown = getAccountSettingBool(accountEnum.ShowStatusTrackingBar2)
            local manager = StatusTrackingBarManager
            local secondary = SecondaryStatusTrackingBarContainer
            if shown == nil then
                return
            end

            setStatusTrackingContainerAvailable(secondary, shown)

            if type(manager) == "table" and type(manager.UpdateBarsShown) == "function" then
                pcall(manager.UpdateBarsShown, manager)
            end
        end
        syncStatusTrackingBars()
    end
    if emm.InitializeAccountSettings then
        emm:InitializeAccountSettings()
        applyAccountSettingOverrides()
    else
        if not emm.accountSettings then
            emm.accountSettings = C_EditMode.GetAccountSettings()
        end
        if emm.UpdateAccountSettingMap then
            pcall(emm.UpdateAccountSettingMap, emm)
        end
        applyAccountSettingOverrides()
    end
    if SetCVarBitfield and Enum and Enum.FrameTutorialAccount
        and Enum.FrameTutorialAccount.EditModeManager then
        pcall(
            SetCVarBitfield,
            "closedInfoFramesAccountWide",
            Enum.FrameTutorialAccount.EditModeManager,
            true
        )
    end
