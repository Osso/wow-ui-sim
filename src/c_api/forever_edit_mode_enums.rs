//! Finite Forever 1.60.1 Edit Mode enum additions from its generated API docs.

use crate::lua_api::methods::{create_table, table_get, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

type EnumFields = (&'static str, &'static [(&'static str, i32)]);

pub(crate) fn register(state: &mut LuaState) {
    let enums = super::helpers::ensure_global_table(state, "Enum");
    for &(name, fields) in ADDITIONS {
        publish_fields(state, enums, name, fields);
    }
    let (name, fields) = crate::lua_api::globals::enum_data::RAID_DISPEL_OVERLAY_TYPE;
    for (index, &field) in fields.iter().enumerate() {
        publish_fields(state, enums, name, &[(field, index as i32)]);
    }
    let (name, fields) = crate::lua_api::globals::enum_data::RAID_DISPEL_OVERLAY_TYPE_META;
    publish_fields(state, enums, name, fields);
}

fn publish_fields(state: &mut LuaState, enums: Val, name: &str, fields: &[(&str, i32)]) {
    let values = match table_get(state, enums, name) {
        value @ Val::Table(_) => value,
        _ => {
            let value = create_table(state);
            table_set(state, enums, name, value);
            value
        }
    };
    for &(field, value) in fields {
        table_set(state, values, field, Val::Num(f64::from(value)));
    }
}

const ADDITIONS: &[EnumFields] = &[
    (
        "EditModeAccountSetting",
        &[
            ("ShowRaidWarning", 33),
            ("ShowTotemActionBar", 34),
            ("ShowGroupFinder", 35),
            ("ShowLossOfControl", 36),
            ("ShowSwingTimer", 37),
        ],
    ),
    (
        "EditModeAccountSettingMeta",
        &[("NumValues", 38), ("MinValue", 0), ("MaxValue", 37)],
    ),
    ("EditModeGroupFinderSetting", &[("Size", 0)]),
    (
        "EditModeGroupFinderSettingMeta",
        &[("NumValues", 1), ("MinValue", 0), ("MaxValue", 0)],
    ),
    ("EditModeLossOfControlSetting", &[("Size", 0)]),
    (
        "EditModeLossOfControlSettingMeta",
        &[("NumValues", 1), ("MinValue", 0), ("MaxValue", 0)],
    ),
    ("EditModeMainActionBarEndCapSetting", &[("Hidden", 0)]),
    (
        "EditModeMainActionBarEndCapSettingMeta",
        &[("NumValues", 1), ("MinValue", 0), ("MaxValue", 0)],
    ),
    (
        "EditModeMainActionBarEndCapSystemIndices",
        &[("EndCapLeft", 1), ("EndCapRight", 2)],
    ),
    (
        "EditModeMainActionBarEndCapSystemIndicesMeta",
        &[("NumValues", 2), ("MinValue", 1), ("MaxValue", 2)],
    ),
    ("EditModeMicroMenuSetting", &[("DeprecatedEyeSize", 3)]),
    (
        "EditModeMicroMenuSettingMeta",
        &[("NumValues", 4), ("MinValue", 0), ("MaxValue", 3)],
    ),
    ("EditModeMinimapSetting", &[("IconScale", 3)]),
    (
        "EditModeMinimapSettingMeta",
        &[("NumValues", 4), ("MinValue", 0), ("MaxValue", 3)],
    ),
    ("EditModePresetLayouts", &[("Gamepad", 2)]),
    (
        "EditModePresetLayoutsMeta",
        &[("NumValues", 3), ("MinValue", 0), ("MaxValue", 2)],
    ),
    ("EditModeRaidWarningSetting", &[("None", 0)]),
    (
        "EditModeRaidWarningSettingMeta",
        &[("NumValues", 1), ("MinValue", 0), ("MaxValue", 0)],
    ),
    (
        "EditModeSwingTimerSetting",
        &[
            ("Scale", 0),
            ("Opacity", 1),
            ("Visibility", 2),
            ("Width", 3),
            ("Height", 4),
            ("ShowBarTitle", 5),
            ("ShowTime", 6),
        ],
    ),
    (
        "EditModeSwingTimerSettingMeta",
        &[("NumValues", 7), ("MinValue", 0), ("MaxValue", 6)],
    ),
    (
        "EditModeSwingTimerSystemIndices",
        &[("MainHand", 1), ("OffHand", 2), ("Ranged", 3)],
    ),
    (
        "EditModeSwingTimerSystemIndicesMeta",
        &[("NumValues", 3), ("MinValue", 1), ("MaxValue", 3)],
    ),
    (
        "EditModeSwingTimerVisibility",
        &[("Always", 0), ("InCombat", 1), ("Hidden", 2)],
    ),
    (
        "EditModeSwingTimerVisibilityMeta",
        &[("NumValues", 3), ("MinValue", 0), ("MaxValue", 2)],
    ),
    (
        "EditModeSystem",
        &[
            ("RaidWarning", 24),
            ("TotemActionBar", 25),
            ("MainActionBarEndCap", 26),
            ("GroupFinder", 27),
            ("LossOfControl", 28),
            ("SwingTimer", 29),
        ],
    ),
    (
        "EditModeSystemMeta",
        &[("NumValues", 30), ("MinValue", 0), ("MaxValue", 29)],
    ),
    (
        "EditModeUnitFrameSetting",
        &[("DebuffIconSize", 19), ("BuffIconSize", 22)],
    ),
    (
        "EditModeUnitFrameSettingMeta",
        &[("NumValues", 23), ("MinValue", 0), ("MaxValue", 22)],
    ),
];
