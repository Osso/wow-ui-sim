//! Additional string constants: HUD, unit frames, fonts, LFG, stats, misc.

use super::{IntDef, StringDef};

// ============================================================================
// HUD Edit Mode Strings
// ============================================================================

pub const HUD_EDIT_MODE_STRINGS: &[StringDef] = &[
    ("HUD_EDIT_MODE_CAST_BAR_LABEL", "Cast Bar"),
    ("HUD_EDIT_MODE_PLAYER_FRAME_LABEL", "Player Frame"),
    ("HUD_EDIT_MODE_TARGET_FRAME_LABEL", "Target Frame"),
    ("HUD_EDIT_MODE_FOCUS_FRAME_LABEL", "Focus Frame"),
    ("HUD_EDIT_MODE_MINIMAP_LABEL", "Minimap"),
    ("HUD_EDIT_MODE_ACTION_BAR_LABEL", "Action Bar %d"),
    ("HUD_EDIT_MODE_STANCE_BAR_LABEL", "Stance Bar"),
    ("HUD_EDIT_MODE_PET_ACTION_BAR_LABEL", "Pet Action Bar"),
    ("HUD_EDIT_MODE_POSSESS_ACTION_BAR_LABEL", "Possess Bar"),
    ("HUD_EDIT_MODE_CHAT_FRAME_LABEL", "Chat Frame"),
    ("HUD_EDIT_MODE_BUFFS_LABEL", "Buffs"),
    ("HUD_EDIT_MODE_DEBUFFS_LABEL", "Debuffs"),
    ("HUD_EDIT_MODE_OBJECTIVE_TRACKER_LABEL", "Objectives"),
    ("HUD_EDIT_MODE_BOSS_FRAMES_LABEL", "Boss Frames"),
    ("HUD_EDIT_MODE_ARENA_FRAMES_LABEL", "Arena Frames"),
    ("HUD_EDIT_MODE_PARTY_FRAMES_LABEL", "Party Frames"),
    ("HUD_EDIT_MODE_RAID_FRAMES_LABEL", "Raid Frames"),
    ("HUD_EDIT_MODE_VEHICLE_LEAVE_BUTTON_LABEL", "Vehicle Exit"),
    ("HUD_EDIT_MODE_ENCOUNTER_BAR_LABEL", "Encounter Bar"),
    ("HUD_EDIT_MODE_EXTRA_ACTION_BUTTON_LABEL", "Extra Action Button"),
    ("HUD_EDIT_MODE_ZONE_ABILITY_FRAME_LABEL", "Zone Ability"),
    ("HUD_EDIT_MODE_BAGS_LABEL", "Bags"),
    ("HUD_EDIT_MODE_MICRO_MENU_LABEL", "Micro Menu"),
    ("HUD_EDIT_MODE_TALKING_HEAD_FRAME_LABEL", "Talking Head"),
    ("HUD_EDIT_MODE_DURABILITY_FRAME_LABEL", "Durability"),
    ("HUD_EDIT_MODE_STATUS_TRACKING_BAR_LABEL", "Status Bars"),
    ("HUD_EDIT_MODE_EXPERIENCE_BAR_LABEL", "Experience Bar"),
    ("HUD_EDIT_MODE_HUD_TOOLTIP_LABEL", "HUD Tooltip"),
    ("HUD_EDIT_MODE_TIMER_BARS_LABEL", "Timer Bars"),
    ("BAG_NAME_BACKPACK", "Backpack"),
    ("LOSS_OF_CONTROL", "Loss of Control"),
    ("COOLDOWN_VIEWER_LABEL", "Cooldown Viewer"),
];

// ============================================================================
// Unit Frame Strings
// ============================================================================

pub const UNIT_FRAME_STRINGS: &[StringDef] = &[
    ("FOCUS", "Focus"),
    ("TARGET", "Target"),
    ("PLAYER", "Player"),
    ("PARTY", "Party"),
    ("RAID", "Raid"),
    ("BOSS", "Boss"),
    ("SHOW_TARGET_OF_TARGET_TEXT", "Target of Target"),
    ("TARGET_OF_TARGET", "Target of Target"),
    ("FOCUS_FRAME_LABEL", "Focus Frame"),
    ("HEALTH", "Health"),
    ("MANA", "Mana"),
    ("RAGE", "Rage"),
    ("ENERGY", "Energy"),
    ("POWER_TYPE_FOCUS", "Focus"),
    ("RUNIC_POWER", "Runic Power"),
    ("SOUL_SHARDS", "Soul Shards"),
    ("SOUL_SHARDS_POWER", "Soul Shards"),
    ("HOLY_POWER", "Holy Power"),
    ("CHI", "Chi"),
    ("CHI_POWER", "Chi"),
    ("INSANITY", "Insanity"),
    ("MAELSTROM", "Maelstrom"),
    ("FURY", "Fury"),
    ("PAIN", "Pain"),
    ("LUNAR_POWER", "Astral Power"),
    ("COMBO_POINTS", "Combo Points"),
    ("COMBO_POINTS_POWER", "Combo Points"),
    ("ARCANE_CHARGES", "Arcane Charges"),
    ("POWER_TYPE_ARCANE_CHARGES", "Arcane Charges"),
    ("POWER_TYPE_ESSENCE", "Essence"),
    ("RUNES", "Runes"),
    ("CLEAR_ALL", "Clear All"),
    ("SHARE_QUEST_ABBREV", "Share"),
    ("BUFFOPTIONS_LABEL", "Buffs and Debuffs"),
    ("DEBUFFOPTIONS_LABEL", "Debuffs"),
    ("BUFFFRAME_LABEL", "Buff Frame"),
    ("DEBUFFFRAME_LABEL", "Debuff Frame"),
    ("UNIT_NAME_FRIENDLY_TOTEMS", "Friendly Totems"),
];

// ============================================================================
// Font Paths
// ============================================================================

pub const FONT_PATH_STRINGS: &[StringDef] = &[
    ("STANDARD_TEXT_FONT", "Fonts\\FRIZQT__.TTF"),
    ("UNIT_NAME_FONT", "Fonts\\FRIZQT__.TTF"),
    ("UNIT_NAME_FONT_CHINESE", "Fonts\\ARKai_T.TTF"),
    ("UNIT_NAME_FONT_CYRILLIC", "Fonts\\FRIZQT___CYR.TTF"),
    ("UNIT_NAME_FONT_KOREAN", "Fonts\\2002.TTF"),
    ("DAMAGE_TEXT_FONT", "Fonts\\FRIZQT__.TTF"),
    ("NAMEPLATE_FONT", "Fonts\\FRIZQT__.TTF"),
];

// ============================================================================
// LFG Strings
// ============================================================================

pub const LFG_STRINGS: &[StringDef] = &[
    ("GROUP_FINDER", "Group Finder"),
    ("STAT_CATEGORY_PVP", "PvP"),
];

pub const LFG_TYPE_STRINGS: &[StringDef] = &[
    ("LFG_TYPE_ZONE", "Zone"),
    ("LFG_TYPE_DUNGEON", "Dungeon"),
    ("LFG_TYPE_RAID", "Raid"),
    ("LFG_TYPE_HEROIC_DUNGEON", "Heroic Dungeon"),
    ("DUNGEONS_BUTTON", "Dungeons"),
    ("RAIDS_BUTTON", "Raids"),
    ("SCENARIOS_BUTTON", "Scenarios"),
    ("PLAYER_V_PLAYER", "Player vs. Player"),
    ("LFG_LIST_LOADING", "Loading..."),
    ("LFG_LIST_SEARCH_PLACEHOLDER", "Enter search..."),
];

pub const LFG_ERROR_STRINGS: &[StringDef] = &[
    ("ERR_LFG_PROPOSAL_FAILED", "The dungeon finder proposal failed."),
    ("ERR_LFG_PROPOSAL_DECLINED", "A player declined the dungeon finder proposal."),
    ("ERR_LFG_ROLE_CHECK_FAILED", "The role check failed."),
    ("ERR_LFG_NO_SLOTS_PLAYER", "You are not in a valid slot."),
    ("ERR_LFG_NO_SLOTS_PARTY", "Your party is not in a valid slot."),
    ("ERR_LFG_MISMATCHED_SLOTS", "You do not meet the requirements for that dungeon."),
    ("ERR_LFG_DESERTER_PLAYER", "You cannot queue because you have the Deserter debuff."),
];

// ============================================================================
// Stat Strings
// ============================================================================

pub const STAT_STRINGS: &[StringDef] = &[
    ("STAT_ARMOR", "Armor"),
    ("STAT_STRENGTH", "Strength"),
    ("STAT_AGILITY", "Agility"),
    ("STAT_STAMINA", "Stamina"),
    ("STAT_INTELLECT", "Intellect"),
    ("STAT_SPIRIT", "Spirit"),
];

pub const ITEM_MOD_STRINGS: &[StringDef] = &[
    // Primary stats
    ("ITEM_MOD_STRENGTH", "Strength"),
    ("ITEM_MOD_STRENGTH_SHORT", "Strength"),
    ("ITEM_MOD_AGILITY", "Agility"),
    ("ITEM_MOD_AGILITY_SHORT", "Agility"),
    ("ITEM_MOD_STAMINA", "Stamina"),
    ("ITEM_MOD_STAMINA_SHORT", "Stamina"),
    ("ITEM_MOD_INTELLECT", "Intellect"),
    ("ITEM_MOD_INTELLECT_SHORT", "Intellect"),
    ("ITEM_MOD_SPIRIT", "Spirit"),
    ("ITEM_MOD_SPIRIT_SHORT", "Spirit"),
    // Secondary stats
    ("ITEM_MOD_CRIT_RATING", "Critical Strike"),
    ("ITEM_MOD_CRIT_RATING_SHORT", "Critical Strike"),
    ("ITEM_MOD_HASTE_RATING", "Haste"),
    ("ITEM_MOD_HASTE_RATING_SHORT", "Haste"),
    ("ITEM_MOD_MASTERY_RATING", "Mastery"),
    ("ITEM_MOD_MASTERY_RATING_SHORT", "Mastery"),
    ("ITEM_MOD_VERSATILITY", "Versatility"),
    // Tertiary and other stats
    ("ITEM_MOD_CR_AVOIDANCE_SHORT", "Avoidance"),
    ("ITEM_MOD_CR_LIFESTEAL_SHORT", "Leech"),
    ("ITEM_MOD_CR_SPEED_SHORT", "Speed"),
    ("ITEM_MOD_CR_STURDINESS_SHORT", "Indestructible"),
    ("ITEM_MOD_ATTACK_POWER_SHORT", "Attack Power"),
    ("ITEM_MOD_SPELL_POWER_SHORT", "Spell Power"),
    ("ITEM_MOD_BLOCK_RATING_SHORT", "Block"),
    ("ITEM_MOD_DODGE_RATING_SHORT", "Dodge"),
    ("ITEM_MOD_PARRY_RATING_SHORT", "Parry"),
    ("ITEM_MOD_HIT_RATING_SHORT", "Hit"),
    ("ITEM_MOD_EXTRA_ARMOR_SHORT", "Bonus Armor"),
    ("ITEM_MOD_PVP_POWER_SHORT", "PvP Power"),
    ("ITEM_MOD_RESILIENCE_RATING_SHORT", "PvP Resilience"),
    ("ITEM_MOD_MANA_SHORT", "Mana"),
    ("ITEM_MOD_MANA_REGENERATION_SHORT", "Mana Regeneration"),
    ("ITEM_MOD_HEALTH_REGENERATION_SHORT", "Health Regeneration"),
    ("ITEM_MOD_DAMAGE_PER_SECOND_SHORT", "Damage Per Second"),
    ("ITEM_MOD_CRAFTING_SPEED_SHORT", "Crafting Speed"),
    ("ITEM_MOD_MULTICRAFT_SHORT", "Multicraft"),
    ("ITEM_MOD_RESOURCEFULNESS_SHORT", "Resourcefulness"),
    ("ITEM_MOD_PERCEPTION_SHORT", "Perception"),
    ("ITEM_MOD_DEFTNESS_SHORT", "Deftness"),
    ("ITEM_MOD_FINESSE_SHORT", "Finesse"),
];

// ============================================================================
// Slash Commands
// ============================================================================

pub const SLASH_COMMAND_STRINGS: &[StringDef] = &[
    ("SLASH_CAST1", "/cast"),
    ("SLASH_CAST2", "/spell"),
    ("SLASH_CAST3", "/use"),
    ("SLASH_CAST4", "/castrandom"),
    ("SLASH_CASTSEQUENCE1", "/castsequence"),
    ("SLASH_CASTRANDOM1", "/castrandom"),
    ("SLASH_CLICK1", "/click"),
    ("SLASH_TARGET1", "/target"),
    ("SLASH_TARGET2", "/tar"),
    ("SLASH_FOCUS1", "/focus"),
    ("SLASH_ASSIST1", "/assist"),
    ("SLASH_FOLLOW1", "/follow"),
    ("SLASH_FOLLOW2", "/fol"),
    ("SLASH_PET_ATTACK1", "/petattack"),
    ("SLASH_PET_FOLLOW1", "/petfollow"),
    ("SLASH_PET_PASSIVE1", "/petpassive"),
    ("SLASH_PET_DEFENSIVE1", "/petdefensive"),
    ("SLASH_PET_AGGRESSIVE1", "/petaggressive"),
    ("SLASH_PET_STAY1", "/petstay"),
    ("SLASH_EQUIP1", "/equip"),
    ("SLASH_EQUIPSLOT1", "/equipslot"),
    ("SLASH_USETALENTS1", "/usetalents"),
    ("SLASH_STOPCASTING1", "/stopcasting"),
    ("SLASH_STOPATTACK1", "/stopattack"),
    ("SLASH_CANCELAURA1", "/cancelaura"),
    ("SLASH_CANCELFORM1", "/cancelform"),
    ("SLASH_DISMOUNT1", "/dismount"),
    ("SLASH_STARTATTACK1", "/startattack"),
];

// ============================================================================
// Binding Names
// ============================================================================

pub const BINDING_NAME_STRINGS: &[StringDef] = &[
    ("BINDING_NAME_EXTRAACTIONBUTTON1", "Extra Action Button"),
    ("BINDING_NAME_BONUSACTIONBUTTON1", "Bonus Action Button"),
    ("BINDING_NAME_ACTIONBUTTON1", "Action Button 1"),
    ("BINDING_NAME_ACTIONBUTTON2", "Action Button 2"),
    ("BINDING_NAME_ACTIONBUTTON3", "Action Button 3"),
    ("BINDING_NAME_ACTIONBUTTON4", "Action Button 4"),
    ("BINDING_NAME_ACTIONBUTTON5", "Action Button 5"),
    ("BINDING_NAME_ACTIONBUTTON6", "Action Button 6"),
    ("BINDING_NAME_ACTIONBUTTON7", "Action Button 7"),
    ("BINDING_NAME_ACTIONBUTTON8", "Action Button 8"),
    ("BINDING_NAME_ACTIONBUTTON9", "Action Button 9"),
    ("BINDING_NAME_ACTIONBUTTON10", "Action Button 10"),
    ("BINDING_NAME_ACTIONBUTTON11", "Action Button 11"),
    ("BINDING_NAME_ACTIONBUTTON12", "Action Button 12"),
];

// ============================================================================
// Loot Error Strings
// ============================================================================

pub const LOOT_ERROR_STRINGS: &[StringDef] = &[
    ("ERR_LOOT_GONE", "Item is no longer available (already looted)"),
    ("ERR_LOOT_NOTILE", "You are too far away to loot that corpse."),
    ("ERR_LOOT_DIDNT_KILL", "You didn't kill that creature."),
    ("ERR_LOOT_ROLL_PENDING", "You cannot loot while the roll is pending."),
    ("ERR_LOOT_WHILE_INVULNERABLE", "You can't loot while invulnerable."),
];

// ============================================================================
// Instance Strings
// ============================================================================

pub const INSTANCE_STRINGS: &[StringDef] = &[
    ("INSTANCE_SAVED", "You are now saved to this instance."),
    ("TRANSFER_ABORT_TOO_MANY_INSTANCES", "You have entered too many instances recently."),
    ("NO_RAID_INSTANCES_SAVED", "You are not saved to any raid instances."),
];

// ============================================================================
// Objective Tracker Strings
// ============================================================================

pub const OBJECTIVE_TRACKER_STRINGS: &[StringDef] = &[
    ("OBJECTIVES_WATCH_TOO_MANY", "You are tracking too many quests."),
    ("OBJECTIVES_TRACKER_LABEL", "Objectives"),
    ("TRACKER_HEADER_WORLD_QUESTS", "World Quests"),
    ("TRACKER_HEADER_BONUS_OBJECTIVES", "Bonus Objectives"),
    ("TRACKER_HEADER_SCENARIO", "Scenario"),
    ("TRACKER_HEADER_OBJECTIVE", "Objective"),
    ("TRACKER_HEADER_PROVINGGROUNDS", "Proving Grounds"),
    ("TRACKER_HEADER_DUNGEON", "Dungeon"),
    ("TRACKER_HEADER_DELVES", "Delves"),
    ("TRACKER_HEADER_CAMPAIGN_QUESTS", "Campaign"),
    ("TRACKER_HEADER_QUESTS", "Quests"),
];

// ============================================================================
// Character Strings
// ============================================================================

pub const CHARACTER_STRINGS: &[StringDef] = &[
    ("CLASS", "Class"),
    ("RACE", "Race"),
    ("LEVEL", "Level"),
    ("GUILD", "Guild"),
    ("REALM", "Realm"),
    ("OFFLINE", "Offline"),
    ("ONLINE", "Online"),
];

// ============================================================================
// Tooltip Strings
// ============================================================================

pub const TOOLTIP_STRINGS: &[StringDef] = &[
    ("TOOLTIP_UNIT_LEVEL", "Level %s"),
    ("TOOLTIP_UNIT_LEVEL_TYPE", "Level %s %s"),
    ("TOOLTIP_UNIT_LEVEL_CLASS", "Level %s %s"),
    ("TOOLTIP_UNIT_LEVEL_RACE_CLASS", "Level %s %s %s"),
    ("ELITE", "Elite"),
    ("RARE", "Rare"),
    ("RAREELITE", "Rare Elite"),
    ("WORLDBOSS", "Boss"),
];

// ============================================================================
// Item Requirement Strings
// ============================================================================

pub const ITEM_REQUIREMENT_STRINGS: &[StringDef] = &[
    ("ITEM_REQ_SKILL", "Requires %s"),
    ("ITEM_REQ_REPUTATION", "Requires %s - %s"),
    ("ITEM_REQ_ALLIANCE", "Alliance"),
    ("ITEM_REQ_HORDE", "Horde"),
    ("ITEM_MIN_LEVEL", "Requires Level %d"),
    ("ITEM_LEVEL", "Item Level %d"),
    ("ITEM_CLASSES_ALLOWED", "Classes: %s"),
    ("ITEM_RACES_ALLOWED", "Races: %s"),
];

// ============================================================================
// Achievement Strings
// ============================================================================

pub const ACHIEVEMENT_STRINGS: &[StringDef] = &[
    ("ACHIEVEMENT_UNLOCKED", "Achievement Unlocked"),
    ("ACHIEVEMENT_POINTS", "Achievement Points"),
    ("TITLES", "Titles"),
    ("TRANSMOG_SETS", "Transmog Sets"),
];

// ============================================================================
// Currency Strings
// ============================================================================

pub const CURRENCY_STRINGS: &[StringDef] = &[
    ("CURRENCY_GAINED_MULTIPLE_BONUS", "You receive currency: %s x%d (Bonus Roll)."),
    ("CURRENCY_TOTAL", "Total: %s"),
];

// ============================================================================
// Spell Error Strings
// ============================================================================

pub const SPELL_ERROR_STRINGS: &[StringDef] = &[
    ("SPELL_FAILED_CUSTOM_ERROR_1029", "Requires Skyriding"),
    ("SPELL_FAILED_NOT_READY", "Spell is not ready"),
    ("SPELL_FAILED_BAD_TARGETS", "Invalid target"),
    ("SPELL_FAILED_NO_VALID_TARGETS", "No valid targets"),
];

// ============================================================================
// Item Upgrade Strings
// ============================================================================

pub const ITEM_UPGRADE_STRINGS: &[StringDef] = &[
    ("UPGRADE", "Upgrade"),
    ("UPGRADE_ITEM", "Upgrade Item"),
    ("UPGRADE_LEVEL", "Upgrade Level"),
];

// ============================================================================
// Spellbook/Encounter Strings
// ============================================================================

pub const SPELLBOOK_STRINGS: &[StringDef] = &[
    ("SPELLBOOK_AVAILABLE_AT", "Available at level %d"),
    ("ENCOUNTER_JOURNAL", "Adventure Guide"),
    ("ENCOUNTER_JOURNAL_ENCOUNTER", "Encounter"),
    ("ENCOUNTER_JOURNAL_DUNGEON", "Dungeon"),
    ("ENCOUNTER_JOURNAL_RAID", "Raid"),
    ("GARRISON_LOCATION_TOOLTIP", "Garrison"),
    ("GARRISON_SHIPYARD", "Shipyard"),
    ("GARRISON_MISSION_COMPLETE", "Mission Complete"),
    ("GARRISON_FOLLOWER", "Follower"),
    ("RAID_BOSSES", "Raid Bosses"),
    ("RAID_INSTANCES", "Raid Instances"),
    ("DUNGEON_BOSSES", "Dungeon Bosses"),
    ("DUNGEON_INSTANCES", "Dungeon Instances"),
    ("WORLD", "World"),
    ("ZONE", "Zone"),
    ("SPECIAL", "Special"),
    ("TUTORIAL_TITLE20", "Tutorial"),
    ("CALENDAR_FILTER_WEEKLY_HOLIDAYS", "Weekly Holidays"),
    ("CHALLENGE_MODE", "Challenge Mode"),
    ("PLAYER_DIFFICULTY_MYTHIC_PLUS", "Mythic+"),
    ("PLAYER_DIFFICULTY1", "Normal"),
    ("PLAYER_DIFFICULTY2", "Heroic"),
    ("PLAYER_DIFFICULTY3", "Mythic"),
    ("PLAYER_DIFFICULTY4", "LFR"),
    ("PLAYER_DIFFICULTY5", "Challenge"),
    ("PLAYER_DIFFICULTY6", "Timewalking"),
];

// ============================================================================
// Dungeon Difficulty Strings
// ============================================================================

pub const DUNGEON_DIFFICULTY_STRINGS: &[StringDef] = &[
    ("DUNGEON_DIFFICULTY1", "Normal"),
    ("DUNGEON_DIFFICULTY2", "Heroic"),
    ("DUNGEON_DIFFICULTY_NORMAL", "Normal"),
    ("DUNGEON_DIFFICULTY_HEROIC", "Heroic"),
    ("DUNGEON_DIFFICULTY_MYTHIC", "Mythic"),
    ("RAID_DIFFICULTY1", "10 Player"),
    ("RAID_DIFFICULTY2", "25 Player"),
    ("RAID_DIFFICULTY3", "10 Player (Heroic)"),
    ("RAID_DIFFICULTY4", "25 Player (Heroic)"),
    ("INSTANCE_RESET_SUCCESS", "%s has been reset."),
    ("INSTANCE_RESET_FAILED", "Cannot reset %s. There are players still inside the instance."),
    ("INSTANCE_RESET_FAILED_OFFLINE", "Cannot reset %s. There are players offline in your party."),
    ("ERR_RAID_DIFFICULTY_CHANGED_S", "Raid difficulty changed to %s."),
    ("ERR_DUNGEON_DIFFICULTY_CHANGED_S", "Dungeon difficulty changed to %s."),
];

// ============================================================================
// Font Color Codes
// ============================================================================

pub const FONT_COLOR_CODE_STRINGS: &[StringDef] = &[
    ("NORMAL_FONT_COLOR_CODE", "|cffffd100"),
    ("HIGHLIGHT_FONT_COLOR_CODE", "|cffffffff"),
    ("RED_FONT_COLOR_CODE", "|cffff2020"),
    ("GREEN_FONT_COLOR_CODE", "|cff20ff20"),
    ("GRAY_FONT_COLOR_CODE", "|cff808080"),
    ("YELLOW_FONT_COLOR_CODE", "|cffffff00"),
    ("LIGHTYELLOW_FONT_COLOR_CODE", "|cffffff9a"),
    ("ORANGE_FONT_COLOR_CODE", "|cffff8040"),
    ("ACHIEVEMENT_COLOR_CODE", "|cffffff00"),
    ("BATTLENET_FONT_COLOR_CODE", "|cff82c5ff"),
    ("DISABLED_FONT_COLOR_CODE", "|cff808080"),
    ("FONT_COLOR_CODE_CLOSE", "|r"),
    ("LINK_FONT_COLOR_CODE", "|cff00ccff"),
];

// ============================================================================
// Item Binding Strings
// ============================================================================

pub const ITEM_BINDING_STRINGS: &[StringDef] = &[
    ("BIND_TRADE_TIME_REMAINING", "You may trade this item with players that were also eligible to loot this item for the next %s."),
    ("BIND_ON_PICKUP", "Binds when picked up"),
    ("BIND_ON_EQUIP", "Binds when equipped"),
    ("BIND_ON_USE", "Binds when used"),
    ("BIND_TO_ACCOUNT", "Binds to Blizzard account"),
    ("BIND_TO_BNETACCOUNT", "Binds to Battle.net account"),
];

// ============================================================================
// Time Strings
// ============================================================================

pub const TIME_STRINGS: &[StringDef] = &[
    ("DAY_ONELETTER_ABBR", "%dd"),
    ("HOUR_ONELETTER_ABBR", "%dh"),
    ("MINUTE_ONELETTER_ABBR", "%dm"),
    ("SECOND_ONELETTER_ABBR", "%ds"),
    ("DAYS_ABBR", "%d Days"),
    ("HOURS_ABBR", "%d Hours"),
    ("MINUTES_ABBR", "%d Min"),
    ("SECONDS_ABBR", "%d Sec"),
    ("DAYS", "Days"),
    ("HOURS", "Hours"),
    ("MINUTES", "Minutes"),
    ("SECONDS", "Seconds"),
];

// ============================================================================
// Totem Slot Constants
// ============================================================================

pub const TOTEM_SLOT_CONSTANTS: &[IntDef] = &[
    ("FIRE_TOTEM_SLOT", 1),
    ("EARTH_TOTEM_SLOT", 2),
    ("WATER_TOTEM_SLOT", 3),
    ("AIR_TOTEM_SLOT", 4),
    ("MAX_TOTEMS", 4),
];

// ============================================================================
// LFG Category Constants
// ============================================================================

pub const LFG_CATEGORY_CONSTANTS: &[IntDef] = &[
    ("LE_LFG_CATEGORY_LFD", 1),
    ("LE_LFG_CATEGORY_RF", 2),
    ("LE_LFG_CATEGORY_SCENARIO", 3),
    ("LE_LFG_CATEGORY_LFR", 4),
    ("LE_LFG_CATEGORY_FLEXRAID", 5),
    ("LE_LFG_CATEGORY_WORLDPVP", 6),
    ("LE_LFG_CATEGORY_BATTLEFIELD", 7),
];
