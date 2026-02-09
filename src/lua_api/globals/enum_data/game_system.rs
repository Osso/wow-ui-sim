//! Game system, report, settings, garrison, club, and tutorial enum data.

use super::{EnumDef, SeqEnumDef};

// ============================================================================
// Game System Enums
// ============================================================================

pub const SCREEN_LOCATION_TYPE: SeqEnumDef = (
    "ScreenLocationType",
    &[
        "Center",
        "Left",
        "Right",
        "Top",
        "Bottom",
        "TopLeft",
        "TopRight",
        "LeftOutside",
        "RightOutside",
        "LeftRight",
        "TopBottom",
        "LeftRightOutside",
    ],
);

pub const START_TIMER_TYPE: SeqEnumDef = (
    "StartTimerType",
    &["PvPBeginTimer", "ChallengeModeCountdown", "PlayerCountdown", "PlunderstormCountdown"],
);

pub const QUEST_SESSION_RESULT: SeqEnumDef = (
    "QuestSessionResult",
    &[
        "Ok", "NotInParty", "InvalidOwner", "AlreadyActive", "NotActive", "InRaid",
        "OwnerRefused", "Timeout", "Disabled", "Started", "Stopped", "Joined", "Left",
        "OwnerLeft", "ReadyCheckFailed", "PartyDestroyed", "MemberTimeout", "AlreadyMember",
        "NotOwner", "AlreadyOwner", "AlreadyJoined", "NotMember", "Busy", "JoinRejected",
        "Logout", "Empty", "QuestNotCompleted", "Resync", "Restricted", "InPetBattle",
        "InvalidPublicParty", "Unknown", "InCombat", "MemberInCombat", "RestrictedCrossFaction",
    ],
);

pub const REPUTATION_SORT_TYPE: SeqEnumDef = (
    "ReputationSortType",
    &["None", "Account", "Character"],
);

pub const QUEST_COMPLETE_SPELL_TYPE: SeqEnumDef = (
    "QuestCompleteSpellType",
    &[
        "LegacyBehavior", "Follower", "Tradeskill", "Ability", "Aura", "Spell", "Unlock",
        "Companion", "QuestlineUnlock", "QuestlineReward", "QuestlineUnlockPart", "PossibleReward",
    ],
);

pub const BANK_LOCKED_REASON: SeqEnumDef = (
    "BankLockedReason",
    &["None", "NoAccountInventoryLock", "BankDisabled", "BankConversionFailed"],
);

pub const QUEST_SESSION_COMMAND: SeqEnumDef = (
    "QuestSessionCommand",
    &["None", "Start", "Stop", "SessionActiveNoCommand"],
);

pub const PLAYER_INTERACTION_TYPE: SeqEnumDef = (
    "PlayerInteractionType",
    &[
        "None", "TradePartner", "Item", "Gossip", "QuestGiver", "Merchant", "TaxiNode",
        "Trainer", "Banker", "AlliedRaceDetailsGiver", "GuildBanker", "Registrar", "Vendor",
        "PetitionVendor", "GuildTabardVendor", "TalentMaster", "SpecializationMaster",
        "MailInfo", "SpiritHealer", "AreaSpiritHealer", "Binder", "Auctioneer", "StableMaster",
        "BattleMaster", "Transmogrifier", "LFGDungeon", "VoidStorageBanker",
        "BlackMarketAuctioneer", "AdventureMap", "WorldMap", "GarrArchitect", "GarrTradeskill",
        "GarrMission", "ShipmentCrafter", "GarrRecruitment", "GarrTalent", "Trophy",
        "PlayerChoice", "ArtifactForge", "ObliterumForge", "ScrappingMachine",
        "ContributionCollector", "AzeriteRespec", "IslandQueue", "ItemInteraction",
        "ChromieTime", "CovenantPreview", "AnimaDiversion", "LegendaryCrafting",
        "WeeklyRewards", "Soulbind", "CovenantSanctum", "NewPlayerGuide", "ItemUpgrade",
        "AdventureJournal", "Renown", "AzeriteForge", "PerksProgramVendor",
        "ProfessionsCraftingOrder", "Professions", "ProfessionsCustomerOrder", "TraitSystem",
        "BarbersChoice", "JailersTowerBuffs", "MajorFactionRenown", "PersonalTabardVendor",
        "ForgeMaster", "CharacterBanker", "AccountBanker", "ProfessionRespec",
        "CornerstoneInteraction", "RenameNeighborhood", "HousingBulletinBoard",
        "HousingPedestal", "CreateGuildNeighborhood", "NeighborhoodCharter", "GuildRename",
        "OpenNeighborhoodCharterConfirmation", "OpenHouseFinder", "PlaceholderType79",
    ],
);

pub const EVENT_TOAST_DISPLAY_TYPE: SeqEnumDef = (
    "EventToastDisplayType",
    &[
        "NormalSingleLine", "NormalBlockText", "NormalTitleAndSubTitle", "NormalTextWithIcon",
        "LargeTextWithIcon", "NormalTextWithIconAndRarity", "Scenario", "ChallengeMode",
        "ScenarioClickExpand", "WeeklyRewardUnlock", "WeeklyRewardUpgrade",
        "FlightpointDiscovered", "CapstoneUnlocked", "SingleLineWithIcon", "Scoreboard",
        "HouseUpgradeAvailable",
    ],
);

pub const WIDGET_UNIT_POWER_BAR_FLASH_MOMENT: SeqEnumDef = (
    "WidgetUnitPowerBarFlashMomentType",
    &["FlashWhenMax", "FlashWhenMin", "FlashWhenEmpty", "FlashWhenFull"],
);

pub const UI_WIDGET_FONT_TYPE: SeqEnumDef = (
    "UIWidgetFontType",
    &["Normal", "Shadow", "Outline"],
);

pub const UI_WIDGET_BLEND_MODE: SeqEnumDef = (
    "UIWidgetBlendModeType",
    &["Default", "Additive"],
);

pub const UI_WIDGET_MOTION_TYPE: SeqEnumDef = (
    "UIWidgetMotionType",
    &["Instant", "Animated"],
);

pub const UI_WIDGET_UPDATE_ANIM_TYPE: SeqEnumDef = (
    "UIWidgetUpdateAnimType",
    &["None", "Flash"],
);

pub const UI_WIDGET_OVERRIDE_STATE: SeqEnumDef = (
    "UIWidgetOverrideState",
    &["NoOverride", "OverrideToShown", "OverrideToHidden"],
);

pub const UI_WIDGET_TEXT_FORMAT_TYPE: SeqEnumDef = (
    "UIWidgetTextFormatType",
    &["Default", "TooltipTitle", "TooltipBodyText"],
);

pub const UI_WIDGET_SPELL_COOLDOWN_TYPE: SeqEnumDef = (
    "UIWidgetSpellButtonCooldownType",
    &["None", "EdgeCooldown"],
);

pub const UI_WIDGET_BUTTON_ENABLED_STATE: SeqEnumDef = (
    "UIWidgetButtonEnabledState",
    &["Disabled", "Enabled", "Yellow", "RedDisabled", "GoldDisabled", "Gold", "Red"],
);

pub const UI_WIDGET_BUTTON_ICON_TYPE: SeqEnumDef = (
    "UIWidgetButtonIconType",
    &["Exit", "Speak", "Undo", "Checkmark", "RedX"],
);

pub const UI_WIDGET_HORIZONTAL_DIRECTION: SeqEnumDef = (
    "UIWidgetHorizontalDirection",
    &["Default", "LeftToRight", "RightToLeft"],
);

pub const UI_WIDGET_LAYOUT_DIRECTION: SeqEnumDef = (
    "UIWidgetLayoutDirection",
    &["Default", "Vertical", "Horizontal", "Overlap"],
);

pub const UI_WIDGET_MODEL_SCENE_LAYER: SeqEnumDef = (
    "UIWidgetModelSceneLayer",
    &["None", "Front", "Back"],
);

// ============================================================================
// Content Tracking Enums
// ============================================================================

pub const CONTENT_TRACKING_TYPE: SeqEnumDef = (
    "ContentTrackingType",
    &["Appearance", "Mount", "Achievement", "Decor"],
);

// ============================================================================
// Report & Moderation Enums
// ============================================================================

pub const SEND_REPORT_RESULT: SeqEnumDef = (
    "SendReportResult",
    &["Success", "GeneralError", "TooManyReports", "RequiresChatLine", "RequiresChatLineOrVoice", "RequiresScreenshot"],
);

// ============================================================================
// Edit Mode Enums (basic)
// ============================================================================

pub const EDIT_MODE_ACTION_BAR_SYSTEM_INDICES: EnumDef = (
    "EditModeActionBarSystemIndices",
    &[
        ("MainBar", 1), ("Bar2", 2), ("Bar3", 3), ("RightBar1", 4), ("RightBar2", 5),
        ("ExtraBar1", 6), ("ExtraBar2", 7), ("ExtraBar3", 8),
        ("StanceBar", 11), ("PetActionBar", 12), ("PossessActionBar", 13),
    ],
);

pub const EDIT_MODE_PRESET_LAYOUTS: SeqEnumDef = (
    "EditModePresetLayouts",
    &["Modern", "Classic"],
);

pub const BAGS_ORIENTATION: SeqEnumDef = (
    "BagsOrientation",
    &["Horizontal", "Vertical"],
);

// ============================================================================
// Settings & UI Enums
// ============================================================================

pub const COLOR_OVERRIDE: SeqEnumDef = (
    "ColorOverride",
    &[
        "ItemQualityPoor", "ItemQualityCommon", "ItemQualityUncommon", "ItemQualityRare",
        "ItemQualityEpic", "ItemQualityLegendary", "ItemQualityArtifact", "ItemQualityAccount",
    ],
);

pub const CLUB_STREAM_TYPE: SeqEnumDef = (
    "ClubStreamType",
    &["General", "Guild", "Officer", "Other"],
);

pub const RECRUIT_A_FRIEND_REWARDS_VERSION: SeqEnumDef = (
    "RecruitAFriendRewardsVersion",
    &["InvalidVersion", "UnusedVersionOne", "VersionTwo", "VersionThree"],
);

pub const MINIMAP_TRACKING_FILTER: EnumDef = (
    "MinimapTrackingFilter",
    &[
        ("Unfiltered", 0), ("Auctioneer", 1), ("Banker", 2), ("Battlemaster", 4),
        ("TaxiNode", 8), ("VenderFood", 16), ("Innkeeper", 32), ("Mailbox", 64),
        ("TrainerProfession", 128), ("VendorReagent", 256), ("Repair", 512),
        ("TrivialQuests", 1024), ("Stablemaster", 2048), ("Transmogrifier", 4096),
        ("POI", 8192), ("Target", 16384), ("Focus", 32768), ("QuestPOIs", 65536),
        ("Digsites", 131072), ("Barber", 262144), ("ItemUpgrade", 524288),
        ("VendorPoison", 1048576), ("AccountCompletedQuests", 2097152), ("AccountBanker", 4194304),
    ],
);

pub const CUSTOM_BINDING_TYPE: SeqEnumDef = (
    "CustomBindingType",
    &["VoicePushToTalk"],
);

pub const CALENDAR_EVENT_TYPE: SeqEnumDef = (
    "CalendarEventType",
    &["Raid", "Dungeon", "PvP", "Meeting", "Other", "HeroicDeprecated"],
);

pub const CAMERA_MODE_ASPECT_RATIO: SeqEnumDef = (
    "CameraModeAspectRatio",
    &["Default", "LegacyLetterbox", "HighDefinition_16_X_9", "Cinemascope_2_Dot_4_X_1"],
);

pub const BAG_SLOT_FLAGS: EnumDef = (
    "BagSlotFlags",
    &[
        ("DisableAutoSort", 1), ("ClassEquipment", 2), ("ClassConsumables", 4),
        ("ClassProfessionGoods", 8), ("ClassJunk", 16), ("ClassQuestItems", 32),
        ("ExcludeJunkSell", 64), ("ClassReagents", 128),
        ("ExpansionCurrent", 256), ("ExpansionLegacy", 512),
    ],
);

pub const GARRISON_FOLLOWER_TYPE: EnumDef = (
    "GarrisonFollowerType",
    &[
        ("FollowerType_6_0_GarrisonFollower", 1),
        ("FollowerType_6_0_Boat", 2),
        ("FollowerType_7_0_GarrisonFollower", 4),
        ("FollowerType_8_0_GarrisonFollower", 22),
        ("FollowerType_9_0_GarrisonFollower", 123),
    ],
);

pub const GARRISON_TYPE: EnumDef = (
    "GarrisonType",
    &[
        ("Type_6_0_Garrison", 2),
        ("Type_7_0_Garrison", 3),
        ("Type_8_0_Garrison", 9),
        ("Type_9_0_Garrison", 111),
    ],
);

pub const CALENDAR_STATUS: SeqEnumDef = (
    "CalendarStatus",
    &["Invited", "Available", "Declined", "Confirmed", "Out", "Standby", "Signedup", "NotSignedup", "Tentative"],
);

pub const HOUSING_ITEM_TOAST_TYPE: SeqEnumDef = (
    "HousingItemToastType",
    &["Room", "Fixture", "Customization", "Decor"],
);

// ============================================================================
// Store & Service Enums
// ============================================================================

pub const VAS_SERVICE_TYPE: SeqEnumDef = (
    "VasServiceType",
    &[
        "FactionChange", "RaceChange", "AppearanceChange", "NameChange",
        "CharacterTransfer", "GuildNameChange", "GuildFactionChange", "GuildTransfer",
    ],
);

pub const ACTION_BAR_ORIENTATION: SeqEnumDef = (
    "ActionBarOrientation",
    &["Horizontal", "Vertical"],
);

pub const WIDGET_OPACITY_TYPE: SeqEnumDef = (
    "WidgetOpacityType",
    &[
        "OneHundred", "Ninety", "Eighty", "Seventy", "Sixty", "Fifty",
        "Forty", "Thirty", "Twenty", "Ten", "Zero",
    ],
);

// ============================================================================
// Edit Mode Setting Enums
// ============================================================================

pub const EDIT_MODE_ACTION_BAR_SETTING: SeqEnumDef = (
    "EditModeActionBarSetting",
    &[
        "Orientation", "NumRows", "NumIcons", "IconSize", "IconPadding",
        "VisibleSetting", "HideBarArt", "DeprecatedSnapToSide", "HideBarScrolling", "AlwaysShowButtons",
    ],
);

// ============================================================================
// Garrison Enums
// ============================================================================

pub const GARR_AUTO_MISSION_EVENT_TYPE: SeqEnumDef = (
    "GarrAutoMissionEventType",
    &[
        "MeleeDamage", "RangeDamage", "SpellMeleeDamage", "SpellRangeDamage",
        "Heal", "PeriodicDamage", "PeriodicHeal", "ApplyAura", "RemoveAura", "Died",
    ],
);

// ============================================================================
// Club / Communities Enums (basic)
// ============================================================================

pub const CLUB_MEMBER_PRESENCE: SeqEnumDef = (
    "ClubMemberPresence",
    &["Unknown", "Online", "OnlineMobile", "Offline", "Away", "Busy"],
);

// ============================================================================
// Tutorial Enums
// ============================================================================

pub const FRAME_TUTORIAL_ACCOUNT: EnumDef = (
    "FrameTutorialAccount",
    &[
        ("HudRevampBagChanges", 1),
        ("PerksProgramActivitiesIntro", 2),
        ("EditModeManager", 3),
        ("TransmogSetsTab", 4),
        ("MountCollectionDragonriding", 5),
        ("LFGList", 6),
        ("HeirloomJournalLevel", 7),
        ("TimerunnersAdvantage", 8),
        ("AccountWideReputation", 9),
        ("TransferableCurrencies", 10),
        ("BindToAccountUntilEquip", 11),
        ("CompletedQuestsFilter", 12),
        ("CompletedQuestsFilterSeen", 13),
        ("ConcentrationCurrency", 14),
        ("MapLegendOpened", 15),
        ("NpcCraftingOrders", 16),
        ("NpcCraftingOrderCreateButton", 17),
        ("NpcCraftingOrderTabNew", 18),
        ("LocalStoriesFilterSeen", 19),
        ("EventSchedulerTabSeen", 20),
        ("AssistedCombatRotationDragSpell", 21),
        ("AssistedCombatRotationActionButton", 22),
        ("HousingDecorCleanup", 23),
        ("HousingDecorPlace", 24),
        ("HousingDecorClippingGrid", 25),
        ("HousingDecorCustomization", 26),
        ("HousingDecorLayout", 27),
        ("HousingHouseFinderMap", 28),
        ("HousingHouseFinderVisitHouse", 29),
        ("HousingItemAcquisition", 30),
        ("HousingNewPip", 31),
        ("PerksProgramActivitiesOpen", 32),
        ("EnconterJournalTutorialsTabSeen", 33),
        ("HousingMarketTab", 34),
        ("HousingTeleportButton", 35),
        ("RPETalentStarterBuild", 36),
        ("HousingInvalidCollision", 37),
        ("HousingModesUnlocked", 38),
        ("HousingExpertMode", 39),
        ("HousingCleanupMode", 40),
    ],
);

pub const ACCOUNT_STORE_CATEGORY_TYPE: EnumDef = (
    "AccountStoreCategoryType",
    &[
        ("Creature", 1),
        ("TransmogSet", 2),
        ("Mount", 3),
        ("Icon", 4),
    ],
);
