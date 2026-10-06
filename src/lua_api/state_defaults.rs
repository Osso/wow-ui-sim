//! Default data for WorldState and PlayerState collections.
//!
//! Separated from state_types.rs to keep type definitions concise.

use super::state_types::*;
use std::collections::HashMap;

pub(super) fn default_equipped_items() -> HashMap<i32, EquippedItem> {
    let e = |item_id| EquippedItem {
        item_id,
        enchant_id: 0,
        gem_ids: [0, 0, 0],
    };
    [
        (1, e(211993)),  // Head: Entombed Seraph's Casque
        (2, e(230637)),  // Neck: Astral Gladiator's Amulet
        (3, e(211991)),  // Shoulder: Entombed Seraph's Plumes
        (5, e(211996)),  // Chest: Entombed Seraph's Breastplate
        (6, e(211990)),  // Waist: Entombed Seraph's Waistguard
        (7, e(211992)),  // Legs: Entombed Seraph's Greaves
        (8, e(211995)),  // Feet: Entombed Seraph's Sabatons
        (9, e(211989)),  // Wrist: Entombed Seraph's Shackles
        (10, e(211994)), // Hands: Entombed Seraph's Castigation
        (11, e(225748)), // Ring1: Seal of the Silent Vigil
        (12, e(215135)), // Ring2: Ring of Earthen Craftsmanship
        (13, e(218715)), // Trinket1: Forged Gladiator's Emblem
        (14, e(236914)), // Trinket2: Unbound Vision Journal
        (15, e(211988)), // Back: Entombed Seraph's Greatcloak
        (16, e(229181)), // MainHand: Ordained Forge Maul
    ]
    .into_iter()
    .collect()
}

pub(super) fn default_mounts() -> Vec<MountData> {
    let m = |mount_id, name: &str, spell_id, icon, collected, mount_type| MountData {
        mount_id,
        name: name.to_string(),
        spell_id,
        icon,
        is_collected: collected,
        is_usable: collected,
        mount_type,
    };
    vec![
        m(6, "Brown Horse", 458, 132261, true, 230), // Ground
        m(18, "Swift Palomino", 23338, 132261, true, 230), // Ground
        m(69, "Swift Stormsaber", 23338, 132226, true, 230), // Ground
        m(107, "Ashes of Al'ar", 40192, 132238, true, 248), // Flying
        m(219, "Invincible", 72286, 132226, true, 248), // Flying
        m(280, "Traveler's Tundra Mammoth", 61425, 236241, true, 230), // Ground (vendor)
        m(376, "Sandstone Drake", 93326, 656576, true, 248), // Flying (2-person)
        m(440, "Grand Expedition Yak", 122708, 646372, true, 230), // Ground (vendor)
        m(678, "Llothien Prowler", 171851, 1394955, true, 230), // Ground
        m(
            1039,
            "Mighty Caravan Brutosaur",
            264058,
            2399241,
            false,
            230,
        ), // Ground (AH mount)
    ]
}

pub(super) fn default_pets() -> Vec<PetData> {
    let mut id = 0u32;
    let p = |id: &mut u32,
             species: u32,
             name: &str,
             icon: u32,
             pet_type: i32,
             level: i32,
             quality: i32,
             collected: bool| {
        *id += 1;
        PetData {
            pet_id: format!("BattlePet-0-{:08X}", *id),
            species_id: species,
            name: name.to_string(),
            icon,
            pet_type,
            level,
            quality,
            is_collected: collected,
            custom_name: None,
            is_favorite: false,
            can_attach_to_decor: false,
            creature_model_scale: Some(1.0),
        }
    };
    vec![
        p(&mut id, 39, "Mechanical Squirrel", 132932, 9, 25, 3, true), // Mechanical
        p(&mut id, 87, "Phoenix Hatchling", 132837, 3, 25, 3, true),   // Elemental
        p(&mut id, 68, "Cat", 132576, 7, 1, 1, true),                  // Beast
        p(&mut id, 254, "Lil' Ragnaros", 134153, 3, 25, 4, true),      // Elemental (legendary)
        p(&mut id, 1266, "Xu-Fu, Cub of Xuen", 648459, 7, 25, 4, true), // Beast (legendary)
        p(&mut id, 630, "Clockwork Gnome", 425954, 9, 25, 3, true),    // Mechanical
        p(&mut id, 846, "Anubisath Idol", 607552, 10, 25, 3, true),    // Humanoid
        p(&mut id, 40, "Bombay Cat", 132576, 7, 1, 1, true),           // Beast
        p(&mut id, 1395, "Iron Starlette", 971374, 9, 25, 3, true),    // Mechanical
        p(&mut id, 2403, "Pocopoc", 4038816, 9, 25, 4, false),         // Mechanical (not collected)
    ]
}

pub(super) fn default_toys() -> Vec<ToyData> {
    let t = |item_id, name: &str, icon, collected| ToyData {
        item_id,
        name: name.to_string(),
        icon,
        is_collected: collected,
        is_usable: collected,
    };
    vec![
        t(166779, "Hearthstone Game Table", 2823166, true),
        t(13379, "Piccolo of the Flaming Fire", 134208, true),
        t(34480, "Romantic Picnic Basket", 236571, true),
        t(33927, "Brewfest Pony Keg", 132790, true),
        t(119210, "Hearthstone Board", 134457, true),
        t(69227, "Fool's Gold", 134112, true),
        t(88589, "Cremating Torch", 135432, true),
        t(86575, "Foxicopter Controller", 463485, true),
        t(104324, "Foot Ball", 620832, true),
        t(187421, "Ashen Liniment", 1500929, false),
    ]
}

struct WarbandSceneSeed {
    id: u32,
    name: &'static str,
    description: &'static str,
    source: &'static str,
    quality: i32,
    texture_kit: &'static str,
    is_collected: bool,
    is_favorite: bool,
    has_fanfare: bool,
    source_type: i32,
}

const DEFAULT_WARBAND_SCENES: [WarbandSceneSeed; 4] = [
    WarbandSceneSeed {
        id: 1,
        name: "Harbor Camp",
        description: "A windswept camp overlooking the harbor.",
        source: "Rewarded from the opening Warband questline.",
        quality: 2,
        texture_kit: "campcollection-bg-image1",
        is_collected: true,
        is_favorite: true,
        has_fanfare: false,
        source_type: 1,
    },
    WarbandSceneSeed {
        id: 2,
        name: "Evergreen Grove",
        description: "A quiet glade with druidic furnishings.",
        source: "Purchased from the Trading Post.",
        quality: 3,
        texture_kit: "campcollection-bg-image2",
        is_collected: true,
        is_favorite: false,
        has_fanfare: false,
        source_type: 2,
    },
    WarbandSceneSeed {
        id: 3,
        name: "Storm Camp",
        description: "A fortified site built for foul weather.",
        source: "Complete a seasonal event to unlock.",
        quality: 3,
        texture_kit: "campcollection-bg-image3",
        is_collected: false,
        is_favorite: false,
        has_fanfare: false,
        source_type: 3,
    },
    WarbandSceneSeed {
        id: 4,
        name: "Dragonflight Perch",
        description: "A high cliffside camp with dragonriding access.",
        source: "Earned from a meta achievement.",
        quality: 4,
        texture_kit: "campcollection-bg-image4",
        is_collected: false,
        is_favorite: false,
        has_fanfare: false,
        source_type: 4,
    },
];

pub(super) fn default_warband_scenes() -> Vec<WarbandSceneData> {
    DEFAULT_WARBAND_SCENES
        .iter()
        .map(warband_scene_from_seed)
        .collect()
}

fn warband_scene_from_seed(seed: &WarbandSceneSeed) -> WarbandSceneData {
    WarbandSceneData {
        warband_scene_id: seed.id,
        name: seed.name.to_string(),
        description: seed.description.to_string(),
        source: seed.source.to_string(),
        quality: seed.quality,
        texture_kit: seed.texture_kit.to_string(),
        is_collected: seed.is_collected,
        is_favorite: seed.is_favorite,
        has_fanfare: seed.has_fanfare,
        source_type: seed.source_type,
    }
}

/// Default transmog appearances: ~5 per armor slot + weapon slots.
///
/// Category IDs from Enum.TransmogCollectionType:
///   Head=1, Shoulder=2, Back=3, Chest=4, Shirt=5, Tabard=6, Wrist=7,
///   Hands=8, Waist=9, Legs=10, Feet=11, OneHSword=14, Staff=23, Shield=18
pub(super) fn default_transmog_appearances() -> Vec<TransmogAppearance> {
    DEFAULT_TRANSMOG_APPEARANCES
        .iter()
        .enumerate()
        .map(transmog_appearance_from_seed)
        .collect()
}

struct TransmogAppearanceSeed {
    category_id: i32,
    item_id: i32,
    is_collected: bool,
    source_type: i32,
}

// Source types: JournalEncounter=1, Quest=2, Vendor=3, WorldDrop=4.
const DEFAULT_TRANSMOG_APPEARANCES: [TransmogAppearanceSeed; 63] = [
    t(1, 31110, true, 1),
    t(1, 34333, true, 1),
    t(1, 44006, true, 2),
    t(1, 77154, true, 1),
    t(1, 99999, false, 4),
    t(2, 30055, true, 1),
    t(2, 34392, true, 1),
    t(2, 50853, true, 1),
    t(2, 77166, true, 1),
    t(2, 99998, false, 3),
    t(3, 27878, true, 1),
    t(3, 34241, true, 1),
    t(3, 65108, true, 2),
    t(3, 77098, true, 4),
    t(3, 99997, false, 4),
    t(4, 30129, true, 1),
    t(4, 34215, true, 1),
    t(4, 50078, true, 1),
    t(4, 77171, true, 1),
    t(4, 99996, false, 3),
    t(5, 16059, true, 3),
    t(5, 4330, true, 3),
    t(6, 5976, true, 3),
    t(7, 30067, true, 1),
    t(7, 34431, true, 1),
    t(7, 50611, true, 1),
    t(7, 77162, true, 4),
    t(7, 99995, false, 4),
    t(8, 30113, true, 1),
    t(8, 34342, true, 1),
    t(8, 50610, true, 1),
    t(8, 77160, true, 1),
    t(8, 99994, false, 2),
    t(9, 30034, true, 1),
    t(9, 34485, true, 1),
    t(9, 50069, true, 1),
    t(9, 77164, true, 4),
    t(9, 99993, false, 3),
    t(10, 30126, true, 1),
    t(10, 34180, true, 1),
    t(10, 50063, true, 1),
    t(10, 77155, true, 1),
    t(10, 99992, false, 2),
    t(11, 30032, true, 1),
    t(11, 34567, true, 1),
    t(11, 50607, true, 1),
    t(11, 77159, true, 4),
    t(11, 99991, false, 4),
    t(14, 28573, true, 1),
    t(14, 34247, true, 1),
    t(14, 50070, true, 1),
    t(14, 77188, true, 1),
    t(14, 99990, false, 1),
    t(23, 30723, true, 1),
    t(23, 34182, true, 1),
    t(23, 50731, true, 4),
    t(23, 77196, true, 1),
    t(23, 99989, false, 4),
    t(18, 28606, true, 1),
    t(18, 34185, true, 1),
    t(18, 50729, true, 1),
    t(18, 77167, true, 4),
    t(18, 99988, false, 3),
];

const fn t(
    category_id: i32,
    item_id: i32,
    is_collected: bool,
    source_type: i32,
) -> TransmogAppearanceSeed {
    TransmogAppearanceSeed {
        category_id,
        item_id,
        is_collected,
        source_type,
    }
}

fn transmog_appearance_from_seed(
    (index, seed): (usize, &TransmogAppearanceSeed),
) -> TransmogAppearance {
    let appearance_id = index as i32 + 1;

    TransmogAppearance {
        source_id: appearance_id,
        visual_id: appearance_id,
        category_id: seed.category_id,
        item_id: seed.item_id,
        is_collected: seed.is_collected,
        source_type: seed.source_type,
        item_mod_id: 0,
    }
}

pub(super) fn default_heirlooms() -> Vec<HeirloomData> {
    DEFAULT_HEIRLOOMS.iter().map(heirloom_from_seed).collect()
}

struct HeirloomSeed {
    item_id: u32,
    name: &'static str,
    equip_loc: &'static str,
    icon: u32,
    upgrade_level: i32,
    source: &'static str,
    min_level: i32,
    max_level: i32,
}

const DEFAULT_HEIRLOOMS: [HeirloomSeed; 11] = [
    heirloom(122245, "Burnished Helm of Might", "INVTYPE_HEAD", 133071, 6),
    heirloom(
        122355,
        "Polished Breastplate of Valor",
        "INVTYPE_CHEST",
        132633,
        6,
    ),
    heirloom(
        122356,
        "Polished Spaulders of Valor",
        "INVTYPE_SHOULDER",
        132633,
        6,
    ),
    heirloom(
        122382,
        "Preened Ironfeather Shoulders",
        "INVTYPE_SHOULDER",
        135033,
        6,
    ),
    heirloom(
        122384,
        "Tattered Dreadmist Robe",
        "INVTYPE_CHEST",
        132673,
        6,
    ),
    heirloom(
        122247,
        "Burnished Legplates of Might",
        "INVTYPE_LEGS",
        133071,
        6,
    ),
    heirloom(
        122250,
        "Mystical Kilt of Elements",
        "INVTYPE_LEGS",
        134188,
        6,
    ),
    heirloom(122266, "Balanced Heartseeker", "INVTYPE_WEAPON", 135274, 6),
    heirloom(
        122389,
        "Bloodied Arcanite Reaper",
        "INVTYPE_2HWEAPON",
        135277,
        6,
    ),
    heirloom(
        122390,
        "Dignified Headmaster's Charge",
        "INVTYPE_2HWEAPON",
        135146,
        6,
    ),
    heirloom(
        187997,
        "Eternal Amulet of the Redeemed",
        "INVTYPE_NECK",
        133280,
        0,
    ),
];

const fn heirloom(
    item_id: u32,
    name: &'static str,
    equip_loc: &'static str,
    icon: u32,
    upgrade_level: i32,
) -> HeirloomSeed {
    HeirloomSeed {
        item_id,
        name,
        equip_loc,
        icon,
        upgrade_level,
        source: "Vendor",
        min_level: 1,
        max_level: 50,
    }
}

fn heirloom_from_seed(seed: &HeirloomSeed) -> HeirloomData {
    HeirloomData {
        item_id: seed.item_id,
        name: seed.name.into(),
        equip_loc: seed.equip_loc.into(),
        icon: seed.icon,
        upgrade_level: seed.upgrade_level,
        source: seed.source.into(),
        min_level: seed.min_level,
        max_level: seed.max_level,
    }
}

pub(super) fn default_premade_listings() -> Vec<PremadeListing> {
    DEFAULT_PREMADE_LISTINGS
        .iter()
        .enumerate()
        .map(premade_listing_from_seed)
        .collect()
}

/// `Enum.LFGEntryGeneralPlaystyle` codes as named constants; raw numbers are
/// otherwise opaque in the seed table.
const PLAYSTYLE_LEARNING: i32 = 1;
const PLAYSTYLE_FUN_RELAXED: i32 = 2;
const PLAYSTYLE_FUN_SERIOUS: i32 = 3;

/// Faction id encoding for `leaderFactionGroup` (`PLAYER_FACTION_GROUP`):
/// 0 = Horde, 1 = Alliance.
const FACTION_HORDE: i32 = 0;
const FACTION_ALLIANCE: i32 = 1;

struct PremadeListingSeed {
    name: &'static str,
    comment: &'static str,
    leader_name: &'static str,
    activity_id: u32,
    tanks: i32,
    healers: i32,
    damagers: i32,
    max_members: i32,
    general_playstyle: i32,
    leader_faction_group: i32,
}

const DEFAULT_PREMADE_LISTINGS: [PremadeListingSeed; 8] = [
    premade_listing(
        "+15 Mists chill run",
        "Know mechanics, 2.5k io",
        "Thrallx",
        1195,
        1,
        1,
        1,
        5,
        PLAYSTYLE_FUN_SERIOUS,
        FACTION_HORDE,
    ),
    premade_listing(
        "+12 Siege weekly",
        "Weekly key, all welcome",
        "Jainavx",
        1188,
        0,
        1,
        1,
        5,
        PLAYSTYLE_FUN_RELAXED,
        FACTION_ALLIANCE,
    ),
    premade_listing(
        "Nerub-ar Palace HC fresh",
        "AOTC prog, be geared",
        "Anduin",
        1296,
        2,
        3,
        7,
        20,
        PLAYSTYLE_FUN_SERIOUS,
        FACTION_ALLIANCE,
    ),
    premade_listing(
        "Nerub-ar Palace N learn",
        "Learning run, patient",
        "Sylvanas",
        1295,
        2,
        2,
        4,
        20,
        PLAYSTYLE_LEARNING,
        FACTION_HORDE,
    ),
    premade_listing(
        "World Boss — Aggregation",
        "Quick kill, summon up",
        "Khadgar",
        1350,
        3,
        4,
        11,
        40,
        PLAYSTYLE_FUN_RELAXED,
        FACTION_ALLIANCE,
    ),
    premade_listing(
        "2v2 Arena chill",
        "Just capping",
        "Garrosh",
        491,
        0,
        0,
        1,
        2,
        PLAYSTYLE_FUN_RELAXED,
        FACTION_HORDE,
    ),
    premade_listing(
        "RBG yolo",
        "Casual RBG, no rage",
        "Velen",
        493,
        1,
        2,
        4,
        10,
        PLAYSTYLE_FUN_RELAXED,
        FACTION_ALLIANCE,
    ),
    premade_listing(
        "WQ group Ringing Deeps",
        "Doing WQs together",
        "Malfurion",
        1700,
        0,
        0,
        3,
        5,
        PLAYSTYLE_FUN_RELAXED,
        FACTION_ALLIANCE,
    ),
];

const fn premade_listing(
    name: &'static str,
    comment: &'static str,
    leader_name: &'static str,
    activity_id: u32,
    tanks: i32,
    healers: i32,
    damagers: i32,
    max_members: i32,
    general_playstyle: i32,
    leader_faction_group: i32,
) -> PremadeListingSeed {
    PremadeListingSeed {
        name,
        comment,
        leader_name,
        activity_id,
        tanks,
        healers,
        damagers,
        max_members,
        general_playstyle,
        leader_faction_group,
    }
}

fn premade_listing_from_seed((index, seed): (usize, &PremadeListingSeed)) -> PremadeListing {
    let search_result_id = index as u32 + 1;

    PremadeListing {
        metadata: Default::default(),
        search_result_id,
        name: seed.name.to_string(),
        comment: seed.comment.to_string(),
        leader_name: seed.leader_name.to_string(),
        activity_id: seed.activity_id,
        num_members: seed.tanks + seed.healers + seed.damagers,
        max_members: seed.max_members,
        voice_chat: String::new(),
        auto_accept: false,
        is_delisted: false,
        party_guid: format!("Party-3-0000-1234-{search_result_id:08X}"),
        tanks: seed.tanks,
        healers: seed.healers,
        damagers: seed.damagers,
        no_role: 0,
        classes_by_role: HashMap::new(),
        general_playstyle: seed.general_playstyle,
        cross_faction_listing: false,
        leader_faction_group: seed.leader_faction_group,
        num_bnet_friends: 0,
        num_char_friends: 0,
        num_guild_mates: 0,
    }
}
