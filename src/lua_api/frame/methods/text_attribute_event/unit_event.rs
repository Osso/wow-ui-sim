use crate::lua_api::methods::{create_string, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::Val;
use rilua::vm::state::LuaState;

/// Lua stack position of the first unit argument after self and the event.
const FIRST_UNIT_ARGUMENT: i32 = 3;
/// R88aj: the handler reads at most four unit arguments.
const MAX_UNIT_ARGUMENTS: i32 = 4;

/// R88aj: read up to four unit arguments, stopping at the first one that is
/// not a string or number. Unknown tokens (and numbers) use a position but are
/// skipped. Tokens are case-insensitive.
///
/// VERIFIED for Mists 70631, Retail 70688 and Forever 70707 (the Forever cache
/// build). INFERRED for the Mists 70268, Retail, PTR, Wrath and other cached
/// builds: the handler is identical in all three decoded builds.
pub(super) fn scan_unit_arguments(state: &LuaState) -> Vec<String> {
    let mut units = Vec::new();
    for index in FIRST_UNIT_ARGUMENT..FIRST_UNIT_ARGUMENT + MAX_UNIT_ARGUMENTS {
        let value = stack_val(state, index);
        if !matches!(value, Val::Str(_) | Val::Num(_)) {
            break;
        }
        let unit = val_to_string(state, value).map(|unit| unit.to_ascii_lowercase());
        units.extend(unit.filter(|unit| is_valid_unit_filter(unit)));
    }
    units
}

pub(super) fn is_valid_unit_filter(unit: &str) -> bool {
    matches!(
        unit,
        "player" | "pet" | "vehicle" | "target" | "focus" | "mouseover" | "npc" | "none"
    ) || has_numbered_unit_prefix(unit)
}

pub(super) fn push_registered_unit(state: &mut LuaState, unit: Option<&str>) {
    if let Some(unit) = unit {
        let unit = create_string(state, unit);
        state.push(unit);
    } else {
        state.push(Val::Nil);
    }
}

fn has_numbered_unit_prefix(unit: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "party",
        "partypet",
        "raid",
        "raidpet",
        "boss",
        "arena",
        "arenapet",
        "nameplate",
    ];
    PREFIXES
        .iter()
        .any(|prefix| has_numeric_suffix(unit, prefix))
}

fn has_numeric_suffix(unit: &str, prefix: &str) -> bool {
    let Some(suffix) = unit.strip_prefix(prefix) else {
        return false;
    };
    !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
}
