//! Additional addon system enum data: clubs, housing, PvP, talents, etc.

use super::{EnumDef, SeqEnumDef};

// ============================================================================
// Club / Communities Enums
// ============================================================================

pub const CLUB_TYPE: SeqEnumDef = ("ClubType", &["BattleNet", "Character", "Guild", "Other"]);

pub const CLUB_FINDER_SETTING_FLAGS: SeqEnumDef = (
    "ClubFinderSettingFlags",
    &[
        "None",
        "Dungeons",
        "Raids",
        "PvP",
        "RP",
        "Social",
        "Small",
        "Medium",
        "Large",
        "Tank",
        "Healer",
        "Damage",
        "EnableListing",
        "MaxLevelOnly",
        "AutoAccept",
        "FactionHorde",
        "FactionAlliance",
        "FactionNeutral",
        "SortRelevance",
        "SortMemberCount",
        "SortNewest",
        "LanguageReserved1",
        "LanguageReserved2",
        "LanguageReserved3",
        "LanguageReserved4",
        "LanguageReserved5",
    ],
);

pub const CLUB_ACTION_TYPE: SeqEnumDef = (
    "ClubActionType",
    &[
        "ErrorClubActionSubscribe",
        "ErrorClubActionCreate",
        "ErrorClubActionEdit",
        "ErrorClubActionDestroy",
        "ErrorClubActionLeave",
        "ErrorClubActionCreateTicket",
        "ErrorClubActionDestroyTicket",
        "ErrorClubActionRedeemTicket",
        "ErrorClubActionGetTicket",
        "ErrorClubActionGetTickets",
        "ErrorClubActionGetBans",
        "ErrorClubActionGetInvitations",
        "ErrorClubActionRevokeInvitation",
        "ErrorClubActionAcceptInvitation",
        "ErrorClubActionDeclineInvitation",
        "ErrorClubActionCreateStream",
        "ErrorClubActionEditStream",
        "ErrorClubActionDestroyStream",
        "ErrorClubActionInviteMember",
        "ErrorClubActionEditMember",
        "ErrorClubActionEditMemberNote",
        "ErrorClubActionKickMember",
        "ErrorClubActionAddBan",
        "ErrorClubActionRemoveBan",
        "ErrorClubActionCreateMessage",
        "ErrorClubActionEditMessage",
        "ErrorClubActionDestroyMessage",
    ],
);

pub const CLUB_ERROR_TYPE: EnumDef = (
    "ClubErrorType",
    &[
        ("ErrorCommunitiesUnknown", 0),
        ("ErrorCommunitiesOther", 1),
        ("ErrorCommunitiesNeutralFaction", 2),
        ("ErrorCommunitiesUnknownRealm", 3),
        ("ErrorCommunitiesBadTarget", 4),
        ("ErrorCommunitiesWrongFaction", 5),
        ("ErrorCommunitiesRestricted", 6),
        ("ErrorCommunitiesIgnored", 7),
        ("ErrorCommunitiesGuild", 8),
        ("ErrorCommunitiesWrongRegion", 9),
        ("ErrorCommunitiesUnknownTicket", 10),
        ("ErrorCommunitiesMissingShortName", 11),
        ("ErrorCommunitiesProfanity", 12),
        ("ErrorCommunitiesTrial", 13),
        ("ErrorCommunitiesVeteranTrial", 14),
        ("ErrorClubFull", 15),
        ("ErrorClubNoClub", 16),
        ("ErrorClubNotMember", 17),
        ("ErrorClubAlreadyMember", 18),
        ("ErrorClubNoSuchMember", 19),
        ("ErrorClubNoSuchInvitation", 20),
        ("ErrorClubInvitationAlreadyExists", 21),
        ("ErrorClubInvalidRoleID", 22),
        ("ErrorClubInsufficientPrivileges", 23),
        ("ErrorClubTooManyClubsJoined", 24),
        ("ErrorClubTooManyCreatedClubsJoined", 25),
        ("ErrorClubVoiceFull", 26),
        ("ErrorClubStreamNoStream", 27),
        ("ErrorClubStreamInvalidName", 28),
        ("ErrorClubStreamCountAtMax", 29),
        ("ErrorClubMemberHasRequiredRole", 30),
        ("ErrorClubSentInvitationCountAtMax", 31),
        ("ErrorClubReceivedInvitationCountAtMax", 32),
        ("ErrorClubTicketCountAtMax", 33),
        ("ErrorClubTicketNoSuchTicket", 34),
        ("ErrorClubTicketHasConsumedAllowedRedeemCount", 35),
        ("ErrorClubBanAlreadyExists", 36),
        ("ErrorClubBanCountAtMax", 37),
        ("ErrorClubNoSuchBan", 38),
        ("ErrorClubUnavailable", 39),
        ("ErrorClubNotOwner", 40),
        ("ErrorClubTargetIsBanned", 41),
        ("ErrorClubStreamCountAtMin", 42),
        ("ErrorCommunitiesChatMute", 43),
        ("ErrorClubDoesntAllowCrossFaction", 44),
        ("ErrorClubEditHasCrossFactionMembers", 45),
    ],
);

pub const CLUB_REMOVED_REASON: EnumDef = (
    "ClubRemovedReason",
    &[
        ("None", 0),
        ("Banned", 1),
        ("Removed", 2),
        ("ClubDestroyed", 3),
    ],
);

pub const CLUB_ROLE_IDENTIFIER: EnumDef = (
    "ClubRoleIdentifier",
    &[("Owner", 1), ("Leader", 2), ("Moderator", 3), ("Member", 4)],
);

// ============================================================================
// Match / PvP Enums
// ============================================================================

pub const MATCH_DETAIL_TYPE: SeqEnumDef = (
    "MatchDetailType",
    &["Placement", "Kills", "PlunderAcquired"],
);

pub const PVP_UNIT_CLASSIFICATION: SeqEnumDef = (
    "PvPUnitClassification",
    &[
        "FlagCarrierHorde",
        "FlagCarrierAlliance",
        "FlagCarrierNeutral",
        "CartRunnerHorde",
        "CartRunnerAlliance",
        "AssassinHorde",
        "AssassinAlliance",
        "OrbCarrierBlue",
        "OrbCarrierGreen",
        "OrbCarrierOrange",
        "OrbCarrierPurple",
    ],
);

pub const END_OF_MATCH_TYPE: SeqEnumDef = ("EndOfMatchType", &["None", "Plunderstorm"]);

pub const ARROW_CALLOUT_DIRECTION: SeqEnumDef =
    ("ArrowCalloutDirection", &["Up", "Down", "Left", "Right"]);

pub const NAVIGATION_STATE: SeqEnumDef = (
    "NavigationState",
    &["Invalid", "Occluded", "InRange", "Disabled"],
);

pub const UI_FRAME_TYPE: SeqEnumDef = ("UIFrameType", &["JailersTowerBuffs", "InterruptTutorial"]);

pub const COOLDOWN_VIEWER_CATEGORY: SeqEnumDef = (
    "CooldownViewerCategory",
    &["Essential", "Utility", "TrackedBuff", "TrackedBar"],
);

#[cfg(feature = "retail-12-1-0")]
pub const COOLDOWN_VIEWER_SOUND: SeqEnumDef = (
    "CooldownViewerSound",
    &[
        "TextToSpeech",
        "AnimalsCat",
        "AnimalsChicken",
        "AnimalsCow",
        "AnimalsGnoll",
        "AnimalsGoat",
        "AnimalsLion",
        "AnimalsPanther",
        "AnimalsRattlesnake",
        "AnimalsSheep",
        "AnimalsWolf",
        "DevicesBoatHorn",
        "DevicesAirHorn",
        "DevicesBikeHorn",
        "DevicesCashRegister",
        "DevicesJackpotBell",
        "DevicesJackpotCoins",
        "DevicesJackpotFail",
        "DevicesRotaryPhoneDial",
        "DevicesRotaryPhoneRing",
        "DevicesStovePipe",
        "DevicesTrashcanLid",
        "ImpactsAnvilStrike",
        "ImpactsBubbleSmash",
        "ImpactsLowThud",
        "ImpactsMetalClanks",
        "ImpactsMetalRattle",
        "ImpactsMetalScrape",
        "ImpactsMetalWarble",
        "ImpactsPopClick",
        "ImpactsStrangeClang",
        "ImpactsSwordScrape",
        "InstrumentsBellRing",
        "InstrumentsBellTrill",
        "InstrumentsBrass",
        "InstrumentsChimeAscending",
        "InstrumentsGuitarChug",
        "InstrumentsGuitarPinch",
        "InstrumentsPitchPipeDistressed",
        "InstrumentsPitchPipeNote",
        "InstrumentsSynthBig",
        "InstrumentsSynthBuzz",
        "InstrumentsSynthHigh",
        "InstrumentsWarhorn",
        "War2AbstractWhoosh",
        "War2Choir",
        "War2Construction",
        "War2MagicChimes",
        "War2PigSqueal",
        "War2Saws",
        "War2Seal",
        "War2Slow",
        "War2Smith",
        "War2SynthStinger",
        "War2TrumpetRally",
        "War2ZippyMagic",
        "War3Bell",
        "War3CrunchyBell",
        "War3DrumSplash",
        "War3Error",
        "War3Fanfare",
        "War3GateOpen",
        "War3Gold",
        "War3MagicShimmer",
        "War3Ringout",
        "War3Rooster",
        "War3ShimmerBell",
        "War3WolfHowl",
        "ShortBellStrike",
        "ShortBellTree",
        "ShortBigPot",
        "ShortBlades",
        "ShortCoffeeMug",
        "ShortCowBell",
        "ShortFingerSnap",
        "ShortGuitar",
        "ShortKalimba",
        "ShortMetalBladeDrop",
        "ShortMetalBladeOnRod",
        "ShortMetalImpact",
        "ShortMiniWoodXylophone",
        "ShortPaperCup",
        "ShortSheetMetal",
        "ShortStovePipe",
        "ShortStovePipeBlade",
        "ShortSwordShing",
        "ShortSynthBleep",
        "ShortSynthBlurp",
        "ShortSynthError",
        "ShortSynthHigh",
        "ShortTriangle",
        "ShortWaterDrop",
        "ShortWineBottle",
        "ShortWoodXylophone",
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const COOLDOWN_VIEWER_SOUND_META: EnumDef = (
    "CooldownViewerSoundMeta",
    &[("MinValue", 0), ("MaxValue", 93), ("NumValues", 94)],
);

pub const TTS_VOICE_TYPE: SeqEnumDef = ("TtsVoiceType", &["Standard", "Alternate"]);

#[cfg(feature = "retail-12-1-0")]
pub const CUSTOM_AURA_BUTTON_DISPEL_TYPE_TEXTURE_STYLE: SeqEnumDef = (
    "CustomAuraButtonDispelTypeTextureStyle",
    &[
        "Border",
        "BorderWithIcon",
        "Icon",
        "PreserveAsset",
        "CustomAsset",
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const CUSTOM_AURA_BUTTON_DISPEL_TYPE_TEXTURE_STYLE_META: EnumDef = (
    "CustomAuraButtonDispelTypeTextureStyleMeta",
    &[("MinValue", 0), ("MaxValue", 4), ("NumValues", 5)],
);

#[cfg(feature = "retail-12-1-0")]
pub const RECENT_ALLIES_FRIEND_TAG: SeqEnumDef = (
    "RecentAlliesFriendTag",
    &[
        "Professions",
        "PvP",
        "Raiding",
        "Dungeons",
        "Delves",
        "Questing",
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const RECENT_ALLIES_FRIEND_TAG_META: EnumDef = (
    "RecentAlliesFriendTagMeta",
    &[("MinValue", 0), ("MaxValue", 5), ("NumValues", 6)],
);

// ============================================================================
// Guild / Social Enums
// ============================================================================

#[cfg(feature = "retail-12-1-0")]
pub const BATTLE_NET_FRIEND_LEVEL: EnumDef = (
    "BattleNetFriendLevel",
    &[("BattleTag", 1), ("RealID", 2), ("Title", 3)],
);

#[cfg(feature = "retail-12-1-0")]
pub const BATTLE_NET_FRIEND_LEVEL_META: EnumDef = (
    "BattleNetFriendLevelMeta",
    &[("MinValue", 1), ("MaxValue", 3), ("NumValues", 3)],
);

#[cfg(feature = "retail-12-1-0")]
pub const BATTLE_NET_FRIEND_TAG: EnumDef = (
    "BattleNetFriendTag",
    &[
        ("Professions", 0),
        ("PvP", 1),
        ("Raiding", 2),
        ("Dungeons", 3),
        ("Delves", 4),
        ("Questing", 5),
        ("Roleplaying", 6),
        ("DamagerRole", 7),
        ("HealerRole", 8),
        ("TankRole", 9),
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const BATTLE_NET_FRIEND_TAG_META: EnumDef = (
    "BattleNetFriendTagMeta",
    &[("MinValue", 0), ("MaxValue", 9), ("NumValues", 10)],
);

#[cfg(feature = "retail-12-1-0")]
pub const SOCIAL_UI_BLOCK_TYPE: EnumDef = (
    "SocialUIBlockType",
    &[("None", 0), ("Ignore", 1), ("BattleNetInviteBlock", 2)],
);

#[cfg(feature = "retail-12-1-0")]
pub const SOCIAL_UI_BLOCK_TYPE_META: EnumDef = (
    "SocialUIBlockTypeMeta",
    &[("MinValue", 0), ("MaxValue", 2), ("NumValues", 3)],
);

#[cfg(feature = "retail-12-1-0")]
pub const SOCIAL_UI_PRESENCE_TYPE: EnumDef = (
    "SocialUIPresenceType",
    &[
        ("Unknown", 0),
        ("Online", 1),
        ("Offline", 2),
        ("Away", 3),
        ("Busy", 4),
        ("AppearOffline", 5),
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const SOCIAL_UI_PRESENCE_TYPE_META: EnumDef = (
    "SocialUIPresenceTypeMeta",
    &[("MinValue", 0), ("MaxValue", 5), ("NumValues", 6)],
);

#[cfg(feature = "retail-12-1-0")]
pub const SOCIAL_SYSTEM_TYPE: SeqEnumDef = (
    "SocialSystemType",
    &[
        "Friends",
        "QuickJoin",
        "RaidList",
        "RecruitAFriend",
        "RecentAllies",
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const SOCIAL_SYSTEM_TYPE_META: EnumDef = (
    "SocialSystemTypeMeta",
    &[("MinValue", 0), ("MaxValue", 4), ("NumValues", 5)],
);

pub const GUILD_ERROR_TYPE: SeqEnumDef = (
    "GuildErrorType",
    &[
        "Success",
        "UnknownError",
        "AlreadyInGuild",
        "TargetAlreadyInGuild",
        "InvitedToGuild",
        "TargetInvitedToGuild",
        "NameInvalid",
        "NameAlreadyExists",
        "NoPermisson",
        "NotInGuild",
        "TargetNotInGuild",
        "PlayerNotFound",
        "WrongFaction",
        "TargetTooHigh",
        "TargetTooLow",
        "TooManyRanks",
        "TooFewRanks",
        "RanksLocked",
        "RankInUse",
        "Ignored",
        "Busy",
        "TargetLevelTooLow",
        "TargetLevelTooHigh",
        "TooManyMembers",
        "InvalidBankTab",
        "WithdrawLimit",
        "NotEnoughMoney",
        "TeamNotFound",
        "BankTabFull",
        "BadItem",
        "TeamsLocked",
        "TooMuchMoney",
        "WrongBankTab",
        "TooManyCreate",
        "RankRequiresAuthenticator",
        "BankTabLocked",
        "TrialAccount",
        "VeteranAccount",
        "UndeletableDueToLevel",
        "LockedForMove",
        "GuildRepTooLow",
        "CantInviteSelf",
        "HasRestriction",
        "BankNotFound",
        "NewLeaderWrongFaction",
        "GuildBankNotAvailable",
        "NewLeaderWrongRealm",
        "DeleteNoAppropriateLeader",
        "RealmMismatch",
        "InCooldown",
        "ReservationExpired",
        "HousingEvictError",
        "Throttled",
    ],
);

pub const ROLODEX_TYPE: EnumDef = (
    "RolodexType",
    &[
        ("None", 0),
        ("PartyMember", 1),
        ("RaidMember", 2),
        ("Trade", 3),
        ("Whisper", 4),
        ("PublicOrderFilledByOther", 5),
        ("PublicOrderFilledByYou", 6),
        ("PersonalOrderFilledByOther", 7),
        ("PersonalOrderFilledByYou", 8),
        ("GuildOrderFilledByOther", 9),
        ("GuildOrderFilledByYou", 10),
        ("CreatureKill", 11),
        ("CompleteDungeon", 12),
        ("KillRaidBoss", 13),
        ("KillLfrBoss", 14),
        ("CompleteDelve", 15),
        ("CompleteArena", 16),
        ("CompleteBg", 17),
        ("Duel", 18),
        ("PetBattle", 19),
        ("PvPKill", 20),
        // 21 and 22 are absent from RolodexConstantsDocumentation.lua; LegacyFriend
        // is 23 there, and Blizzard_RecentAlliesUtil.lua indexes a table by it at
        // file scope, so the value has to match rather than follow the sequence.
        ("LegacyFriend", 23),
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const ROLODEX_TYPE_LEGACY_FRIEND: EnumDef = ("RolodexType", &[("LegacyFriend", 23)]);

#[cfg(feature = "retail-12-1-0")]
pub const ROLODEX_TYPE_META: EnumDef = (
    "RolodexTypeMeta",
    &[("MinValue", 0), ("MaxValue", 23), ("NumValues", 22)],
);

pub const INVALID_PLOT_SCREENSHOT_REASON: SeqEnumDef = (
    "InvalidPlotScreenshotReason",
    &[
        "None",
        "OutOfBounds",
        "Facing",
        "NoNeighborhoodFound",
        "NoActivePlayer",
    ],
);

pub const WARBAND_SCENE_ANIMATION_EVENT: EnumDef = (
    "WarbandSceneAnimationEvent",
    &[
        ("StartingPose", 0),
        ("Idle", 1),
        ("Mouseover", 2),
        ("Select", 3),
        ("Deselect", 4),
        ("Insert", 5),
        ("EnterWorld", 6),
        ("Spin", 7),
        ("Poke", 8),
        ("Ffx", 9),
    ],
);

// ============================================================================
// Housing Enums
// ============================================================================

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_RESULT: SeqEnumDef = (
    "HousingResult",
    &[
        "Success",
        "AccountBanned",
        "ActionLockedByCombat",
        "BlueprintCodeInvalid",
        "BlueprintDyeFailed",
        "BlueprintGenericExportError",
        "BlueprintGenericImportError",
        "BlueprintLocationInvalid",
        "BlueprintNameInvalid",
        "BlueprintNotFound",
        "BlueprintRequirementsUnmet",
        "BlueprintRoomPlacementRequired",
        "BlueprintTypeInvalid",
        "BlueprintTypeLocationInvalid",
        "BlueprintStorageLimit",
        "BlueprintVersionInvalid",
        "BoundsFailureChildren",
        "BoundsFailurePlot",
        "BoundsFailureRoom",
        "BoundToStartingArea",
        "CannotAfford",
        "CharterComplete",
        "CollisionInvalid",
        "DbError",
        "DecorCannotBeRedeemed",
        "DecorItemNotDestroyable",
        "DecorNotFound",
        "DecorNotFoundInStorage",
        "DuplicateCharterSignature",
        "FilterRejected",
        "FixtureCantDeleteDoor",
        "FixtureHookEmpty",
        "FixtureHookOccupied",
        "FixtureHouseTypeMismatch",
        "FixtureNotFound",
        "FixtureSizeMismatch",
        "FixtureTypeMismatch",
        "GenericFailure",
        "GuildMoreAccountsNeeded",
        "GuildMoreActivePlayersNeeded",
        "GuildNotLoaded",
        "HouseEditLockFailed",
        "HouseExteriorAlreadyThatSize",
        "HouseExteriorAlreadyThatType",
        "HouseExteriorRootNotFound",
        "HouseExteriorTypeNeighborhoodMismatch",
        "HouseExteriorTypeNotFound",
        "HouseExteriorTypeSizeMismatch",
        "HouseExteriorSizeNotAvailable",
        "HookNotChildOfFixture",
        "HouseNotFound",
        "IncorrectFaction",
        "InvalidDecorItem",
        "InvalidDistance",
        "InvalidExteriorDocument",
        "InvalidGuild",
        "InvalidHouse",
        "InvalidInstance",
        "InvalidInteraction",
        "InvalidInteriorDocument",
        "InvalidLightOverlap",
        "InvalidMap",
        "InvalidNeighborhoodName",
        "InvalidRoomLayout",
        "InsufficientRoomBudget",
        "LockedByOtherPlayer",
        "LockOperationFailed",
        "MaxPlacedDecorReached",
        "MaxPetDecorReached",
        "MaxPreviewDecorReached",
        "MaxStorageDecorReached",
        "MissingCoreFixture",
        "MissingDye",
        "MissingExpansionAccess",
        "MissingFactionMap",
        "MissingPrivateNeighborhoodInvite",
        "MoreHouseSlotsNeeded",
        "MoreSignaturesNeeded",
        "NeighborhoodNotFound",
        "NoNeighborhoodOwnershipRequests",
        "NotInDecorEditMode",
        "NotInFixtureEditMode",
        "NotInLayoutEditMode",
        "NotInsideHouse",
        "NotOnOwnedPlot",
        "OperationAborted",
        "OwnerNotInGuild",
        "PermissionDenied",
        "PlacementTargetInvalid",
        "PlayerNotFound",
        "PlayerNotInInstance",
        "PlotNotFound",
        "PlotNotVacant",
        "PlotReservationCooldown",
        "PlotReserved",
        "RoomNotFound",
        "RoomPlacementOutOfBounds",
        "RoomUpdateFailed",
        "RpcFailure",
        "ServiceNotAvailable",
        "StaticDataNotFound",
        "TimeoutLimit",
        "TimerunningNotAllowed",
        "TokenRequired",
        "TooManyRequests",
        "TransactionFailure",
        "UncollectedExteriorFixture",
        "UncollectedHouseType",
        "UncollectedRoom",
        "UncollectedRoomMaterial",
        "UncollectedRoomTheme",
        "UnlockOperationFailed",
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_RESULT_META: EnumDef = (
    "HousingResultMeta",
    &[("MinValue", 0), ("MaxValue", 111), ("NumValues", 112)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_PET_BEHAVIOR_TYPE: SeqEnumDef =
    ("HousingPetBehaviorType", &["Stationary", "Wander"]);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_PET_BEHAVIOR_TYPE_META: EnumDef = (
    "HousingPetBehaviorTypeMeta",
    &[("MinValue", 0), ("MaxValue", 1), ("NumValues", 2)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSE_EDITOR_PLAYER_TYPE: SeqEnumDef =
    ("HouseEditorPlayerType", &["None", "Owner", "Visitor"]);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSE_EDITOR_PLAYER_TYPE_META: EnumDef = (
    "HouseEditorPlayerTypeMeta",
    &[("MinValue", 0), ("MaxValue", 2), ("NumValues", 3)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSE_SETTING_FLAGS: EnumDef = (
    "HouseSettingFlags",
    &[
        ("None", 0),
        ("HouseAccessAnyone", 1),
        ("HouseAccessNeighbors", 2),
        ("HouseAccessGuild", 4),
        ("HouseAccessFriends", 8),
        ("HouseAccessParty", 16),
        ("PlotAccessAnyone", 32),
        ("PlotAccessNeighbors", 64),
        ("PlotAccessGuild", 128),
        ("PlotAccessFriends", 256),
        ("PlotAccessParty", 512),
        ("BlueprintExportAnyone", 1024),
        ("BlueprintExportNeighbors", 2048),
        ("BlueprintExportGuild", 4096),
        ("BlueprintExportFriends", 8192),
        ("BlueprintExportParty", 16384),
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSE_SETTING_FLAGS_META: EnumDef = (
    "HouseSettingFlagsMeta",
    &[("MinValue", 0), ("MaxValue", 16384), ("NumValues", 16)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_BUDGET_TYPE: EnumDef = (
    "HousingBudgetType",
    &[("RoomPlacement", 0), ("DecorPlacement", 1), ("PetDecor", 2)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_BUDGET_TYPE_META: EnumDef = (
    "HousingBudgetTypeMeta",
    &[("MinValue", 0), ("MaxValue", 2), ("NumValues", 3)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_HOUSE_SCOPE: EnumDef = (
    "HousingHouseScope",
    &[("None", 0), ("Interior", 1), ("Exterior", 2)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_HOUSE_SCOPE_META: EnumDef = (
    "HousingHouseScopeMeta",
    &[("MinValue", 0), ("MaxValue", 2), ("NumValues", 3)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_BLUEPRINT_TYPE: SeqEnumDef = (
    "HousingBlueprintType",
    &["None", "House", "Room", "Interior", "Exterior"],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_BLUEPRINT_TYPE_META: EnumDef = (
    "HousingBlueprintTypeMeta",
    &[("MinValue", 0), ("MaxValue", 4), ("NumValues", 5)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_BLUEPRINT_CONTENT_TYPE: SeqEnumDef = (
    "HousingBlueprintContentType",
    &[
        "None",
        "HouseType",
        "Room",
        "Decor",
        "Dye",
        "Fixture",
        "Other",
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_BLUEPRINT_CONTENT_TYPE_META: EnumDef = (
    "HousingBlueprintContentTypeMeta",
    &[("MinValue", 0), ("MaxValue", 6), ("NumValues", 7)],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_BLUEPRINT_UNMET_REQUIREMENT_FLAGS: EnumDef = (
    "HousingBlueprintUnmetRequirementFlags",
    &[
        ("InsufficientBudget", 1),
        ("MissingRoom", 2),
        ("MissingFixture", 4),
        ("MissingDecor", 8),
        ("MissingDye", 16),
        ("MismatchedExteriorFaction", 32),
        ("HouseTypeLocked", 64),
        ("HouseSizeLocked", 128),
    ],
);

#[cfg(feature = "retail-12-1-0")]
pub const HOUSING_BLUEPRINT_UNMET_REQUIREMENT_FLAGS_META: EnumDef = (
    "HousingBlueprintUnmetRequirementFlagsMeta",
    &[("MinValue", 1), ("MaxValue", 128), ("NumValues", 8)],
);

pub const HOUSE_OWNER_ERROR: EnumDef = (
    "HouseOwnerError",
    &[("Faction", 0), ("Guild", 1), ("GenericPermission", 2)],
);

pub const HOUSING_FIXTURE_DECOR_ACTION: EnumDef =
    ("HousingFixtureDecorAction", &[("Store", 0), ("Detach", 1)]);

pub const HOUSING_LAYOUT_RESTRICTION: SeqEnumDef = (
    "HousingLayoutRestriction",
    &[
        "None",
        "RoomNotFound",
        "NotInsideHouse",
        "NotHouseOwner",
        "IsBaseRoom",
        "RoomNotLeaf",
        "StairwellConnection",
        "LastRoom",
        "UnreachableRoom",
        "SingleDoor",
    ],
);

pub const HOUSING_EXPERT_SUBMODE_RESTRICTION: SeqEnumDef = (
    "HousingExpertSubmodeRestriction",
    &[
        "None",
        "NotInExpertMode",
        "NoHouseExteriorScale",
        "NoWMOScale",
    ],
);

pub const NEIGHBORHOOD_OWNER_TYPE: SeqEnumDef =
    ("NeighborhoodOwnerType", &["None", "Guild", "Charter"]);

// ============================================================================
// Delves Enums
// ============================================================================

#[cfg(feature = "retail-12-1-0")]
pub const TIERED_ENTRANCE_TYPE: EnumDef = (
    "TieredEntranceType",
    &[
        ("Invalid", 0),
        ("Delve", 1),
        ("Sites", 2),
        ("WorldTier", 3),
        ("Lairs", 4),
    ],
);

pub const TIERED_ENTRANCE_REWARD_TYPE: EnumDef =
    ("TieredEntranceRewardType", &[("Item", 0), ("Currency", 1)]);

pub const WORLD_TIER_DIFFICULTY: EnumDef = (
    "WorldTierDifficulty",
    &[("Normal", 1), ("Heroic", 2), ("Mythic", 3)],
);

// ============================================================================
// Ping / Voice Enums
// ============================================================================

pub const PING_RESULT: SeqEnumDef = (
    "PingResult",
    &[
        "Success",
        "FailedGeneric",
        "FailedSpamming",
        "FailedDisabledByLeader",
        "FailedDisabledBySettings",
        "FailedOutOfPingArea",
        "FailedSquelched",
        "FailedUnspecified",
    ],
);

pub const VOICE_CHAT_STATUS_CODE: SeqEnumDef = (
    "VoiceChatStatusCode",
    &[
        "Success",
        "OperationPending",
        "TooManyRequests",
        "LoginProhibited",
        "ClientNotInitialized",
        "ClientNotLoggedIn",
        "ClientAlreadyLoggedIn",
        "ChannelNameTooShort",
        "ChannelNameTooLong",
        "ChannelAlreadyExists",
        "AlreadyInChannel",
        "TargetNotFound",
        "Failure",
        "ServiceLost",
        "UnableToLaunchProxy",
        "ProxyConnectionTimeOut",
        "ProxyConnectionUnableToConnect",
        "ProxyConnectionUnexpectedDisconnect",
        "Disabled",
        "UnsupportedChatChannelType",
        "InvalidCommunityStream",
        "PlayerSilenced",
        "PlayerVoiceChatParentalDisabled",
        "InvalidInputDevice",
        "InvalidOutputDevice",
    ],
);

// ============================================================================
// Talent / Trait Enums
// ============================================================================

pub const TRAIT_NODE_TYPE: SeqEnumDef = (
    "TraitNodeType",
    &["Single", "Tiered", "Selection", "SubTreeSelection"],
);

pub const TRAIT_NODE_FLAG: EnumDef = (
    "TraitNodeFlag",
    &[
        ("ShowMultipleIcons", 1),
        ("NeverPurchasable", 2),
        ("TestPositionLocked", 4),
        ("TestGridPositioned", 8),
        ("ActiveAtFirstRank", 16),
        ("ShowExpandedSelection", 32),
        ("HideMaxRank", 64),
        ("HighestChosenRank", 128),
    ],
);

pub const TRAIT_NODE_ENTRY_TYPE: SeqEnumDef = (
    "TraitNodeEntryType",
    &[
        "SpendHex",
        "SpendSquare",
        "SpendCircle",
        "SpendSmallCircle",
        "DeprecatedSelect",
        "DragAndDrop",
        "SpendDiamond",
        "ProfPath",
        "ProfPerk",
        "ProfPathUnlock",
        "RedButton",
        "ArmorSet",
        "SpendInfinite",
        "SpendCapstoneCircle",
        "SpendCapstoneSquare",
    ],
);

pub const TRAIT_DEFINITION_SUB_TYPE: SeqEnumDef = (
    "TraitDefinitionSubType",
    &[
        "DragonflightRed",
        "DragonflightBlue",
        "DragonflightGreen",
        "DragonflightBronze",
        "DragonflightBlack",
    ],
);

pub const TRAIT_EDGE_VISUAL_STYLE: SeqEnumDef = ("TraitEdgeVisualStyle", &["None", "Straight"]);

// ============================================================================
// Currency / Token Enums
// ============================================================================

pub const ACCOUNT_CURRENCY_TRANSFER_RESULT: SeqEnumDef = (
    "AccountCurrencyTransferResult",
    &[
        "Success",
        "InvalidCharacter",
        "CharacterLoggedIn",
        "InsufficientCurrency",
        "MaxQuantity",
        "InvalidCurrency",
        "NoValidSourceCharacter",
        "ServerError",
        "CannotUseCurrency",
        "TransactionInProgress",
        "CurrencyTransferDisabled",
    ],
);

pub const CURRENCY_FILTER_TYPE: SeqEnumDef = (
    "CurrencyFilterType",
    &[
        "None",
        "DiscoveredOnly",
        "DiscoveredAndAllAccountTransferable",
    ],
);

// ============================================================================
// Delves Enums
// ============================================================================

pub const CURIO_RARITY: EnumDef = (
    "CurioRarity",
    &[("Common", 1), ("Uncommon", 2), ("Rare", 3), ("Epic", 4)],
);

// ============================================================================
// Navigation / Tracking Enums
// ============================================================================

pub const SUPER_TRACKING_TYPE: SeqEnumDef = (
    "SuperTrackingType",
    &[
        "Quest",
        "UserWaypoint",
        "Corpse",
        "Scenario",
        "Content",
        "PartyMember",
        "MapPin",
        "Vignette",
    ],
);

pub const SUPER_TRACKING_MAP_PIN_TYPE: SeqEnumDef = (
    "SuperTrackingMapPinType",
    &[
        "AreaPOI",
        "QuestOffer",
        "TaxiNode",
        "DigSite",
        "HousingPlot",
    ],
);

// ============================================================================
// SpellBook Enums
// ============================================================================

pub const SPELL_BOOK_SPELL_BANK: SeqEnumDef = ("SpellBookSpellBank", &["Player", "Pet"]);

pub const SPELL_BOOK_ITEM_TYPE: SeqEnumDef = (
    "SpellBookItemType",
    &["None", "Spell", "FutureSpell", "PetAction", "Flyout"],
);

pub const SPELL_BOOK_SKILL_LINE_INDEX: EnumDef = (
    "SpellBookSkillLineIndex",
    &[
        ("General", 1),
        ("Class", 2),
        ("MainSpec", 3),
        ("OffSpecStart", 4),
    ],
);
