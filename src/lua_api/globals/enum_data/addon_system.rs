//! Additional addon system enum data: clubs, housing, PvP, talents, etc.

use super::{EnumDef, SeqEnumDef};

// ============================================================================
// Club / Communities Enums
// ============================================================================

pub const CLUB_TYPE: SeqEnumDef = (
    "ClubType",
    &["BattleNet", "Character", "Guild", "Other"],
);

pub const CLUB_FINDER_SETTING_FLAGS: SeqEnumDef = (
    "ClubFinderSettingFlags",
    &[
        "None", "Dungeons", "Raids", "PvP", "RP", "Social", "Small", "Medium", "Large",
        "Tank", "Healer", "Damage", "EnableListing", "MaxLevelOnly", "AutoAccept",
        "FactionHorde", "FactionAlliance", "FactionNeutral", "SortRelevance", "SortMemberCount",
        "SortNewest", "LanguageReserved1", "LanguageReserved2", "LanguageReserved3",
        "LanguageReserved4", "LanguageReserved5",
    ],
);

pub const CLUB_ACTION_TYPE: SeqEnumDef = (
    "ClubActionType",
    &[
        "ErrorClubActionSubscribe", "ErrorClubActionCreate", "ErrorClubActionEdit",
        "ErrorClubActionDestroy", "ErrorClubActionLeave", "ErrorClubActionCreateTicket",
        "ErrorClubActionDestroyTicket", "ErrorClubActionRedeemTicket", "ErrorClubActionGetTicket",
        "ErrorClubActionGetTickets", "ErrorClubActionGetBans", "ErrorClubActionGetInvitations",
        "ErrorClubActionRevokeInvitation", "ErrorClubActionAcceptInvitation",
        "ErrorClubActionDeclineInvitation", "ErrorClubActionCreateStream",
        "ErrorClubActionEditStream", "ErrorClubActionDestroyStream",
        "ErrorClubActionInviteMember", "ErrorClubActionEditMember",
        "ErrorClubActionEditMemberNote", "ErrorClubActionKickMember",
        "ErrorClubActionAddBan", "ErrorClubActionRemoveBan",
        "ErrorClubActionCreateMessage", "ErrorClubActionEditMessage",
        "ErrorClubActionDestroyMessage",
    ],
);

pub const CLUB_ERROR_TYPE: EnumDef = (
    "ClubErrorType",
    &[
        ("ErrorCommunitiesUnknown", 0), ("ErrorCommunitiesOther", 1),
        ("ErrorCommunitiesNeutralFaction", 2), ("ErrorCommunitiesUnknownRealm", 3),
        ("ErrorCommunitiesBadTarget", 4), ("ErrorCommunitiesWrongFaction", 5),
        ("ErrorCommunitiesRestricted", 6), ("ErrorCommunitiesIgnored", 7),
        ("ErrorCommunitiesGuild", 8), ("ErrorCommunitiesWrongRegion", 9),
        ("ErrorCommunitiesUnknownTicket", 10), ("ErrorCommunitiesMissingShortName", 11),
        ("ErrorCommunitiesProfanity", 12), ("ErrorCommunitiesTrial", 13),
        ("ErrorCommunitiesVeteranTrial", 14), ("ErrorClubFull", 15),
        ("ErrorClubNoClub", 16), ("ErrorClubNotMember", 17),
        ("ErrorClubAlreadyMember", 18), ("ErrorClubNoSuchMember", 19),
        ("ErrorClubNoSuchInvitation", 20), ("ErrorClubInvitationAlreadyExists", 21),
        ("ErrorClubInvalidRoleID", 22), ("ErrorClubInsufficientPrivileges", 23),
        ("ErrorClubTooManyClubsJoined", 24), ("ErrorClubTooManyCreatedClubsJoined", 25),
        ("ErrorClubVoiceFull", 26), ("ErrorClubStreamNoStream", 27),
        ("ErrorClubStreamInvalidName", 28), ("ErrorClubStreamCountAtMax", 29),
        ("ErrorClubMemberHasRequiredRole", 30), ("ErrorClubSentInvitationCountAtMax", 31),
        ("ErrorClubReceivedInvitationCountAtMax", 32),
        ("ErrorClubTicketCountAtMax", 33), ("ErrorClubTicketNoSuchTicket", 34),
        ("ErrorClubTicketHasConsumedAllowedRedeemCount", 35),
        ("ErrorClubBanAlreadyExists", 36), ("ErrorClubBanCountAtMax", 37),
        ("ErrorClubNoSuchBan", 38), ("ErrorClubUnavailable", 39),
        ("ErrorClubNotOwner", 40), ("ErrorClubTargetIsBanned", 41),
        ("ErrorClubStreamCountAtMin", 42), ("ErrorCommunitiesChatMute", 43),
        ("ErrorClubDoesntAllowCrossFaction", 44),
        ("ErrorClubEditHasCrossFactionMembers", 45),
    ],
);

pub const CLUB_REMOVED_REASON: EnumDef = (
    "ClubRemovedReason",
    &[("None", 0), ("Banned", 1), ("Removed", 2), ("ClubDestroyed", 3)],
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
        "FlagCarrierHorde", "FlagCarrierAlliance", "FlagCarrierNeutral",
        "CartRunnerHorde", "CartRunnerAlliance",
        "AssassinHorde", "AssassinAlliance",
        "OrbCarrierBlue", "OrbCarrierGreen", "OrbCarrierOrange", "OrbCarrierPurple",
    ],
);

pub const END_OF_MATCH_TYPE: SeqEnumDef = (
    "EndOfMatchType",
    &["None", "Plunderstorm"],
);

// ============================================================================
// Expansion / Feature Enums
// ============================================================================

pub const EXPANSION_LANDING_PAGE_TYPE: SeqEnumDef = (
    "ExpansionLandingPageType",
    &["None", "Dragonflight", "WarWithin"],
);

pub const ARROW_CALLOUT_DIRECTION: SeqEnumDef = (
    "ArrowCalloutDirection",
    &["Up", "Down", "Left", "Right"],
);

pub const NAVIGATION_STATE: SeqEnumDef = (
    "NavigationState",
    &["Invalid", "Occluded", "InRange", "Disabled"],
);

pub const UI_FRAME_TYPE: SeqEnumDef = (
    "UIFrameType",
    &["JailersTowerBuffs", "InterruptTutorial"],
);

pub const COOLDOWN_VIEWER_CATEGORY: SeqEnumDef = (
    "CooldownViewerCategory",
    &["Essential", "Utility", "TrackedBuff", "TrackedBar"],
);

pub const TTS_VOICE_TYPE: SeqEnumDef = (
    "TtsVoiceType",
    &["Standard", "Alternate"],
);

// ============================================================================
// Guild / Social Enums
// ============================================================================

pub const GUILD_ERROR_TYPE: SeqEnumDef = (
    "GuildErrorType",
    &[
        "Success", "UnknownError", "AlreadyInGuild", "TargetAlreadyInGuild",
        "InvitedToGuild", "TargetInvitedToGuild", "NameInvalid", "NameAlreadyExists",
        "NoPermisson", "NotInGuild", "TargetNotInGuild", "PlayerNotFound",
        "WrongFaction", "TargetTooHigh", "TargetTooLow", "TooManyRanks",
        "TooFewRanks", "RanksLocked", "RankInUse", "Ignored", "Busy",
        "TargetLevelTooLow", "TargetLevelTooHigh", "TooManyMembers",
        "InvalidBankTab", "WithdrawLimit", "NotEnoughMoney", "TeamNotFound",
        "BankTabFull", "BadItem", "TeamsLocked", "TooMuchMoney", "WrongBankTab",
        "TooManyCreate", "RankRequiresAuthenticator", "BankTabLocked",
        "TrialAccount", "VeteranAccount", "UndeletableDueToLevel", "LockedForMove",
        "GuildRepTooLow", "CantInviteSelf", "HasRestriction", "BankNotFound",
        "NewLeaderWrongFaction", "GuildBankNotAvailable", "NewLeaderWrongRealm",
        "DeleteNoAppropriateLeader", "RealmMismatch", "InCooldown",
        "ReservationExpired", "HousingEvictError", "Throttled",
    ],
);

pub const ROLODEX_TYPE: SeqEnumDef = (
    "RolodexType",
    &[
        "None", "PartyMember", "RaidMember", "Trade", "Whisper",
        "PublicOrderFilledByOther", "PublicOrderFilledByYou",
        "PersonalOrderFilledByOther", "PersonalOrderFilledByYou",
        "GuildOrderFilledByOther", "GuildOrderFilledByYou",
        "CreatureKill", "CompleteDungeon", "KillRaidBoss", "KillLfrBoss",
        "CompleteDelve", "CompleteArena", "CompleteBg", "Duel", "PetBattle", "PvPKill",
    ],
);

pub const INVALID_PLOT_SCREENSHOT_REASON: SeqEnumDef = (
    "InvalidPlotScreenshotReason",
    &["None", "OutOfBounds", "Facing", "NoNeighborhoodFound", "NoActivePlayer"],
);

// ============================================================================
// Housing Enums
// ============================================================================

pub const HOUSING_RESULT: SeqEnumDef = (
    "HousingResult",
    &[
        "Success", "ActionLockedByCombat", "BoundsFailureChildren",
        "BoundsFailurePlot", "BoundsFailureRoom", "CannotAfford",
        "CharterComplete", "CollisionInvalid", "DbError",
        "DecorCannotBeRedeemed", "DecorItemNotDestroyable", "DecorNotFound",
        "DecorNotFoundInStorage", "DuplicateCharterSignature", "FilterRejected",
        "FixtureCantDeleteDoor", "FixtureHookEmpty", "FixtureHookOccupied",
        "FixtureHouseTypeMismatch", "FixtureNotFound", "FixtureNotOwned", "FixtureSizeMismatch",
        "FixtureTypeMismatch", "GenericFailure", "GuildMoreAccountsNeeded",
        "GuildMoreActivePlayersNeeded", "GuildNotLoaded", "HouseEditLockFailed",
        "HouseExteriorAlreadyThatSize", "HouseExteriorAlreadyThatType",
        "HouseExteriorRootNotFound", "HouseExteriorTypeNeighborhoodMismatch",
        "HouseExteriorTypeNotFound", "HouseExteriorTypeSizeMismatch",
        "HouseExteriorSizeNotAvailable", "HookNotChildOfFixture", "HouseNotFound",
        "IncorrectFaction", "InvalidDecorItem", "InvalidDistance", "InvalidGuild",
        "InvalidHouse", "InvalidInstance", "InvalidInteraction", "InvalidMap",
        "InvalidNeighborhoodName", "InvalidRoomLayout", "LockedByOtherPlayer",
        "LockOperationFailed", "MaxDecorReached", "MaxPreviewDecorReached",
        "MissingCoreFixture", "MissingDye", "MissingExpansionAccess",
        "MissingFactionMap", "MissingPrivateNeighborhoodInvite", "MoreHouseSlotsNeeded",
        "MoreSignaturesNeeded", "NeighborhoodNotFound",
        "NoNeighborhoodOwnershipRequests", "NotInDecorEditMode",
        "NotInFixtureEditMode", "NotInLayoutEditMode", "NotInsideHouse",
        "NotOnOwnedPlot", "OperationAborted", "OwnerNotInGuild",
        "PermissionDenied", "PlacementTargetInvalid", "PlayerNotFound",
        "PlayerNotInInstance", "PlotNotFound", "PlotNotVacant",
        "PlotReservationCooldown", "PlotReserved", "RoomNotFound",
        "RoomUpdateFailed", "RpcFailure", "ServiceNotAvailable",
        "StaticDataNotFound", "TimeoutLimit", "TimerunningNotAllowed",
        "TokenRequired", "TooManyRequests", "TransactionFailure",
        "UncollectedExteriorFixture", "UncollectedHouseType", "UncollectedRoom",
        "UncollectedRoomMaterial", "UncollectedRoomTheme", "UnlockOperationFailed",
    ],
);

pub const HOUSE_SETTING_FLAGS: EnumDef = (
    "HouseSettingFlags",
    &[
        ("HouseAccessAnyone", 1), ("HouseAccessNeighbors", 2),
        ("HouseAccessGuild", 4), ("HouseAccessFriends", 8), ("HouseAccessParty", 16),
    ],
);

pub const NEIGHBORHOOD_OWNER_TYPE: SeqEnumDef = (
    "NeighborhoodOwnerType",
    &["None", "Guild", "Charter"],
);

// ============================================================================
// Ping / Voice Enums
// ============================================================================

pub const PING_RESULT: SeqEnumDef = (
    "PingResult",
    &[
        "Success", "FailedGeneric", "FailedSpamming", "FailedDisabledByLeader",
        "FailedDisabledBySettings", "FailedOutOfPingArea", "FailedSquelched", "FailedUnspecified",
    ],
);

pub const VOICE_CHAT_STATUS_CODE: SeqEnumDef = (
    "VoiceChatStatusCode",
    &[
        "Success", "OperationPending", "TooManyRequests", "LoginProhibited",
        "ClientNotInitialized", "ClientNotLoggedIn", "ClientAlreadyLoggedIn",
        "ChannelNameTooShort", "ChannelNameTooLong", "ChannelAlreadyExists",
        "AlreadyInChannel", "TargetNotFound", "Failure", "ServiceLost",
        "UnableToLaunchProxy", "ProxyConnectionTimeOut", "ProxyConnectionUnableToConnect",
        "ProxyConnectionUnexpectedDisconnect", "Disabled", "UnsupportedChatChannelType",
        "InvalidCommunityStream", "PlayerSilenced", "PlayerVoiceChatParentalDisabled",
        "InvalidInputDevice", "InvalidOutputDevice",
    ],
);

// ============================================================================
// Talent / Trait Enums
// ============================================================================

pub const TRAIT_NODE_ENTRY_TYPE: SeqEnumDef = (
    "TraitNodeEntryType",
    &[
        "SpendHex", "SpendSquare", "SpendCircle", "SpendSmallCircle",
        "DeprecatedSelect", "DragAndDrop", "SpendDiamond", "ProfPath",
        "ProfPerk", "ProfPathUnlock", "RedButton", "ArmorSet",
        "SpendInfinite", "SpendCapstoneCircle", "SpendCapstoneSquare",
    ],
);

pub const TRAIT_DEFINITION_SUB_TYPE: SeqEnumDef = (
    "TraitDefinitionSubType",
    &["DragonflightRed", "DragonflightBlue", "DragonflightGreen", "DragonflightBronze", "DragonflightBlack"],
);

pub const TRAIT_EDGE_VISUAL_STYLE: SeqEnumDef = (
    "TraitEdgeVisualStyle",
    &["None", "Straight"],
);

// ============================================================================
// Currency / Token Enums
// ============================================================================

pub const ACCOUNT_CURRENCY_TRANSFER_RESULT: SeqEnumDef = (
    "AccountCurrencyTransferResult",
    &[
        "Success", "InvalidCharacter", "CharacterLoggedIn", "InsufficientCurrency",
        "MaxQuantity", "InvalidCurrency", "NoValidSourceCharacter", "ServerError",
        "CannotUseCurrency", "TransactionInProgress", "CurrencyTransferDisabled",
    ],
);

pub const CURRENCY_FILTER_TYPE: SeqEnumDef = (
    "CurrencyFilterType",
    &["None", "DiscoveredOnly", "DiscoveredAndAllAccountTransferable"],
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
    &["Quest", "UserWaypoint", "Corpse", "Scenario", "Content", "PartyMember", "MapPin", "Vignette"],
);

pub const SUPER_TRACKING_MAP_PIN_TYPE: SeqEnumDef = (
    "SuperTrackingMapPinType",
    &["AreaPOI", "QuestOffer", "TaxiNode", "DigSite", "HousingPlot"],
);
