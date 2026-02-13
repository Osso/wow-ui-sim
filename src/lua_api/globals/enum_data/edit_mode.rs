//! Edit mode detail enums, transmog, and map enum data.

use super::{EnumDef, SeqEnumDef};

// ============================================================================
// Edit Mode Enums (from EditModeManagerConstantsDocumentation.lua)
// ============================================================================

pub const ACTION_BAR_VISIBLE_SETTING: SeqEnumDef = (
    "ActionBarVisibleSetting",
    &["Always", "InCombat", "OutOfCombat", "Hidden"],
);

pub const EDIT_MODE_SYSTEM: SeqEnumDef = (
    "EditModeSystem",
    &[
        "ActionBar", "CastBar", "Minimap", "UnitFrame", "EncounterBar",
        "ExtraAbilities", "AuraFrame", "TalkingHeadFrame", "ChatFrame",
        "VehicleLeaveButton", "LootFrame", "HudTooltip", "ObjectiveTracker",
        "MicroMenu", "Bags", "StatusTrackingBar", "DurabilityFrame",
        "TimerBars", "VehicleSeatIndicator", "ArchaeologyBar", "CooldownViewer",
        "PersonalResourceDisplay", "EncounterEvents", "DamageMeter",
    ],
);

pub const EDIT_MODE_CHAT_FRAME_SETTING: SeqEnumDef = (
    "EditModeChatFrameSetting",
    &["WidthHundreds", "WidthTensAndOnes", "HeightHundreds", "HeightTensAndOnes"],
);

pub const EDIT_MODE_ACCOUNT_SETTING: SeqEnumDef = (
    "EditModeAccountSetting",
    &[
        "ShowGrid", "GridSpacing", "SettingsExpanded", "ShowTargetAndFocus",
        "ShowStanceBar", "ShowPetActionBar", "ShowPossessActionBar", "ShowCastBar",
        "ShowEncounterBar", "ShowExtraAbilities", "ShowBuffsAndDebuffs",
        "DeprecatedShowDebuffFrame", "ShowPartyFrames", "ShowRaidFrames",
        "ShowTalkingHeadFrame", "ShowVehicleLeaveButton", "ShowBossFrames",
        "ShowArenaFrames", "ShowLootFrame", "ShowHudTooltip", "ShowStatusTrackingBar2",
        "ShowDurabilityFrame", "EnableSnap", "EnableAdvancedOptions", "ShowPetFrame",
        "ShowTimerBars", "ShowVehicleSeatIndicator", "ShowArchaeologyBar", "ShowCooldownViewer",
        "ShowPersonalResourceDisplay", "ShowEncounterEvents", "ShowDamageMeter",
        "ShowExternalDefensives",
    ],
);

pub const EDIT_MODE_LAYOUT_TYPE: SeqEnumDef = (
    "EditModeLayoutType",
    &["Preset", "Account", "Character", "Override"],
);

pub const EDIT_MODE_UNIT_FRAME_SETTING: SeqEnumDef = (
    "EditModeUnitFrameSetting",
    &[
        "HidePortrait", "CastBarUnderneath", "BuffsOnTop", "UseLargerFrame",
        "UseRaidStylePartyFrames", "ShowPartyFrameBackground", "UseHorizontalGroups",
        "CastBarOnSide", "ShowCastTime", "ViewRaidSize", "FrameWidth", "FrameHeight",
        "DisplayBorder", "RaidGroupDisplayType", "SortPlayersBy", "RowSize",
        "FrameSize", "ViewArenaSize", "AuraOrganizationType", "IconSize", "Opacity",
    ],
);

pub const EDIT_MODE_UNIT_FRAME_SYSTEM_INDICES: EnumDef = (
    "EditModeUnitFrameSystemIndices",
    &[
        ("Player", 1), ("Target", 2), ("Focus", 3), ("Party", 4),
        ("Raid", 5), ("Boss", 6), ("Arena", 7), ("Pet", 8),
    ],
);

pub const EDIT_MODE_CAST_BAR_SETTING: SeqEnumDef = (
    "EditModeCastBarSetting",
    &["BarSize", "LockToPlayerFrame", "ShowCastTime"],
);

pub const EDIT_MODE_MINIMAP_SETTING: SeqEnumDef = (
    "EditModeMinimapSetting",
    &["HeaderUnderneath", "RotateMinimap", "Size"],
);

pub const EDIT_MODE_AURA_FRAME_SETTING: SeqEnumDef = (
    "EditModeAuraFrameSetting",
    &[
        "Orientation", "IconWrap", "IconDirection", "IconLimitBuffFrame",
        "IconLimitDebuffFrame", "IconSize", "IconPadding", "DeprecatedShowFull",
        "VisibleSetting", "Opacity", "ShowDispelType",
    ],
);

pub const EDIT_MODE_AURA_FRAME_SYSTEM_INDICES: EnumDef = (
    "EditModeAuraFrameSystemIndices",
    &[("BuffFrame", 1), ("DebuffFrame", 2), ("ExternalDefensivesFrame", 3)],
);

pub const EDIT_MODE_BAGS_SETTING: SeqEnumDef = (
    "EditModeBagsSetting",
    &["Orientation", "Direction", "Size", "BagSlotPadding"],
);

pub const EDIT_MODE_MICRO_MENU_SETTING: SeqEnumDef = (
    "EditModeMicroMenuSetting",
    &["Orientation", "Order", "Size", "EyeSize"],
);

pub const EDIT_MODE_OBJECTIVE_TRACKER_SETTING: SeqEnumDef = (
    "EditModeObjectiveTrackerSetting",
    &["Height", "Opacity", "TextSize"],
);

pub const EDIT_MODE_STATUS_TRACKING_BAR_SETTING: SeqEnumDef = (
    "EditModeStatusTrackingBarSetting",
    &["Height", "Width", "TextSize"],
);

pub const EDIT_MODE_STATUS_TRACKING_BAR_SYSTEM_INDICES: EnumDef = (
    "EditModeStatusTrackingBarSystemIndices",
    &[("StatusTrackingBar1", 1), ("StatusTrackingBar2", 2)],
);

pub const EDIT_MODE_DURABILITY_FRAME_SETTING: SeqEnumDef = (
    "EditModeDurabilityFrameSetting",
    &["Size"],
);

pub const EDIT_MODE_TIMER_BARS_SETTING: SeqEnumDef = (
    "EditModeTimerBarsSetting",
    &["Size"],
);

pub const EDIT_MODE_VEHICLE_SEAT_INDICATOR_SETTING: SeqEnumDef = (
    "EditModeVehicleSeatIndicatorSetting",
    &["Size"],
);

pub const EDIT_MODE_ARCHAEOLOGY_BAR_SETTING: SeqEnumDef = (
    "EditModeArchaeologyBarSetting",
    &["Size"],
);

pub const EDIT_MODE_COOLDOWN_VIEWER_SETTING: SeqEnumDef = (
    "EditModeCooldownViewerSetting",
    &[
        "Orientation", "IconLimit", "IconDirection", "IconSize", "IconPadding",
        "Opacity", "VisibleSetting", "BarContent", "HideWhenInactive", "ShowTimer",
        "ShowTooltips", "BarWidthScale",
    ],
);

pub const EDIT_MODE_COOLDOWN_VIEWER_SYSTEM_INDICES: EnumDef = (
    "EditModeCooldownViewerSystemIndices",
    &[("Essential", 1), ("Utility", 2), ("BuffIcon", 3), ("BuffBar", 4)],
);

pub const AURA_FRAME_ICON_DIRECTION: EnumDef = (
    "AuraFrameIconDirection",
    &[("Down", 0), ("Up", 1), ("Left", 0), ("Right", 1)],
);

pub const AURA_FRAME_ICON_WRAP: EnumDef = (
    "AuraFrameIconWrap",
    &[("Down", 0), ("Up", 1), ("Left", 0), ("Right", 1)],
);

pub const AURA_FRAME_ORIENTATION: SeqEnumDef = (
    "AuraFrameOrientation",
    &["Horizontal", "Vertical"],
);

pub const BAGS_DIRECTION: EnumDef = (
    "BagsDirection",
    &[("Left", 0), ("Right", 1), ("Up", 0), ("Down", 1)],
);

pub const CLUB_FINDER_REQUEST_TYPE: EnumDef = (
    "ClubFinderRequestType",
    &[("None", 0), ("Guild", 1), ("Community", 2), ("All", 3)],
);

pub const MICRO_MENU_ORDER: SeqEnumDef = (
    "MicroMenuOrder",
    &["Default", "Reverse"],
);

pub const MICRO_MENU_ORIENTATION: SeqEnumDef = (
    "MicroMenuOrientation",
    &["Horizontal", "Vertical"],
);

pub const RAID_GROUP_DISPLAY_TYPE: SeqEnumDef = (
    "RaidGroupDisplayType",
    &["SeparateGroupsVertical", "SeparateGroupsHorizontal", "CombineGroupsVertical", "CombineGroupsHorizontal"],
);

pub const SORT_PLAYERS_BY: SeqEnumDef = (
    "SortPlayersBy",
    &["Role", "Group", "Alphabetical"],
);

pub const VIEW_ARENA_SIZE: SeqEnumDef = (
    "ViewArenaSize",
    &["Two", "Three"],
);

pub const VIEW_RAID_SIZE: SeqEnumDef = (
    "ViewRaidSize",
    &["Ten", "TwentyFive", "Forty"],
);

pub const COOLDOWN_VIEWER_BAR_CONTENT: SeqEnumDef = (
    "CooldownViewerBarContent",
    &["IconAndName", "IconOnly", "NameOnly"],
);

pub const COOLDOWN_VIEWER_ICON_DIRECTION: SeqEnumDef = (
    "CooldownViewerIconDirection",
    &["Left", "Right"],
);

pub const COOLDOWN_VIEWER_ORIENTATION: SeqEnumDef = (
    "CooldownViewerOrientation",
    &["Horizontal", "Vertical"],
);

pub const COOLDOWN_VIEWER_VISIBLE_SETTING: SeqEnumDef = (
    "CooldownViewerVisibleSetting",
    &["Always", "InCombat", "Hidden"],
);

pub const EDIT_MODE_PERSONAL_RESOURCE_DISPLAY_SETTING: SeqEnumDef = (
    "EditModePersonalResourceDisplaySetting",
    &["HideHealthAndPower", "OnlyShowInCombat"],
);

pub const EDIT_MODE_ENCOUNTER_EVENTS_SETTING: SeqEnumDef = (
    "EditModeEncounterEventsSetting",
    &[
        "Orientation", "IconDirection", "ShowSpellName", "IconSize", "OverallSize",
        "BackgroundTransparency", "Transparency", "Visibility", "TooltipAnchor",
        "ShowTimer", "ViewType", "FlipHorizontally", "BarWidth", "Padding",
    ],
);

pub const ENCOUNTER_EVENTS_VIEW_TYPE: SeqEnumDef = (
    "EncounterEventsViewType",
    &["Timeline", "Bars"],
);

pub const ENCOUNTER_EVENTS_ORIENTATION: SeqEnumDef = (
    "EncounterEventsOrientation",
    &["Horizontal", "Vertical"],
);

pub const ENCOUNTER_EVENTS_ICON_DIRECTION: SeqEnumDef = (
    "EncounterEventsIconDirection",
    &["Left", "Right", "Bottom", "Top"],
);

pub const ENCOUNTER_EVENTS_VISIBILITY: SeqEnumDef = (
    "EncounterEventsVisibility",
    &["Always", "InEncounter", "DeprecatedHidden"],
);

pub const ENCOUNTER_EVENTS_TOOLTIP_ANCHOR: SeqEnumDef = (
    "EncounterEventsTooltipAnchor",
    &["Hidden", "Default", "Cursor"],
);

pub const EDIT_MODE_DAMAGE_METER_SETTING: SeqEnumDef = (
    "EditModeDamageMeterSetting",
    &[
        "Visibility", "Style", "Numbers", "FrameWidth", "FrameHeight",
        "Padding", "Transparency", "ObsoleteReuse1", "ShowSpecIcon",
        "ShowClassColor", "BarHeight", "TextSize", "BackgroundTransparency",
    ],
);

pub const DAMAGE_METER_STYLE: SeqEnumDef = (
    "DamageMeterStyle",
    &["Default", "Thin", "Bordered", "FullBackground"],
);

pub const DAMAGE_METER_NUMBERS: SeqEnumDef = (
    "DamageMeterNumbers",
    &["Minimal", "Compact", "Complete"],
);

pub const DAMAGE_METER_VISIBILITY: SeqEnumDef = (
    "DamageMeterVisibility",
    &["Always", "InCombat", "Hidden"],
);

pub const RAID_AURA_ORGANIZATION_TYPE: SeqEnumDef = (
    "RaidAuraOrganizationType",
    &["Legacy", "BuffsTopDebuffsBottom", "BuffsRightDebuffsLeft"],
);

pub const AURA_FRAME_VISIBLE_SETTING: SeqEnumDef = (
    "AuraFrameVisibleSetting",
    &["Always", "InCombat", "Hidden"],
);

pub const EDIT_MODE_ENCOUNTER_EVENTS_SYSTEM_INDICES: EnumDef = (
    "EditModeEncounterEventsSystemIndices",
    &[
        ("Timeline", 1), ("CriticalWarnings", 2),
        ("MediumWarnings", 3), ("NormalWarnings", 4),
    ],
);

pub const EDIT_MODE_SETTING_DISPLAY_TYPE: SeqEnumDef = (
    "EditModeSettingDisplayType",
    &["Dropdown", "Checkbox", "Slider"],
);

// ============================================================================
// Transmog Meta Enums
// ============================================================================

pub const TRANSMOG_COLLECTION_TYPE_META: EnumDef = (
    "TransmogCollectionTypeMeta",
    &[("NumValues", 30)],
);

// ============================================================================
// Map / Vignette / Housing enums
// ============================================================================

pub const MAP_CANVAS_POSITION: EnumDef = (
    "MapCanvasPosition",
    &[
        ("None", 0),
        ("BottomLeft", 1),
        ("BottomRight", 2),
        ("TopLeft", 3),
        ("TopRight", 4),
    ],
);

pub const VIGNETTE_OBJECTIVE_TYPE: EnumDef = (
    "VignetteObjectiveType",
    &[
        ("None", 0),
        ("Defeat", 1),
        ("DefeatShowRemainingHealth", 2),
    ],
);

pub const HOUSING_PLOT_OWNER_TYPE: EnumDef = (
    "HousingPlotOwnerType",
    &[
        ("None", 0),
        ("Stranger", 1),
        ("Friend", 2),
        ("Self", 3),
    ],
);
