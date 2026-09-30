//! Documented 12.0.5 numeric publication; no downstream domain behavior implied.

use crate::lua_api::methods::{table_get, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) {
    let enums = super::ensure_global_table(state, "Enum");
    for &(name, member, value) in ADDITIONS {
        let values = table_get(state, enums, name);
        table_set(state, values, member, Val::Num(f64::from(value)));
        publish_metadata(state, enums, name, values);
    }
}

pub(crate) fn refresh_house_finder_metadata(state: &mut LuaState) {
    let enums = super::ensure_global_table(state, "Enum");
    let name = "HouseFinderSuggestionReason";
    let values = table_get(state, enums, name);
    publish_metadata(state, enums, name, values);
}

fn publish_metadata(state: &mut LuaState, enums: Val, name: &str, values: Val) {
    let Val::Table(values_ref) = values else {
        panic!("Enum.{name} must be initialized before 12.0.5 additions");
    };
    let members = state.gc.tables.get(values_ref).unwrap().hash_entries();
    let numbers = members.iter().filter_map(|(_, value)| match value {
        Val::Num(number) => Some(*number),
        _ => None,
    });
    let (count, min, max) = numbers.fold(
        (0, f64::INFINITY, f64::NEG_INFINITY),
        |(count, min, max), value| (count + 1, min.min(value), max.max(value)),
    );
    let metadata = table_get(state, enums, &format!("{name}Meta"));
    table_set(state, metadata, "NumValues", Val::Num(f64::from(count)));
    table_set(state, metadata, "MinValue", Val::Num(min));
    table_set(state, metadata, "MaxValue", Val::Num(max));
}

// Cached official CurrencyConstantsDocumentation.lua:95–113,
// TransmogSharedDocumentation.lua:46–57, PlayerHousingConstantsDocumentation.lua:52–69.
// Only the three names added by retained 12.0.5 patch notes are published here.
const ADDITIONS: &[(&str, &str, i32)] = &[
    ("CurrencyFlagsB", "CurrencyBNoBonusXP", 2048),
    ("TransmogIllusionFlags", "AllowedRangedShieldsHoldables", 4),
    ("HouseFinderSuggestionReason", "HomeOwner", 64),
];
