//! Frame strata projection of supplied game-rule values.
use crate::lua_api::globals::game_rules::resolve_rule_name_from_enum_id;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const USAGE: &str = "Usage: local frameStrata = C_GameRules.GetGameRuleAsFrameStrata(gameRule)";
const STRATA: [&str; 10] = [
    "WORLD",
    "BACKGROUND",
    "LOW",
    "MEDIUM",
    "HIGH",
    "DIALOG",
    "FULLSCREEN",
    "FULLSCREEN_DIALOG",
    "TOOLTIP",
    "BLIZZARD",
];

pub(crate) fn query(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    #[cfg(feature = "retail-12-1-0")]
    let value = rilua::table_security::unwrap_secret(state, value)?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(text) => state
            .gc
            .string_arena
            .get(text)
            .and_then(|s| std::str::from_utf8(s.data()).ok())
            .and_then(|s| s.trim().parse::<f64>().ok()),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error(USAGE))?;
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(rilua::runtime_error(format!(
            "{USAGE} - integer overflow attempting to store {number}"
        )));
    }
    let name = resolve_rule_name_from_enum_id(state, number.trunc() as i64)
        .ok_or_else(|| rilua::runtime_error(USAGE))?;
    let payload = borrow_state(state)?
        .game_rules
        .rules
        .get(&name)
        .map(|rule| rule.as_int);
    let result = payload.map(|value| {
        usize::try_from(value)
            .ok()
            .and_then(|index| STRATA.get(index))
            .copied()
            .unwrap_or("UNKNOWN")
    });
    let result = match result {
        Some(text) => create_string(state, text),
        None => Val::Nil,
    };
    state.push(result);
    Ok(1)
}
