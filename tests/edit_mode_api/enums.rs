use super::*;

#[test]
fn compact_raid_group_type_enum_matches_edit_mode_unit_frame_indices() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let (party, raid, arena, edit_party, edit_raid, edit_arena): (i32, i32, i32, i32, i32, i32) =
        env.eval(
            r#"
            return CompactRaidGroupTypeEnum.Party,
                CompactRaidGroupTypeEnum.Raid,
                CompactRaidGroupTypeEnum.Arena,
                Enum.EditModeUnitFrameSystemIndices.Party,
                Enum.EditModeUnitFrameSystemIndices.Raid,
                Enum.EditModeUnitFrameSystemIndices.Arena
            "#,
        )
        .expect("read compact raid group type enum");

    assert_eq!(party, edit_party);
    assert_eq!(raid, edit_raid);
    assert_eq!(arena, edit_arena);
}

#[test]
fn unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let (big_defensive_icon_size, min_value, max_value, num_values): (i32, i32, i32, i32) = env
        .eval(
            r#"
            local setting = Enum.EditModeUnitFrameSetting
            local meta = Enum.EditModeUnitFrameSettingMeta
            return setting.BigDefensiveIconSize,
                meta.MinValue,
                meta.MaxValue,
                meta.NumValues
            "#,
        )
        .expect("read unit frame edit mode setting enum");

    assert_eq!(big_defensive_icon_size, 21);
    assert_eq!(min_value, 0);
    assert_eq!(max_value, 21);
    assert_eq!(num_values, 22);
}

#[test]
fn cooldown_viewer_edit_mode_setting_ids_match_blizzard_order() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let (
        orientation,
        icon_limit,
        icon_direction,
        icon_size,
        icon_padding,
        bar_width_scale,
        opacity,
        visible_setting,
        bar_content,
        hide_when_inactive,
        show_timer,
        show_tooltips,
    ): (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            local setting = Enum.EditModeCooldownViewerSetting
            return setting.Orientation,
                setting.IconLimit,
                setting.IconDirection,
                setting.IconSize,
                setting.IconPadding,
                setting.BarWidthScale,
                setting.Opacity,
                setting.VisibleSetting,
                setting.BarContent,
                setting.HideWhenInactive,
                setting.ShowTimer,
                setting.ShowTooltips
            "#,
        )
        .expect("read cooldown viewer edit mode setting enum");

    assert_eq!(orientation, 0);
    assert_eq!(icon_limit, 1);
    assert_eq!(icon_direction, 2);
    assert_eq!(icon_size, 3);
    assert_eq!(icon_padding, 4);
    assert_eq!(opacity, 5);
    assert_eq!(visible_setting, 6);
    assert_eq!(bar_content, 7);
    assert_eq!(hide_when_inactive, 8);
    assert_eq!(show_timer, 9);
    assert_eq!(show_tooltips, 10);
    assert_eq!(bar_width_scale, 11);
}

#[test]
fn status_tracking_bar_edit_mode_setting_ids_match_live_docs() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let (height, width, text_size, size, min_value, max_value, num_values): (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
    ) = env
        .eval(
            r#"
            local setting = Enum.EditModeStatusTrackingBarSetting
            local meta = Enum.EditModeStatusTrackingBarSettingMeta
            return setting.Height,
                setting.Width,
                setting.TextSize,
                setting.Size,
                meta.MinValue,
                meta.MaxValue,
                meta.NumValues
            "#,
        )
        .expect("read status tracking bar edit mode setting enum");

    assert_eq!(height, 0);
    assert_eq!(width, 1);
    assert_eq!(text_size, 2);
    assert_eq!(size, 3);
    assert_eq!(min_value, 0);
    assert_eq!(max_value, 3);
    assert_eq!(num_values, 4);
}

#[test]
fn edit_mode_account_setting_ids_include_totem_action_bar() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let (raid_warning, totem_action_bar, loss_of_control, min_value, max_value, num_values): (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
    ) = env
        .eval(
            r#"
            local setting = Enum.EditModeAccountSetting
            local meta = Enum.EditModeAccountSettingMeta
            return setting.ShowRaidWarning,
                setting.ShowTotemActionBar,
                setting.ShowLossOfControl,
                meta.MinValue,
                meta.MaxValue,
                meta.NumValues
            "#,
        )
        .expect("read edit mode account setting enum");

    assert_eq!(raid_warning, 33);
    assert_eq!(totem_action_bar, 34);
    assert_eq!(loss_of_control, 35);
    assert_eq!(min_value, 0);
    assert_eq!(max_value, 35);
    assert_eq!(num_values, 36);
}

#[test]
fn encounter_events_icon_direction_matches_blizzard_docs() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let (left, right, top, bottom, min_value, max_value, num_values): (
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
    ) = env
        .eval(
            r#"
            local direction = Enum.EncounterEventsIconDirection
            local meta = Enum.EncounterEventsIconDirectionMeta
            return direction.Left,
                direction.Right,
                direction.Top,
                direction.Bottom,
                meta.MinValue,
                meta.MaxValue,
                meta.NumValues
            "#,
        )
        .expect("read encounter events icon direction enum");

    assert_eq!(left, 0);
    assert_eq!(right, 1);
    assert_eq!(top, 0);
    assert_eq!(bottom, 1);
    assert_eq!(min_value, 0);
    assert_eq!(max_value, 1);
    assert_eq!(num_values, 4);
}

#[test]
fn edit_mode_profile_option_enums_match_blizzard_docs() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    env.exec(
        r#"
        local expected = {
            ActionBarOrientation = { Horizontal = 0, Vertical = 1 },
            ActionBarVisibleSetting = { Always = 0, InCombat = 1, OutOfCombat = 2, Hidden = 3 },
            AuraFrameIconDirection = { Down = 0, Up = 1, Left = 0, Right = 1 },
            AuraFrameIconWrap = { Down = 0, Up = 1, Left = 0, Right = 1 },
            AuraFrameOrientation = { Horizontal = 0, Vertical = 1 },
            AuraFrameVisibleSetting = { Always = 0, InCombat = 1, Hidden = 2 },
            BagsDirection = { Left = 0, Right = 1, Up = 0, Down = 1 },
            BagsOrientation = { Horizontal = 0, Vertical = 1 },
            CooldownViewerBarContent = { IconAndName = 0, IconOnly = 1, NameOnly = 2 },
            CooldownViewerIconDirection = { Left = 0, Right = 1 },
            CooldownViewerOrientation = { Horizontal = 0, Vertical = 1 },
            CooldownViewerVisibleSetting = { Always = 0, InCombat = 1, Hidden = 2 },
            DamageMeterNumbers = { Minimal = 0, Compact = 1, Complete = 2 },
            DamageMeterSourceDisplayType = { None = 0, Ally = 1, Enemy = 2 },
            DamageMeterStyle = { Default = 0, Thin = 1, Bordered = 2, FullBackground = 3 },
            DamageMeterVisibility = { Always = 0, InCombat = 1, Hidden = 2 },
            EditModeActionBarSystemIndices = {
                MainBar = 1, Bar2 = 2, Bar3 = 3, RightBar1 = 4, RightBar2 = 5,
                ExtraBar1 = 6, ExtraBar2 = 7, ExtraBar3 = 8,
                StanceBar = 11, PetActionBar = 12, PossessActionBar = 13,
            },
            EditModeAuraFrameSystemIndices = { BuffFrame = 1, DebuffFrame = 2, ExternalDefensivesFrame = 3 },
            EditModeCooldownViewerSystemIndices = { Essential = 1, Utility = 2, BuffIcon = 3, BuffBar = 4 },
            EditModeEncounterEventsSystemIndices = { Timeline = 1, CriticalWarnings = 2, MediumWarnings = 3, NormalWarnings = 4 },
            EditModeStatusTrackingBarSystemIndices = { StatusTrackingBar1 = 1, StatusTrackingBar2 = 2 },
            EditModeUnitFrameSystemIndices = { Player = 1, Target = 2, Focus = 3, Party = 4, Raid = 5, Boss = 6, Arena = 7, Pet = 8 },
            EncounterEventsIconDirection = { Left = 0, Right = 1, Top = 0, Bottom = 1 },
            EncounterEventsOrientation = { Horizontal = 0, Vertical = 1 },
            EncounterEventsTooltipAnchor = { Hidden = 0, Default = 1, Cursor = 2 },
            EncounterEventsViewType = { Timeline = 0, Bars = 1 },
            EncounterEventsVisibility = { Always = 0, InEncounter = 1, DeprecatedHidden = 2 },
            MicroMenuOrder = { Default = 0, Reverse = 1 },
            MicroMenuOrientation = { Horizontal = 0, Vertical = 1 },
            PersonalResourceDisplayVisibleSetting = { Always = 0, InCombat = 1, Hidden = 2 },
            RaidAuraOrganizationType = { Legacy = 0, BuffsTopDebuffsBottom = 1, BuffsRightDebuffsLeft = 2 },
            ViewArenaSize = { Two = 0, Three = 1 },
            ViewRaidSize = { Ten = 0, TwentyFive = 1, Forty = 2 },
            WidgetOpacityType = {
                OneHundred = 0, Ninety = 1, Eighty = 2, Seventy = 3, Sixty = 4,
                Fifty = 5, Forty = 6, Thirty = 7, Twenty = 8, Ten = 9, Zero = 10,
            },
        }

        local settingEnums = {
            EditModeAccountSetting = {
                "ShowGrid", "GridSpacing", "SettingsExpanded", "ShowTargetAndFocus",
                "ShowStanceBar", "ShowPetActionBar", "ShowPossessActionBar", "ShowCastBar",
                "ShowEncounterBar", "ShowExtraAbilities", "ShowBuffsAndDebuffs",
                "DeprecatedShowDebuffFrame", "ShowPartyFrames", "ShowRaidFrames",
                "ShowTalkingHeadFrame", "ShowVehicleLeaveButton", "ShowBossFrames",
                "ShowArenaFrames", "ShowLootFrame", "ShowHudTooltip", "ShowStatusTrackingBar2",
                "ShowDurabilityFrame", "EnableSnap", "EnableAdvancedOptions", "ShowPetFrame",
                "ShowTimerBars", "ShowVehicleSeatIndicator", "ShowArchaeologyBar",
                "ShowCooldownViewer", "ShowPersonalResourceDisplay", "ShowEncounterEvents",
                "ShowDamageMeter", "ShowExternalDefensives", "ShowRaidWarning",
                "ShowTotemActionBar", "ShowLossOfControl",
            },
            EditModeActionBarSetting = {
                "Orientation", "NumRows", "NumIcons", "IconSize", "IconPadding",
                "VisibleSetting", "HideBarArt", "DeprecatedSnapToSide",
                "HideBarScrolling", "AlwaysShowButtons",
            },
            EditModeArchaeologyBarSetting = { "Size" },
            EditModeAuraFrameSetting = {
                "Orientation", "IconWrap", "IconDirection", "IconLimitBuffFrame",
                "IconLimitDebuffFrame", "IconSize", "IconPadding", "DeprecatedShowFull",
                "VisibleSetting", "Opacity", "ShowDispelType",
            },
            EditModeBagsSetting = { "Orientation", "Direction", "Size", "BagSlotPadding" },
            EditModeCastBarSetting = { "BarSize", "LockToPlayerFrame", "ShowCastTime" },
            EditModeChatFrameSetting = { "WidthHundreds", "WidthTensAndOnes", "HeightHundreds", "HeightTensAndOnes" },
            EditModeCooldownViewerSetting = {
                "Orientation", "IconLimit", "IconDirection", "IconSize", "IconPadding",
                "Opacity", "VisibleSetting", "BarContent", "HideWhenInactive",
                "ShowTimer", "ShowTooltips", "BarWidthScale",
            },
            EditModeDamageMeterSetting = {
                "Visibility", "Style", "Numbers", "FrameWidth", "FrameHeight",
                "Padding", "Transparency", "ObsoleteReuse1", "ShowSpecIcon",
                "ShowClassColor", "BarHeight", "TextSize", "BackgroundTransparency",
            },
            EditModeDurabilityFrameSetting = { "Size" },
            EditModeEncounterEventsSetting = {
                "Orientation", "IconDirection", "ShowSpellName", "IconSize", "OverallSize",
                "BackgroundTransparency", "Transparency", "Visibility", "TooltipAnchor",
                "ShowTimer", "ViewType", "FlipHorizontally", "BarWidth", "Padding",
            },
            EditModeMicroMenuSetting = { "Orientation", "Order", "Size", "EyeSize" },
            EditModeMinimapSetting = { "HeaderUnderneath", "RotateMinimap", "Size" },
            EditModeObjectiveTrackerSetting = { "Height", "Opacity", "TextSize" },
            EditModePersonalResourceDisplaySetting = {
                "HideHealth", "DeprecatedOnlyShowInCombat", "HidePower", "HideClassInfo",
                "HealthBarHeight", "PowerBarHeight", "Padding", "Opacity", "VisibleSetting",
                "Size", "HideClassInfoOnPlayerFrame", "ShowClassColor", "BarWidth",
                "ShowBarText", "HideAltPower",
            },
            EditModeStatusTrackingBarSetting = { "Height", "Width", "TextSize", "Size" },
            EditModeTimerBarsSetting = { "Size" },
            EditModeUnitFrameSetting = {
                "HidePortrait", "CastBarUnderneath", "BuffsOnTop", "UseLargerFrame",
                "UseRaidStylePartyFrames", "ShowPartyFrameBackground", "UseHorizontalGroups",
                "CastBarOnSide", "ShowCastTime", "ViewRaidSize", "FrameWidth",
                "FrameHeight", "DisplayBorder", "RaidGroupDisplayType", "SortPlayersBy",
                "RowSize", "FrameSize", "ViewArenaSize", "AuraOrganizationType",
                "IconSize", "Opacity", "BigDefensiveIconSize",
            },
            EditModeVehicleSeatIndicatorSetting = { "Size" },
        }

        for enumName, values in pairs(settingEnums) do
            expected[enumName] = expected[enumName] or {}
            for index, fieldName in ipairs(values) do
                expected[enumName][fieldName] = index - 1
            end
        end

        for enumName, fields in pairs(expected) do
            local actual = Enum[enumName]
            if type(actual) ~= "table" then
                error(enumName .. " is not registered")
            end
            for fieldName, expectedValue in pairs(fields) do
                if actual[fieldName] ~= expectedValue then
                    error(string.format("%s.%s expected %s got %s", enumName, fieldName, tostring(expectedValue), tostring(actual[fieldName])))
                end
            end
        end
        "#,
    )
    .expect("profile option enums should match Blizzard generated docs");
}
