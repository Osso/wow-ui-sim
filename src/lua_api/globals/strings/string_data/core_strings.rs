//! Core string constants: errors, game values, UI categories, items, combat.

use super::{FloatDef, IntDef, StringDef};

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
    ("NUM_TOTAL_EQUIPPED_BAG_SLOTS", 5),
    ("BACKPACK_CONTAINER", 0),
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
// Autocomplete Priority Constants (LE_AUTOCOMPLETE_PRIORITY_*)
// ============================================================================

pub const AUTOCOMPLETE_CONSTANTS: &[IntDef] = &[
    ("LE_AUTOCOMPLETE_PRIORITY_OTHER", 0),
    ("LE_AUTOCOMPLETE_PRIORITY_INTERACTED", 1),
    ("LE_AUTOCOMPLETE_PRIORITY_IN_GROUP", 2),
    ("LE_AUTOCOMPLETE_PRIORITY_GUILD", 3),
    ("LE_AUTOCOMPLETE_PRIORITY_FRIEND", 4),
    ("LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER", 5),
    ("LE_AUTOCOMPLETE_PRIORITY_ACCOUNT_CHARACTER_SAME_REALM", 6),
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
    ("ADDON_LOAD_FAILED", "Addon %s could not be loaded: %s"),
    ("ADDON_DEPENDENCIES", "Dependencies"),
    ("ADDON_DEP_DISABLED", "Dependency Disabled"),
    ("ADDON_MISSING", "Missing"),
    ("ADDON_DISABLED", "Disabled"),
    ("ADDON_CORRUPT", "Corrupt"),
    ("ADDON_INCOMPATIBLE", "Incompatible"),
    ("ADDON_DEP_MISSING", "Dependency Missing"),
    ("ADDON_DEP_INCOMPATIBLE", "Dependency Incompatible"),
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
