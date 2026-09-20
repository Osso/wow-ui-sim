//! Source-published Forever 1.60.1 minimap, ping, transmog and possess-bar data.

use crate::lua_api::methods::{create_table, table_get, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) {
    let enums = super::helpers::ensure_global_table(state, "Enum");
    register_tracking_filters(state, enums);
    register_ping_results(state, enums);
    publish(state, enums, "GamepadPossessBarOverride", POSSESS_OVERRIDES);
    publish(
        state,
        enums,
        "GamepadPossessBarOverrideMeta",
        &[("MinValue", 1), ("MaxValue", 12), ("NumValues", 12)],
    );
    let constants = super::helpers::ensure_global_table(state, "Constants");
    publish(state, constants, "Transmog", &[("NoTransmogID", 0)]);
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
