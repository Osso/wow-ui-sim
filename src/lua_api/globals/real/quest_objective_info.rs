//! Legacy quest-objective rows exposed to objective tracker Lua.

use crate::c_api::quest_objectives::{QuestObjective, objectives_for_quest};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetQuestObjectiveInfo", get_quest_objective_info)?;
    Ok(())
}

fn get_quest_objective_info(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let objective_index = i32::from_stack(state, 2)?;
    let Some(objective) = read_objective(state, quest_id, objective_index)? else {
        return Ok(0);
    };
    push_objective_fields(state, &objective);
    Ok(5)
}

fn read_objective(
    state: &LuaState,
    quest_id: i32,
    objective_index: i32,
) -> LuaResult<Option<QuestObjective>> {
    let Some(index) = objective_index
        .checked_sub(1)
        .and_then(|i| usize::try_from(i).ok())
    else {
        return Ok(None);
    };
    Ok(
        objectives_for_quest(&borrow_state(state)?.quest_log_entries, quest_id)
            .get(index)
            .cloned(),
    )
}

// INFERRED R88cq: preserve the stored objective row fields in the legacy
// five-value order consumed by Blizzard_BonusObjectiveTracker.lua:367.
fn push_objective_fields(state: &mut LuaState, objective: &QuestObjective) {
    let text = create_string(state, &objective.text);
    state.push(text);
    let objective_type = create_string(state, &objective.objective_type);
    state.push(objective_type);
    state.push(Val::Bool(objective.finished));
    state.push(Val::Num(f64::from(objective.num_fulfilled)));
    state.push(Val::Num(f64::from(objective.num_required)));
}
