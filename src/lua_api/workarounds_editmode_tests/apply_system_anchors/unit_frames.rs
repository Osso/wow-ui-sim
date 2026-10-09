use super::*;

#[test]
fn apply_system_anchors_replays_buffs_on_top_when_aura_update_exists() {
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
            systemIndex = 2,
            name = "TargetFrame",
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
            self.anchorApplied = true
        end

        function frame:UpdateAuras()
            self.aurasUpdated = true
        end

        function frame:UpdateSystemSetting(setting, entireSystemUpdate)
            table.insert(self.updatedSettings, tostring(setting) .. ":" .. tostring(entireSystemUpdate))
            if setting == Enum.EditModeUnitFrameSetting.BuffsOnTop then
                self:UpdateAuras()
            end
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = { frame },
        }

        function EditModeManagerFrame:InitSystemAnchors()
            self.initSystemAnchorsCalled = true
        end

        function EditModeManagerFrame:GetActiveLayoutSystemInfo()
            return {
                system = Enum.EditModeSystem.UnitFrame,
                systemIndex = 2,
                isInDefaultPosition = false,
                anchorInfo = {
                    point = "TOPLEFT",
                    relativeTo = UIParent,
                    relativePoint = "TOPLEFT",
                    offsetX = 10,
                    offsetY = -10,
                },
                settings = {
                    { setting = Enum.EditModeUnitFrameSetting.BuffsOnTop, value = 0 },
                    { setting = Enum.EditModeUnitFrameSetting.FrameSize, value = 0 },
                },
            }
        end

        function EditModeManagerFrame:UpdateSystem()
            error("target frame startup should not run the full update path")
        end
        "#,
    )
    .expect("install target frame stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply target frame anchors");

    let (updated_settings, auras_updated): (String, bool) = env
        .eval(
            r#"
            local frame = EditModeManagerFrame.registeredSystemFrames[1]
            return table.concat(frame.updatedSettings, ","),
                frame.aurasUpdated or false
            "#,
        )
        .expect("read target frame state");

    assert_eq!(
        updated_settings, "2:true,16:true",
        "BuffsOnTop and FrameSize should both replay for unit frames with UpdateAuras"
    );
    assert!(
        auras_updated,
        "BuffsOnTop replay should be able to refresh unit-frame auras"
    );
}

#[test]
fn apply_system_anchors_replays_active_widescreen_unit_frame_settings() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                UnitFrame = 3,
            },
            EditModeUnitFrameSetting = {
                CastBarUnderneath = 1,
                BuffsOnTop = 2,
                UseLargerFrame = 3,
                UseRaidStylePartyFrames = 4,
                ShowPartyFrameBackground = 5,
                UseHorizontalGroups = 6,
                ViewRaidSize = 9,
                FrameWidth = 10,
                FrameHeight = 11,
                DisplayBorder = 12,
                RaidGroupDisplayType = 13,
                SortPlayersBy = 14,
                RowSize = 15,
                FrameSize = 16,
                ViewArenaSize = 17,
            },
        }

        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }

        local setting = Enum.EditModeUnitFrameSetting
        local activeRows = {
            [1] = { setting.CastBarUnderneath, setting.FrameSize },
            [2] = { setting.BuffsOnTop, setting.FrameSize },
            [3] = { setting.BuffsOnTop, setting.UseLargerFrame, setting.FrameSize },
            [4] = {
                setting.UseRaidStylePartyFrames,
                setting.ShowPartyFrameBackground,
                setting.UseHorizontalGroups,
                setting.FrameWidth,
                setting.FrameHeight,
                setting.DisplayBorder,
                setting.SortPlayersBy,
                setting.FrameSize,
            },
            [5] = {
                setting.ViewRaidSize,
                setting.FrameWidth,
                setting.FrameHeight,
                setting.DisplayBorder,
                setting.RaidGroupDisplayType,
                setting.SortPlayersBy,
                setting.RowSize,
            },
            [7] = {
                setting.FrameWidth,
                setting.FrameHeight,
                setting.DisplayBorder,
                setting.ViewArenaSize,
            },
        }

        local function copiedSettings(systemIndex)
            local copied = {}
            for _, settingId in ipairs(activeRows[systemIndex] or {}) do
                table.insert(copied, { setting = settingId, value = 0 })
            end
            return copied
        end

        local function newFrame(name, systemIndex)
            local frame = {
                system = Enum.EditModeSystem.UnitFrame,
                systemIndex = systemIndex,
                name = name,
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
                self.anchorApplied = true
            end

            function frame:UpdateAuras()
                self.aurasUpdated = true
            end

            function frame:UpdateSystemSetting(settingId, entireSystemUpdate)
                table.insert(self.updatedSettings, tostring(settingId) .. ":" .. tostring(entireSystemUpdate))
            end

            return frame
        end

        EditModeManagerFrame = {
            layoutInfo = {},
            requestedIndices = {},
            registeredSystemFrames = {
                newFrame("PlayerFrame", 1),
                newFrame("TargetFrame", 2),
                newFrame("FocusFrame", 3),
                newFrame("PartyFrame", 4),
                newFrame("CompactRaidFrameContainer", 5),
                newFrame("CompactArenaFrame", 7),
            },
        }

        function EditModeManagerFrame:InitSystemAnchors()
            self.initSystemAnchorsCalled = true
        end

        function EditModeManagerFrame:GetActiveLayoutSystemInfo(system, systemIndex)
            table.insert(self.requestedIndices, systemIndex)
            return {
                system = system,
                systemIndex = systemIndex,
                isInDefaultPosition = false,
                anchorInfo = {
                    point = "CENTER",
                    relativeTo = UIParent,
                    relativePoint = "CENTER",
                    offsetX = 0,
                    offsetY = 0,
                },
                settings = copiedSettings(systemIndex),
            }
        end

        function EditModeManagerFrame:UpdateSystem()
            error("active unit frame shortcut rows should not run full startup UpdateSystem")
        end
        "#,
    )
    .expect("install active unit frame row stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply active unit frame row settings");

    let (requested_indices, replayed_settings): (String, String) = env
        .eval(
            r#"
            local replayed = {}
            for _, frame in ipairs(EditModeManagerFrame.registeredSystemFrames) do
                table.insert(replayed, table.concat(frame.updatedSettings, ","))
            end
            return table.concat(EditModeManagerFrame.requestedIndices, ","),
                table.concat(replayed, "|")
            "#,
        )
        .expect("read active unit frame replayed settings");

    assert_eq!(requested_indices, "1,2,3,4,5,7");
    assert_eq!(
        replayed_settings,
        "1:true,16:true|2:true,16:true|2:true,3:true,16:true|4:true,5:true,6:true,10:true,11:true,12:true,14:true,16:true|9:true,10:true,11:true,12:true,13:true,14:true,15:true|10:true,11:true,12:true,17:true",
        "every active Widescreen unit-frame setting on shortcut frames should replay"
    );
}

#[test]
fn apply_system_anchors_batches_compact_unit_frame_startup_refreshes() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                UnitFrame = 3,
            },
        }

        UIParent = { name = "UIParent" }
        EditModeUtil = {
            IsBottomAnchoredActionBar = function() return false end,
            IsRightAnchoredActionBar = function() return false end,
        }

        EditModeSystemMixin = {}
        function EditModeSystemMixin.UpdateSystemSetting(self, setting, entireSystemUpdate)
            table.insert(self.baseUpdates, tostring(setting) .. ":" .. tostring(entireSystemUpdate))
        end

        local function newFrame(name, systemIndex)
            local frame = {
                system = Enum.EditModeSystem.UnitFrame,
                systemIndex = systemIndex,
                name = name,
                baseUpdates = {},
                clearedSettings = {},
            }

            function frame:GetName()
                return self.name
            end

            function frame:SetHasActiveChanges(value)
                self.hasActiveChanges = value
            end

            function frame:UpdateSettingMap(updateDirtySettings)
                self.settingMapUpdated = updateDirtySettings
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

            function frame:ApplySystemAnchor()
                self.anchorApplied = true
            end

            function frame:ClearDirtySetting(setting)
                table.insert(self.clearedSettings, tostring(setting))
            end

            function frame:UpdateSystemSetting()
                error("compact unit frame startup should use batched refresh")
            end

            return frame
        end

        PartyFrame = newFrame("PartyFrame", 4)
        function PartyFrame:UpdatePartyMemberBackground()
            self.backgroundUpdates = (self.backgroundUpdates or 0) + 1
        end
        function PartyFrame:UpdatePaddingAndLayout()
            self.paddingUpdates = (self.paddingUpdates or 0) + 1
        end
        function PartyFrame:UpdateSelectionVerticalState()
            self.selectionUpdates = (self.selectionUpdates or 0) + 1
        end
        function PartyFrame:UpdateSystemSettingFrameSize()
            self.frameSizeUpdates = (self.frameSizeUpdates or 0) + 1
        end

        CompactRaidFrameContainer = newFrame("CompactRaidFrameContainer", 5)
        function CompactRaidFrameContainer:TryUpdate()
            self.tryUpdates = (self.tryUpdates or 0) + 1
        end
        function CompactRaidFrameContainer:Layout()
            self.layouts = (self.layouts or 0) + 1
        end
        function CompactRaidFrameContainer:ApplyMultipleToFrames()
            self.applyMultiple = (self.applyMultiple or 0) + 1
        end

        CompactArenaFrame = newFrame("CompactArenaFrame", 7)
        function CompactArenaFrame:RefreshMembers()
            self.refreshMembers = (self.refreshMembers or 0) + 1
        end

        CompactPartyFrame = {
            refreshMembers = 0,
            RefreshMembers = function(self)
                self.refreshMembers = self.refreshMembers + 1
            end,
        }
        compactRaidGroupBorderUpdates = 0
        function CompactRaidGroup_UpdateBorder()
            compactRaidGroupBorderUpdates = compactRaidGroupBorderUpdates + 1
        end
        raidAndPartyUpdates = 0
        function UpdateRaidAndPartyFrames()
            raidAndPartyUpdates = raidAndPartyUpdates + 1
        end
        function DefaultCompactMiniFrameSetup() end
        function CompactUnitFrame_UpdateAllFromEditMode() end

        EditModeManagerFrame = {
            layoutInfo = {},
            registeredSystemFrames = {
                PartyFrame,
                CompactRaidFrameContainer,
                CompactArenaFrame,
            },
            raidContainerFlowUpdates = 0,
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

        function EditModeManagerFrame:GetActiveLayoutSystemInfo(system, systemIndex)
            return {
                system = system,
                systemIndex = systemIndex,
                isInDefaultPosition = false,
                anchorInfo = {
                    point = "CENTER",
                    relativeTo = UIParent,
                    relativePoint = "CENTER",
                    offsetX = 0,
                    offsetY = 0,
                },
                settings = {
                    { setting = 10, value = 1 },
                    { setting = 11, value = 1 },
                    { setting = 12, value = 1 },
                },
            }
        end

        function EditModeManagerFrame:UpdateSystem()
            error("compact unit frame startup should not run full UpdateSystem")
        end

        function EditModeManagerFrame:UpdateRaidContainerFlow()
            self.raidContainerFlowUpdates = self.raidContainerFlowUpdates + 1
        end
        "#,
    )
    .expect("install compact unit frame batch stubs");

    env.exec(APPLY_SYSTEM_ANCHORS_LUA)
        .expect("apply compact unit frame batched settings");

    let summary: String = env
        .eval(
            r#"
            return table.concat({
                table.concat(PartyFrame.baseUpdates, ","),
                tostring(PartyFrame.backgroundUpdates),
                tostring(PartyFrame.paddingUpdates),
                tostring(PartyFrame.selectionUpdates),
                tostring(PartyFrame.frameSizeUpdates),
                table.concat(CompactRaidFrameContainer.baseUpdates, ","),
                tostring(CompactRaidFrameContainer.tryUpdates),
                tostring(CompactRaidFrameContainer.layouts),
                tostring(CompactRaidFrameContainer.applyMultiple),
                tostring(EditModeManagerFrame.raidContainerFlowUpdates),
                table.concat(CompactArenaFrame.baseUpdates, ","),
                tostring(CompactArenaFrame.refreshMembers),
                tostring(compactRaidGroupBorderUpdates),
                tostring(CompactPartyFrame.refreshMembers),
                tostring(raidAndPartyUpdates),
            }, "|")
            "#,
        )
        .expect("read compact unit frame batch counters");

    assert_eq!(
        summary,
        "10:true,11:true,12:true|1|2|1|1|10:true,11:true,12:true|1|1|2|1|10:true,11:true,12:true|1|1|1|1",
        "compact unit startup should mirror settings and run one batched refresh per subsystem"
    );
}
