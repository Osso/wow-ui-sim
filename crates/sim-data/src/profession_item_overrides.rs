use crate::items::ItemInfo;

pub fn get_item(id: u32) -> Option<&'static ItemInfo> {
    PROFESSION_ITEM_OVERRIDES
        .iter()
        .find(|(item_id, _)| *item_id == id)
        .map(|(_, item)| item)
}

const fn profession_item(
    name: &'static str,
    quality: u8,
    item_level: u16,
    inventory_type: u8,
    sell_price: u32,
    stackable: u32,
    bonding: u8,
    expansion_id: u8,
    icon_file_data_id: u32,
) -> ItemInfo {
    ItemInfo {
        name,
        quality,
        item_level,
        required_level: 1,
        inventory_type,
        sell_price,
        stackable,
        bonding,
        expansion_id,
        icon_file_data_id,
        stat_percent_editor: [0; 10],
        stat_modifier_bonus_stat: [-1; 10],
        artifact_id: 0,
        socket_types: [0; 3],
        gem_properties: 0,
    }
}

static PROFESSION_ITEM_OVERRIDES: &[(u32, ItemInfo)] = &[
    (
        2835,
        profession_item("Rough Stone", 1, 10, 0, 2, 1000, 0, 0, 135232),
    ),
    (
        2840,
        profession_item("Copper Bar", 1, 10, 0, 10, 1000, 0, 0, 133216),
    ),
    (
        2852,
        profession_item("Copper Chain Pants", 1, 4, 7, 79, 1, 2, 0, 134583),
    ),
    (
        2862,
        profession_item("Rough Sharpening Stone", 1, 3, 0, 3, 20, 0, 0, 135248),
    ),
    (
        18567,
        profession_item("Elemental Flux", 1, 10, 0, 7500, 1000, 0, 0, 135839),
    ),
    (
        23445,
        profession_item("Fel Iron Bar", 1, 15, 0, 2000, 1000, 0, 1, 133230),
    ),
    (
        23482,
        profession_item("Fel Iron Plate Gloves", 2, 30, 10, 20269, 1, 2, 1, 132937),
    ),
    (
        23484,
        profession_item("Fel Iron Plate Belt", 2, 30, 6, 1500, 1, 2, 1, 132510),
    ),
    (
        36916,
        profession_item("Cobalt Bar", 1, 20, 0, 2400, 1000, 0, 2, 133228),
    ),
    (
        39086,
        profession_item("Cobalt Legplates", 2, 32, 7, 1500, 1, 2, 2, 134679),
    ),
    (
        39087,
        profession_item("Cobalt Belt", 2, 32, 6, 1500, 1, 2, 2, 132520),
    ),
    (
        54849,
        profession_item("Obsidium Bar", 1, 25, 0, 4800, 1000, 0, 3, 135241),
    ),
    (
        54850,
        profession_item("Hardened Obsidium Bracers", 2, 35, 9, 1500, 1, 2, 3, 463455),
    ),
    (
        65365,
        profession_item("Folded Obsidium", 1, 25, 0, 5000, 1000, 0, 3, 135657),
    ),
    (
        72096,
        profession_item("Ghost Iron Bar", 1, 30, 0, 100, 1000, 0, 4, 538438),
    ),
    (
        80811,
        profession_item("Spiritguard Helm", 2, 37, 1, 1500, 1, 2, 4, 648027),
    ),
    (
        82896,
        profession_item("Spiritguard Shoulders", 2, 37, 3, 1500, 1, 2, 4, 648033),
    ),
    (
        108257,
        profession_item("Truesteel Ingot", 2, 35, 0, 3600, 1000, 1, 5, 1046264),
    ),
    (
        109118,
        profession_item("Blackrock Ore", 1, 35, 0, 350, 1000, 0, 5, 962047),
    ),
    (
        109119,
        profession_item("True Iron Ore", 1, 35, 0, 1500, 1000, 0, 5, 962048),
    ),
    (
        116426,
        profession_item("Smoldering Helm", 3, 43, 1, 1500, 1, 2, 5, 134400),
    ),
    (
        123897,
        profession_item("Leystone Waistguard", 3, 46, 6, 1500, 1, 2, 6, 134400),
    ),
    (
        123898,
        profession_item("Leystone Armguards", 3, 46, 9, 1500, 1, 2, 6, 134400),
    ),
    (
        123918,
        profession_item("Leystone Ore", 1, 40, 0, 1, 1000, 0, 6, 1394960),
    ),
    (
        152512,
        profession_item("Monelite Ore", 1, 45, 0, 1, 1000, 0, 7, 2037638),
    ),
    (
        152812,
        profession_item("Monel-Hardened Hoofplates", 1, 50, 0, 0, 200, 0, 7, 1405823),
    ),
    (
        152813,
        profession_item("Monel-Hardened Stirrups", 1, 50, 0, 0, 200, 0, 7, 1405822),
    ),
    (
        160298,
        profession_item("Durable Flux", 1, 45, 0, 750, 1000, 0, 7, 134387),
    ),
    (
        171374,
        profession_item(
            "Ceremonious Breastplate",
            2,
            100,
            5,
            503120,
            1,
            2,
            8,
            134400,
        ),
    ),
    (
        171428,
        profession_item("Shadowghast Ingot", 2, 50, 0, 12500, 1000, 0, 8, 3528421),
    ),
    (
        171828,
        profession_item("Laestrite Ore", 1, 50, 0, 650, 1000, 0, 8, 3594132),
    ),
    (
        171829,
        profession_item("Solenium Ore", 2, 50, 0, 650, 1000, 0, 8, 3731242),
    ),
    (
        171830,
        profession_item("Oxxein Ore", 2, 50, 0, 650, 1000, 0, 8, 3608331),
    ),
    (
        171831,
        profession_item("Phaedrum Ore", 2, 50, 0, 650, 1000, 0, 8, 3537032),
    ),
    (
        171832,
        profession_item("Sinvyr Ore", 2, 50, 0, 650, 1000, 0, 8, 3616941),
    ),
    (
        180733,
        profession_item("Luminous Flux", 1, 50, 0, 22500, 1000, 0, 0, 3615503),
    ),
    (
        189541,
        profession_item("Primal Molten Alloy", 3, 70, 0, 50000, 1000, 0, 9, 4622288),
    ),
    (
        190505,
        profession_item(
            "Primal Molten Shortblade",
            4,
            350,
            13,
            1352148,
            1,
            1,
            9,
            134400,
        ),
    ),
    (
        190508,
        profession_item(
            "Primal Molten Warglaive",
            4,
            350,
            13,
            1366616,
            1,
            1,
            9,
            134400,
        ),
    ),
    (
        210930,
        profession_item("Bismuth", 1, 70, 0, 500, 1000, 0, 10, 5728051),
    ),
    (
        210931,
        profession_item("Bismuth", 2, 70, 0, 500, 1000, 0, 10, 5728051),
    ),
    (
        210934,
        profession_item("Aqirite", 2, 70, 0, 1500, 1000, 0, 10, 134573),
    ),
    (
        210935,
        profession_item("Aqirite", 3, 70, 0, 1500, 1000, 0, 10, 134573),
    ),
    (
        210937,
        profession_item("Ironclaw Ore", 2, 70, 0, 1500, 1000, 0, 10, 5735005),
    ),
    (
        217143,
        profession_item(
            "Algari Competitor's Plate Breastplate",
            2,
            577,
            5,
            469106,
            1,
            2,
            10,
            134400,
        ),
    ),
    (
        217144,
        profession_item(
            "Algari Competitor's Plate Sabatons",
            2,
            577,
            8,
            330180,
            1,
            2,
            10,
            134400,
        ),
    ),
    (
        222426,
        profession_item("Ironclaw Alloy", 3, 70, 0, 70200, 1000, 0, 10, 5931154),
    ),
    (
        237366,
        profession_item("Dazzling Thorium", 3, 80, 0, 1500, 1000, 0, 11, 7549223),
    ),
    (
        238017,
        profession_item(
            "Sun-Blessed Leatherworker's Knife",
            3,
            106,
            29,
            320200,
            1,
            2,
            11,
            7456228,
        ),
    ),
    (
        238018,
        profession_item(
            "Sun-Blessed Blacksmith's Hammer",
            3,
            106,
            29,
            320500,
            1,
            2,
            11,
            134400,
        ),
    ),
    (
        238528,
        profession_item("Majestic Claw", 4, 80, 0, 10000, 1000, 0, 11, 7549227),
    ),
];
