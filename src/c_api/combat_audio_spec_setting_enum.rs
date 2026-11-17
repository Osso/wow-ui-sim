//! Cached profile data for `Enum.CombatAudioAlertSpecSetting`.
//!
//! Retail 12.1.5.70783, PTR 12.1.7.70493, Forever 1.60.1.70707, and
//! Era/Anniversary 1.15.9.70003 document the same nine members in
//! `CombatAudioAlertSharedDocumentation.lua`. Mists uses its existing native
//! publisher. Wrath has no cached declaration and stays unpublished.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_table, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

const MEMBERS: [(&str, i32); 9] = [
    ("Resource1Percent", 0),
    ("Resource1Format", 1),
    ("Resource1Voice", 2),
    ("Resource1Volume", 3),
    ("Resource2Percent", 4),
    ("Resource2Format", 5),
    ("Resource2Voice", 6),
    ("Resource2Volume", 7),
    ("SayIfTargeted", 8),
];

const MIN_VALUE: i32 = 0;
const MAX_VALUE: i32 = 8;
const NUM_VALUES: i32 = 9;

fn publishes_cached_enum_for_active_profile() -> bool {
    match ACTIVE {
        ClientProfile::Retail => cfg!(feature = "client-retail"),
        ClientProfile::Ptr
        | ClientProfile::WowForever
        | ClientProfile::Era
        | ClientProfile::Anniversary => true,
        ClientProfile::Wrath | ClientProfile::Mists => false,
    }
}

pub(crate) fn register(state: &mut LuaState) {
    if !publishes_cached_enum_for_active_profile() {
        return;
    }

    let enums = super::helpers::ensure_global_table(state, "Enum");
    let values = create_table(state);
    table_set(state, enums, "CombatAudioAlertSpecSetting", values);
    for (name, value) in MEMBERS {
        table_set(state, values, name, Val::Num(f64::from(value)));
    }

    let meta = create_table(state);
    table_set(state, enums, "CombatAudioAlertSpecSettingMeta", meta);
    for (name, value) in [
        ("MinValue", MIN_VALUE),
        ("MaxValue", MAX_VALUE),
        ("NumValues", NUM_VALUES),
    ] {
        table_set(state, meta, name, Val::Num(f64::from(value)));
    }
}
