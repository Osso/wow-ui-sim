use super::*;

#[test]
fn setup_layout_info_merges_default_action_bar_settings_into_saved_layout() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.exec(
        r#"
        Enum = {
            EditModeSystem = {
                CastBar = 1,
                UnitFrame = 2,
                ActionBar = 3,
            },
            EditModeCastBarSetting = {
                LockToPlayerFrame = 101,
            },
            EditModeLayoutType = {
                Preset = 0,
                Account = 1,
            },
            EditModeUnitFrameSetting = {
                CastBarUnderneath = 201,
                UseRaidStylePartyFrames = 202,
            },
            EditModeUnitFrameSystemIndices = {
                Player = 301,
                Party = 302,
            },
            EditModeActionBarSetting = {
                HideBarArt = 6,
                AlwaysShowButtons = 9,
            },
        }

        C_EditMode = {
            GetLayouts = function()
                return {
                    activeLayout = 1,
                    layouts = {
                        {
                            layoutIndex = 77,
                            layoutName = "Saved Sparse",
                            layoutType = Enum.EditModeLayoutType.Account,
                            systems = {
                                {
                                    system = Enum.EditModeSystem.ActionBar,
                                    systemIndex = 1,
                                    isInDefaultPosition = false,
                                    anchorInfo = { point = "BOTTOM" },
                                    settings = {
                                        { setting = Enum.EditModeActionBarSetting.AlwaysShowButtons, value = 1 },
                                    },
                                },
                            },
                        },
                    },
                }
            end,
            GetAccountSettings = function()
                return {}
            end,
        }

        EditModePresetLayoutManager = {
            presetLayoutInfo = {},
        }

        function EditModePresetLayoutManager:GetAllDefaultSettingsForSystem(system, systemIndex)
            if system == Enum.EditModeSystem.ActionBar and systemIndex == 1 then
                return {
                    [Enum.EditModeActionBarSetting.HideBarArt] = 0,
                    [Enum.EditModeActionBarSetting.AlwaysShowButtons] = 0,
                }
            end
            return {}
        end

        function tAppendAll(tbl, addedArray)
            for i, element in ipairs(addedArray) do
                table.insert(tbl, element)
            end
        end

        EditModeManagerFrame = {}
        "#,
    )
    .expect("install sparse saved layout stubs");

    env.exec(SETUP_LAYOUT_INFO_LUA)
        .expect("run setup layout info");

    let (hide_bar_art, always_show_buttons): (i32, i32) = env
        .eval(
            r#"
            local settings = EditModeManagerFrame.layoutInfo.layouts[1].systems[1].settings
            local values = {}
            for _, settingInfo in ipairs(settings) do
                values[settingInfo.setting] = settingInfo.value
            end
            return values[Enum.EditModeActionBarSetting.HideBarArt],
                values[Enum.EditModeActionBarSetting.AlwaysShowButtons]
            "#,
        )
        .expect("read merged action bar settings");

    assert_eq!(hide_bar_art, 0, "default side-art setting should be merged");
    assert_eq!(
        always_show_buttons, 1,
        "saved values must override default settings"
    );
}

const ACCOUNT_SETTING_STUBS: &str = r#"
        Enum = {
            EditModeLayoutType = {
                Preset = 0,
                Account = 1,
            },
            EditModeSystem = {
                StatusTrackingBar = 15,
            },
            EditModeStatusTrackingBarSystemIndices = {
                StatusTrackingBar1 = 1,
                StatusTrackingBar2 = 2,
            },
            EditModeAccountSetting = {
                ShowGrid = 0,
                GridSpacing = 1,
                SettingsExpanded = 2,
                ShowTargetAndFocus = 3,
                ShowPartyFrames = 4,
                ShowRaidFrames = 5,
                ShowStanceBar = 6,
                ShowPetActionBar = 7,
                ShowPossessActionBar = 8,
                ShowCastBar = 9,
                ShowEncounterBar = 10,
                ShowExtraAbilities = 11,
                ShowBuffsAndDebuffs = 12,
                ShowExternalDefensives = 13,
                ShowTalkingHeadFrame = 14,
                ShowVehicleLeaveButton = 15,
                ShowBossFrames = 16,
                ShowArenaFrames = 17,
                ShowLootFrame = 18,
                ShowHudTooltip = 19,
                ShowStatusTrackingBar2 = 20,
                ShowDurabilityFrame = 21,
                EnableSnap = 22,
                EnableAdvancedOptions = 23,
                ShowPetFrame = 24,
                ShowTimerBars = 25,
                ShowVehicleSeatIndicator = 26,
                ShowArchaeologyBar = 27,
                ShowCooldownViewer = 28,
                ShowPersonalResourceDisplay = 29,
                ShowEncounterEvents = 31,
                ShowDamageMeter = 32,
                ShowTotemActionBar = 33,
            },
        }

        C_EditMode = {
            GetLayouts = function()
                return {
                    activeLayout = 1,
                    layouts = {
                        {
                            layoutIndex = 1,
                            layoutName = "Hidden XP",
                            layoutType = Enum.EditModeLayoutType.Account,
                            systems = {
                                {
                                    system = Enum.EditModeSystem.StatusTrackingBar,
                                    systemIndex = Enum.EditModeStatusTrackingBarSystemIndices.StatusTrackingBar1,
                                    hidden = true,
                                    isInDefaultPosition = false,
                                    anchorInfo = { point = "CENTER" },
                                    settings = {},
                                },
                            },
                        },
                    },
                }
            end,
            GetAccountSettings = function()
                return {
                    { setting = Enum.EditModeAccountSetting.ShowGrid, value = 1 },
                    { setting = Enum.EditModeAccountSetting.GridSpacing, value = 42 },
                    { setting = Enum.EditModeAccountSetting.SettingsExpanded, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowExternalDefensives, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowStatusTrackingBar2, value = 0 },
                    { setting = Enum.EditModeAccountSetting.EnableSnap, value = 0 },
                    { setting = Enum.EditModeAccountSetting.EnableAdvancedOptions, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowTimerBars, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowVehicleSeatIndicator, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowArchaeologyBar, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowCooldownViewer, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowPersonalResourceDisplay, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowEncounterEvents, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowDamageMeter, value = 0 },
                    { setting = Enum.EditModeAccountSetting.ShowTotemActionBar, value = 0 },
                }
            end,
        }

        EditModePresetLayoutManager = {
            presetLayoutInfo = {},
        }

        function tAppendAll(tbl, addedArray)
            for i, element in ipairs(addedArray) do
                table.insert(tbl, element)
            end
        end

        EditModeManagerFrame = {}
        function EditModeManagerFrame:InitializeAccountSettings()
            self.accountSettings = C_EditMode.GetAccountSettings()
            self.accountSettingsInitialized = true
            self.showGrid = self.accountSettings[1].value
            self.gridSpacing = self.accountSettings[2].value
        end
        function EditModeManagerFrame:SetGridShown(value)
            self.gridShown = value
        end
        function EditModeManagerFrame:SetGridSpacing(value)
            self.gridSpacingSetter = value
        end
        function EditModeManagerFrame:SetEnableSnap(value)
            self.snapEnabled = value
        end
        function EditModeManagerFrame:SetEnableAdvancedOptions(value)
            self.advancedOptionsEnabled = value
        end

        EditModeManagerFrame.AccountSettings = {
            timerBarsShown = true,
            vehicleSeatIndicatorShown = true,
            archaeologyBarShown = true,
            cooldownViewerShown = true,
            personalResourceDisplayShown = true,
            encounterEventsShown = true,
            damageMeterShown = true,
            externalDefensivesShown = true,
            statusTrackingBar2Shown = true,
            statusTrackingBar2Refreshed = false,
            totemActionBarShown = true,
            expanded = true,
        }
        function EditModeManagerFrame.AccountSettings:SetExpandedState(value)
            self.expanded = value
        end
        function EditModeManagerFrame.AccountSettings:SetExternalDefensivesShown(value)
            self.externalDefensivesShown = value
        end
        function EditModeManagerFrame.AccountSettings:SetStatusTrackingBar2Shown(value)
            self.statusTrackingBar2Shown = value
        end
        function EditModeManagerFrame.AccountSettings:RefreshStatusTrackingBar2()
            self.statusTrackingBar2Refreshed = true
        end
        function EditModeManagerFrame.AccountSettings:SetTimerBarsShown(value)
            self.timerBarsShown = value
        end
        function EditModeManagerFrame.AccountSettings:SetVehicleSeatIndicatorShown(value)
            self.vehicleSeatIndicatorShown = value
        end
        function EditModeManagerFrame.AccountSettings:SetArchaeologyBarShown(value)
            self.archaeologyBarShown = value
        end
        function EditModeManagerFrame.AccountSettings:SetCooldownViewerShown(value)
            self.cooldownViewerShown = value
        end
        function EditModeManagerFrame.AccountSettings:SetPersonalResourceDisplayShown(value)
            self.personalResourceDisplayShown = value
        end
        function EditModeManagerFrame.AccountSettings:SetEncounterEventsShown(value)
            self.encounterEventsShown = value
        end
        function EditModeManagerFrame.AccountSettings:SetDamageMeterShown(value)
            self.damageMeterShown = value
        end
        function EditModeManagerFrame.AccountSettings:SetTotemActionBarShown(value)
            self.totemActionBarShown = value
        end

        StatusTrackingBarInfo = {
            BarsEnum = {
                None = -1,
            },
        }
        MainStatusTrackingBarContainer = {
            hidden = false,
            shownBar = 4,
        }
        function MainStatusTrackingBarContainer:SetShownBar(barIndex)
            self.shownBar = barIndex
        end
        function MainStatusTrackingBarContainer:Hide()
            self.hidden = true
        end
        SecondaryStatusTrackingBarContainer = {
            hidden = false,
            shownBar = 1,
        }
        function SecondaryStatusTrackingBarContainer:SetShownBar(barIndex)
            self.shownBar = barIndex
        end
        function SecondaryStatusTrackingBarContainer:Hide()
            self.hidden = true
        end
        StatusTrackingBarManager = {
            barContainers = {
                MainStatusTrackingBarContainer,
                SecondaryStatusTrackingBarContainer,
            },
            updated = false,
        }
        function StatusTrackingBarManager:UpdateBarsShown()
            self.updated = true
        end
        "#;

fn install_account_setting_stubs(env: &WowLuaEnv) {
    env.exec(ACCOUNT_SETTING_STUBS)
        .expect("install account setting stubs");
}

#[test]
fn setup_layout_info_initializes_account_settings_from_saved_cache() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    install_account_setting_stubs(&env);

    env.exec(SETUP_LAYOUT_INFO_LUA)
        .expect("setup layout info should initialize account settings");

    let (
        initialized,
        show_grid,
        grid_spacing,
        grid_shown,
        grid_spacing_setter,
        snap_enabled,
        advanced_options_enabled,
        settings_expanded,
        external_defensives_shown,
        status_tracking_bar2_shown,
        status_tracking_bar2_refreshed,
        timer_bars_shown,
        vehicle_seat_indicator_shown,
        archaeology_bar_shown,
        cooldown_viewer_shown,
        personal_resource_display_shown,
        encounter_events_shown,
        damage_meter_shown,
        totem_action_bar_shown,
    ): (
        bool,
        i32,
        i32,
        bool,
        i32,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            return EditModeManagerFrame.accountSettingsInitialized,
                EditModeManagerFrame.showGrid,
                EditModeManagerFrame.gridSpacing,
                EditModeManagerFrame.gridShown,
                EditModeManagerFrame.gridSpacingSetter,
                EditModeManagerFrame.snapEnabled,
                EditModeManagerFrame.advancedOptionsEnabled,
                EditModeManagerFrame.AccountSettings.expanded,
                EditModeManagerFrame.AccountSettings.externalDefensivesShown,
                EditModeManagerFrame.AccountSettings.statusTrackingBar2Shown,
                EditModeManagerFrame.AccountSettings.statusTrackingBar2Refreshed,
                EditModeManagerFrame.AccountSettings.timerBarsShown,
                EditModeManagerFrame.AccountSettings.vehicleSeatIndicatorShown,
                EditModeManagerFrame.AccountSettings.archaeologyBarShown,
                EditModeManagerFrame.AccountSettings.cooldownViewerShown,
                EditModeManagerFrame.AccountSettings.personalResourceDisplayShown,
                EditModeManagerFrame.AccountSettings.encounterEventsShown,
                EditModeManagerFrame.AccountSettings.damageMeterShown,
                EditModeManagerFrame.AccountSettings.totemActionBarShown
            "#,
        )
        .expect("read account setting state");

    assert!(
        initialized,
        "saved account settings should be applied through Blizzard's initializer"
    );
    assert_eq!(show_grid, 1);
    assert_eq!(grid_spacing, 42);
    assert!(grid_shown);
    assert_eq!(grid_spacing_setter, 42);
    assert!(!snap_enabled);
    assert!(!advanced_options_enabled);
    assert!(!settings_expanded);
    assert!(!external_defensives_shown);
    assert!(
        !status_tracking_bar2_shown,
        "saved status tracking bar 2 visibility should be applied during account initialization"
    );
    assert!(
        !status_tracking_bar2_refreshed,
        "RefreshStatusTrackingBar2 is edit-mode-only (it highlights the container); startup must not call it"
    );

    let (
        status_tracking_container_count,
        status_tracking_main_hidden,
        status_tracking_main_bar,
        status_tracking_secondary_hidden,
        status_tracking_secondary_bar,
        status_tracking_manager_updated,
    ): (i32, bool, i32, bool, i32, bool) = env
        .eval(
            r#"
            return #StatusTrackingBarManager.barContainers,
                MainStatusTrackingBarContainer.hidden,
                MainStatusTrackingBarContainer.shownBar,
                SecondaryStatusTrackingBarContainer.hidden,
                SecondaryStatusTrackingBarContainer.shownBar,
                StatusTrackingBarManager.updated
            "#,
        )
        .expect("read status tracking manager profile state");

    assert_eq!(
        status_tracking_container_count, 0,
        "profile-hidden status tracking bars should be removed from manager capacity"
    );
    assert!(
        status_tracking_main_hidden,
        "profile-hidden status tracking bar 1 should hide the main container"
    );
    assert_eq!(
        status_tracking_main_bar, -1,
        "profile-hidden status tracking bar 1 should clear the main shown bar"
    );
    assert!(
        status_tracking_secondary_hidden,
        "disabled status tracking bar 2 should hide the secondary container"
    );
    assert_eq!(
        status_tracking_secondary_bar, -1,
        "disabled status tracking bar 2 should clear the secondary shown bar"
    );
    assert!(
        status_tracking_manager_updated,
        "status tracking manager should recompute visible bars after profile visibility changes"
    );
    assert!(
        !timer_bars_shown,
        "saved timer bars visibility should be applied during account initialization"
    );
    assert!(
        !vehicle_seat_indicator_shown,
        "saved vehicle seat visibility should be applied during account initialization"
    );
    assert!(
        !archaeology_bar_shown,
        "saved archaeology bar visibility should be applied during account initialization"
    );
    assert!(
        !cooldown_viewer_shown,
        "saved cooldown viewer visibility should be applied during account initialization"
    );
    assert!(
        !personal_resource_display_shown,
        "saved personal resource display visibility should be applied during account initialization"
    );
    assert!(
        !encounter_events_shown,
        "saved encounter events visibility should be applied during account initialization"
    );
    assert!(
        !damage_meter_shown,
        "saved damage meter visibility should be applied during account initialization"
    );
    assert!(
        !totem_action_bar_shown,
        "saved totem action bar visibility should be applied during account initialization"
    );
}
