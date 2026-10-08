//! Bonus objective queries backed by the existing ordered scenario steps.

use super::{ensure_namespace, helpers::set_table_array};
use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Scenario")?;
    table_set_rust_fn_static(state, namespace, "GetBonusSteps", get_bonus_steps)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetBonusStepRewardQuestID",
        get_bonus_step_reward_quest_id,
    )
}

fn get_bonus_steps(state: &mut LuaState) -> LuaResult<u32> {
    let step_ids: Vec<i32> = {
        let sim = borrow_state(state)?;
        sim.scenario
            .steps
            .iter()
            .filter(|step| sim.scenario.in_scenario && step.is_bonus_step)
            .map(|step| step.step_id)
            .collect()
    };
    let table = create_table(state);
    for (index, step_id) in step_ids.iter().enumerate() {
        set_table_array(
            state,
            table,
            index as i64 + 1,
            Val::Num(f64::from(*step_id)),
        );
    }
    state.push(table);
    Ok(1)
}

fn get_bonus_step_reward_quest_id(state: &mut LuaState) -> LuaResult<u32> {
    let step_id = i32::from_stack(state, 1)?;
    let quest_id = {
        let sim = borrow_state(state)?;
        sim.scenario
            .steps
            .iter()
            .find(|step| sim.scenario.in_scenario && step.is_bonus_step && step.step_id == step_id)
            .and_then(|step| step.bonus_reward_quest_id)
    };
    state.push(quest_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}
