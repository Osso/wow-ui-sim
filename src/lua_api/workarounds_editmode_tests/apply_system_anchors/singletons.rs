use super::*;

#[test]
fn apply_system_anchors_maps_nil_system_index_to_saved_singleton_index() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                CastBar = 1,
            },
        }

        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }

        local frame = {
            system = Enum.EditModeSystem.CastBar,
            systemIndex = nil,
            name = "PlayerCastingBarFrame",
            updatedSettings = {},
        }

        function frame:GetName()
            return self.name
        end

        function frame:SetHasActiveChanges(value)
            self.hasActiveChanges = value
        end

        function frame:UpdateSettingMap()
            self.settingMapUpdated = true
        end

        function frame:ApplySystemAnchor()
            error("cast bar startup should not apply a scale-affecting anchor")
        end

        function frame:UpdateSystemSetting(setting, entireSystemUpdate)
            table.insert(self.updatedSettings, {
                setting = setting,
                entireSystemUpdate = entireSystemUpdate,
            })
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = { frame },
            layoutApplyInProgress = false,
            requestedSystemIndex = nil,
        }

        function EditModeManagerFrame:InitSystemAnchors()
            self.initSystemAnchorsCalled = true
        end

        function EditModeManagerFrame:GetActiveLayoutSystemInfo(_system, systemIndex)
            self.requestedSystemIndex = systemIndex
            if systemIndex ~= nil then
                return nil
            end
            return {
                system = Enum.EditModeSystem.CastBar,
                systemIndex = nil,
                isInDefaultPosition = false,
                anchorInfo = {
                    point = "CENTER",
                    relativeTo = UIParent,
                    relativePoint = "CENTER",
                    offsetX = 0,
                    offsetY = -174,
                },
                settings = {
                    { setting = 1, value = 0 },
                },
            }
        end

        function EditModeManagerFrame:UpdateSystem()
            error("nil-index singleton should be seeded directly")
        end
        "#,
    )
    .expect("install nil-index singleton stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply nil-index singleton anchors");

    let (requested_system_index, has_active_changes, setting_map_updated, updated_setting): (
        String,
        bool,
        bool,
        i32,
    ) = env
        .eval(
            r#"
            local frame = EditModeManagerFrame.registeredSystemFrames[1]
            return tostring(EditModeManagerFrame.requestedSystemIndex),
                frame.hasActiveChanges,
                frame.settingMapUpdated,
                frame.updatedSettings[1] and frame.updatedSettings[1].setting
            "#,
        )
        .expect("read nil-index singleton state");

    assert_eq!(requested_system_index, "nil");
    assert!(!has_active_changes, "system should be seeded as clean");
    assert!(
        setting_map_updated,
        "system settings should still be mapped"
    );
    assert_eq!(
        updated_setting, 1,
        "nil-index singleton settings should be applied after seeding"
    );
}

#[test]
fn apply_system_anchors_falls_back_to_minus_one_for_nil_singletons() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                CastBar = 1,
            },
        }

        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }

        local frame = {
            system = Enum.EditModeSystem.CastBar,
            systemIndex = nil,
            name = "PlayerCastingBarFrame",
            updatedSettings = {},
        }

        function frame:GetName() return self.name end
        function frame:SetHasActiveChanges(value) self.hasActiveChanges = value end
        function frame:UpdateSettingMap() self.settingMapUpdated = true end
        function frame:ClearAllPoints()
            self.currentAnchorInfo = nil
        end
        function frame:SetPoint(point, relativeTo, relativePoint, offsetX, offsetY)
            self.currentAnchorInfo = {
                point = point,
                relativeTo = relativeTo,
                relativePoint = relativePoint,
                offsetX = offsetX,
                offsetY = offsetY,
            }
        end
        function frame:ApplySystemAnchor() end
        function frame:UpdateSystemSetting(setting, entireSystemUpdate)
            table.insert(self.updatedSettings, {
                setting = setting,
                entireSystemUpdate = entireSystemUpdate,
            })
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = { frame },
            requestedSystemIndices = {},
        }

        function EditModeManagerFrame:InitSystemAnchors()
            for _, systemFrame in ipairs(self.registeredSystemFrames) do
                if not ((systemFrame.isBottomManagedFrame or systemFrame.isRightManagedFrame)
                    and systemFrame:IsInDefaultPosition()) then
                    systemFrame:ClearAllPoints()
                    systemFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
                end
            end
        end

        function EditModeManagerFrame:GetActiveLayoutSystemInfo(_system, systemIndex)
            table.insert(self.requestedSystemIndices, tostring(systemIndex))
            if systemIndex ~= -1 then
                return nil
            end
            return {
                system = Enum.EditModeSystem.CastBar,
                systemIndex = -1,
                isInDefaultPosition = false,
                anchorInfo = {
                    point = "CENTER",
                    relativeTo = UIParent,
                    relativePoint = "CENTER",
                    offsetX = 0,
                    offsetY = -174,
                },
                settings = {
                    { setting = 1, value = 0 },
                },
            }
        end

        function EditModeManagerFrame:UpdateSystem()
            error("fallback singleton should be seeded directly")
        end
        "#,
    )
    .expect("install nil-index fallback stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply nil-index fallback singleton anchors");

    let (requested_system_indices, setting_map_updated, updated_setting): (String, bool, i32) = env
        .eval(
            r#"
            local frame = EditModeManagerFrame.registeredSystemFrames[1]
            return table.concat(EditModeManagerFrame.requestedSystemIndices, ","),
                frame.settingMapUpdated,
                frame.updatedSettings[1] and frame.updatedSettings[1].setting
            "#,
        )
        .expect("read nil-index fallback singleton state");

    assert_eq!(requested_system_indices, "nil,-1");
    assert!(
        setting_map_updated,
        "fallback system settings should still be mapped"
    );
    assert_eq!(
        updated_setting, 1,
        "fallback singleton settings should be applied after seeding"
    );
}

#[test]
fn apply_system_anchors_replays_active_widescreen_singleton_settings() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                CastBar = 1,
                Minimap = 2,
                ChatFrame = 8,
                ObjectiveTracker = 12,
                MicroMenu = 13,
                Bags = 14,
                DurabilityFrame = 16,
                TimerBars = 17,
                VehicleSeatIndicator = 18,
                ArchaeologyBar = 19,
            },
        }

        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }

        local widescreenRows = {
            [Enum.EditModeSystem.CastBar] = { {0, 0}, {1, 0}, {2, 0} },
            [Enum.EditModeSystem.Minimap] = { {0, 0}, {1, 0}, {2, 5} },
            [Enum.EditModeSystem.ChatFrame] = { {0, 3}, {1, 48}, {2, 1}, {3, 20} },
            [Enum.EditModeSystem.ObjectiveTracker] = { {0, 1}, {1, 0}, {2, 0} },
            [Enum.EditModeSystem.MicroMenu] = { {0, 1}, {1, 0}, {2, 6}, {3, 4} },
            [Enum.EditModeSystem.Bags] = { {0, 0}, {1, 0}, {2, 5} },
            [Enum.EditModeSystem.DurabilityFrame] = { {0, 5} },
            [Enum.EditModeSystem.TimerBars] = { {0, 0} },
            [Enum.EditModeSystem.VehicleSeatIndicator] = { {0, 10} },
            [Enum.EditModeSystem.ArchaeologyBar] = { {0, 0} },
        }

        local function settingsFor(system)
            local settings = {}
            for _, pair in ipairs(widescreenRows[system] or {}) do
                table.insert(settings, { setting = pair[1], value = pair[2] })
            end
            return settings
        end

        local function newSingletonFrame(system, name)
            local frame = {
                system = system,
                systemIndex = nil,
                name = name,
                replayedSettings = {},
                replayedValues = {},
            }

            function frame:GetName()
                return self.name
            end
            function frame:SetHasActiveChanges(value)
                self.hasActiveChanges = value
            end
            function frame:UpdateSettingMap()
                self.settingMapUpdated = true
            end
            function frame:GetSettingValue(setting)
                for _, settingInfo in ipairs(self.systemInfo.settings or {}) do
                    if settingInfo.setting == setting then
                        return settingInfo.value
                    end
                end
            end
            function frame:UpdateSystemSetting(setting, entireSystemUpdate)
                table.insert(self.replayedSettings, setting)
                table.insert(self.replayedValues, tostring(setting) .. "=" .. tostring(self:GetSettingValue(setting)))
                self.entireSystemUpdate = entireSystemUpdate
            end
            function frame:UpdateSystem(systemInfo)
                self.updateSystemCalls = (self.updateSystemCalls or 0) + 1
                self.systemInfo = systemInfo
                for _, settingInfo in ipairs(systemInfo.settings or {}) do
                    self:UpdateSystemSetting(settingInfo.setting, true)
                end
            end
            function frame:ApplySystemAnchor()
                self.anchorCalls = (self.anchorCalls or 0) + 1
            end

            return frame
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            requestedRows = {},
            registeredSystemFrames = {
                newSingletonFrame(Enum.EditModeSystem.CastBar, "PlayerCastingBarFrame"),
                newSingletonFrame(Enum.EditModeSystem.Minimap, "MinimapCluster"),
                newSingletonFrame(Enum.EditModeSystem.ChatFrame, "ChatFrame1"),
                newSingletonFrame(Enum.EditModeSystem.ObjectiveTracker, "ObjectiveTrackerFrame"),
                newSingletonFrame(Enum.EditModeSystem.MicroMenu, "MicroMenu"),
                newSingletonFrame(Enum.EditModeSystem.Bags, "BagsBar"),
                newSingletonFrame(Enum.EditModeSystem.DurabilityFrame, "DurabilityFrame"),
                newSingletonFrame(Enum.EditModeSystem.TimerBars, "TimerBarsFrame"),
                newSingletonFrame(Enum.EditModeSystem.VehicleSeatIndicator, "VehicleSeatIndicator"),
                newSingletonFrame(Enum.EditModeSystem.ArchaeologyBar, "ArchaeologyBarFrame"),
            },
        }

        function EditModeManagerFrame:InitSystemAnchors()
            self.initSystemAnchorsCalled = true
        end
        function EditModeManagerFrame:GetActiveLayoutSystemInfo(system, systemIndex)
            table.insert(self.requestedRows, tostring(system) .. ":" .. tostring(systemIndex))
            return {
                system = system,
                systemIndex = systemIndex,
                isInDefaultPosition = false,
                anchorInfo = { point = "CENTER", relativeTo = UIParent, relativePoint = "CENTER", offsetX = 0, offsetY = 0 },
                settings = settingsFor(system),
            }
        end
        function EditModeManagerFrame:UpdateSystem(systemFrame)
            -- Mirrors EditModeManagerFrameMixin:UpdateSystem -- the manager
            -- resolves the active layout itself; nothing pre-seeds the frame.
            systemFrame:UpdateSystem(self:GetActiveLayoutSystemInfo(systemFrame.system, systemFrame.systemIndex))
        end
        "#,
    )
    .expect("install active singleton stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply active singleton settings");

    let (requested_rows, replayed_values, update_system_calls, anchor_calls): (
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            r#"
            local replayedRows = {}
            local updateRows = {}
            local anchorRows = {}
            for _, frame in ipairs(EditModeManagerFrame.registeredSystemFrames) do
                table.insert(replayedRows, tostring(frame.system) .. ":" .. table.concat(frame.replayedValues, ","))
                table.insert(updateRows, tostring(frame.updateSystemCalls or 0))
                table.insert(anchorRows, tostring(frame.system) .. ":" .. tostring(frame.anchorCalls or 0))
            end
            return table.concat(EditModeManagerFrame.requestedRows, "|"),
                table.concat(replayedRows, "|"),
                table.concat(updateRows, ","),
                table.concat(anchorRows, "|")
            "#,
        )
        .expect("read singleton replay state");

    assert_eq!(
        requested_rows, "1:nil|2:nil|8:nil|12:nil|13:nil|14:nil|16:nil|17:nil|18:nil|19:nil",
        "singleton Widescreen systems should preserve nil system indices when the active layout has nil rows"
    );
    assert_eq!(
        replayed_values,
        "1:0=0,1=0,2=0|2:0=0,1=0,2=5|8:0=3,1=48,2=1,3=20|12:0=1,1=0,2=0|13:0=1,1=0,2=6,3=4|14:0=0,1=0,2=5|16:0=5|17:0=0|18:0=10|19:0=0",
        "every active Widescreen singleton option row should replay its saved setting values"
    );
    assert_eq!(
        update_system_calls, "0,1,1,1,1,1,1,1,1,1",
        "only the cast bar should use the direct startup replay branch"
    );
    assert_eq!(
        anchor_calls, "1:0|2:1|8:1|12:1|13:1|14:1|16:1|17:1|18:1|19:1",
        "ordinary singleton systems such as Minimap should apply saved anchors after UpdateSystem"
    );
}

#[test]
fn apply_system_anchors_preserves_addon_hidden_singleton_visibility() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                MicroMenu = 13,
            },
        }

        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }

        local frame = {
            system = Enum.EditModeSystem.MicroMenu,
            systemIndex = nil,
            name = "MicroMenuContainer",
            shown = false,
        }

        function frame:GetName()
            return self.name
        end
        function frame:SetHasActiveChanges(value)
            self.hasActiveChanges = value
        end
        function frame:UpdateSettingMap()
            self.settingMapUpdated = true
        end
        function frame:IsShown()
            return self.shown
        end
        function frame:Show()
            self.shown = true
        end
        function frame:Hide()
            self.shown = false
            self.hideCalls = (self.hideCalls or 0) + 1
        end
        function frame:ApplySystemAnchor()
            self.anchorCalls = (self.anchorCalls or 0) + 1
        end
        function frame:UpdateSystem(systemInfo)
            self.updateSystemCalls = (self.updateSystemCalls or 0) + 1
            self.systemInfo = systemInfo
            self:Show()
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = { frame },
        }

        function EditModeManagerFrame:InitSystemAnchors()
            self.initSystemAnchorsCalled = true
        end
        function EditModeManagerFrame:GetActiveLayoutSystemInfo(system, systemIndex)
            return {
                system = system,
                systemIndex = systemIndex,
                isInDefaultPosition = false,
                anchorInfo = { point = "TOP", relativeTo = UIParent, relativePoint = "TOP", offsetX = 0, offsetY = 0 },
                settings = {},
            }
        end
        function EditModeManagerFrame:UpdateSystem(systemFrame)
            -- Mirrors EditModeManagerFrameMixin:UpdateSystem -- the manager
            -- resolves the active layout itself; nothing pre-seeds the frame.
            systemFrame:UpdateSystem(self:GetActiveLayoutSystemInfo(systemFrame.system, systemFrame.systemIndex))
        end
        "#,
    )
    .expect("install hidden singleton stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply singleton anchors without resurrecting addon-hidden frame");

    let (shown, update_calls, anchor_calls, hide_calls): (bool, i64, i64, i64) = env
        .eval(
            r#"
            local frame = EditModeManagerFrame.registeredSystemFrames[1]
            return frame:IsShown(),
                frame.updateSystemCalls or 0,
                frame.anchorCalls or 0,
                frame.hideCalls or 0
            "#,
        )
        .expect("read hidden singleton replay state");

    assert!(
        !shown,
        "startup EditMode replay must not resurrect addon-hidden frames"
    );
    assert_eq!(
        update_calls, 1,
        "hidden singleton still needs UpdateSystem for layout state"
    );
    assert_eq!(
        anchor_calls, 1,
        "hidden singleton still needs anchor replay for later re-show"
    );
    assert_eq!(
        hide_calls, 1,
        "frame should be hidden again after UpdateSystem temporarily shows it"
    );
}

#[test]
fn apply_system_anchors_replays_remaining_active_widescreen_system_settings() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                EncounterBar = 4,
                ExtraAbilities = 5,
                TalkingHeadFrame = 7,
                VehicleLeaveButton = 9,
                LootFrame = 10,
                HudTooltip = 11,
                StatusTrackingBar = 15,
                PersonalResourceDisplay = 21,
                EncounterEvents = 22,
                DamageMeter = 23,
            },
        }

        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }

        local layoutRows = {
            "4:-1:",
            "5:-1:",
            "7:-1:",
            "9:-1:",
            "10:-1:",
            "11:-1:",
            "15:1:3=10",
            "15:2:3=10",
            "21:-1:0=0,1=0",
            "22:1:0=1,1=1,2=0,3=5,4=5,5=0,6=50,7=1,8=1,9=1,10=0,11=0,12=50,13=2",
            "22:2:3=5,4=5,6=50,7=0,8=1",
            "22:3:3=5,4=5,6=50,7=0,8=1",
            "22:4:3=5,4=5,6=50,7=0,8=1",
            "23:-1:0=0,1=0,2=1,3=0,4=13,5=2,6=50,8=1,9=1,10=1,11=5,12=50",
        }

        local systemsByKey = {}
        for _, row in ipairs(layoutRows) do
            local system, systemIndex, settings = string.match(row, "([^:]+):([^:]+):(.*)")
            local key = system .. ":" .. systemIndex
            local settingRows = {}
            for setting, value in string.gmatch(settings or "", "([^=,]+)=([^=,]+)") do
                table.insert(settingRows, {
                    setting = tonumber(setting),
                    value = tonumber(value),
                })
            end
            systemsByKey[key] = {
                system = tonumber(system),
                systemIndex = tonumber(systemIndex),
                isInDefaultPosition = false,
                anchorInfo = {
                    point = "CENTER",
                    relativeTo = UIParent,
                    relativePoint = "CENTER",
                    offsetX = 0,
                    offsetY = 0,
                },
                settings = settingRows,
            }
        end

        local function newSystemFrame(system, systemIndex, name)
            local frame = {
                system = system,
                systemIndex = systemIndex,
                name = name,
                replayedValues = {},
            }

            function frame:GetName()
                return self.name
            end
            function frame:SetHasActiveChanges(value)
                self.hasActiveChanges = value
            end
            function frame:UpdateSettingMap()
                self.settingMapUpdated = true
            end
            function frame:GetSettingValue(setting)
                for _, settingInfo in ipairs(self.systemInfo.settings or {}) do
                    if settingInfo.setting == setting then
                        return settingInfo.value
                    end
                end
            end
            function frame:UpdateSystemSetting(setting, entireSystemUpdate)
                table.insert(self.replayedValues, tostring(setting) .. "=" .. tostring(self:GetSettingValue(setting)))
                self.entireSystemUpdate = entireSystemUpdate
            end
            function frame:UpdateSystem(systemInfo)
                self.updateSystemCalls = (self.updateSystemCalls or 0) + 1
                self.systemInfo = systemInfo
                for _, settingInfo in ipairs(systemInfo.settings or {}) do
                    self:UpdateSystemSetting(settingInfo.setting, true)
                end
            end

            return frame
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            requestedRows = {},
            registeredSystemFrames = {
                newSystemFrame(Enum.EditModeSystem.EncounterBar, nil, "EncounterBar"),
                newSystemFrame(Enum.EditModeSystem.ExtraAbilities, nil, "ExtraAbilities"),
                newSystemFrame(Enum.EditModeSystem.TalkingHeadFrame, nil, "TalkingHeadFrame"),
                newSystemFrame(Enum.EditModeSystem.VehicleLeaveButton, nil, "VehicleLeaveButton"),
                newSystemFrame(Enum.EditModeSystem.LootFrame, nil, "LootFrame"),
                newSystemFrame(Enum.EditModeSystem.HudTooltip, nil, "HudTooltip"),
                newSystemFrame(Enum.EditModeSystem.StatusTrackingBar, 1, "StatusTrackingBar1"),
                newSystemFrame(Enum.EditModeSystem.StatusTrackingBar, 2, "StatusTrackingBar2"),
                newSystemFrame(Enum.EditModeSystem.PersonalResourceDisplay, nil, "PersonalResourceDisplay"),
                newSystemFrame(Enum.EditModeSystem.EncounterEvents, 1, "EncounterEventsTimeline"),
                newSystemFrame(Enum.EditModeSystem.EncounterEvents, 2, "EncounterEventsCriticalWarnings"),
                newSystemFrame(Enum.EditModeSystem.EncounterEvents, 3, "EncounterEventsMediumWarnings"),
                newSystemFrame(Enum.EditModeSystem.EncounterEvents, 4, "EncounterEventsNormalWarnings"),
                newSystemFrame(Enum.EditModeSystem.DamageMeter, nil, "DamageMeter"),
            },
        }

        function EditModeManagerFrame:InitSystemAnchors()
            self.initSystemAnchorsCalled = true
        end
        function EditModeManagerFrame:GetActiveLayoutSystemInfo(system, systemIndex)
            local key = tostring(system) .. ":" .. tostring(systemIndex)
            table.insert(self.requestedRows, key)
            return systemsByKey[key]
        end
        function EditModeManagerFrame:UpdateSystem(systemFrame)
            -- Mirrors EditModeManagerFrameMixin:UpdateSystem -- the manager
            -- resolves the active layout itself; nothing pre-seeds the frame.
            systemFrame:UpdateSystem(self:GetActiveLayoutSystemInfo(systemFrame.system, systemFrame.systemIndex))
        end
        "#,
    )
    .expect("install remaining widescreen system stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply remaining widescreen system settings");

    let (requested_rows, replayed_values, update_system_calls): (String, String, String) = env
        .eval(
            r#"
            local replayedRows = {}
            local updateRows = {}
            for _, frame in ipairs(EditModeManagerFrame.registeredSystemFrames) do
                table.insert(
                    replayedRows,
                    tostring(frame.system) .. ":"
                        .. tostring(frame.systemIndex or -1) .. ":"
                        .. table.concat(frame.replayedValues, ",")
                )
                table.insert(updateRows, tostring(frame.updateSystemCalls or 0))
            end
            return table.concat(EditModeManagerFrame.requestedRows, "|"),
                table.concat(replayedRows, "|"),
                table.concat(updateRows, ",")
            "#,
        )
        .expect("read remaining widescreen replay state");

    assert_eq!(
        requested_rows,
        "4:nil|4:-1|5:nil|5:-1|7:nil|7:-1|9:nil|9:-1|10:nil|10:-1|11:nil|11:-1|15:1|15:2|21:nil|21:-1|22:1|22:2|22:3|22:4|23:nil|23:-1",
        "remaining Widescreen nil-index systems should try nil before saved-cache -1 fallback"
    );
    assert_eq!(
        replayed_values,
        "4:-1:|5:-1:|7:-1:|9:-1:|10:-1:|11:-1:|15:1:3=10|15:2:3=10|21:-1:0=0,1=0|22:1:0=1,1=1,2=0,3=5,4=5,5=0,6=50,7=1,8=1,9=1,10=0,11=0,12=50,13=2|22:2:3=5,4=5,6=50,7=0,8=1|22:3:3=5,4=5,6=50,7=0,8=1|22:4:3=5,4=5,6=50,7=0,8=1|23:-1:0=0,1=0,2=1,3=0,4=13,5=2,6=50,8=1,9=1,10=1,11=5,12=50",
        "remaining active Widescreen settings should replay saved values"
    );
    assert_eq!(
        update_system_calls, "1,1,1,1,1,1,1,1,1,1,1,1,1,1",
        "remaining systems should apply settings through their normal UpdateSystem path"
    );
}
