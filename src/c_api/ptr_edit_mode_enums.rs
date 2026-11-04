//! PTR 12.1.7.70493 Edit Mode declarations, not Lua initialization replacements.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_table, table_get, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) {
    if ACTIVE != ClientProfile::Ptr {
        return;
    }
    let enums = super::helpers::ensure_global_table(state, "Enum");
    for &(name, fields) in DECLARATIONS {
        let values = match table_get(state, enums, name) {
            values @ Val::Table(_) => values,
            _ => {
                let values = create_table(state);
                table_set(state, enums, name, values);
                values
            }
        };
        for &(field, value) in fields {
            table_set(state, values, field, Val::Num(f64::from(value)));
        }
    }
}

type EnumFields = (&'static str, &'static [(&'static str, i32)]);
const THREE_VALUES_META: &[(&str, i32)] = &[("NumValues", 3), ("MinValue", 0), ("MaxValue", 2)];

// Pinned Blizzard_APIDocumentationGenerated/EditModeManagerConstantsDocumentation.lua:
// BossFrameGrowDirection at 111–122; EditModePresetLayouts at 639–650;
// EditModeSwingTimerSetting at 702–717; EditModeSwingTimerVisibility at 732–743.
const DECLARATIONS: &[EnumFields] = &[
    // Same pinned document: individual system declarations at 778–782.
    (
        "EditModeSystem",
        &[
            ("MainActionBarEndCap", 26),
            ("GroupFinder", 27),
            ("LossOfControl", 28),
            ("SwingTimer", 29),
            ("GamepadMainActionBar", 30),
        ],
    ),
    // System metadata at 745–749; account settings and metadata at 215–260.
    (
        "EditModeSystemMeta",
        &[("NumValues", 31), ("MinValue", 0), ("MaxValue", 30)],
    ),
    (
        "EditModeAccountSetting",
        &[
            ("ShowGroupFinder", 35),
            ("ShowLossOfControl", 36),
            ("ShowSwingTimer", 37),
        ],
    ),
    (
        "EditModeAccountSettingMeta",
        &[("NumValues", 38), ("MinValue", 0), ("MaxValue", 37)],
    ),
    // Individual setting declarations at 505–509, 520, 556 and 635.
    (
        "EditModeGamepadMainActionBarSetting",
        &[
            ("Compact", 0),
            ("ShowEmptyBars", 1),
            ("ShowButtonPrompts", 2),
            ("ShowHighlight", 3),
            ("ShouldExpand", 4),
        ],
    ),
    ("EditModeGroupFinderSetting", &[("Size", 0)]),
    ("EditModeMainActionBarEndCapSetting", &[("Hidden", 0)]),
    // Same pinned document: endcap indices and metadata at 560–568.
    (
        "EditModeMainActionBarEndCapSystemIndices",
        &[("EndCapLeft", 1), ("EndCapRight", 2)],
    ),
    (
        "EditModeMainActionBarEndCapSystemIndicesMeta",
        &[("NumValues", 2), ("MinValue", 1), ("MaxValue", 2)],
    ),
    // Same pinned document: SwingTimer indices and metadata at 719–728.
    (
        "EditModeSwingTimerSystemIndices",
        &[("MainHand", 1), ("OffHand", 2), ("Ranged", 3)],
    ),
    (
        "EditModeSwingTimerSystemIndicesMeta",
        &[("NumValues", 3), ("MinValue", 1), ("MaxValue", 3)],
    ),
    (
        "EditModePersonalResourceDisplaySetting",
        &[("AllowMouseoverThrough", 15)],
    ),
    // Unit-frame declarations and metadata at 797–828.
    (
        "EditModeUnitFrameSetting",
        &[
            ("DebuffIconSize", 19),
            ("BuffIconSize", 22),
            ("UseHorizontalLayout", 23),
            ("GrowDirection", 24),
        ],
    ),
    (
        "EditModeUnitFrameSettingMeta",
        &[("NumValues", 25), ("MinValue", 0), ("MaxValue", 24)],
    ),
    (
        "BossFrameGrowDirection",
        &[("Left", 0), ("Center", 1), ("Right", 2)],
    ),
    ("BossFrameGrowDirectionMeta", THREE_VALUES_META),
    (
        "EditModePresetLayouts",
        &[("Modern", 0), ("Classic", 1), ("Gamepad", 2)],
    ),
    ("EditModePresetLayoutsMeta", THREE_VALUES_META),
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
        "EditModeSwingTimerVisibility",
        &[("Always", 0), ("InCombat", 1), ("Hidden", 2)],
    ),
    ("EditModeSwingTimerVisibilityMeta", THREE_VALUES_META),
];
