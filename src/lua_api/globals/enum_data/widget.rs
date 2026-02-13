//! UI Widget, Character, Store, and Animation enum data.

use super::{EnumDef, SeqEnumDef};

// ============================================================================
// Role & Character Enums
// ============================================================================

pub const LFG_ROLE: EnumDef = ("LFGRole", &[("Tank", 0), ("Healer", 1), ("Damage", 2)]);

pub const UNIT_SEX: EnumDef = ("UnitSex", &[("Male", 2), ("Female", 3)]);

pub const GAME_MODE: SeqEnumDef = ("GameMode", &["Standard", "Plunderstorm", "WoWHack"]);

pub const PARTY_PLAYLIST_ENTRY: EnumDef = (
    "PartyPlaylistEntry",
    &[("NormalGameMode", 0), ("TrainingGameMode", 1)],
);

pub const CHAT_CHANNEL_TYPE: EnumDef = (
    "ChatChannelType",
    &[("None", 0), ("Custom", 1), ("PrivateParty", 2), ("PublicParty", 3), ("Communities", 4)],
);

pub const PVP_MATCH_STATE: EnumDef = (
    "PvPMatchState",
    &[("Inactive", 0), ("Waiting", 1), ("StartUp", 2), ("Engaged", 3), ("PostRound", 4), ("Complete", 5)],
);

pub const WORLD_ELAPSED_TIMER_TYPES: EnumDef = (
    "WorldElapsedTimerTypes",
    &[("ChallengeMode", 0), ("ProvingGround", 1)],
);

pub const PLAYER_MENTORSHIP_STATUS: SeqEnumDef = (
    "PlayerMentorshipStatus",
    &["None", "Mentor", "Newcomer"],
);

pub const RELATIVE_CONTENT_DIFFICULTY: SeqEnumDef = (
    "RelativeContentDifficulty",
    &["Trivial", "Easy", "Fair", "Difficult", "Impossible"],
);

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
        "DbHouseOwnerRestriction",
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
    &[
        ("Accountbanktab", -3),
        ("AccountBankTab_1", 12),
        ("AccountBankTab_2", 13),
        ("AccountBankTab_3", 14),
        ("AccountBankTab_4", 15),
        ("AccountBankTab_5", 16),
        ("Backpack", 0),
        ("Bag_1", 1),
        ("Bag_2", 2),
        ("Bag_3", 3),
        ("Bag_4", 4),
        ("Characterbanktab", -2),
        ("CharacterBankTab_1", 6),
        ("CharacterBankTab_2", 7),
        ("CharacterBankTab_3", 8),
        ("CharacterBankTab_4", 9),
        ("CharacterBankTab_5", 10),
        ("CharacterBankTab_6", 11),
        ("Keyring", -1),
        ("ReagentBag", 5),
    ],
);

// ============================================================================
// NamePlate Enums
// ============================================================================

pub const NAME_PLATE_SIZE: EnumDef = (
    "NamePlateSize",
    &[("Small", 1), ("Medium", 2), ("Large", 3), ("ExtraLarge", 4), ("Huge", 5)],
);
