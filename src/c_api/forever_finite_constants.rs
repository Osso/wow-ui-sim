//! Source-published Forever 1.60.1 bag, minimap, ping, transmog and possess-bar data.

use crate::lua_api::methods::{create_table, table_get, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) {
    register_enums(state);
    register_constants(state);
}

fn register_enums(state: &mut LuaState) {
    let enums = super::helpers::ensure_global_table(state, "Enum");
    publish(
        state,
        enums,
        "WeaponSlot",
        &[("MainHand", 0), ("OffHand", 1), ("Ranged", 2)],
    );
    publish(
        state,
        enums,
        "WeaponSlotMeta",
        &[("MinValue", 0), ("MaxValue", 2), ("NumValues", 3)],
    );
    register_bag_indices(state, enums);
    register_tracking_filters(state, enums);
    register_ping_results(state, enums);
    register_gamepad_overrides(state, enums);
}

fn register_gamepad_overrides(state: &mut LuaState, enums: Val) {
    publish(state, enums, "GamepadPossessBarOverride", POSSESS_OVERRIDES);
    publish(
        state,
        enums,
        "GamepadPossessBarOverrideMeta",
        &[("MinValue", 1), ("MaxValue", 12), ("NumValues", 12)],
    );
    publish(state, enums, "GamepadStanceBarOverride", STANCE_OVERRIDES);
    publish(
        state,
        enums,
        "GamepadStanceBarOverrideMeta",
        &[("MinValue", 1), ("MaxValue", 12), ("NumValues", 12)],
    );
}

fn register_constants(state: &mut LuaState) {
    let constants = super::helpers::ensure_global_table(state, "Constants");
    publish(state, constants, "Transmog", &[("NoTransmogID", 0)]);
    publish(
        state,
        constants,
        "TransmogOutfitDataConsts",
        &[("CLEAR_TRANSMOG_OUTFIT_MANUAL_SPELL_ID", 1_247_917)],
    );
    publish(state, constants, "LegacyConsts", LEGACY_CONSTANTS);
    publish(
        state,
        constants,
        "QuestLogConsts",
        &[("MAXIMUM_NUM_QUESTS_LOG_CAN_ACCEPT", 40)],
    );
    publish(
        state,
        constants,
        "LevelConstsExposed",
        &[
            ("MIN_RES_SICKNESS_LEVEL", 10),
            ("MIN_ACHIEVEMENT_LEVEL", 10),
            ("MIN_TALENT_LEVEL", 10),
        ],
    );
}

fn register_bag_indices(state: &mut LuaState, enums: Val) {
    // Forever 1.60.1.69913 BagIndexConstantsDocumentation.lua. Character tabs
    // 1–6 retain their shared IDs; account tabs shift beyond all nine of them.
    publish(
        state,
        enums,
        "BagIndex",
        &[
            ("CharacterBankTab_7", 12),
            ("CharacterBankTab_8", 13),
            ("CharacterBankTab_9", 14),
            ("AccountBankTab_1", 15),
            ("AccountBankTab_2", 16),
            ("AccountBankTab_3", 17),
            ("AccountBankTab_4", 18),
            ("AccountBankTab_5", 19),
            ("AccountBankTab_6", 20),
            ("AccountBankTab_7", 21),
            ("AccountBankTab_8", 22),
            ("AccountBankTab_9", 23),
        ],
    );
    publish(
        state,
        enums,
        "BagIndexMeta",
        &[("MinValue", -3), ("MaxValue", 23), ("NumValues", 27)],
    );
}

fn register_tracking_filters(state: &mut LuaState, enums: Val) {
    publish(
        state,
        enums,
        "MinimapTrackingFilter",
        &[("TrainerClass", 8388608), ("VendorAmmo", 16777216)],
    );
    publish(
        state,
        enums,
        "MinimapTrackingFilterMeta",
        &[("MinValue", 0), ("MaxValue", 16777216), ("NumValues", 26)],
    );
}

fn register_ping_results(state: &mut LuaState, enums: Val) {
    publish(state, enums, "PingResult", &[("FailedSilent", 8)]);
    publish(
        state,
        enums,
        "PingResultMeta",
        &[("MinValue", 0), ("MaxValue", 8), ("NumValues", 9)],
    );
}

fn publish(state: &mut LuaState, root: Val, name: &str, fields: &[(&str, i32)]) {
    let values = match table_get(state, root, name) {
        value @ Val::Table(_) => value,
        _ => {
            let value = create_table(state);
            table_set(state, root, name, value);
            value
        }
    };
    for &(field, value) in fields {
        table_set(state, values, field, Val::Num(f64::from(value)));
    }
}

// Forever LegacyConstantsDocumentation.lua.
const LEGACY_CONSTANTS: &[(&str, i32)] = &[
    ("LEGACY_REWARD_TRACK_FACTION_ID", 2802),
    ("LEGACY_POINTS_TRAIT_CURRENCY_ID", 4225),
    ("LEGACY_TREE_PROFESSIONS_ID", 1187),
    ("LEGACY_TREE_ADVENTURE_ID", 1188),
    ("LEGACY_TREE_PROGRESSION_ID", 1189),
    ("LEGACY_TREE_ADVENTURE_TALENTED_NODE_ID", 110298),
];

// Forever GamepadUIDocumentation.lua; None deliberately differs from possession.
const STANCE_OVERRIDES: &[(&str, i32)] = &[
    ("None", 1),
    ("Page1LeftBar", 2),
    ("Page1RightBar", 3),
    ("Page1BottomBar", 4),
    ("Page2TopBar", 5),
    ("Page2LeftBar", 6),
    ("Page2RightBar", 7),
    ("Page2BottomBar", 8),
    ("Page3TopBar", 9),
    ("Page3LeftBar", 10),
    ("Page3RightBar", 11),
    ("Page3BottomBar", 12),
];

const POSSESS_OVERRIDES: &[(&str, i32)] = &[
    ("SpecialPageTopBar", 1),
    ("Page1LeftBar", 2),
    ("Page1RightBar", 3),
    ("Page1BottomBar", 4),
    ("Page2TopBar", 5),
    ("Page2LeftBar", 6),
    ("Page2RightBar", 7),
    ("Page2BottomBar", 8),
    ("Page3TopBar", 9),
    ("Page3LeftBar", 10),
    ("Page3RightBar", 11),
    ("Page3BottomBar", 12),
];
