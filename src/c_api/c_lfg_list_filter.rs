//! Saved Group Finder filter inputs, copied into simulator state.
use crate::lua_api::methods::{borrow_state_mut, table_get};
use crate::lua_api::state_types::LfgAdvancedFilter;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

fn read_field(state: &mut LuaState, data: Val, name: &str) -> LuaResult<Val> {
    let value = table_get(state, data, name);
    rilua::table_security::unwrap_secret(state, value)
}

fn read_bool(state: &mut LuaState, data: Val, name: &str) -> LuaResult<bool> {
    match read_field(state, data, name)? {
        Val::Nil => Ok(false),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "AdvancedFilterOptions.{name} must be a boolean"
        ))),
    }
}

fn read_activities(state: &mut LuaState, data: Val) -> LuaResult<Vec<u32>> {
    let activities = read_field(state, data, "activities")?;
    let Val::Table(reference) = activities else {
        return Err(runtime_error(
            "AdvancedFilterOptions.activities must be a table",
        ));
    };
    let mut out = Vec::new();
    for index in 1.. {
        let value = state
            .gc
            .tables
            .get(reference)
            .ok_or_else(|| runtime_error("activities table is unavailable"))?
            .get_int(index);
        match rilua::table_security::unwrap_secret(state, value)? {
            Val::Nil => return Ok(out),
            Val::Num(id)
                if id.is_finite() && id.fract() == 0.0 && id >= 0.0 && id <= u32::MAX as f64 =>
            {
                out.push(id as u32)
            }
            _ => return Err(runtime_error("activities entries must be u32 numbers")),
        }
    }
    unreachable!()
}

pub(crate) fn save_advanced_filter(state: &mut LuaState) -> LuaResult<u32> {
    let data = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(data, Val::Table(_)) {
        return Err(runtime_error(
            "options must be an AdvancedFilterOptions table",
        ));
    }
    let minimum_rating = match read_field(state, data, "minimumRating")? {
        Val::Nil => 0,
        Val::Num(value) if value.is_finite() => value as i32,
        _ => return Err(runtime_error("minimumRating must be a number")),
    };
    let filter = LfgAdvancedFilter {
        needs_tank: read_bool(state, data, "needsTank")?,
        needs_healer: read_bool(state, data, "needsHealer")?,
        needs_damage: read_bool(state, data, "needsDamage")?,
        needs_my_class: read_bool(state, data, "needsMyClass")?,
        has_tank: read_bool(state, data, "hasTank")?,
        has_healer: read_bool(state, data, "hasHealer")?,
        activities: read_activities(state, data)?,
        minimum_rating,
        difficulty_normal: read_bool(state, data, "difficultyNormal")?,
        difficulty_heroic: read_bool(state, data, "difficultyHeroic")?,
        difficulty_mythic: read_bool(state, data, "difficultyMythic")?,
        difficulty_mythic_plus: read_bool(state, data, "difficultyMythicPlus")?,
        general_playstyle1: read_bool(state, data, "generalPlaystyle1")?,
        general_playstyle2: read_bool(state, data, "generalPlaystyle2")?,
        general_playstyle3: read_bool(state, data, "generalPlaystyle3")?,
        general_playstyle4: read_bool(state, data, "generalPlaystyle4")?,
    };
    borrow_state_mut(state)?.lfg_advanced_filter = filter;
    Ok(0)
}
