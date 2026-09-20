//! Source-confirmed enum group shared by retail 12.1 and Forever.
use super::{BATTLE_NET_FRIEND_TAG, BATTLE_NET_FRIEND_TAG_META, EnumDef, SeqEnumDef};

pub const ENUMS: (&[EnumDef], &[SeqEnumDef]) = if cfg!(any(
    feature = "retail-12-1-0",
    feature = "client-wowforever"
)) {
    (
        &[
            BATTLE_NET_FRIEND_LEVEL,
            BATTLE_NET_FRIEND_TAG,
            BATTLE_NET_FRIEND_TAG_META,
            BATTLE_NET_FRIEND_LEVEL_META,
            VISUAL_ALERT_TYPE,
            VISUAL_ALERT_TYPE_META,
            COOLDOWN_VIEWER_SOUND_META,
        ],
        &[COOLDOWN_VIEWER_SOUND],
    )
} else {
    (&[], &[])
};

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
