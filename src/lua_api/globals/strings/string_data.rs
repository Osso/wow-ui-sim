//! Static string constant data for WoW UI.
//!
//! This file contains all UI string constants as static arrays,
//! separated from registration logic for maintainability.

/// String constant definition: (name, value)
pub type StringDef = (&'static str, &'static str);

/// Integer constant definition: (name, value)
pub type IntDef = (&'static str, i32);

/// Float constant definition: (name, value)
pub type FloatDef = (&'static str, f64);

// ============================================================================
// Error Message Strings (ERR_*)
// ============================================================================

pub const ERROR_STRINGS: &[StringDef] = &[
    ("ERR_CHAT_PLAYER_NOT_FOUND_S", "%s is not online"),
    ("ERR_NOT_IN_COMBAT", "You can't do that while in combat"),
    ("ERR_GENERIC_NO_TARGET", "You have no target"),
    ("ERR_FRIEND_OFFLINE_S", "%s is offline."),
    ("ERR_FRIEND_ONLINE_SS", "|Hplayer:%s|h[%s]|h has come online."),
    ("ERR_FRIEND_NOT_FOUND", "That player is not on your friends list."),
    ("ERR_FRIEND_ADDED_S", "%s added to friends."),
    ("ERR_FRIEND_REMOVED_S", "%s removed from friends."),
    ("ERR_IGNORE_ADDED_S", "%s added to ignore list."),
    ("ERR_IGNORE_REMOVED_S", "%s removed from ignore list."),
];

// ============================================================================
// Game Constants (NUM_*, MAX_*, etc.)
// ============================================================================

pub const GAME_INT_CONSTANTS: &[IntDef] = &[
    ("NUM_PET_ACTION_SLOTS", 10),
    ("NUM_ACTIONBAR_BUTTONS", 12),
    ("NUM_BAG_SLOTS", 5),
    ("MAX_SKILLLINE_TABS", 8),
    ("MAX_PLAYER_LEVEL", 80),
    ("MAX_NUM_TALENTS", 20),
    ("MAX_BOSS_FRAMES", 8),
    ("MAX_PARTY_MEMBERS", 4),
    ("MAX_RAID_MEMBERS", 40),
];

pub const GAME_STRING_CONSTANTS: &[StringDef] = &[
    ("BAGSLOTTEXT", "Bag Slot"),
    ("BOOKTYPE_SPELL", "spell"),
    ("BOOKTYPE_PET", "pet"),
];

// ============================================================================
// Expansion Constants (LE_EXPANSION_*)
// ============================================================================

pub const EXPANSION_CONSTANTS: &[IntDef] = &[
    ("LE_EXPANSION_CLASSIC", 0),
    ("LE_EXPANSION_BURNING_CRUSADE", 1),
    ("LE_EXPANSION_WRATH_OF_THE_LICH_KING", 2),
    ("LE_EXPANSION_CATACLYSM", 3),
    ("LE_EXPANSION_MISTS_OF_PANDARIA", 4),
    ("LE_EXPANSION_WARLORDS_OF_DRAENOR", 5),
    ("LE_EXPANSION_LEGION", 6),
    ("LE_EXPANSION_BATTLE_FOR_AZEROTH", 7),
    ("LE_EXPANSION_SHADOWLANDS", 8),
    ("LE_EXPANSION_DRAGONFLIGHT", 9),
    ("LE_EXPANSION_WAR_WITHIN", 10),
    ("LE_EXPANSION_LEVEL_CURRENT", 10),
];

// ============================================================================
// Inventory Slot Constants (INVSLOT_*)
// ============================================================================

pub const INVENTORY_SLOT_CONSTANTS: &[IntDef] = &[
    ("INVSLOT_AMMO", 0),
    ("INVSLOT_HEAD", 1),
    ("INVSLOT_NECK", 2),
    ("INVSLOT_SHOULDER", 3),
    ("INVSLOT_BODY", 4),
    ("INVSLOT_CHEST", 5),
    ("INVSLOT_WAIST", 6),
    ("INVSLOT_LEGS", 7),
    ("INVSLOT_FEET", 8),
    ("INVSLOT_WRIST", 9),
    ("INVSLOT_HAND", 10),
    ("INVSLOT_FINGER1", 11),
    ("INVSLOT_FINGER2", 12),
    ("INVSLOT_TRINKET1", 13),
    ("INVSLOT_TRINKET2", 14),
    ("INVSLOT_BACK", 15),
    ("INVSLOT_MAINHAND", 16),
    ("INVSLOT_OFFHAND", 17),
    ("INVSLOT_RANGED", 18),
    ("INVSLOT_TABARD", 19),
    ("INVSLOT_FIRST_EQUIPPED", 1),
    ("INVSLOT_LAST_EQUIPPED", 19),
];

// ============================================================================
// Raid Target Markers
// ============================================================================

pub const RAID_TARGET_STRINGS: &[StringDef] = &[
    ("RAID_TARGET_1", "Star"),
    ("RAID_TARGET_2", "Circle"),
    ("RAID_TARGET_3", "Diamond"),
    ("RAID_TARGET_4", "Triangle"),
    ("RAID_TARGET_5", "Moon"),
    ("RAID_TARGET_6", "Square"),
    ("RAID_TARGET_7", "Cross"),
    ("RAID_TARGET_8", "Skull"),
];

// ============================================================================
// Taxi/Flight Path Constants
// ============================================================================

pub const TAXI_FLOAT_CONSTANTS: &[FloatDef] = &[
    ("TAXIROUTE_LINEFACTOR", 128.0 / 126.0),
    ("TAXIROUTE_LINEFACTOR_2", 1.0),
];

// ============================================================================
// Keyboard Modifier Text
// ============================================================================

pub const KEYBOARD_MODIFIER_STRINGS: &[StringDef] = &[
    ("SHIFT_KEY_TEXT", "Shift"),
    ("ALT_KEY_TEXT", "Alt"),
    ("CTRL_KEY_TEXT", "Ctrl"),
];

// ============================================================================
// UI Category Strings
// ============================================================================

pub const UI_CATEGORY_STRINGS: &[StringDef] = &[
    ("SPECIALIZATION", "Specialization"),
    ("TALENT", "Talent"),
    ("ITEMS", "Items"),
    ("SPELLS", "Spells"),
    ("MOUNTS", "Mounts"),
    ("TOYS", "Toys"),
    ("PETS", "Pets"),
    ("HEIRLOOMS", "Heirlooms"),
    ("APPEARANCES", "Appearances"),
    ("TRANSMOG", "Transmog"),
    ("WARDROBE", "Wardrobe"),
    ("COLLECTIONS", "Collections"),
    ("ACHIEVEMENTS", "Achievements"),
    ("DUNGEONS", "Dungeons"),
    ("RAIDS", "Raids"),
    ("SCENARIO", "Scenario"),
    ("PVP", "PvP"),
    ("ARENA", "Arena"),
    ("BATTLEGROUND", "Battleground"),
    ("VENDOR", "Vendor"),
    ("MERCHANT", "Merchant"),
    ("TRAINER", "Trainer"),
    ("AUCTION_HOUSE", "Auction House"),
    ("GUILD_BANK", "Guild Bank"),
    ("MAIL", "Mail"),
    ("BANK", "Bank"),
    ("LOOT", "Loot"),
    ("TRADE", "Trade"),
    ("QUESTS", "Quests"),
    ("REPUTATION", "Reputation"),
    ("CURRENCY", "Currency"),
    ("PROFESSIONS", "Professions"),
    ("RECIPES", "Recipes"),
    ("NONE", "None"),
    ("DEFAULT", "Default"),
    ("UNKNOWN", "Unknown"),
    ("RETRIEVING_ITEM_INFO", "Retrieving item information"),
    ("RETRIEVING_DATA", "Retrieving data..."),
];

// ============================================================================
// UI Button Strings
// ============================================================================

pub const UI_BUTTON_STRINGS: &[StringDef] = &[
    ("YES", "Yes"),
    ("NO", "No"),
    ("OKAY", "Okay"),
    ("CANCEL", "Cancel"),
    ("ACCEPT", "Accept"),
    ("DECLINE", "Decline"),
    ("ENABLE", "Enable"),
    ("DISABLE", "Disable"),
    ("ADDON_LIST", "Addons"),
    ("ENABLE_ALL_ADDONS", "Enable All"),
    ("DISABLE_ALL_ADDONS", "Disable All"),
    ("ADDON_LOADED", "Loaded"),
    ("ADDON_DEPENDENCIES", "Dependencies"),
    ("ADDON_DEP_DISABLED", "Dependency Disabled"),
    ("HIGHLIGHTING", "Highlighting:"),
    ("READY", "Ready"),
    ("NOT_READY", "Not Ready"),
    ("BUSY", "Busy"),
    ("AFK", "Away"),
    ("DND", "Do Not Disturb"),
];

// ============================================================================
// Item Strings
// ============================================================================

pub const ITEM_STRINGS: &[StringDef] = &[
    ("SELL_PRICE", "Sell Price"),
    ("BUY_PRICE", "Buy Price"),
    ("PVP_ITEM_LEVEL_TOOLTIP", "PvP Item Level %d"),
    ("ITEM_UNIQUE_MULTIPLE", "Unique (%d)"),
    ("ITEM_UNIQUE", "Unique"),
    ("ITEM_UNIQUE_EQUIPPABLE", "Unique-Equipped"),
    ("ITEM_ACCOUNTBOUND", "Warbound"),
    ("ITEM_ACCOUNTBOUND_UNTIL_EQUIP", "Warbound until equipped"),
    ("ITEM_BNETACCOUNTBOUND", "Battle.net Account Bound"),
    ("ITEM_SOULBOUND", "Soulbound"),
    ("ITEM_BIND_ON_EQUIP", "Binds when equipped"),
    ("ITEM_BIND_ON_PICKUP", "Binds when picked up"),
    ("ITEM_BIND_ON_USE", "Binds when used"),
];

pub const SOCKET_STRINGS: &[StringDef] = &[
    ("EMPTY_SOCKET_BLUE", "blue socket"),
    ("EMPTY_SOCKET_RED", "red socket"),
    ("EMPTY_SOCKET_YELLOW", "yellow socket"),
    ("EMPTY_SOCKET_META", "meta socket"),
    ("EMPTY_SOCKET_PRISMATIC", "prismatic socket"),
    ("EMPTY_SOCKET_NO_COLOR", "prismatic socket"),
    ("EMPTY_SOCKET_COGWHEEL", "cogwheel socket"),
    ("EMPTY_SOCKET_HYDRAULIC", "sha-touched"),
    ("EMPTY_SOCKET_CYPHER", "crystallic socket"),
    ("EMPTY_SOCKET_DOMINATION", "domination socket"),
    ("EMPTY_SOCKET_PRIMORDIAL", "primordial socket"),
    ("EMPTY_SOCKET_PUNCHCARDBLUE", "blue punchcard socket"),
    ("EMPTY_SOCKET_PUNCHCARDRED", "red punchcard socket"),
    ("EMPTY_SOCKET_PUNCHCARDYELLOW", "yellow punchcard socket"),
    ("EMPTY_SOCKET_TINKER", "tinker socket"),
    ("EMPTY_SOCKET_SINGINGSEA", "singing sea socket"),
    ("EMPTY_SOCKET_SINGINGTHUNDER", "singing thunder socket"),
    ("EMPTY_SOCKET_SINGINGWIND", "singing wind socket"),
];

// ============================================================================
// Binding Header Strings
// ============================================================================

pub const BINDING_HEADER_STRINGS: &[StringDef] = &[
    ("BINDING_HEADER_RAID_TARGET", "Raid Target"),
    ("BINDING_HEADER_ACTIONBAR", "Action Bar"),
    ("BINDING_HEADER_MULTIACTIONBAR", "Multi-Action Bar"),
    ("BINDING_HEADER_MOVEMENT", "Movement"),
    ("BINDING_HEADER_CHAT", "Chat"),
    ("BINDING_HEADER_TARGETING", "Targeting"),
    ("BINDING_HEADER_INTERFACE", "Interface"),
    ("BINDING_HEADER_MISC", "Miscellaneous"),
    ("BINDING_HEADER_HOUSING_SYSTEM", "Housing System"),
    ("NOT_BOUND", "Not bound"),
    ("KEY_BOUND", "Key bound"),
];

// ============================================================================
// Misc UI Strings
// ============================================================================

pub const MISC_UI_STRINGS: &[StringDef] = &[
    ("SOURCE", "Source:"),
    ("APPEARANCE_LABEL", "Appearance"),
    ("COLOR", "Color"),
    ("COMPACT_UNIT_FRAME_PROFILE_SORTBY_ALPHABETICAL", "Alphabetical"),
    ("ITEM_QUALITY6_DESC", "Artifact"),
    ("ITEM_COOLDOWN_TIME", "%s Cooldown"),
    ("TOY", "Toy"),
    ("MOUNT", "Mount"),
    ("PET", "Pet"),
    ("EQUIPMENT", "Equipment"),
    ("REAGENT", "Reagent"),
    ("APPEARANCE", "Appearance"),
    ("TRANSMOG_SOURCE_LABEL", "Source:"),
    ("TRANSMOGRIFY", "Transmogrify"),
    ("WORLD_QUEST_REWARD_FILTERS_ANIMA", "Anima"),
    ("WORLD_QUEST_REWARD_FILTERS_EQUIPMENT", "Equipment"),
    ("WORLD_QUEST_REWARD_FILTERS_GOLD", "Gold"),
    ("WORLD_QUEST_REWARD_FILTERS_RESOURCES", "Resources"),
];

// ============================================================================
// Duel Strings
// ============================================================================

pub const DUEL_STRINGS: &[StringDef] = &[
    ("DUEL_WINNER_KNOCKOUT", "%1$s has defeated %2$s in a duel"),
    ("DUEL_WINNER_RETREAT", "%1$s has defeated %2$s in a duel (retreat)"),
];

// ============================================================================
// Loot Strings
// ============================================================================

pub const LOOT_STRINGS: &[StringDef] = &[
    ("LOOT_ITEM_PUSHED_SELF", "You receive loot: %s."),
    ("LOOT_ITEM_SELF", "You receive loot: %s."),
    ("LOOT_ITEM_PUSHED_SELF_MULTIPLE", "You receive loot: %sx%d."),
    ("LOOT_ITEM_SELF_MULTIPLE", "You receive loot: %sx%d."),
    ("CHANGED_OWN_ITEM", "Changed %s to %s."),
    ("LOOT_ITEM", "%s receives loot: %s."),
    ("LOOT_ITEM_MULTIPLE", "%s receives loot: %sx%d."),
    ("CURRENCY_GAINED", "You receive currency: %s."),
    ("CURRENCY_GAINED_MULTIPLE", "You receive currency: %s x%d."),
    ("YOU_LOOT_MONEY", "You loot %s"),
];

// ============================================================================
// XP and Quest Strings
// ============================================================================

pub const XP_QUEST_STRINGS: &[StringDef] = &[
    ("COMBATLOG_XPGAIN_EXHAUSTION1", "%s dies, you gain %d experience (+%d exp Rested bonus)."),
    ("COMBATLOG_XPGAIN_QUEST", "You gain %d experience (+%d exp bonus)."),
    ("COMBATLOG_XPGAIN_FIRSTPERSON", "%s dies, you gain %d experience."),
    ("COMBATLOG_XPGAIN_FIRSTPERSON_UNNAMED", "You gain %d experience."),
    ("ERR_QUEST_REWARD_EXP_I", "Experience gained: %d."),
    ("ERR_QUEST_REWARD_MONEY_S", "Received: %s"),
];

// ============================================================================
// Chat Format Strings
// ============================================================================

pub const CHAT_FORMAT_STRINGS: &[StringDef] = &[
    ("CHAT_MONSTER_SAY_GET", "%s says: "),
    ("CHAT_MONSTER_YELL_GET", "%s yells: "),
    ("CHAT_MONSTER_WHISPER_GET", "%s whispers: "),
    ("CHAT_SAY_GET", "%s says: "),
    ("CHAT_WHISPER_GET", "%s whispers: "),
    ("CHAT_WHISPER_INFORM_GET", "To %s: "),
    ("CHAT_BN_WHISPER_GET", "%s whispers: "),
    ("CHAT_BN_WHISPER_INFORM_GET", "To %s: "),
    ("ACHIEVEMENT_BROADCAST", "%s has earned the achievement %s!"),
    ("WHO_LIST_FORMAT", "%s - Level %d %s %s"),
    ("WHO_LIST_GUILD_FORMAT", "%s - Level %d %s %s <%s>"),
];

// ============================================================================
// Guild News Constants
// ============================================================================

pub const GUILD_NEWS_CONSTANTS: &[IntDef] = &[
    ("NEWS_ITEM_LOOTED", 0),
    ("NEWS_LEGENDARY_LOOTED", 1),
    ("NEWS_GUILD_ACHIEVEMENT", 2),
    ("NEWS_PLAYER_ACHIEVEMENT", 3),
    ("NEWS_DUNGEON_ENCOUNTER", 4),
    ("NEWS_GUILD_LEVEL", 5),
    ("NEWS_GUILD_CREATE", 6),
    ("NEWS_ITEM_CRAFTED", 7),
    ("NEWS_ITEM_PURCHASED", 8),
    ("NEWS_GUILD_MOTD", 9),
];

// ============================================================================
// Duration Strings
// ============================================================================

pub const DURATION_STRINGS: &[StringDef] = &[
    ("SPELL_DURATION_SEC", "%.1f sec"),
    ("SPELL_DURATION_MIN", "%.1f min"),
    ("SECONDS_ABBR", "%d sec"),
    ("MINUTES_ABBR", "%d min"),
    ("HOURS_ABBR", "%d hr"),
];

// ============================================================================
// Combat Text Strings
// ============================================================================

pub const COMBAT_TEXT_STRINGS: &[StringDef] = &[
    ("SHOW_COMBAT_HEALING", "Healing"),
    ("SHOW_COMBAT_HEALING_TEXT", "Show Healing"),
    ("SHOW_COMBAT_HEALING_ABSORB_SELF", "Self Absorbs"),
    ("SHOW_COMBAT_HEALING_ABSORB_TARGET", "Target Absorbs"),
    ("OPTION_TOOLTIP_SHOW_COMBAT_HEALING", "Show combat healing numbers"),
    ("OPTION_TOOLTIP_SHOW_COMBAT_HEALING_ABSORB_SELF", "Show self absorbs"),
    ("OPTION_TOOLTIP_SHOW_COMBAT_HEALING_ABSORB_TARGET", "Show target absorbs"),
    ("COMBAT_TEXT_SHOW_COMBO_POINTS_TEXT", "Combo Points"),
    ("COMBAT_TEXT_SHOW_FRIENDLY_NAMES_TEXT", "Friendly Names"),
    ("COMBAT_TEXT_SHOW_DODGE_PARRY_MISS_TEXT", "Dodge/Parry/Miss"),
    ("COMBAT_TEXT_SHOW_MANA_TEXT", "Show Mana"),
    ("COMBAT_TEXT_SHOW_HONOR_GAINED_TEXT", "Honor Gained"),
    ("COMBAT_TEXT_SHOW_REACTIVES_TEXT", "Reactives"),
    ("COMBAT_TEXT_SHOW_RESISTANCES_TEXT", "Resistances"),
    ("COMBAT_TEXT_SHOW_ENERGIZE_TEXT", "Energize"),
    ("TEXT_MODE_A_STRING_RESULT_OVERKILLING", "(Overkill)"),
    ("TEXT_MODE_A_STRING_RESULT_RESIST", "(Resisted)"),
    ("TEXT_MODE_A_STRING_RESULT_BLOCK", "(Blocked)"),
    ("TEXT_MODE_A_STRING_RESULT_ABSORB", "(Absorbed)"),
    ("TEXT_MODE_A_STRING_RESULT_CRITICAL", "(Critical)"),
];

pub const COMBAT_LOG_RAID_TARGET_CONSTANTS: &[IntDef] = &[
    ("COMBATLOG_OBJECT_RAIDTARGET1", 0x00100000),
    ("COMBATLOG_OBJECT_RAIDTARGET2", 0x00200000),
    ("COMBATLOG_OBJECT_RAIDTARGET3", 0x00400000),
    ("COMBATLOG_OBJECT_RAIDTARGET4", 0x00800000),
    ("COMBATLOG_OBJECT_RAIDTARGET5", 0x01000000),
    ("COMBATLOG_OBJECT_RAIDTARGET6", 0x02000000),
    ("COMBATLOG_OBJECT_RAIDTARGET7", 0x04000000),
    ("COMBATLOG_OBJECT_RAIDTARGET8", 0x08000000),
];

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
// Icon List Data (for register_icon_list)
// ============================================================================

pub const ICON_LIST_DATA: &[(&str, i32)] = &[
    ("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_1:", 1),
    ("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_2:", 2),
    ("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_3:", 3),
    ("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_4:", 4),
    ("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_5:", 5),
    ("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_6:", 6),
    ("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_7:", 7),
    ("|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_8:", 8),
];

// ============================================================================
// Item Quality Colors Data (for register_item_quality_colors)
// ============================================================================

/// (quality_index, r, g, b, hex)
pub const ITEM_QUALITY_COLORS_DATA: &[(i32, f64, f64, f64, &str)] = &[
    (0, 0.62, 0.62, 0.62, "ff9d9d9d"), // Poor (gray)
    (1, 1.00, 1.00, 1.00, "ffffffff"), // Common (white)
    (2, 0.12, 1.00, 0.00, "ff1eff00"), // Uncommon (green)
    (3, 0.00, 0.44, 0.87, "ff0070dd"), // Rare (blue)
    (4, 0.64, 0.21, 0.93, "ffa335ee"), // Epic (purple)
    (5, 1.00, 0.50, 0.00, "ffff8000"), // Legendary (orange)
    (6, 0.90, 0.80, 0.50, "ffe6cc80"), // Artifact (light gold)
    (7, 0.00, 0.80, 1.00, "ff00ccff"), // Heirloom (light blue)
    (8, 0.00, 0.80, 1.00, "ff00ccff"), // WoW Token
];

// ============================================================================
// Class Names Data (for register_class_name_tables)
// ============================================================================

pub const CLASS_NAMES_DATA: &[(&str, &str)] = &[
    ("WARRIOR", "Warrior"),
    ("PALADIN", "Paladin"),
    ("HUNTER", "Hunter"),
    ("ROGUE", "Rogue"),
    ("PRIEST", "Priest"),
    ("DEATHKNIGHT", "Death Knight"),
    ("SHAMAN", "Shaman"),
    ("MAGE", "Mage"),
    ("WARLOCK", "Warlock"),
    ("MONK", "Monk"),
    ("DRUID", "Druid"),
    ("DEMONHUNTER", "Demon Hunter"),
    ("EVOKER", "Evoker"),
];

// ============================================================================
// Tooltip Default Colors Data
// ============================================================================

/// (r, g, b, a)
pub const TOOLTIP_DEFAULT_COLOR: (f64, f64, f64, f64) = (1.0, 1.0, 1.0, 1.0);
pub const TOOLTIP_DEFAULT_BG_COLOR: (f64, f64, f64, f64) = (0.0, 0.0, 0.0, 1.0);

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

// ============================================================================
// Game Error String Constants (LE_GAME_ERR_*)
// These are string constants used as table keys in UIErrorsFrame.lua
// ============================================================================

pub const GAME_ERROR_STRINGS: &[StringDef] = &[
    ("LE_GAME_ERR_SPELL_FAILED_TOTEMS", "You don't have the required totem."),
    ("LE_GAME_ERR_SPELL_FAILED_EQUIPPED_ITEM", "You need to equip the required item."),
    ("LE_GAME_ERR_SPELL_ALREADY_KNOWN_S", "You already know %s."),
    ("LE_GAME_ERR_SPELL_FAILED_SHAPESHIFT_FORM_S", "Can't do that while %s."),
    ("LE_GAME_ERR_SPELL_FAILED_ALREADY_AT_FULL_MANA", "Already at full mana."),
    ("LE_GAME_ERR_OUT_OF_MANA", "Not enough mana."),
    ("LE_GAME_ERR_SPELL_OUT_OF_RANGE", "Out of range."),
    ("LE_GAME_ERR_SPELL_FAILED_S", "%s failed."),
    ("LE_GAME_ERR_SPELL_FAILED_REAGENTS", "Missing reagent."),
    ("LE_GAME_ERR_SPELL_FAILED_REAGENTS_GENERIC", "Missing reagent."),
    ("LE_GAME_ERR_SPELL_FAILED_NOTUNSHEATHED", "You need to unsheathe your weapon."),
    ("LE_GAME_ERR_SPELL_UNLEARNED_S", "You have unlearned %s."),
    ("LE_GAME_ERR_SPELL_FAILED_EQUIPPED_SPECIFIC_ITEM", "You need to equip a specific item."),
    ("LE_GAME_ERR_SPELL_FAILED_ALREADY_AT_FULL_POWER_S", "Already at full %s."),
    ("LE_GAME_ERR_SPELL_FAILED_EQUIPPED_ITEM_CLASS_S", "You need to equip a %s."),
    ("LE_GAME_ERR_SPELL_FAILED_ALREADY_AT_FULL_HEALTH", "Already at full health."),
    ("LE_GAME_ERR_SPELL_FAILED_CANT_FLY_HERE", "You can't fly here."),
    ("LE_GAME_ERR_GENERIC_NO_VALID_TARGETS", "No valid targets."),
    ("LE_GAME_ERR_ITEM_COOLDOWN", "Item is not ready yet."),
    ("LE_GAME_ERR_CANT_USE_ITEM", "You can't use that item."),
    ("LE_GAME_ERR_SPELL_FAILED_ANOTHER_IN_PROGRESS", "Another action is in progress."),
    ("LE_GAME_ERR_ABILITY_COOLDOWN", "Ability is not ready yet."),
    ("LE_GAME_ERR_SPELL_COOLDOWN", "Spell is not ready yet."),
    ("LE_GAME_ERR_OUT_OF_HOLY_POWER", "Not enough Holy Power."),
    ("LE_GAME_ERR_OUT_OF_POWER_DISPLAY", "Not enough power."),
    ("LE_GAME_ERR_OUT_OF_SOUL_SHARDS", "Not enough Soul Shards."),
    ("LE_GAME_ERR_OUT_OF_FOCUS", "Not enough Focus."),
    ("LE_GAME_ERR_OUT_OF_COMBO_POINTS", "Not enough Combo Points."),
    ("LE_GAME_ERR_OUT_OF_CHI", "Not enough Chi."),
    ("LE_GAME_ERR_OUT_OF_PAIN", "Not enough Pain."),
    ("LE_GAME_ERR_OUT_OF_HEALTH", "Not enough Health."),
    ("LE_GAME_ERR_OUT_OF_RAGE", "Not enough Rage."),
    ("LE_GAME_ERR_OUT_OF_ENERGY", "Not enough Energy."),
    ("LE_GAME_ERR_OUT_OF_ARCANE_CHARGES", "Not enough Arcane Charges."),
    ("LE_GAME_ERR_OUT_OF_RUNES", "Not enough Runes."),
    ("LE_GAME_ERR_OUT_OF_RUNIC_POWER", "Not enough Runic Power."),
    ("LE_GAME_ERR_OUT_OF_LUNAR_POWER", "Not enough Astral Power."),
    ("LE_GAME_ERR_OUT_OF_INSANITY", "Not enough Insanity."),
    ("LE_GAME_ERR_OUT_OF_MAELSTROM", "Not enough Maelstrom."),
    ("LE_GAME_ERR_OUT_OF_FURY", "Not enough Fury."),
    ("LE_GAME_ERR_OUT_OF_RANGE", "Out of range."),
    ("LE_GAME_ERR_OUT_OF_ESSENCE", "Not enough Essence."),
];
