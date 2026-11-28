//! Auto-generated WoW Forever UiMap and UiMapAssignment data (Wago exports of 1.60.1.70707).
//! Do not edit manually - regenerate with: wow-cli generate ui-map-assignments

pub use super::ui_maps::UiMapInfo;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiMapAssignment {
    pub map_id: i32,
    pub area_id: u32,
    pub ui_min: [f32; 2],
    pub ui_max: [f32; 2],
    pub region_min: [f32; 2],
    pub region_max: [f32; 2],
}

pub static UI_MAP_DB: phf::Map<u32, UiMapInfo> = ::phf::Map {
    key: 12913932095322966823,
    disps: &[
        (12, 51),
        (1, 35),
        (1, 12),
        (0, 46),
        (0, 51),
        (1, 17),
        (1, 40),
        (0, 4),
        (6, 23),
        (28, 44),
        (0, 0),
        (7, 10),
    ],
    entries: &[
        (1428, UiMapInfo { name: "Burning Steppes", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1451, UiMapInfo { name: "Silithus", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1430, UiMapInfo { name: "Deadwind Pass", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1432, UiMapInfo { name: "Loch Modan", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1458, UiMapInfo { name: "Undercity", map_type: 3, parent_map_id: 1415, flags: 0 }),
        (1449, UiMapInfo { name: "Un'Goro Crater", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1436, UiMapInfo { name: "Westfall", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1425, UiMapInfo { name: "The Hinterlands", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (2652, UiMapInfo { name: "Shen'dralas", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1457, UiMapInfo { name: "Darnassus", map_type: 3, parent_map_id: 1414, flags: 0 }),
        (1440, UiMapInfo { name: "Ashenvale", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1431, UiMapInfo { name: "Duskwood", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (2521, UiMapInfo { name: "Zephras Isle", map_type: 3, parent_map_id: 947, flags: 2 }),
        (1415, UiMapInfo { name: "Eastern Kingdoms", map_type: 2, parent_map_id: 947, flags: 0 }),
        (1452, UiMapInfo { name: "Winterspring", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1435, UiMapInfo { name: "Swamp of Sorrows", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1419, UiMapInfo { name: "Blasted Lands", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (2548, UiMapInfo { name: "Riverglades", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1453, UiMapInfo { name: "Stormwind City", map_type: 3, parent_map_id: 1415, flags: 0 }),
        (1417, UiMapInfo { name: "Arathi Highlands", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1418, UiMapInfo { name: "Badlands", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1463, UiMapInfo { name: "Eastern Kingdoms", map_type: 2, parent_map_id: 0, flags: 0 }),
        (1427, UiMapInfo { name: "Searing Gorge", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1459, UiMapInfo { name: "Alterac Valley", map_type: 6, parent_map_id: 1424, flags: 2 }),
        (1450, UiMapInfo { name: "Moonglade", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1448, UiMapInfo { name: "Felwood", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (2665, UiMapInfo { name: "Zephras Isle", map_type: 3, parent_map_id: 0, flags: 0 }),
        (1411, UiMapInfo { name: "Durotar", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (947, UiMapInfo { name: "Azeroth", map_type: 1, parent_map_id: 0, flags: 2304 }),
        (1446, UiMapInfo { name: "Tanaris", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1456, UiMapInfo { name: "Thunder Bluff", map_type: 3, parent_map_id: 1414, flags: 0 }),
        (1439, UiMapInfo { name: "Darkshore", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1460, UiMapInfo { name: "Warsong Gulch", map_type: 6, parent_map_id: 1413, flags: 0 }),
        (2524, UiMapInfo { name: "Darkspear Islands", map_type: 6, parent_map_id: 1414, flags: 0 }),
        (1426, UiMapInfo { name: "Dun Morogh", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1424, UiMapInfo { name: "Hillsbrad Foothills", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1455, UiMapInfo { name: "Ironforge", map_type: 3, parent_map_id: 1415, flags: 0 }),
        (1433, UiMapInfo { name: "Redridge Mountains", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1437, UiMapInfo { name: "Wetlands", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1461, UiMapInfo { name: "Arathi Basin", map_type: 6, parent_map_id: 1417, flags: 0 }),
        (1445, UiMapInfo { name: "Dustwallow Marsh", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1464, UiMapInfo { name: "Kalimdor", map_type: 2, parent_map_id: 0, flags: 0 }),
        (1441, UiMapInfo { name: "Thousand Needles", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1422, UiMapInfo { name: "Western Plaguelands", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1414, UiMapInfo { name: "Kalimdor", map_type: 2, parent_map_id: 947, flags: 0 }),
        (1420, UiMapInfo { name: "Tirisfal Glades", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1438, UiMapInfo { name: "Teldrassil", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1434, UiMapInfo { name: "Stranglethorn Vale", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1447, UiMapInfo { name: "Azshara", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1454, UiMapInfo { name: "Orgrimmar", map_type: 3, parent_map_id: 1414, flags: 0 }),
        (1443, UiMapInfo { name: "Desolace", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1444, UiMapInfo { name: "Feralas", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1416, UiMapInfo { name: "Alterac Mountains", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1413, UiMapInfo { name: "The Barrens", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1429, UiMapInfo { name: "Elwynn Forest", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1423, UiMapInfo { name: "Eastern Plaguelands", map_type: 3, parent_map_id: 1415, flags: 2 }),
        (1412, UiMapInfo { name: "Mulgore", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1442, UiMapInfo { name: "Stonetalon Mountains", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (2482, UiMapInfo { name: "Mount Hyjal", map_type: 3, parent_map_id: 1414, flags: 2 }),
        (1421, UiMapInfo { name: "Silverpine Forest", map_type: 3, parent_map_id: 1415, flags: 2 }),
    ],
};

pub fn get_ui_map(id: u32) -> Option<&'static UiMapInfo> {
    UI_MAP_DB.get(&id)
}

pub static UI_MAP_ASSIGNMENTS: &[(u32, &[UiMapAssignment])] = &[
    (947, &[
        UiMapAssignment { map_id: 0, area_id: 0, ui_min: [0.5505f32, 0.0994f32], ui_max: [0.8966f32, 0.8691f32], region_min: [-16000.0f32, -7466.7f32], region_max: [6933.3f32, 8000.0f32] },
        UiMapAssignment { map_id: 1, area_id: 0, ui_min: [0.0399f32, 0.0855f32], ui_max: [0.4083f32, 0.9234f32], region_min: [-12800.0f32, -9600.0f32], region_max: [12266.7f32, 6933.3f32] },
    ]),
    (1411, &[
        UiMapAssignment { map_id: 1, area_id: 14, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-1716.6666f32, -7249.9995f32], region_max: [1808.3333f32, -1962.4999f32] },
    ]),
    (1412, &[
        UiMapAssignment { map_id: 1, area_id: 215, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-3835.416f32, -3675.0f32], region_max: [266.66602f32, 2479.167f32] },
    ]),
    (1413, &[
        UiMapAssignment { map_id: 1, area_id: 17, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-5143.75f32, -7510.4165f32], region_max: [1612.4999f32, 2622.9165f32] },
    ]),
    (1414, &[
        UiMapAssignment { map_id: 1, area_id: 0, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-11733.3f32, -19733.21f32], region_max: [12799.9f32, 17066.6f32] },
    ]),
    (1415, &[
        UiMapAssignment { map_id: 0, area_id: 0, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-16000.0f32, -19199.9f32], region_max: [7466.6f32, 16000.0f32] },
    ]),
    (1416, &[
        UiMapAssignment { map_id: 0, area_id: 36, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-366.66666f32, -2016.6666f32], region_max: [1500.0f32, 783.3333f32] },
    ]),
    (1417, &[
        UiMapAssignment { map_id: 0, area_id: 45, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-2533.3333f32, -4466.6665f32], region_max: [-133.33333f32, -866.6666f32] },
    ]),
    (1418, &[
        UiMapAssignment { map_id: 0, area_id: 3, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-7547.9165f32, -4566.6665f32], region_max: [-5889.583f32, -2079.1665f32] },
    ]),
    (1419, &[
        UiMapAssignment { map_id: 0, area_id: 4, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-12800.0f32, -4591.6665f32], region_max: [-10566.666f32, -1241.6666f32] },
    ]),
    (1420, &[
        UiMapAssignment { map_id: 0, area_id: 85, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [824.99994f32, -1485.4166f32], region_max: [3837.4998f32, 3033.3333f32] },
    ]),
    (1421, &[
        UiMapAssignment { map_id: 0, area_id: 130, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-1133.3333f32, -750.0f32], region_max: [1666.6666f32, 3449.9998f32] },
    ]),
    (1422, &[
        UiMapAssignment { map_id: 0, area_id: 28, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [499.99997f32, -3883.3333f32], region_max: [3366.6665f32, 416.66666f32] },
    ]),
    (1423, &[
        UiMapAssignment { map_id: 0, area_id: 139, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [825.0f32, -6558.334f32], region_max: [3691.667f32, -2256.25f32] },
    ]),
    (1424, &[
        UiMapAssignment { map_id: 0, area_id: 267, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-1733.3333f32, -2133.3333f32], region_max: [400.0f32, 1066.6666f32] },
    ]),
    (1425, &[
        UiMapAssignment { map_id: 0, area_id: 47, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-1100.0f32, -5425.0f32], region_max: [1466.6666f32, -1575.0f32] },
    ]),
    (1426, &[
        UiMapAssignment { map_id: 0, area_id: 1, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-7160.4165f32, -3122.9165f32], region_max: [-3877.0833f32, 1802.0833f32] },
    ]),
    (1427, &[
        UiMapAssignment { map_id: 0, area_id: 51, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-7587.4995f32, -2554.1665f32], region_max: [-6100.0f32, -322.91666f32] },
    ]),
    (1428, &[
        UiMapAssignment { map_id: 0, area_id: 46, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-8983.333f32, -3195.8333f32], region_max: [-7031.2495f32, -266.66666f32] },
    ]),
    (1429, &[
        UiMapAssignment { map_id: 0, area_id: 12, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-10254.166f32, -1935.4166f32], region_max: [-7939.583f32, 1535.4166f32] },
    ]),
    (1430, &[
        UiMapAssignment { map_id: 0, area_id: 41, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-11533.333f32, -3333.3333f32], region_max: [-9866.666f32, -833.3333f32] },
    ]),
    (1431, &[
        UiMapAssignment { map_id: 0, area_id: 10, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-11516.666f32, -1866.6666f32], region_max: [-9716.666f32, 833.3333f32] },
    ]),
    (1432, &[
        UiMapAssignment { map_id: 0, area_id: 38, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-6327.083f32, -4752.083f32], region_max: [-4487.5f32, -1993.7499f32] },
    ]),
    (1433, &[
        UiMapAssignment { map_id: 0, area_id: 44, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-10022.916f32, -3852.084f32], region_max: [-8575.0f32, -1681.25f32] },
    ]),
    (1434, &[
        UiMapAssignment { map_id: 0, area_id: 33, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-15422.916f32, -4160.4165f32], region_max: [-11168.75f32, 2220.8333f32] },
    ]),
    (1435, &[
        UiMapAssignment { map_id: 0, area_id: 8, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-11150.0f32, -4516.6665f32], region_max: [-9620.833f32, -2222.9165f32] },
    ]),
    (1436, &[
        UiMapAssignment { map_id: 0, area_id: 40, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-11733.333f32, -483.3333f32], region_max: [-9400.0f32, 3016.6665f32] },
    ]),
    (1437, &[
        UiMapAssignment { map_id: 0, area_id: 11, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-4904.1665f32, -4525.0f32], region_max: [-2147.9165f32, -389.5833f32] },
    ]),
    (1438, &[
        UiMapAssignment { map_id: 1, area_id: 141, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [8437.5f32, -1277.0833f32], region_max: [11831.25f32, 3814.5833f32] },
    ]),
    (1439, &[
        UiMapAssignment { map_id: 1, area_id: 148, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [3966.6665f32, -3608.3333f32], region_max: [8333.333f32, 2941.6665f32] },
    ]),
    (1440, &[
        UiMapAssignment { map_id: 1, area_id: 331, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [829.1666f32, -4066.6665f32], region_max: [4672.9165f32, 1699.9999f32] },
    ]),
    (1441, &[
        UiMapAssignment { map_id: 1, area_id: 400, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-6899.9995f32, -4833.333f32], region_max: [-3966.6665f32, -433.3333f32] },
    ]),
    (1442, &[
        UiMapAssignment { map_id: 1, area_id: 406, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-339.5833f32, -1637.4999f32], region_max: [2916.6665f32, 3245.8333f32] },
    ]),
    (1443, &[
        UiMapAssignment { map_id: 1, area_id: 405, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-2545.8333f32, -262.5f32], region_max: [452.0833f32, 4233.333f32] },
    ]),
    (1444, &[
        UiMapAssignment { map_id: 1, area_id: 357, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-6999.9995f32, -1508.3333f32], region_max: [-2366.6665f32, 5441.6665f32] },
    ]),
    (1445, &[
        UiMapAssignment { map_id: 1, area_id: 15, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-5533.333f32, -6225.0f32], region_max: [-2033.3333f32, -974.99994f32] },
    ]),
    (1446, &[
        UiMapAssignment { map_id: 1, area_id: 440, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-10475.0f32, -7118.7495f32], region_max: [-5875.0f32, -218.74998f32] },
    ]),
    (1447, &[
        UiMapAssignment { map_id: 1, area_id: 16, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [1960.4166f32, -8347.916f32], region_max: [5341.6665f32, -3277.0833f32] },
    ]),
    (1448, &[
        UiMapAssignment { map_id: 1, area_id: 361, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [3299.9998f32, -4108.333f32], region_max: [7133.333f32, 1641.6666f32] },
    ]),
    (1449, &[
        UiMapAssignment { map_id: 1, area_id: 490, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-8433.333f32, -3166.6665f32], region_max: [-5966.6665f32, 533.3333f32] },
    ]),
    (1450, &[
        UiMapAssignment { map_id: 1, area_id: 493, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [6952.083f32, -3689.5833f32], region_max: [8491.666f32, -1381.25f32] },
    ]),
    (1451, &[
        UiMapAssignment { map_id: 1, area_id: 1377, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-8281.25f32, -945.834f32], region_max: [-5958.334f32, 2537.5f32] },
    ]),
    (1452, &[
        UiMapAssignment { map_id: 1, area_id: 618, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [3799.9998f32, -7416.6665f32], region_max: [8533.333f32, -316.66666f32] },
    ]),
    (1453, &[
        UiMapAssignment { map_id: 0, area_id: 1519, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-9154.17f32, -14.584f32], region_max: [-7995.83f32, 1722.92f32] },
    ]),
    (1454, &[
        UiMapAssignment { map_id: 1, area_id: 1637, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [1338.4606f32, -5083.2056f32], region_max: [2273.8772f32, -3680.601f32] },
    ]),
    (1455, &[
        UiMapAssignment { map_id: 0, area_id: 1537, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-5096.8457f32, -1504.2164f32], region_max: [-4569.241f32, -713.5914f32] },
    ]),
    (1456, &[
        UiMapAssignment { map_id: 1, area_id: 1638, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-1545.8333f32, -527.0833f32], region_max: [-849.99994f32, 516.6666f32] },
    ]),
    (1457, &[
        UiMapAssignment { map_id: 1, area_id: 1657, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [9532.587f32, 1880.0295f32], region_max: [10238.316f32, 2938.3628f32] },
    ]),
    (1458, &[
        UiMapAssignment { map_id: 0, area_id: 1497, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [1237.8412f32, -86.1824f32], region_max: [1877.9453f32, 873.1926f32] },
    ]),
    (1459, &[
        UiMapAssignment { map_id: 30, area_id: 2597, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-1739.5833f32, -2456.25f32], region_max: [1085.4166f32, 1781.2499f32] },
    ]),
    (1460, &[
        UiMapAssignment { map_id: 489, area_id: 3277, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [862.49994f32, 895.8333f32], region_max: [1627.0833f32, 2041.6666f32] },
    ]),
    (1461, &[
        UiMapAssignment { map_id: 529, area_id: 3358, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [337.5f32, 102.08333f32], region_max: [1508.3333f32, 1858.3333f32] },
    ]),
    (1463, &[
        UiMapAssignment { map_id: 0, area_id: 0, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-15980.0f32, -11880.0f32], region_max: [5817.0f32, 9917.0f32] },
    ]),
    (1464, &[
        UiMapAssignment { map_id: 1, area_id: 0, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-11870.0f32, -13370.0f32], region_max: [12470.0f32, 10970.0f32] },
    ]),
    (2482, &[
        UiMapAssignment { map_id: 1, area_id: 616, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [3989.583f32, -4395.834f32], region_max: [6304.166f32, -922.916f32] },
    ]),
    (2521, &[
        UiMapAssignment { map_id: 2991, area_id: 16593, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [1247.917f32, -1331.25f32], region_max: [4956.25f32, 4231.25f32] },
    ]),
    (2524, &[
        UiMapAssignment { map_id: 2997, area_id: 16606, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-835.416f32, 993.75f32], region_max: [447.91602f32, 2918.75f32] },
    ]),
    (2548, &[
        UiMapAssignment { map_id: 0, area_id: 16591, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-9700.0f32, -6741.666f32], region_max: [-6466.666f32, -1891.666f32] },
    ]),
    (2652, &[
        UiMapAssignment { map_id: 1, area_id: 16651, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [-3266.666f32, -25.0f32], region_max: [-1900.0f32, 2025.0f32] },
    ]),
    (2665, &[
        UiMapAssignment { map_id: 2991, area_id: 0, ui_min: [0.0f32, 0.0f32], ui_max: [1.0f32, 1.0f32], region_min: [1247.92f32, -1331.25f32], region_max: [4956.25f32, 4231.25f32] },
    ]),
];

pub fn get_ui_map_assignments(ui_map_id: u32) -> Option<&'static [UiMapAssignment]> {
    let index = UI_MAP_ASSIGNMENTS.binary_search_by_key(&ui_map_id, |(id, _)| *id).ok()?;
    Some(UI_MAP_ASSIGNMENTS[index].1)
}
