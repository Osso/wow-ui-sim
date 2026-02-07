//! Enum constant data for WoW API.
//!
//! This file contains only DATA - static arrays of enum values.
//! The registration logic is in enum_api.rs.

/// Enum definition: (enum_name, &[(variant_name, value)])
pub type EnumDef = (&'static str, &'static [(&'static str, i32)]);

/// Sequential enum: (enum_name, &[variant_names]) - values are 0, 1, 2, ...
pub type SeqEnumDef = (&'static str, &'static [&'static str]);

// ============================================================================
// Role & Character Enums
// ============================================================================

pub const LFG_ROLE: EnumDef = ("LFGRole", &[("Tank", 0), ("Healer", 1), ("Damage", 2)]);

pub const UNIT_SEX: EnumDef = ("UnitSex", &[("Male", 2), ("Female", 3)]);

pub const GAME_MODE: SeqEnumDef = ("GameMode", &["Standard", "Plunderstorm", "WoWHack"]);

pub const PROFESSION: EnumDef = (
    "Profession",
    &[
        ("Mining", 1),
        ("Skinning", 2),
        ("Herbalism", 3),
        ("Blacksmithing", 4),
        ("Leatherworking", 5),
        ("Alchemy", 6),
        ("Tailoring", 7),
        ("Engineering", 8),
        ("Enchanting", 9),
        ("Fishing", 10),
        ("Cooking", 11),
        ("Jewelcrafting", 12),
        ("Inscription", 13),
        ("Archaeology", 14),
    ],
);

// ============================================================================
// Store & Transaction Enums
// ============================================================================

pub const VAS_TRANSACTION_PURCHASE_RESULT: SeqEnumDef = (
    "VasTransactionPurchaseResult",
    &[
        "Ok",
        "NotAvailable",
        "InProgress",
        "OnlyOneVasAtATime",
        "InvalidDestinationAccount",
        "InvalidSourceAccount",
        "InvalidCharacter",
        "NotEnoughMoney",
        "NotEligible",
        "TransferServiceDisabled",
        "DifferentRegion",
        "RealmNotEligible",
        "CharacterNotOnAccount",
        "TooManyCharacters",
        "InternalError",
        "PendingOtherProduct",
        "PendingItemDelivery",
        "PurchaseInProgress",
        "GenericError",
        "DisallowedSourceAccount",
        "DisallowedDestinationAccount",
        "LowerBoxLevel",
        "MaxCharactersOnServer",
        "CantAffordService",
        "ServiceAvailable",
        "CharacterHasGuildBank",
        "NameNotAvailable",
        "CharacterBelongsToGuild",
        "LockedForVas",
        "MoveInProgress",
        "AgeRestriction",
        "UnderMinAge",
        "BoostedTooRecently",
        "NewPlayerRestrictions",
        "CannotRestore",
        "GuildHasGuildBank",
        "CharacterArenaTeam",
        "CharacterTransferInProgress",
        "CharacterTransferPending",
        "RaceClassComboNotEligible",
        "InvalidStartingLevel",
        "ProxyBadRequestContained",
        "ProxyCharacterTransferredNoBoostInProgress",
        "DbRealmNotEligible",
        "DbCannotMoveGuildmaster",
        "DbMaxCharactersOnServer",
        "DbNoMixedAlliance",
        "DbDuplicateCharacterName",
        "DbHasMail",
        "DbMoveInProgress",
        "DbUnderMinLevelReq",
        "DbIneligibleTargetRealm",
        "DbTransferDateTooSoon",
        "DbCharLocked",
        "DbAllianceNotEligible",
        "DbTooMuchMoneyForLevel",
        "DbHasAuctions",
        "DbLastSaveTooRecent",
        "DbNameNotAvailable",
        "DbLastRenameTooRecent",
        "DbAlreadyRenameFlagged",
        "DbCustomizeAlreadyRequested",
        "DbLastCustomizeTooSoon",
        "DbFactionChangeTooSoon",
        "DbRaceClassComboIneligible",
        "DbPendingItemAudit",
        "DbGuildRankInsufficient",
        "DbCharacterWithoutGuild",
        "DbGmSenorityInsufficient",
        "DbAuthenticatorInsufficient",
        "DbIneligibleMapID",
        "DbBpayDeliveryPending",
        "DbHasBpayToken",
        "DbHasHeirloomItem",
        "DbResultAccountRestricted",
        "DbLastSaveTooDistant",
        "DbCagedPetInInventory",
        "DbOnBoostCooldown",
        "DbPvEPvPTransferNotAllowed",
        "DbNewLeaderInvalid",
        "DbNeedsLevelSquish",
        "DbHasNewPlayerExperienceRestriction",
        "DbHasCraftingOrders",
        "DbInvalidName",
        "DbNeedsEraChoice",
        "DbCannotMoveArenaCaptn",
    ],
);

pub const STORE_ERROR: SeqEnumDef = (
    "StoreError",
    &[
        "InvalidPaymentMethod",
        "PaymentFailed",
        "WrongCurrency",
        "BattlepayDisabled",
        "InsufficientBalance",
        "Other",
        "AlreadyOwned",
        "ParentalControlsNoPurchase",
        "PurchaseDenied",
        "ConsumableTokenOwned",
        "TooManyTokens",
        "ItemUnavailable",
        "ClientRestricted",
    ],
);

pub const GAME_RULE: SeqEnumDef = (
    "GameRule",
    &[
        "PlayerCastBarDisabled",
        "TargetCastBarDisabled",
        "NameplateCastBarDisabled",
        "UserAddonsDisabled",
        "EncounterJournalDisabled",
        "EjSuggestedContentDisabled",
        "EjDungeonsDisabled",
        "EjRaidsDisabled",
        "EjItemSetsDisabled",
        "ExperienceBarDisabled",
        "ActionButtonTypeOverlayStrategy",
        "MinimapDisabled",
        "WorldMapDisabled",
        "IngameMailNotificationDisabled",
        "IngameTrackingDisabled",
        "LandingPageFactionID",
    ],
);

// ============================================================================
// Animation Enums
// ============================================================================

pub const SCRIPTED_ANIMATION_BEHAVIOR: SeqEnumDef = (
    "ScriptedAnimationBehavior",
    &[
        "None",
        "TargetShake",
        "TargetKnockBack",
        "SourceRecoil",
        "SourceCollideWithTarget",
        "UIParentShake",
        "Bounce",
        "Jump",
        "Spiral",
        "JumpCurve",
        "RunAway",
        "SpellCastDirectional",
    ],
);

pub const SCRIPTED_ANIMATION_TRAJECTORY: SeqEnumDef = (
    "ScriptedAnimationTrajectory",
    &[
        "AtSource",
        "AtTarget",
        "Straight",
        "CurveLeft",
        "CurveRight",
        "CurveRandom",
        "HalfwayBetween",
    ],
);

// ============================================================================
// UI Widget Enums
// ============================================================================

pub const UI_WIDGET_VISUALIZATION_TYPE: SeqEnumDef = (
    "UIWidgetVisualizationType",
    &[
        "IconAndText",
        "CaptureBar",
        "StatusBar",
        "DoubleStatusBar",
        "IconTextAndBackground",
        "DoubleIconAndText",
        "StackedResourceTracker",
        "IconTextAndCurrencies",
        "TextWithState",
        "HorizontalCurrencies",
        "BulletTextList",
        "ScenarioHeaderCurrenciesAndBackground",
        "TextureAndText",
        "SpellDisplay",
        "DoubleStateIconRow",
        "TextureAndTextRow",
        "ZoneControl",
        "CaptureZone",
        "TextureWithAnimation",
        "DiscreteProgressSteps",
        "ScenarioHeaderTimer",
        "TextColumnRow",
        "Spacer",
        "UnitPowerBar",
        "FillUpFrames",
        "TextWithSubtext",
        "MapPinAnimation",
        "ItemDisplay",
        "TugOfWar",
        "ControlZone",
        "SpellWithChargesDisplay",
    ],
);

pub const UI_WIDGET_TOOLTIP_LOCATION: SeqEnumDef = (
    "UIWidgetTooltipLocation",
    &["Default", "BottomLeft", "Left", "TopLeft", "Top", "TopRight", "Right", "BottomRight", "Bottom"],
);

pub const UI_WIDGET_TEXT_SIZE_TYPE: EnumDef = (
    "UIWidgetTextSizeType",
    &[
        ("Small12Pt", 0), ("Medium16Pt", 1), ("Large24Pt", 2), ("Huge27Pt", 3),
        ("Standard14Pt", 4), ("Small10Pt", 5), ("Small11Pt", 6), ("Medium18Pt", 7),
        ("Large20Pt", 8),
    ],
);

pub const UI_WIDGET_FLAG: EnumDef = (
    "UIWidgetFlag",
    &[("UniversalWidget", 1), ("KeepCenteredHorizontallyWithShift", 2)],
);

pub const FLIGHT_PATH_FACTION: SeqEnumDef = (
    "FlightPathFaction",
    &[
        "Horde",
        "Alliance",
        "Neutral",
        "FriendlyToHordeAndAlliance",
        "RestedOnlyOnGround",
        "RequiresHordeQuestline",
        "RequiresAllianceQuestline",
        "TaxiNodeFlyable",
        "NotAccountCompletable",
        "RestedAllowMount",
    ],
);

pub const UI_WIDGET_SCALE: SeqEnumDef = (
    "UIWidgetScale",
    &[
        "OneHundred", "Ninty", "Eighty", "Seventy", "Sixty", "Fifty",
        "OneHundredTen", "OneHundredTwenty", "OneHundredThirty", "OneHundredForty",
        "OneHundredFifty", "OneHundredSixty", "OneHundredSeventy", "OneHundredEighty",
        "OneHundredNinety", "TwoHundred",
    ],
);

pub const WIDGET_REWARD_SHOWN_STATE: SeqEnumDef = (
    "UIWidgetRewardShownState",
    &["Hidden", "ShownEarned", "ShownUnearned"],
);

pub const WIDGET_ICON_SIZE_TYPE: SeqEnumDef = (
    "WidgetIconSizeType",
    &["Standard", "Small", "Medium", "Large"],
);

// ============================================================================
// Spell Display Enums
// ============================================================================

pub const SPELL_DISPLAY_BORDER_COLOR: SeqEnumDef = (
    "SpellDisplayBorderColor",
    &["None", "Black", "White", "Red", "Yellow", "Orange", "Purple", "Green", "Blue"],
);

pub const SPELL_DISPLAY_ICON_DISPLAY_TYPE: SeqEnumDef = (
    "SpellDisplayIconDisplayType",
    &["Buff", "Debuff", "Circular"],
);

pub const SPELL_DISPLAY_TEXT_SHOWN_STATE: SeqEnumDef = (
    "SpellDisplayTextShownStateType",
    &["Shown", "Hidden", "ShowOnMouseover"],
);

pub const SPELL_DISPLAY_TINT: SeqEnumDef = (
    "SpellDisplayTint",
    &[
        "None",
        "Red",
        "Yellow",
        "Green",
        "White",
        "Bronze",
        "Purple",
        "RedGrayscale",
        "YellowGrayscale",
        "GreenGrayscale",
    ],
);

// ============================================================================
// Status Bar Enums
// ============================================================================

pub const STATUS_BAR_COLOR_TINT_VALUE: SeqEnumDef = (
    "StatusBarColorTintValue",
    &["None", "Black", "White", "Red", "Yellow", "Orange", "Purple", "Green", "Blue"],
);

pub const STATUS_BAR_OVERRIDE_TEXT_SHOWN: SeqEnumDef = (
    "StatusBarOverrideBarTextShownType",
    &["Never", "Always", "OnlyOnMouseover", "OnlyNotOnMouseover"],
);

pub const STATUS_BAR_VALUE_TEXT_TYPE: SeqEnumDef = (
    "StatusBarValueTextType",
    &["Hidden", "Percentage", "Value", "Time", "TimeShowOneLevelOnly", "ValueOverMax", "ValueOverMaxNormalized"],
);

// ============================================================================
// Widget State Enums
// ============================================================================

pub const WIDGET_SHOWN_STATE: SeqEnumDef = (
    "WidgetShownState",
    &["Shown", "Hidden", "ShownOnMouseover", "ShownIfNotEmpty", "ShownIfEmpty"],
);

pub const WIDGET_ENABLED_STATE: SeqEnumDef = (
    "WidgetEnabledState",
    &["Disabled", "Enabled", "Red", "White", "Green", "Gold", "Artifact", "Black", "ColorTwo", "Yellow"],
);

pub const WIDGET_ANIMATION_TYPE: SeqEnumDef = ("WidgetAnimationType", &["None", "Fade"]);

pub const WIDGET_SHOW_GLOW_STATE: SeqEnumDef = (
    "WidgetShowGlowState",
    &["HideGlow", "ShowGlow"],
);

pub const WIDGET_GLOW_ANIM_TYPE: SeqEnumDef = (
    "WidgetGlowAnimType",
    &["None", "Pulse", "FullPulse"],
);

pub const ICON_AND_TEXT_WIDGET_STATE: SeqEnumDef = (
    "IconAndTextWidgetState",
    &["Hidden", "Shown", "ShownWithDynamicIconFlashing", "ShownWithDynamicIconNotFlashing"],
);

pub const ICON_STATE: SeqEnumDef = (
    "IconState",
    &["Hidden", "ShowState1", "ShowState2", "ShowState1Flashing", "ShowState2Flashing"],
);

// ============================================================================
// Zone Control Enums
// ============================================================================

pub const ZONE_CONTROL_STATE: SeqEnumDef = ("ZoneControlState", &["State1", "State2", "State3"]);

pub const ZONE_CONTROL_MODE: SeqEnumDef = (
    "ZoneControlMode",
    &["TwoSections", "ThreeSections", "FiveSections"],
);

pub const ZONE_CONTROL_ACTIVE_STATE: SeqEnumDef = (
    "ZoneControlActiveState",
    &["Inactive", "State1Active", "State2Active", "BothStatesActive"],
);

pub const ZONE_CONTROL_FILL_TYPE: SeqEnumDef = (
    "ZoneControlFillType",
    &["SingleFillClockwise", "SingleFillCounterClockwise", "DoubleFillClockwise", "DoubleFillCounterClockwise"],
);

pub const ZONE_CONTROL_DANGER_FLASH_TYPE: SeqEnumDef = (
    "ZoneControlDangerFlashType",
    &["ShowOnHazardousState", "AlwaysShow"],
);

pub const ZONE_CONTROL_LEADING_EDGE_TYPE: SeqEnumDef = (
    "ZoneControlLeadingEdgeType",
    &["None", "UseLeadingEdge"],
);

pub const CAPTURE_BAR_FILL_DIRECTION: SeqEnumDef = (
    "CaptureBarWidgetFillDirectionType",
    &["RightToLeft", "LeftToRight"],
);

// ============================================================================
// Misc Widget Enums
// ============================================================================

pub const UI_WIDGET_TEXTURE_TEXT_SIZE: SeqEnumDef = (
    "UIWidgetTextureAndTextSizeType",
    &["Small", "Medium", "Large", "Huge", "Standard", "Medium2"],
);

pub const MAP_PIN_ANIMATION_TYPE: SeqEnumDef = (
    "MapPinAnimationType",
    &["None", "Pulse"],
);

pub const TUG_OF_WAR_MARKER_ARROW: SeqEnumDef = (
    "TugOfWarMarkerArrowShownState",
    &["Hidden", "Shown", "ShownWithPulseAnim"],
);

pub const ICON_AND_TEXT_SHIFT_TYPE: SeqEnumDef = (
    "IconAndTextShiftTextType",
    &["None", "ShiftRight", "ShiftLeft"],
);

pub const ITEM_DISPLAY_TEXT_STYLE: SeqEnumDef = (
    "ItemDisplayTextDisplayStyle",
    &["Default", "ShowName"],
);

pub const WIDGET_ICON_SOURCE_TYPE: SeqEnumDef = (
    "WidgetIconSourceType",
    &["Default", "Spell"],
);

pub const WIDGET_TEXT_HORIZONTAL_ALIGNMENT: EnumDef = (
    "WidgetTextHorizontalAlignmentType",
    &[("Left", 0), ("Center", 1), ("Right", 2)],
);

pub const BAG_INDEX: EnumDef = (
    "BagIndex",
    &[("Backpack", 0), ("ReagentBag", 5)],
);

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
// Report & Moderation Enums
// ============================================================================

pub const SEND_REPORT_RESULT: SeqEnumDef = (
    "SendReportResult",
    &["Success", "GeneralError", "TooManyReports", "RequiresChatLine", "RequiresChatLineOrVoice", "RequiresScreenshot"],
);

// ============================================================================
// Edit Mode Enums
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
// Club / Communities Enums
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
        "FrameSize", "ViewArenaSize",
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
    ],
);

pub const EDIT_MODE_AURA_FRAME_SYSTEM_INDICES: EnumDef = (
    "EditModeAuraFrameSystemIndices",
    &[("BuffFrame", 1), ("DebuffFrame", 2)],
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
        "ShowTooltips",
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

// ============================================================================
// All sequential enums (for batch registration)
// ============================================================================

pub const SEQUENTIAL_ENUMS: &[SeqEnumDef] = &[
    GAME_MODE,
    VAS_TRANSACTION_PURCHASE_RESULT,
    STORE_ERROR,
    GAME_RULE,
    SCRIPTED_ANIMATION_BEHAVIOR,
    SCRIPTED_ANIMATION_TRAJECTORY,
    UI_WIDGET_VISUALIZATION_TYPE,
    UI_WIDGET_TOOLTIP_LOCATION,
    FLIGHT_PATH_FACTION,
    UI_WIDGET_SCALE,
    WIDGET_REWARD_SHOWN_STATE,
    WIDGET_ICON_SIZE_TYPE,
    SPELL_DISPLAY_BORDER_COLOR,
    SPELL_DISPLAY_ICON_DISPLAY_TYPE,
    SPELL_DISPLAY_TEXT_SHOWN_STATE,
    SPELL_DISPLAY_TINT,
    STATUS_BAR_COLOR_TINT_VALUE,
    STATUS_BAR_OVERRIDE_TEXT_SHOWN,
    STATUS_BAR_VALUE_TEXT_TYPE,
    WIDGET_SHOWN_STATE,
    WIDGET_ENABLED_STATE,
    WIDGET_ANIMATION_TYPE,
    WIDGET_SHOW_GLOW_STATE,
    WIDGET_GLOW_ANIM_TYPE,
    ICON_AND_TEXT_WIDGET_STATE,
    ICON_STATE,
    ZONE_CONTROL_STATE,
    ZONE_CONTROL_MODE,
    ZONE_CONTROL_ACTIVE_STATE,
    ZONE_CONTROL_FILL_TYPE,
    ZONE_CONTROL_DANGER_FLASH_TYPE,
    ZONE_CONTROL_LEADING_EDGE_TYPE,
    CAPTURE_BAR_FILL_DIRECTION,
    UI_WIDGET_TEXTURE_TEXT_SIZE,
    MAP_PIN_ANIMATION_TYPE,
    TUG_OF_WAR_MARKER_ARROW,
    ICON_AND_TEXT_SHIFT_TYPE,
    ITEM_DISPLAY_TEXT_STYLE,
    WIDGET_ICON_SOURCE_TYPE,
    WIDGET_UNIT_POWER_BAR_FLASH_MOMENT,
    UI_WIDGET_FONT_TYPE,
    UI_WIDGET_BLEND_MODE,
    UI_WIDGET_MOTION_TYPE,
    UI_WIDGET_UPDATE_ANIM_TYPE,
    UI_WIDGET_OVERRIDE_STATE,
    UI_WIDGET_TEXT_FORMAT_TYPE,
    UI_WIDGET_SPELL_COOLDOWN_TYPE,
    UI_WIDGET_BUTTON_ENABLED_STATE,
    UI_WIDGET_BUTTON_ICON_TYPE,
    UI_WIDGET_HORIZONTAL_DIRECTION,
    UI_WIDGET_LAYOUT_DIRECTION,
    UI_WIDGET_MODEL_SCENE_LAYER,
    SCREEN_LOCATION_TYPE,
    START_TIMER_TYPE,
    QUEST_SESSION_RESULT,
    REPUTATION_SORT_TYPE,
    QUEST_COMPLETE_SPELL_TYPE,
    BANK_LOCKED_REASON,
    QUEST_SESSION_COMMAND,
    PLAYER_INTERACTION_TYPE,
    EVENT_TOAST_DISPLAY_TYPE,
    SEND_REPORT_RESULT,
    EDIT_MODE_PRESET_LAYOUTS,
    BAGS_ORIENTATION,
    COLOR_OVERRIDE,
    CLUB_STREAM_TYPE,
    RECRUIT_A_FRIEND_REWARDS_VERSION,
    CUSTOM_BINDING_TYPE,
    CALENDAR_EVENT_TYPE,
    CAMERA_MODE_ASPECT_RATIO,
    CALENDAR_STATUS,
    HOUSING_ITEM_TOAST_TYPE,
    VAS_SERVICE_TYPE,
    ACTION_BAR_ORIENTATION,
    WIDGET_OPACITY_TYPE,
    EDIT_MODE_ACTION_BAR_SETTING,
    GARR_AUTO_MISSION_EVENT_TYPE,
    CLUB_MEMBER_PRESENCE,
    ACTION_BAR_VISIBLE_SETTING,
    EDIT_MODE_SYSTEM,
    EDIT_MODE_CHAT_FRAME_SETTING,
    EDIT_MODE_ACCOUNT_SETTING,
    EDIT_MODE_LAYOUT_TYPE,
    EDIT_MODE_UNIT_FRAME_SETTING,
    EDIT_MODE_CAST_BAR_SETTING,
    EDIT_MODE_MINIMAP_SETTING,
    EDIT_MODE_AURA_FRAME_SETTING,
    EDIT_MODE_BAGS_SETTING,
    EDIT_MODE_MICRO_MENU_SETTING,
    EDIT_MODE_OBJECTIVE_TRACKER_SETTING,
    EDIT_MODE_STATUS_TRACKING_BAR_SETTING,
    EDIT_MODE_DURABILITY_FRAME_SETTING,
    EDIT_MODE_TIMER_BARS_SETTING,
    EDIT_MODE_VEHICLE_SEAT_INDICATOR_SETTING,
    EDIT_MODE_ARCHAEOLOGY_BAR_SETTING,
    EDIT_MODE_COOLDOWN_VIEWER_SETTING,
    AURA_FRAME_ORIENTATION,
    MICRO_MENU_ORDER,
    MICRO_MENU_ORIENTATION,
    RAID_GROUP_DISPLAY_TYPE,
    SORT_PLAYERS_BY,
    VIEW_ARENA_SIZE,
    VIEW_RAID_SIZE,
    COOLDOWN_VIEWER_BAR_CONTENT,
    COOLDOWN_VIEWER_ICON_DIRECTION,
    COOLDOWN_VIEWER_ORIENTATION,
    COOLDOWN_VIEWER_VISIBLE_SETTING,
];

// All explicit value enums (for batch registration)
pub const EXPLICIT_ENUMS: &[EnumDef] = &[
    LFG_ROLE,
    UNIT_SEX,
    PROFESSION,
    UI_WIDGET_TEXT_SIZE_TYPE,
    UI_WIDGET_FLAG,
    WIDGET_TEXT_HORIZONTAL_ALIGNMENT,
    BAG_INDEX,
    EDIT_MODE_ACTION_BAR_SYSTEM_INDICES,
    MINIMAP_TRACKING_FILTER,
    BAG_SLOT_FLAGS,
    GARRISON_FOLLOWER_TYPE,
    GARRISON_TYPE,
    FRAME_TUTORIAL_ACCOUNT,
    TRANSMOG_COLLECTION_TYPE_META,
    EDIT_MODE_UNIT_FRAME_SYSTEM_INDICES,
    EDIT_MODE_AURA_FRAME_SYSTEM_INDICES,
    EDIT_MODE_STATUS_TRACKING_BAR_SYSTEM_INDICES,
    EDIT_MODE_COOLDOWN_VIEWER_SYSTEM_INDICES,
    AURA_FRAME_ICON_DIRECTION,
    AURA_FRAME_ICON_WRAP,
    BAGS_DIRECTION,
    CLUB_FINDER_REQUEST_TYPE,
    MAP_CANVAS_POSITION,
    VIGNETTE_OBJECTIVE_TYPE,
    HOUSING_PLOT_OWNER_TYPE,
];
