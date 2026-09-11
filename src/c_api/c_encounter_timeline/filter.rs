use super::layout;
use crate::lua_api::methods::{borrow_state, table_set_num};
use crate::lua_bridge::stack_val;
use rilua::vm::{state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

fn optional_number(state: &LuaState, index: i32) -> LuaResult<Option<f64>> {
    match stack_val(state, index) {
        Val::Nil => Ok(None),
        Val::Num(value) if value.is_finite() && value >= 0.0 => Ok(Some(value)),
        _ => Err(runtime_error(
            "timeline filter must be a finite nonnegative number",
        )),
    }
}

fn optional_bool(state: &LuaState, index: i32) -> LuaResult<bool> {
    match stack_val(state, index) {
        Val::Nil => Ok(true),
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error("timeline exclusion filter must be a boolean")),
    }
}

pub(super) fn sorted_list(state: &mut LuaState) -> LuaResult<u32> {
    let count = optional_number(state, 1)?;
    if count.is_some_and(|count| count.fract() != 0.0) {
        return Err(runtime_error(
            "timeline maximum event count must be an integer",
        ));
    }
    let duration = optional_number(state, 2)?;
    let exclude_terminal = optional_bool(state, 3)?;
    let exclude_hidden = optional_bool(state, 4)?;
    let ids = filtered_ids(state, duration, exclude_terminal, exclude_hidden)?;
    let limit = count.map_or(ids.len(), |count| count.min(ids.len() as f64) as usize);
    let result = state.gc.alloc_table(Table::with_sizes(limit, 0));
    for (index, id) in ids.into_iter().take(limit).enumerate() {
        table_set_num(state, result, (index + 1) as f64, Val::Num(f64::from(id)));
    }
    state.push(Val::Table(result));
    Ok(1)
}

fn filtered_ids(
    state: &LuaState,
    duration: Option<f64>,
    terminal: bool,
    hidden: bool,
) -> LuaResult<Vec<u32>> {
    let sim = borrow_state(state)?;
    let timeline = &sim.encounter_timeline;
    Ok(layout::sorted_ids(timeline)
        .into_iter()
        .filter(|id| {
            let event = &timeline.events[id];
            let terminal_excluded = terminal && event.state.terminal();
            let hidden_excluded = hidden && event.track == layout::INDETERMINATE;
            let duration_allowed =
                duration.is_none_or(|maximum| layout::remaining(event) <= maximum);
            !terminal_excluded && !hidden_excluded && duration_allowed
        })
        .collect())
}
