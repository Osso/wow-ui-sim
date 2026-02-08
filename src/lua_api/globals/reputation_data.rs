//! Reputation/faction data for the WoW UI simulator.
//!
//! Provides a static list of factions with standings for the reputation frame UI.
//! The list is hierarchical: headers group factions into categories.

/// Standing levels (matches WoW's Reaction enum).
pub const HATED: i32 = 1;
pub const HOSTILE: i32 = 2;
pub const UNFRIENDLY: i32 = 3;
pub const NEUTRAL: i32 = 4;
pub const FRIENDLY: i32 = 5;
pub const HONORED: i32 = 6;
pub const REVERED: i32 = 7;
pub const EXALTED: i32 = 8;

/// A faction entry in the reputation list.
pub struct FactionEntry {
    pub faction_id: i32,
    pub name: &'static str,
    pub description: &'static str,
    pub reaction: i32,
    /// Current rep within this level.
    pub standing: i32,
    /// Rep needed to fill bar at this level.
    pub top_value: i32,
    pub is_header: bool,
    pub is_collapsed: bool,
    pub is_child: bool,
    pub is_account_wide: bool,
}

const fn faction_header(name: &'static str) -> FactionEntry {
    FactionEntry {
        faction_id: 0,
        name,
        description: "",
        reaction: 0,
        standing: 0,
        top_value: 0,
        is_header: true,
        is_collapsed: false,
        is_child: false,
        is_account_wide: false,
    }
}

const fn faction(
    faction_id: i32,
    name: &'static str,
    description: &'static str,
    reaction: i32,
    standing: i32,
    top_value: i32,
) -> FactionEntry {
    FactionEntry {
        faction_id,
        name,
        description,
        reaction,
        standing,
        top_value,
        is_header: false,
        is_collapsed: false,
        is_child: true,
        is_account_wide: false,
    }
}

/// Static faction list (headers + entries).
static FACTION_LIST: &[FactionEntry] = &[
    faction_header("The War Within"),
    faction(2590, "Council of Dornogal", "The governing body of Dornogal.", HONORED, 8200, 12000),
    faction(2570, "Hallowfall Arathi", "The Arathi settlers of Hallowfall.", REVERED, 4500, 21000),
    faction(2600, "The Assembly of the Deeps", "United denizens of the deep.", HONORED, 11000, 12000),
    faction(2605, "The Severed Threads", "A coalition of Nerubian outcasts.", FRIENDLY, 4800, 6000),
    faction_header("Dragonflight"),
    faction(2507, "Dragonscale Expedition", "Explorers of the Dragon Isles.", EXALTED, 999, 1000),
    faction(2510, "Valdrakken Accord", "The united dragonflights.", EXALTED, 999, 1000),
    faction(2511, "Iskaara Tuskarr", "The Tuskarr fishing community.", REVERED, 18000, 21000),
    faction(2503, "Maruuk Centaur", "The centaur clans of the Ohn'ahran Plains.", REVERED, 15000, 21000),
    faction_header("Shadowlands"),
    faction(2407, "The Ascended", "Servants of the Archon in Bastion.", HONORED, 3000, 12000),
    faction(2410, "The Undying Army", "Defenders of Maldraxxus.", FRIENDLY, 2000, 6000),
    faction(2413, "Court of Night", "Denizens of Ardenweald.", HONORED, 7500, 12000),
    faction_header("Classic"),
    faction(72, "Stormwind", "The Kingdom of Stormwind.", EXALTED, 999, 1000),
    faction(47, "Ironforge", "The Dwarven capital.", EXALTED, 999, 1000),
    faction(69, "Darnassus", "The Night Elf capital.", REVERED, 19000, 21000),
    faction(930, "Exodar", "The Draenei city.", HONORED, 9500, 12000),
    faction(1134, "Gilneas", "The Worgen homeland.", FRIENDLY, 3200, 6000),
];

/// Number of factions in the list (visible, considering expansion state).
pub fn num_factions() -> i32 {
    FACTION_LIST.len() as i32
}

/// Get a faction entry by 1-based index.
pub fn get_faction_by_index(index: i32) -> Option<&'static FactionEntry> {
    FACTION_LIST.get((index - 1) as usize)
}

/// Get a faction entry by faction ID.
pub fn get_faction_by_id(faction_id: i32) -> Option<&'static FactionEntry> {
    FACTION_LIST
        .iter()
        .find(|f| !f.is_header && f.faction_id == faction_id)
}

/// Get the first watched (non-header) faction.
pub fn watched_faction() -> Option<&'static FactionEntry> {
    // Return the first War Within faction as watched
    FACTION_LIST.iter().find(|f| f.faction_id == 2590)
}
