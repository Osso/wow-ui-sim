use super::inputs::{array, field};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set_static,
};
use crate::lua_api::state::ScenarioStep;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const NUMBERS: [&str; 8] = [
    "criteriaType",
    "quantity",
    "totalQuantity",
    "flags",
    "assetID",
    "criteriaID",
    "duration",
    "elapsed",
];
const CRITERIA_ID_INDEX: usize = 5;
const FLAGS: [&str; 4] = ["completed", "failed", "isWeightedProgress", "isFormatted"];

/// The pinned 14-field record; numeric/boolean defaults come from the declaration.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Criterion {
    description: String,
    quantity_string: String,
    numbers: [f64; 8],
    flags: [bool; 4],
}

fn decode_criterion(state: &mut LuaState, row: Val) -> LuaResult<Criterion> {
    let mut numbers = [0.0; 8];
    for (index, name) in NUMBERS.iter().enumerate() {
        numbers[index] = if *name == "criteriaID" {
            field(state, row, name)?
        } else {
            field::<Option<f64>>(state, row, name)?.unwrap_or(0.0)
        };
        if !numbers[index].is_finite() {
            return Err(rilua::runtime_error("criteria numbers must be finite"));
        }
    }
    let mut flags = [false; 4];
    for (index, name) in FLAGS.iter().enumerate() {
        flags[index] = field::<Option<bool>>(state, row, name)?.unwrap_or(false);
    }
    Ok(Criterion {
        description: field(state, row, "description")?,
        quantity_string: field(state, row, "quantityString")?,
        numbers,
        flags,
    })
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = crate::c_api::ensure_namespace(state, "C_ScenarioInfo")?;
    table_set_rust_fn_static(state, ns, "GetCriteriaInfo", current_criterion)?;
    table_set_rust_fn_static(state, ns, "GetCriteriaInfoByStep", step_criterion)?;
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        let scenario = crate::c_api::ensure_namespace(state, "C_Scenario")?;
        table_set_rust_fn_static(state, scenario, "ShouldShowCriteria", should_show_criteria)?;
    }
    Ok(())
}

/// Undocumented native; consumed by the Retail/PTR scenario tracker. Best guess,
/// native UNKNOWN: criteria are shown unless a scenario supplies otherwise.
fn should_show_criteria(state: &mut LuaState) -> LuaResult<u32> {
    let shown = !borrow_state(state)?.challenge_run.criteria_hidden;
    state.push(Val::Bool(shown));
    Ok(1)
}

pub(super) fn set_criteria_shown(state: &mut LuaState) -> LuaResult<u32> {
    let shown = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.challenge_run.criteria_hidden = !shown;
    dispatch_event_now(
        state,
        "SCENARIO_CRITERIA_SHOW_STATE_UPDATE",
        &[Val::Bool(shown)],
    )?;
    Ok(0)
}

pub(super) fn set_criteria(state: &mut LuaState) -> LuaResult<u32> {
    let row = stack_val(state, 1);
    let rows = array(state, stack_val(state, 2))?;
    let criteria = rows
        .into_iter()
        .map(|row| decode_criterion(state, row))
        .collect::<LuaResult<Vec<_>>>()?;
    let step = ScenarioStep {
        step_id: field(state, row, "stepID")?,
        title: field(state, row, "title")?,
        description: field(state, row, "description")?,
        num_criteria: criteria.len() as i32,
        completed: false,
        is_bonus_step: false,
        bonus_reward_quest_id: None,
    };
    let changed_ids = store_criteria(state, step, criteria)?;
    for id in changed_ids {
        dispatch_event_now(state, "SCENARIO_CRITERIA_UPDATE", &[Val::Num(id)])?;
    }
    Ok(0)
}

fn store_criteria(
    state: &LuaState,
    step: ScenarioStep,
    criteria: Vec<Criterion>,
) -> LuaResult<Vec<f64>> {
    let mut sim = borrow_state_mut(state)?;
    let old = sim.challenge_run.criteria.get(&step.step_id);
    let changed = criteria
        .iter()
        .enumerate()
        .filter(|(i, row)| old.and_then(|rows| rows.get(*i)) != Some(*row))
        .map(|(_, row)| row.numbers[CRITERIA_ID_INDEX])
        .collect();
    sim.challenge_run.criteria.insert(step.step_id, criteria);
    sim.scenario.current_step = step.step_id;
    sim.scenario.in_scenario = true;
    sim.scenario.steps.retain(|old| old.step_id != step.step_id);
    sim.scenario.steps.push(step);
    sim.scenario.num_steps = sim.scenario.steps.len() as i32;
    Ok(changed)
}

fn current_criterion(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let step = borrow_state(state)?.scenario.current_step;
    push_criterion(state, step, index)
}

fn step_criterion(state: &mut LuaState) -> LuaResult<u32> {
    let step = i32::from_stack(state, 1)?;
    let index = i32::from_stack(state, 2)?;
    push_criterion(state, step, index)
}

fn push_criterion(state: &mut LuaState, step: i32, index: i32) -> LuaResult<u32> {
    let row = {
        let sim = borrow_state(state)?;
        usize::try_from(index.saturating_sub(1))
            .ok()
            .and_then(|index| sim.challenge_run.criteria.get(&step)?.get(index).cloned())
    };
    let Some(row) = row.filter(|_| index > 0) else {
        return Ok(0);
    };
    let table = create_table(state);
    state.push(table);
    let text = create_string(state, &row.description);
    table_set_static(state, table, "description", text);
    let text = create_string(state, &row.quantity_string);
    table_set_static(state, table, "quantityString", text);
    for (name, value) in NUMBERS.into_iter().zip(row.numbers) {
        table_set_static(state, table, name, Val::Num(value));
    }
    for (name, value) in FLAGS.into_iter().zip(row.flags) {
        table_set_static(state, table, name, Val::Bool(value));
    }
    Ok(1)
}
