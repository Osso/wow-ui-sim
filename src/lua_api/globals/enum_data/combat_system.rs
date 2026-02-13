//! Combat, encounter, damage meter, and spell enum data.

use super::{EnumDef, SeqEnumDef};

// ============================================================================
// Combat Audio Alert Enums
// ============================================================================

pub const COMBAT_AUDIO_ALERT_CATEGORY: SeqEnumDef = (
    "CombatAudioAlertCategory",
    &[
        "General", "PlayerHealth", "TargetHealth", "PartyHealth",
        "PlayerResource1", "PlayerResource2", "PlayerCast", "TargetCast",
        "PlayerDebuffs",
    ],
);

pub const COMBAT_AUDIO_ALERT_TYPE: SeqEnumDef = (
    "CombatAudioAlertType",
    &["Health", "Cast"],
);

pub const COMBAT_AUDIO_ALERT_THROTTLE: SeqEnumDef = (
    "CombatAudioAlertThrottle",
    &[
        "Sample", "PlayerHealth", "TargetHealth", "PlayerCast",
        "TargetCast", "PlayerResource1", "PlayerResource2",
    ],
);

pub const COMBAT_AUDIO_ALERT_UNIT: SeqEnumDef = (
    "CombatAudioAlertUnit",
    &["Player", "Target"],
);

pub const COMBAT_AUDIO_ALERT_SPEC_SETTING: SeqEnumDef = (
    "CombatAudioAlertSpecSetting",
    &[
        "Resource1Percent", "Resource1Format", "Resource2Percent",
        "Resource2Format", "SayIfTargeted",
    ],
);

// ============================================================================
// Combat Log Object Enums (bitmask flags)
// ============================================================================

pub const COMBAT_LOG_OBJECT: EnumDef = (
    "CombatLogObject",
    &[
        ("Empty", 0),
        ("AffiliationMine", 1),
        ("AffiliationParty", 2),
        ("AffiliationRaid", 4),
        ("AffiliationOutsider", 8),
        ("ReactionFriendly", 16),
        ("ReactionNeutral", 32),
        ("ReactionHostile", 64),
        ("ControlPlayer", 256),
        ("ControlNpc", 512),
        ("TypePlayer", 1024),
        ("TypeNpc", 2048),
        ("TypePet", 4096),
        ("TypeGuardian", 8192),
        ("TypeObject", 16384),
        ("Target", 65536),
        ("Focus", 131072),
        ("Maintank", 262144),
        ("Mainassist", 524288),
        ("None", -2147483648),  // 0x80000000
    ],
);

pub const COMBAT_LOG_OBJECT_TARGET: EnumDef = (
    "CombatLogObjectTarget",
    &[
        ("Raidtarget1", 1),
        ("Raidtarget2", 2),
        ("Raidtarget3", 4),
        ("Raidtarget4", 8),
        ("Raidtarget5", 16),
        ("Raidtarget6", 32),
        ("Raidtarget7", 64),
        ("Raidtarget8", 128),
        ("RaidNone", -2147483648),
    ],
);

// ============================================================================
// Damage Class Enum (bitmask for spell schools)
// ============================================================================

pub const DAMAGECLASS: EnumDef = (
    "Damageclass",
    &[
        ("MaskNone", 0),
        ("MaskPhysical", 1),
        ("MaskHoly", 2),
        ("MaskFire", 4),
        ("MaskNature", 8),
        ("MaskFrost", 16),
        ("MaskShadow", 32),
        ("MaskArcane", 64),
    ],
);

// ============================================================================
// Cooldown Viewer Enums
// ============================================================================

pub const COOLDOWN_VIEWER_ALERT_TYPE: SeqEnumDef = (
    "CooldownViewerAlertType",
    &["Sound", "Visual"],
);

pub const COOLDOWN_VIEWER_ALERT_EVENT_TYPE: SeqEnumDef = (
    "CooldownViewerAlertEventType",
    &["Available", "PandemicTime", "OnCooldown", "ChargeGained", "OnAuraApplied", "OnAuraRemoved"],
);

pub const COOLDOWN_VIEWER_ALERT_EVENT_TYPE_META: EnumDef = (
    "CooldownViewerAlertEventTypeMeta",
    &[("MinValue", 0), ("MaxValue", 5), ("NumValues", 6)],
);

pub const COOLDOWN_VIEWER_ADD_ALERT_STATUS: SeqEnumDef = (
    "CooldownViewerAddAlertStatus",
    &["InvalidAlertType", "InvalidEventType", "Success"],
);

// ============================================================================
// Damage Meter Enums
// ============================================================================

pub const DAMAGE_METER_SESSION_TYPE: SeqEnumDef = (
    "DamageMeterSessionType",
    &["Overall", "Current", "Expired"],
);

pub const DAMAGE_METER_TYPE: SeqEnumDef = (
    "DamageMeterType",
    &[
        "DamageDone", "Dps", "HealingDone", "Hps", "Absorbs",
        "Interrupts", "Dispels", "DamageTaken", "AvoidableDamageTaken",
        "Deaths", "EnemyDamageTaken",
    ],
);

// ============================================================================
// Encounter Timeline / Warnings Enums
// ============================================================================

pub const ENCOUNTER_TIMELINE_TRACK: SeqEnumDef = (
    "EncounterTimelineTrack",
    &["Queued", "Short", "Medium", "Long", "Indeterminate"],
);

pub const ENCOUNTER_TIMELINE_EVENT_STATE: SeqEnumDef = (
    "EncounterTimelineEventState",
    &["Active", "Paused", "Finished", "Canceled"],
);

pub const ENCOUNTER_EVENT_SEVERITY: SeqEnumDef = (
    "EncounterEventSeverity",
    &["Low", "Medium", "High"],
);

pub const ENCOUNTER_TIMELINE_ICON_SET: SeqEnumDef = (
    "EncounterTimelineIconSet",
    &["TankAlert", "HealerAlert", "DamageAlert", "Deadly", "Dispel", "Enrage"],
);

pub const ENCOUNTER_TIMELINE_VIEW_TYPE: SeqEnumDef = (
    "EncounterTimelineViewType",
    &["Timeline", "Bars"],
);

pub const STATUS_BAR_TIMER_DIRECTION: SeqEnumDef = (
    "StatusBarTimerDirection",
    &["RemainingTime", "ElapsedTime"],
);

// ============================================================================
// Spell Aura Enums
// ============================================================================

pub const SPELL_AURA_VISIBILITY_TYPE: SeqEnumDef = (
    "SpellAuraVisibilityType",
    &["RaidInCombat", "RaidOutOfCombat", "EnemyTarget"],
);

// ============================================================================
// Lua Curve Type Enum
// ============================================================================

pub const LUA_CURVE_TYPE: SeqEnumDef = (
    "LuaCurveType",
    &["Bezier", "Linear"],
);

// ============================================================================
// Transmog Outfit Enums (stubs to prevent C_TransmogOutfitInfo errors)
// ============================================================================

pub const TRANSMOG_OUTFIT_FLAGS: SeqEnumDef = (
    "TransmogOutfitFlags",
    &["None", "NoMainhand", "NoOffhand"],
);

// ============================================================================
// Prey / Hunt / UI Widget State Enums
// ============================================================================

pub const PREY_HUNT_PROGRESS_STATE: SeqEnumDef = (
    "PreyHuntProgressState",
    &["Cold", "Warm", "Hot", "Final"],
);

// ============================================================================
// Status Bar Enums
// ============================================================================

pub const STATUS_BAR_FILL_STYLE: SeqEnumDef = (
    "StatusBarFillStyle",
    &["Standard", "StandardNoRangeFill", "Center", "Reverse"],
);

// ============================================================================
// Raid Dispel Display
// ============================================================================

pub const RAID_DISPEL_DISPLAY_TYPE: SeqEnumDef = (
    "RaidDispelDisplayType",
    &["Disabled", "DispellableByMe", "DisplayAll"],
);

// ============================================================================
// Combat Audio Alert Percent Values
// ============================================================================

pub const COMBAT_AUDIO_ALERT_PERCENT_VALUES: SeqEnumDef = (
    "CombatAudioAlertPercentValues",
    &["Off", "Every10Percent", "Every20Percent", "Every30Percent",
      "Every40Percent", "Every50Percent"],
);

pub const COMBAT_AUDIO_ALERT_PERCENT_VALUES_META: EnumDef = (
    "CombatAudioAlertPercentValuesMeta",
    &[("MinValue", 0), ("MaxValue", 5), ("NumValues", 6)],
);

pub const COMBAT_AUDIO_ALERT_CAST_STATE: SeqEnumDef = (
    "CombatAudioAlertCastState",
    &["Off", "OnCastStart", "OnCastEnd"],
);

pub const COMBAT_AUDIO_ALERT_CAST_STATE_META: EnumDef = (
    "CombatAudioAlertCastStateMeta",
    &[("MinValue", 0), ("MaxValue", 2), ("NumValues", 3)],
);

pub const COMBAT_AUDIO_ALERT_PLAYER_RESOURCE_FORMAT_VALUES: SeqEnumDef = (
    "CombatAudioAlertPlayerResourceFormatValues",
    &["ResourceFull", "ResourceNoPercent", "ResourceNoPercentDiv10",
      "NoResourceFull", "NoResourceNoPercent", "NoResourceNoPercentDiv10"],
);

pub const COMBAT_AUDIO_ALERT_PLAYER_RESOURCE_FORMAT_VALUES_META: EnumDef = (
    "CombatAudioAlertPlayerResourceFormatValuesMeta",
    &[("MinValue", 0), ("MaxValue", 5), ("NumValues", 6)],
);

// All remaining CombatAudioAlert Meta enums (used by TextToSpeechCommands)

pub const COMBAT_AUDIO_ALERT_CATEGORY_META: EnumDef = (
    "CombatAudioAlertCategoryMeta",
    &[("MinValue", 0), ("MaxValue", 8), ("NumValues", 9)],
);
pub const COMBAT_AUDIO_ALERT_TYPE_META: EnumDef = (
    "CombatAudioAlertTypeMeta",
    &[("MinValue", 0), ("MaxValue", 1), ("NumValues", 2)],
);
pub const COMBAT_AUDIO_ALERT_THROTTLE_META: EnumDef = (
    "CombatAudioAlertThrottleMeta",
    &[("MinValue", 0), ("MaxValue", 6), ("NumValues", 7)],
);
pub const COMBAT_AUDIO_ALERT_UNIT_META: EnumDef = (
    "CombatAudioAlertUnitMeta",
    &[("MinValue", 0), ("MaxValue", 1), ("NumValues", 2)],
);
pub const COMBAT_AUDIO_ALERT_SPEC_SETTING_META: EnumDef = (
    "CombatAudioAlertSpecSettingMeta",
    &[("MinValue", 0), ("MaxValue", 4), ("NumValues", 5)],
);
pub const COMBAT_AUDIO_ALERT_PARTY_PERCENT_VALUES_META: EnumDef = (
    "CombatAudioAlertPartyPercentValuesMeta",
    &[("MinValue", 0), ("MaxValue", 10), ("NumValues", 11)],
);
pub const COMBAT_AUDIO_ALERT_PLAYER_CAST_FORMAT_VALUES_META: EnumDef = (
    "CombatAudioAlertPlayerCastFormatValuesMeta",
    &[("MinValue", 0), ("MaxValue", 4), ("NumValues", 5)],
);
pub const COMBAT_AUDIO_ALERT_PLAYER_HEALTH_FORMAT_VALUES_META: EnumDef = (
    "CombatAudioAlertPlayerHealthFormatValuesMeta",
    &[("MinValue", 0), ("MaxValue", 5), ("NumValues", 6)],
);
pub const COMBAT_AUDIO_ALERT_SAY_IF_TARGETED_TYPE_META: EnumDef = (
    "CombatAudioAlertSayIfTargetedTypeMeta",
    &[("MinValue", 0), ("MaxValue", 3), ("NumValues", 4)],
);
pub const COMBAT_AUDIO_ALERT_TARGET_CAST_FORMAT_VALUES_META: EnumDef = (
    "CombatAudioAlertTargetCastFormatValuesMeta",
    &[("MinValue", 0), ("MaxValue", 6), ("NumValues", 7)],
);
pub const COMBAT_AUDIO_ALERT_TARGET_DEATH_BEHAVIOR_META: EnumDef = (
    "CombatAudioAlertTargetDeathBehaviorMeta",
    &[("MinValue", 0), ("MaxValue", 1), ("NumValues", 2)],
);
pub const COMBAT_AUDIO_ALERT_TARGET_HEALTH_FORMAT_VALUES_META: EnumDef = (
    "CombatAudioAlertTargetHealthFormatValuesMeta",
    &[("MinValue", 0), ("MaxValue", 8), ("NumValues", 9)],
);
pub const COMBAT_AUDIO_ALERT_DEBUFF_SELF_ALERT_VALUES_META: EnumDef = (
    "CombatAudioAlertDebuffSelfAlertValuesMeta",
    &[("MinValue", 0), ("MaxValue", 1), ("NumValues", 2)],
);
pub const COMBAT_AUDIO_ALERT_PLAYER_DEBUFF_FORMAT_VALUES_META: EnumDef = (
    "CombatAudioAlertPlayerDebuffFormatValuesMeta",
    &[("MinValue", 0), ("MaxValue", 1), ("NumValues", 2)],
);

// ============================================================================
// NamePlate Style
// ============================================================================

pub const NAME_PLATE_STYLE: SeqEnumDef = (
    "NamePlateStyle",
    &["Modern", "Thin", "Block", "HealthFocus", "CastFocus", "Legacy"],
);
