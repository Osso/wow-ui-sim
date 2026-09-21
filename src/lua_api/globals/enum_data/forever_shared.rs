//! Source-confirmed enum group shared by retail 12.1 and Forever.
use super::{
    BATTLE_NET_FRIEND_TAG, BATTLE_NET_FRIEND_TAG_META, EnumDef, ROLODEX_TYPE_LEGACY_FRIEND,
    ROLODEX_TYPE_META, SOCIAL_SYSTEM_TYPE, SOCIAL_SYSTEM_TYPE_META, SOCIAL_UI_BLOCK_TYPE,
    SOCIAL_UI_BLOCK_TYPE_META, SOCIAL_UI_PRESENCE_TYPE, SOCIAL_UI_PRESENCE_TYPE_META, SeqEnumDef,
};

pub const ENUMS: (&[EnumDef], &[SeqEnumDef]) = if cfg!(any(
    feature = "retail-12-1-0",
    feature = "client-wowforever"
)) {
    (
        &[
            ACTION_BAR_SET,
            ACTION_BAR_SET_META,
            BATTLE_NET_FRIEND_LEVEL,
            BATTLE_NET_FRIEND_TAG,
            BATTLE_NET_FRIEND_TAG_META,
            BATTLE_NET_FRIEND_LEVEL_META,
            VISUAL_ALERT_TYPE,
            VISUAL_ALERT_TYPE_META,
            COOLDOWN_VIEWER_SOUND_META,
            SOCIAL_UI_PRESENCE_TYPE,
            SOCIAL_UI_PRESENCE_TYPE_META,
            SOCIAL_SYSTEM_TYPE_META,
            SOCIAL_UI_BLOCK_TYPE,
            SOCIAL_UI_BLOCK_TYPE_META,
            ROLODEX_TYPE_LEGACY_FRIEND,
            ROLODEX_TYPE_META,
        ],
        &[COOLDOWN_VIEWER_SOUND, SOCIAL_SYSTEM_TYPE],
    )
} else {
    (&[], &[])
};

const ACTION_BAR_SET: EnumDef = (
    "ActionBarSet",
    &[("None", 0), ("Mkb", 1), ("Gamepad", 2), ("All", 3)],
);

const ACTION_BAR_SET_META: EnumDef = (
    "ActionBarSetMeta",
    &[("MinValue", 0), ("MaxValue", 3), ("NumValues", 4)],
);

// Forever 1.60.1.69913 CombatAudioAlertSharedDocumentation.lua.
#[cfg(feature = "client-wowforever")]
pub(super) const COMBAT_AUDIO_ALERT_PULSE_PERCENT_VALUES: EnumDef = (
    "CombatAudioAlertPulsePercentValues",
    &[
        ("Off", 0),
        ("Under90Percent", 1),
        ("Under80Percent", 2),
        ("Under70Percent", 3),
        ("Under60Percent", 4),
        ("Under50Percent", 5),
        ("Under40Percent", 6),
        ("Under30Percent", 7),
        ("Under20Percent", 8),
        ("Under10Percent", 9),
    ],
);

#[cfg(feature = "client-wowforever")]
pub(super) const COMBAT_AUDIO_ALERT_PULSE_PERCENT_VALUES_META: EnumDef = (
    "CombatAudioAlertPulsePercentValuesMeta",
    &[("MinValue", 0), ("MaxValue", 9), ("NumValues", 10)],
);

#[cfg(feature = "client-wowforever")]
pub(super) const RECENT_ALLIES_INTERACTION_CATEGORY_FILTER: EnumDef = (
    "RecentAlliesInteractionCategoryFilter",
    &[
        ("Professions", 0),
        ("PvP", 1),
        ("Raiding", 2),
        ("Dungeons", 3),
        ("Delves", 4),
        ("Questing", 5),
    ],
);

#[cfg(feature = "client-wowforever")]
pub(super) const RECENT_ALLIES_INTERACTION_CATEGORY_FILTER_META: EnumDef = (
    "RecentAlliesInteractionCategoryFilterMeta",
    &[("MinValue", 0), ("MaxValue", 5), ("NumValues", 6)],
);

const COOLDOWN_VIEWER_SOUND: SeqEnumDef = (
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

const COOLDOWN_VIEWER_SOUND_META: EnumDef = (
    "CooldownViewerSoundMeta",
    &[("MinValue", 0), ("MaxValue", 93), ("NumValues", 94)],
);

const BATTLE_NET_FRIEND_LEVEL: EnumDef = (
    "BattleNetFriendLevel",
    &[("BattleTag", 1), ("RealID", 2), ("Title", 3)],
);

const BATTLE_NET_FRIEND_LEVEL_META: EnumDef = (
    "BattleNetFriendLevelMeta",
    &[("MinValue", 1), ("MaxValue", 3), ("NumValues", 3)],
);

const VISUAL_ALERT_TYPE: EnumDef = (
    "VisualAlertType",
    &[
        ("MarchingAnts", 1),
        ("MarchingAntsCyan", 2),
        ("MarchingAntsRed", 3),
        ("MarchingAntsGreen", 4),
        ("MarchingAntsBlue", 5),
        ("Flash", 6),
        ("FlashCyan", 7),
        ("FlashRed", 8),
        ("FlashGreen", 9),
        ("FlashBlue", 10),
    ],
);

const VISUAL_ALERT_TYPE_META: EnumDef = (
    "VisualAlertTypeMeta",
    &[("MinValue", 1), ("MaxValue", 10), ("NumValues", 10)],
);
