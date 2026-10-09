use super::*;

#[test]
fn apply_system_anchors_converts_raw_action_bar_icon_size_before_scaling() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                ActionBar = 0,
            },
            EditModeActionBarSetting = {
                IconSize = 3,
            },
            ActionBarOrientation = {
                Horizontal = 0,
                Vertical = 1,
            },
            EditModeSettingDisplayType = {
                Slider = 1,
            },
        }
        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return true end,
            IsRightAnchoredActionBar = function() return false end,
        }
        EditModeSettingDisplayInfoManager = {
            GetSystemSettingDisplayInfoMap = function()
                return {
                    [Enum.EditModeActionBarSetting.IconSize] = {
                        minValue = 50,
                        maxValue = 200,
                        stepSize = 10,
                        ConvertValueForDisplay = function(self, value)
                            return math.max(self.minValue, math.min(self.maxValue, (value * self.stepSize) + self.minValue))
                        end,
                    },
                }
            end,
        }

        local button = { container = {} }
        function button.container:SetScale(value)
            if value <= 0 then
                error("container received non-positive scale")
            end
            self.scale = value
        end

        local frame = {
            system = Enum.EditModeSystem.ActionBar,
            systemIndex = 1,
            systemInfo = {
                settings = {
                    { setting = Enum.EditModeActionBarSetting.IconSize, value = 0 },
                },
            },
            actionButtons = { button },
            Selection = {},
            dirtySettings = {},
        }
        function frame:GetName()
            return "MainActionBar"
        end
        function frame:SetHasActiveChanges() end
        function frame:UpdateSettingMap()
            -- Simulate startup frames whose Blizzard OnSystemLoad did not
            -- populate settingDisplayInfoMap before the fast replay path.
            self.settingMap = {}
            for _, settingInfo in ipairs(self.systemInfo.settings) do
                local displayInfo = self.settingDisplayInfoMap and self.settingDisplayInfoMap[settingInfo.setting]
                self.settingMap[settingInfo.setting] = {
                    value = settingInfo.value,
                    displayValue = displayInfo and displayInfo:ConvertValueForDisplay(settingInfo.value) or nil,
                }
            end
        end
        function frame:GetSettingValue(setting, useRawValue)
            local settingInfo = self.settingMap[setting]
            if useRawValue then
                return settingInfo and settingInfo.value
            end
            return settingInfo and (settingInfo.displayValue or settingInfo.value)
        end
        function frame:ApplySystemAnchor() end
        function frame:EditModeSetScale(value)
            if value <= 0 then
                error("frame received non-positive scale")
            end
            self.editModeScale = value
        end
        function frame:Layout() end

        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = { frame },
            InitSystemAnchors = function() end,
            GetActiveLayoutSystemInfo = function()
                return frame.systemInfo
            end,
        }
        "#,
    )
    .expect("install raw icon-size replay stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply system anchors should convert raw icon size before scaling");

    let (frame_scale, button_scale): (f64, f64) = env
        .eval(
            r#"
            local frame = EditModeManagerFrame.registeredSystemFrames[1]
            return frame.editModeScale, frame.actionButtons[1].container.scale
            "#,
        )
        .expect("read replayed scales");

    assert_eq!(frame_scale, 0.5);
    assert_eq!(button_scale, 0.5);
}

#[test]
fn apply_system_anchors_seeds_display_info_before_cast_bar_scale_replay() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                CastBar = 1,
            },
            EditModeCastBarSetting = {
                BarSize = 0,
                LockToPlayerFrame = 1,
            },
            EditModeSettingDisplayType = {
                Slider = 1,
            },
        }
        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }
        EditModeSettingDisplayInfoManager = {
            GetSystemSettingDisplayInfoMap = function()
                return {
                    [Enum.EditModeCastBarSetting.BarSize] = {
                        minValue = 100,
                        maxValue = 150,
                        stepSize = 10,
                        ConvertValueForDisplay = function(self, value)
                            return math.max(self.minValue, math.min(self.maxValue, (value * self.stepSize) + self.minValue))
                        end,
                    },
                }
            end,
        }

        local frame = {
            system = Enum.EditModeSystem.CastBar,
            systemIndex = 1,
            systemInfo = {
                settings = {
                    { setting = Enum.EditModeCastBarSetting.BarSize, value = 0 },
                    { setting = Enum.EditModeCastBarSetting.LockToPlayerFrame, value = 0 },
                },
                anchorInfo = { point = "BOTTOM", relativeTo = UIParent, relativePoint = "BOTTOM", offsetX = 0, offsetY = 0 },
            },
            dirtySettings = {},
            setPointBaseCalls = 0,
        }
        function frame:GetName()
            return "PlayerCastingBarFrame"
        end
        function frame:SetHasActiveChanges() end
        function frame:UpdateSettingMap()
            self.settingMap = {}
            for _, settingInfo in ipairs(self.systemInfo.settings) do
                local displayInfo = self.settingDisplayInfoMap and self.settingDisplayInfoMap[settingInfo.setting]
                self.settingMap[settingInfo.setting] = {
                    value = settingInfo.value,
                    displayValue = displayInfo and displayInfo:ConvertValueForDisplay(settingInfo.value) or nil,
                }
                self.dirtySettings[settingInfo.setting] = true
            end
        end
        function frame:GetSettingValue(setting)
            local settingInfo = self.settingMap[setting]
            return settingInfo and (settingInfo.displayValue or settingInfo.value)
        end
        function frame:GetSettingValueBool(setting)
            return self:GetSettingValue(setting) == 1
        end
        function frame:IsSettingDirty(setting)
            return self.dirtySettings[setting]
        end
        function frame:SetScale(value)
            if value <= 0 then
                error("cast bar received non-positive scale")
            end
            self.scale = value
        end
        function frame:ClearAllPoints()
            error("startup cast-bar anchor replay should not call the EditMode ClearAllPoints override")
        end
        function frame:ClearAllPointsBase()
            self.clearedBasePoints = true
        end
        function frame:SetPoint()
            error("startup cast-bar anchor replay should not call the EditMode SetPoint override")
        end
        function frame:SetPointBase(point, relativeTo, relativePoint, offsetX, offsetY)
            self.setPointBaseCalls = self.setPointBaseCalls + 1
            self.point = point
            self.relativeTo = relativeTo
            self.relativePoint = relativePoint
            self.offsetX = offsetX
            self.offsetY = offsetY
        end
        function frame:ApplySystemAnchor()
            self.anchorCalls = (self.anchorCalls or 0) + 1
        end
        function frame:UpdateSystemSetting(setting)
            if setting == Enum.EditModeCastBarSetting.BarSize then
                self:SetScale(self:GetSettingValue(setting) / 100)
            elseif setting == Enum.EditModeCastBarSetting.LockToPlayerFrame then
                error("lock replay should not call full Blizzard cast-bar update")
            end
        end
        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = { frame },
            InitSystemAnchors = function() end,
            GetActiveLayoutSystemInfo = function()
                return frame.systemInfo
            end,
            OnEditModeSystemAnchorChanged = function()
                error("startup cast-bar anchor replay should not notify full EditMode layout")
            end,
        }
        "#,
    )
    .expect("install cast bar raw-size replay stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply system anchors should seed display info before cast bar scale replay");

    let (scale, set_point_calls, anchor_calls, point, relative_point): (
        f64,
        i64,
        i64,
        String,
        String,
    ) = env
        .eval(
            r#"
            local f = EditModeManagerFrame.registeredSystemFrames[1]
            return f.scale, f.setPointBaseCalls, f.anchorCalls or 0, f.point, f.relativePoint
            "#,
        )
        .expect("read converted cast bar scale and anchor replay");

    assert_eq!(scale, 1.0);
    assert_eq!(
        set_point_calls, 1,
        "unlocked cast bars should apply saved anchorInfo through base SetPoint"
    );
    assert_eq!(
        anchor_calls, 0,
        "direct cast-bar anchor replay should avoid the hookable ApplySystemAnchor path"
    );
    assert_eq!(point, "BOTTOM");
    assert_eq!(relative_point, "BOTTOM");
}

#[test]
fn apply_system_anchors_replays_player_frame_size_without_cast_bar_side_effect() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = { UnitFrame = 2 },
            EditModeUnitFrameSetting = { FrameSize = 16 },
            EditModeUnitFrameSystemIndices = { Player = 1 },
        }
        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }
        EditModeSettingDisplayInfoManager = {
            GetSystemSettingDisplayInfoMap = function()
                return {
                    [Enum.EditModeUnitFrameSetting.FrameSize] = {
                        minValue = 100,
                        maxValue = 200,
                        stepSize = 5,
                        ConvertValueForDisplay = function(self, value)
                            return math.max(self.minValue, math.min(self.maxValue, (value * self.stepSize) + self.minValue))
                        end,
                    },
                }
            end,
        }
        PlayerCastingBarFrame = {
            UpdateSystemSettingBarSize = function()
                error("player frame size replay should not update cast bar size during startup")
            end,
        }

        local frame = {
            system = Enum.EditModeSystem.UnitFrame,
            systemIndex = Enum.EditModeUnitFrameSystemIndices.Player,
            systemInfo = {
                settings = {
                    { setting = Enum.EditModeUnitFrameSetting.FrameSize, value = 0 },
                },
                anchorInfo = { point = "BOTTOM", relativeTo = UIParent, relativePoint = "BOTTOM", offsetX = 0, offsetY = 0 },
            },
            dirtySettings = {},
            setPointBaseCalls = 0,
        }
        function frame:GetName() return "PlayerFrame" end
        function frame:SetHasActiveChanges() end
        function frame:UpdateSettingMap()
            self.settingMap = {}
            for _, settingInfo in ipairs(self.systemInfo.settings) do
                local displayInfo = self.settingDisplayInfoMap and self.settingDisplayInfoMap[settingInfo.setting]
                self.settingMap[settingInfo.setting] = {
                    value = settingInfo.value,
                    displayValue = displayInfo and displayInfo:ConvertValueForDisplay(settingInfo.value) or nil,
                }
                self.dirtySettings[settingInfo.setting] = true
            end
        end
        function frame:GetSettingValue(setting, useRawValue)
            local settingInfo = self.settingMap[setting]
            if useRawValue then return settingInfo and settingInfo.value end
            return settingInfo and (settingInfo.displayValue or settingInfo.value)
        end
        function frame:SetScale(value)
            if value <= 0 then error("player frame received non-positive scale") end
            self.scale = value
        end
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
        function frame:ClearAllPointsBase()
            self.clearedPoints = true
        end
        function frame:SetPointBase(point, relativeTo, relativePoint, offsetX, offsetY)
            self.setPointBaseCalls = self.setPointBaseCalls + 1
            self.point = point
            self.relativeTo = relativeTo
            self.relativePoint = relativePoint
            self.offsetX = offsetX
            self.offsetY = offsetY
        end
        function frame:ApplySystemAnchor()
            self.anchorCalls = (self.anchorCalls or 0) + 1
            if PlayerCastingBarFrame.ApplySystemAnchor then
                PlayerCastingBarFrame:ApplySystemAnchor()
            end
        end
        function frame:UpdateSystemSetting(setting)
            if setting == Enum.EditModeUnitFrameSetting.FrameSize then
                self:SetScale(self:GetSettingValue(setting) / 100)
                PlayerCastingBarFrame:UpdateSystemSettingBarSize()
            end
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = { frame },
            GetActiveLayoutSystemInfo = function() return frame.systemInfo end,
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
        "#,
    )
    .expect("install player frame replay stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply system anchors should avoid player-frame cast-bar side effects");

    let (scale, set_point_calls, anchor_calls, point, relative_point): (
        f64,
        i64,
        i64,
        String,
        String,
    ) = env
        .eval(
            r#"
            local f = EditModeManagerFrame.registeredSystemFrames[1]
            return f.scale, f.setPointBaseCalls, f.anchorCalls or 0, f.point, f.relativePoint
            "#,
        )
        .expect("read converted player frame scale and direct anchor replay");

    assert_eq!(scale, 1.0);
    assert_eq!(
        set_point_calls, 1,
        "player frame should apply its saved anchorInfo through base SetPoint"
    );
    assert_eq!(
        anchor_calls, 0,
        "direct player-frame anchor replay should avoid cast-bar ApplySystemAnchor side effects"
    );
    assert_eq!(point, "BOTTOM");
    assert_eq!(relative_point, "BOTTOM");
}

#[test]
fn apply_system_anchors_seeds_unit_frame_settings_without_full_startup_update() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                UnitFrame = 3,
            },
            EditModeUnitFrameSetting = {
                BuffsOnTop = 2,
                FrameSize = 16,
            },
        }

        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }

        local frame = {
            system = Enum.EditModeSystem.UnitFrame,
            systemIndex = 1,
            name = "PlayerFrame",
            anchorCalls = 0,
            setPointBaseCalls = 0,
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
            self.anchorCalls = self.anchorCalls + 1
        end

        function frame:SetPointBase(point, relativeTo, relativePoint, offsetX, offsetY)
            self.setPointBaseCalls = self.setPointBaseCalls + 1
            self.point = point
            self.relativeTo = relativeTo
            self.relativePoint = relativePoint
            self.offsetX = offsetX
            self.offsetY = offsetY
        end

        function frame:UpdateSystemSetting(setting, entireSystemUpdate)
            if setting == Enum.EditModeUnitFrameSetting.BuffsOnTop then
                self.buffsAttempted = true
                error("BuffsOnTop should not call UpdateSystemSetting without UpdateAuras")
            end
            table.insert(self.updatedSettings, {
                setting = setting,
                entireSystemUpdate = entireSystemUpdate,
            })
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = { frame },
            layoutApplyInProgress = false,
        }

        function EditModeManagerFrame:InitSystemAnchors()
            self.initSystemAnchorsCalled = true
        end

        function EditModeManagerFrame:GetActiveLayoutSystemInfo()
            return {
                system = Enum.EditModeSystem.UnitFrame,
                systemIndex = 1,
                isInDefaultPosition = false,
                anchorInfo = {
                    point = "TOPLEFT",
                    relativeTo = UIParent,
                    relativePoint = "TOPLEFT",
                    offsetX = -292.4,
                    offsetY = -144.4,
                },
                settings = {
                    { setting = Enum.EditModeUnitFrameSetting.BuffsOnTop, value = 1 },
                    { setting = Enum.EditModeUnitFrameSetting.FrameSize, value = 0 },
                },
            }
        end

        function EditModeManagerFrame:UpdateSystem()
            error("unit frame startup should not run the full update path")
        end
        "#,
    )
    .expect("install unit frame stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply unit frame anchors");

    let (
        anchor_calls,
        set_point_base_calls,
        has_active_changes,
        setting_map_updated,
        updated_setting,
        entire_update,
        buffs_on_top,
        buffs_attempted,
    ): (i32, i32, bool, bool, i32, bool, bool, bool) = env
        .eval(
            r#"
            local frame = EditModeManagerFrame.registeredSystemFrames[1]
            return frame.anchorCalls,
                frame.setPointBaseCalls,
                frame.hasActiveChanges,
                frame.settingMapUpdated,
                frame.updatedSettings[1] and frame.updatedSettings[1].setting,
                frame.updatedSettings[1] and frame.updatedSettings[1].entireSystemUpdate,
                frame.buffsOnTop,
                frame.buffsAttempted or false
            "#,
        )
        .expect("read unit frame state");

    assert_eq!(
        set_point_base_calls, 1,
        "saved unit frame anchor should apply through base SetPoint"
    );
    assert_eq!(
        anchor_calls, 0,
        "PlayerFrame direct anchor replay should avoid ApplySystemAnchor side effects"
    );
    assert!(!has_active_changes, "system should be seeded as clean");
    assert!(
        setting_map_updated,
        "system settings should still be mapped"
    );
    assert_eq!(
        updated_setting, 16,
        "saved unit-frame settings should be applied even when full UpdateSystem is skipped"
    );
    assert!(
        entire_update,
        "startup setting application should use the full-update flag"
    );
    assert!(
        buffs_on_top,
        "BuffsOnTop should still apply its saved value without refreshing auras"
    );
    assert!(
        !buffs_attempted,
        "BuffsOnTop should not call the unsafe aura refresh path without UpdateAuras"
    );
}
