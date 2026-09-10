use super::*;

pub(in crate::lua_api::state) fn default_lfg_category_info()
-> std::collections::HashMap<i32, LfgCategoryInfo> {
    let cat = |name: &str, order: i32, allow_cross_faction: bool| -> LfgCategoryInfo {
        LfgCategoryInfo {
            name: name.into(),
            order,
            separate_recommended: false,
            prefer_current_area: false,
            allow_cross_faction,
            auto_choose_activity: false,
            show_playstyle_dropdown: false,
        }
    };
    let mut map = std::collections::HashMap::new();
    map.insert(2, cat("Dungeons", 1, true));
    map.insert(3, cat("Raids", 2, true));
    map.insert(4, cat("Arenas", 3, false));
    map.insert(6, cat("Custom", 4, true));
    map.insert(9, cat("Battlegrounds", 5, false));
    map
}

pub(in crate::lua_api::state) fn default_lfg_activity_groups() -> Vec<LfgActivityGroupInfo> {
    vec![
        lfg_activity_group(295, 2, "The War Within Mythic+", 1, LFG_FILTER_PVE),
        lfg_activity_group(296, 2, "The War Within Heroic Dungeons", 2, LFG_FILTER_PVE),
        lfg_activity_group(320, 3, "The War Within Raids", 1, LFG_FILTER_PVE),
        lfg_activity_group(350, 4, "Arenas", 1, LFG_FILTER_PVP),
        lfg_activity_group(360, 9, "Rated Battlegrounds", 1, LFG_FILTER_PVP),
        lfg_activity_group(400, 6, "Custom — World Content", 1, LFG_FILTER_PVE),
    ]
}

const LFG_FILTER_PVE: u32 = 1;
const LFG_FILTER_PVP: u32 = 2;
const LFG_MIN_LEVEL_TWW: i32 = 80;
const LFG_MIN_LEVEL_PVP: i32 = 70;

#[derive(Clone, Copy)]
struct LfgActivitySeed {
    activity_id: u32,
    group_id: u32,
    category_id: i32,
    full_name: &'static str,
    short_name: &'static str,
    min_level: i32,
    max_players: i32,
    filters: u32,
    order_index: i32,
    use_honor_level: bool,
    difficulty_id: i32,
    allow_cross_faction: bool,
    is_current_raid_activity: bool,
}

impl From<LfgActivitySeed> for LfgActivityInfo {
    fn from(seed: LfgActivitySeed) -> Self {
        Self {
            activity_id: seed.activity_id,
            group_id: seed.group_id,
            category_id: seed.category_id,
            full_name: seed.full_name.into(),
            short_name: seed.short_name.into(),
            min_level: seed.min_level,
            max_players: seed.max_players,
            item_level: 0,
            filters: seed.filters,
            display_type: 0,
            order_index: seed.order_index,
            use_honor_level: seed.use_honor_level,
            difficulty_id: seed.difficulty_id,
            allow_cross_faction: seed.allow_cross_faction,
            is_current_raid_activity: seed.is_current_raid_activity,
        }
    }
}

fn lfg_activity_group(
    group_id: u32,
    category_id: i32,
    name: &str,
    order_index: i32,
    filters: u32,
) -> LfgActivityGroupInfo {
    LfgActivityGroupInfo {
        group_id,
        category_id,
        name: name.into(),
        order_index,
        filters,
    }
}

pub(in crate::lua_api::state) fn default_lfg_activities() -> Vec<LfgActivityInfo> {
    mythic_plus_activity_seeds()
        .into_iter()
        .chain(raid_activity_seeds())
        .chain(world_content_activity_seeds())
        .chain(pvp_activity_seeds())
        .map(LfgActivityInfo::from)
        .collect()
}

fn mythic_plus_activity_seeds() -> [LfgActivitySeed; 3] {
    [
        pve_activity(1195, 295, 2, "Mists of Tirna Scithe (M+)", "MoTS M+", 5, 1),
        pve_activity(1188, 295, 2, "The Stonevault (M+)", "Stonevault M+", 5, 2),
        pve_activity(
            1190,
            295,
            2,
            "Ara-Kara, City of Echoes (M+)",
            "Ara-Kara M+",
            5,
            3,
        ),
    ]
}

fn raid_activity_seeds() -> [LfgActivitySeed; 2] {
    [
        raid_activity(1296, "Nerub-ar Palace (Heroic)", "Nerub-ar HC", 1, 15, true),
        raid_activity(1295, "Nerub-ar Palace (Normal)", "Nerub-ar N", 2, 14, true),
    ]
}

fn world_content_activity_seeds() -> [LfgActivitySeed; 2] {
    [
        pve_activity(
            1350,
            400,
            6,
            "World Boss — Aggregation of Horrors",
            "World Boss",
            40,
            1,
        ),
        pve_activity(
            1700,
            400,
            6,
            "World Quests — Khaz Algar",
            "World Quests",
            5,
            2,
        ),
    ]
}

fn pvp_activity_seeds() -> [LfgActivitySeed; 3] {
    [
        pvp_activity(491, 350, 4, "2v2 Arena Skirmish", "2v2 Arena", 2, 1),
        pvp_activity(492, 350, 4, "3v3 Arena Skirmish", "3v3 Arena", 3, 2),
        pvp_activity(493, 360, 9, "Rated Battlegrounds", "RBG", 10, 1),
    ]
}

fn pve_activity(
    activity_id: u32,
    group_id: u32,
    category_id: i32,
    full_name: &'static str,
    short_name: &'static str,
    max_players: i32,
    order_index: i32,
) -> LfgActivitySeed {
    LfgActivitySeed {
        activity_id,
        group_id,
        category_id,
        full_name,
        short_name,
        min_level: LFG_MIN_LEVEL_TWW,
        max_players,
        filters: LFG_FILTER_PVE,
        order_index,
        use_honor_level: false,
        difficulty_id: 0,
        allow_cross_faction: true,
        is_current_raid_activity: false,
    }
}

fn pvp_activity(
    activity_id: u32,
    group_id: u32,
    category_id: i32,
    full_name: &'static str,
    short_name: &'static str,
    max_players: i32,
    order_index: i32,
) -> LfgActivitySeed {
    LfgActivitySeed {
        activity_id,
        group_id,
        category_id,
        full_name,
        short_name,
        min_level: LFG_MIN_LEVEL_PVP,
        max_players,
        filters: LFG_FILTER_PVP,
        order_index,
        use_honor_level: true,
        difficulty_id: 0,
        allow_cross_faction: false,
        is_current_raid_activity: false,
    }
}

fn raid_activity(
    activity_id: u32,
    full_name: &'static str,
    short_name: &'static str,
    order_index: i32,
    difficulty_id: i32,
    is_current_raid_activity: bool,
) -> LfgActivitySeed {
    LfgActivitySeed {
        difficulty_id,
        is_current_raid_activity,
        ..pve_activity(activity_id, 320, 3, full_name, short_name, 20, order_index)
    }
}

#[derive(Clone, Copy)]
struct LfdDungeonSeed {
    dungeon_id: i32,
    name: &'static str,
    type_id: i32,
    subtype_id: i32,
    min_level: i32,
    max_level: i32,
    rec_level: i32,
    max_players: i32,
    expansion_level: i32,
    texture_filename: &'static str,
    description: &'static str,
    is_random: bool,
    is_follower_dungeon: bool,
}

impl From<LfdDungeonSeed> for LfdDungeonInfo {
    fn from(seed: LfdDungeonSeed) -> Self {
        Self {
            dungeon_id: seed.dungeon_id,
            name: seed.name.into(),
            type_id: seed.type_id,
            subtype_id: seed.subtype_id,
            min_level: seed.min_level,
            max_level: seed.max_level,
            rec_level: seed.rec_level,
            min_rec_level: seed.rec_level,
            max_rec_level: seed.rec_level,
            expansion_level: seed.expansion_level,
            group_id: 0,
            texture_filename: seed.texture_filename.into(),
            difficulty: 0,
            max_players: seed.max_players,
            description: seed.description.into(),
            is_holiday: false,
            min_players: 1,
            map_name: seed.name.into(),
            min_gear: 0,
            is_scaling_dungeon: false,
            is_random: seed.is_random,
            is_follower_dungeon: seed.is_follower_dungeon,
            training_ground_kind: None,
        }
    }
}

const LFD_RANDOM_DUNGEON_SEEDS: [LfdDungeonSeed; 2] = [
    // Header row (negative id). Retail data keeps headers as categories;
    // only positive-id "random heroic dungeon" rows carry is_random=true.
    LfdDungeonSeed {
        dungeon_id: -1,
        name: "Random Heroic Dungeons",
        type_id: 6,
        subtype_id: 1,
        min_level: 80,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "",
        description: "",
        is_random: false,
        is_follower_dungeon: false,
    },
    // GetRandomDungeonBestChoice returns this id; the header above must stay
    // non-random so the LFDQueueFrame_SetType("specific") fallback works.
    LfdDungeonSeed {
        dungeon_id: 999,
        name: "Random Heroic Dungeon",
        type_id: 6,
        subtype_id: 2,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-BACKGROUND-HEROIC",
        description: "A random heroic dungeon.",
        is_random: true,
        is_follower_dungeon: false,
    },
];

const LFD_WAR_WITHIN_DUNGEON_SEEDS: [LfdDungeonSeed; 8] = [
    LfdDungeonSeed {
        dungeon_id: 1201,
        name: "Ara-Kara, City of Echoes",
        type_id: 2,
        subtype_id: 2,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-DUNGEON-ARAKARA",
        description: "A spider-city dungeon.",
        is_random: false,
        is_follower_dungeon: false,
    },
    LfdDungeonSeed {
        dungeon_id: 1202,
        name: "City of Threads",
        type_id: 2,
        subtype_id: 2,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-DUNGEON-CITYOFTHREADS",
        description: "Nerubian city.",
        is_random: false,
        is_follower_dungeon: true,
    },
    LfdDungeonSeed {
        dungeon_id: 1203,
        name: "Mists of Tirna Scithe",
        type_id: 2,
        subtype_id: 2,
        min_level: 60,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-DUNGEON-MISTSOFTIRNA",
        description: "A fae forest dungeon.",
        is_random: false,
        is_follower_dungeon: false,
    },
    LfdDungeonSeed {
        dungeon_id: 1204,
        name: "The Stonevault",
        type_id: 2,
        subtype_id: 2,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-DUNGEON-STONEVAULT",
        description: "Earthen vault.",
        is_random: false,
        is_follower_dungeon: true,
    },
    LfdDungeonSeed {
        dungeon_id: 1205,
        name: "Grim Batol",
        type_id: 2,
        subtype_id: 2,
        min_level: 15,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-DUNGEON-GRIMBATOL",
        description: "Ancient dragon bastion.",
        is_random: false,
        is_follower_dungeon: false,
    },
    LfdDungeonSeed {
        dungeon_id: 1206,
        name: "The Dawnbreaker",
        type_id: 2,
        subtype_id: 2,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-DUNGEON-DAWNBREAKER",
        description: "Arathi warship.",
        is_random: false,
        is_follower_dungeon: false,
    },
    LfdDungeonSeed {
        dungeon_id: 1207,
        name: "Darkflame Cleft",
        type_id: 2,
        subtype_id: 2,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-DUNGEON-DARKFLAMECLEFT",
        description: "A torch-lit cavern.",
        is_random: false,
        is_follower_dungeon: true,
    },
    LfdDungeonSeed {
        dungeon_id: 1208,
        name: "The Rookery",
        type_id: 2,
        subtype_id: 2,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 5,
        expansion_level: 10,
        texture_filename: "Interface/LFGFRAME/UI-LFG-DUNGEON-ROOKERY",
        description: "Stormrook fortress.",
        is_random: false,
        is_follower_dungeon: false,
    },
];

const LFR_DUNGEON_SEEDS: [LfdDungeonSeed; 2] = [
    LfdDungeonSeed {
        dungeon_id: 416,
        name: "Mogu'shan Vaults",
        type_id: 3,
        subtype_id: 1,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 25,
        expansion_level: 4,
        texture_filename: "Interface/LFGFRAME/UI-LFR-BACKGROUND-MOGUSHANVAULTS",
        description: "Enter the ancient mogu vaults in Raid Finder.",
        is_random: false,
        is_follower_dungeon: false,
    },
    LfdDungeonSeed {
        dungeon_id: 417,
        name: "Heart of Fear",
        type_id: 3,
        subtype_id: 1,
        min_level: 70,
        max_level: 80,
        rec_level: 80,
        max_players: 25,
        expansion_level: 4,
        texture_filename: "Interface/LFGFRAME/UI-LFR-BACKGROUND-HEARTOFFEAR",
        description: "Assault the mantid heartland in Raid Finder.",
        is_random: false,
        is_follower_dungeon: false,
    },
];

pub(in crate::lua_api::state) fn default_lfd_dungeons() -> Vec<LfdDungeonInfo> {
    LFD_RANDOM_DUNGEON_SEEDS
        .into_iter()
        .chain(LFD_WAR_WITHIN_DUNGEON_SEEDS)
        .chain(LFR_DUNGEON_SEEDS)
        .map(LfdDungeonInfo::from)
        .collect()
}

// Seed the `SimState.auction_browse_results` list with two
// representative Browse-tab rows (a crafting mat and a gear piece).
