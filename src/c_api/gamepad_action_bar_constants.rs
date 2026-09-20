//! Forever 1.60.1 GamepadUIDocumentation constants; no input-device behavior.

use crate::lua_api::methods::{create_table, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(super) fn register(state: &mut LuaState) {
    let constants = super::helpers::ensure_global_table(state, "Constants");
    let values = create_table(state);
    for (name, value) in [
        ("NUM_SLOTS_PER_GAMEPAD_ACTION_BAR_GROUP", 4),
        ("NUM_GROUPS_PER_GAMEPAD_ACTION_BAR", 2),
        ("NUM_SLOTS_PER_GAMEPAD_ACTION_BAR", 8),
        ("NUM_PAGEABLE_BARS_IN_GAMEPAD_ACTION_BAR_PAGE_UNIT", 4),
        (
            "NUM_PAGEABLE_SLOTS_PER_GAMEPAD_ACTION_BAR_PAGE_UNIT_STANDARD_PAGE",
            32,
        ),
        ("NUM_RESERVED_SLOTS_PER_GAMEPAD_ACTION_BAR_PAGE_UNIT", 4),
        ("NUM_PAGES_PER_GAMEPAD_ACTION_BAR_PAGE_UNIT", 4),
        ("NUM_STANDARD_PAGES_PER_GAMEPAD_ACTION_BAR_PAGE_UNIT", 3),
        ("GAMEPAD_ACTION_BAR_PAGE_UNIT_SPECIAL_PAGE_INDEX", 4),
        ("NUM_PAGEABLE_SLOTS_PER_GAMEPAD_ACTION_BAR_PAGE_UNIT", 96),
        ("NUM_GAMEPAD_STANCE_ACTION_BARS", 5),
    ] {
        table_set(state, values, name, Val::Num(f64::from(value)));
    }
    table_set(state, constants, "GamepadActionBarConstants", values);
}
