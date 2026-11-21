//! Explicit simulator-only objective producer. No native completion/reward policy.

use crate::c_api::quest_objectives::{
    QuestObjective, QuestObjectiveProgress, QuestProgressCurrency, QuestProgressItem,
    QuestProgressRequirements, changed_criteria, commit_objective_progress,
    commit_progress_requirements, valid_progress, valid_progress_requirements,
};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state_mut, create_string, table_get};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn set_quest_objectives(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let progress = decode_progress(state)?;
    let criteria = commit_supplied_progress(state, quest_id, progress)?;
    let Some((criteria, autocomplete)) = criteria else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    for objective in criteria {
        dispatch_criteria(state, quest_id, &objective)?;
    }
    dispatch_event_now(
        state,
        "QUEST_WATCH_UPDATE",
        &[Val::Num(f64::from(quest_id))],
    )?;
    dispatch_event_now(state, "QUEST_LOG_UPDATE", &[])?;
    crate::c_api::quest_readiness::notify_readiness(state, quest_id, autocomplete)?;
    state.push(Val::Bool(true));
    Ok(1)
}

pub(super) fn set_quest_progress_requirements(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let requirements = decode_progress_requirements(state)?;
    let committed = commit_progress_requirements(
        &mut borrow_state_mut(state)?.quest_log_entries,
        quest_id,
        requirements,
    );
    state.push(Val::Bool(committed));
    Ok(1)
}

fn decode_progress_requirements(state: &mut LuaState) -> LuaResult<QuestProgressRequirements> {
    let table = stack_val(state, 2);
    let requirements = QuestProgressRequirements {
        money: read_field(state, table, "money")?,
        hide_required_items: read_field(state, table, "hideRequiredItems")?,
        items: decode_progress_items(state, table)?,
        currencies: decode_progress_currencies(state, table)?,
        text: read_field(state, table, "text")?,
        portrait_display_id: read_field(state, table, "portraitDisplayID")?,
        portrait_name: read_field(state, table, "portraitName")?,
        portrait_text: read_field(state, table, "portraitText")?,
    };
    if !valid_progress_requirements(&requirements) {
        return Err(rilua::runtime_error(
            "SetQuestProgressRequirements: invalid values",
        ));
    }
    Ok(requirements)
}

fn decode_progress_items(state: &mut LuaState, table: Val) -> LuaResult<Vec<QuestProgressItem>> {
    decode_rows(state, table, "items", |state, row| {
        Ok(QuestProgressItem {
            name: read_field(state, row, "name")?,
            texture: read_field(state, row, "texture")?,
            required_amount: read_field(state, row, "requiredAmount")?,
            hidden: read_field(state, row, "hidden")?,
        })
    })
}

fn decode_progress_currencies(
    state: &mut LuaState,
    table: Val,
) -> LuaResult<Vec<QuestProgressCurrency>> {
    decode_rows(state, table, "currencies", |state, row| {
        Ok(QuestProgressCurrency {
            currency_id: read_field(state, row, "currencyID")?,
            name: read_field(state, row, "name")?,
            texture: read_field(state, row, "texture")?,
            quality: read_field(state, row, "quality")?,
            required_amount: read_field(state, row, "requiredAmount")?,
        })
    })
}

fn decode_rows<T>(
    state: &mut LuaState,
    table: Val,
    field: &str,
    mut decode: impl FnMut(&mut LuaState, Val) -> LuaResult<T>,
) -> LuaResult<Vec<T>> {
    let rows = table_get(state, table, field);
    let Val::Table(rows_ref) = rows else {
        return Err(rilua::runtime_error(
            "SetQuestProgressRequirements: rows must be a table",
        ));
    };
    let row_values = {
        let rows_table = state.gc.tables.get(rows_ref).ok_or_else(|| {
            rilua::runtime_error("SetQuestProgressRequirements: rows unavailable")
        })?;
        (1..=rows_table.len(&state.gc.string_arena))
            .map(|index| rows_table.get_int(index as i64))
            .collect::<Vec<_>>()
    };
    row_values
        .into_iter()
        .map(|row| decode(state, row))
        .collect()
}

fn commit_supplied_progress(
    state: &mut LuaState,
    quest_id: i32,
    progress: QuestObjectiveProgress,
) -> LuaResult<Option<(Vec<QuestObjective>, bool)>> {
    let mut sim = borrow_state_mut(state)?;
    let previous = sim
        .quest_log_entries
        .objective_progress
        .get(&quest_id)
        .cloned();
    if !commit_objective_progress(&mut sim.quest_log_entries, quest_id, progress.clone()) {
        return Ok(None);
    }
    let autocomplete =
        crate::c_api::quest_readiness::commit_objective_readiness(&mut sim, quest_id);
    Ok(Some((
        changed_criteria(previous.as_ref(), &progress),
        autocomplete,
    )))
}

fn decode_progress(state: &mut LuaState) -> LuaResult<QuestObjectiveProgress> {
    let rows = read_objective_rows(state)?;
    let objectives = rows
        .into_iter()
        .map(|row| decode_objective(state, row))
        .collect::<LuaResult<Vec<_>>>()?;
    let progress = QuestObjectiveProgress {
        objectives,
        percentage: Option::<f64>::from_stack(state, 3)?,
        declared_objective_count: None,
    };
    if !valid_progress(&progress) {
        return Err(rilua::runtime_error(
            "SetQuestObjectives: counts must be nonnegative, criteria IDs positive, percentage finite in 0..100",
        ));
    }
    Ok(progress)
}

fn read_objective_rows(state: &LuaState) -> LuaResult<Vec<Val>> {
    let Val::Table(rows_ref) = stack_val(state, 2) else {
        return Err(rilua::runtime_error(
            "SetQuestObjectives: rows must be a table",
        ));
    };
    let table = state
        .gc
        .tables
        .get(rows_ref)
        .ok_or_else(|| rilua::runtime_error("SetQuestObjectives: rows table unavailable"))?;
    Ok((1..=table.len(&state.gc.string_arena))
        .map(|index| table.get_int(index as i64))
        .collect())
}

fn decode_objective(state: &mut LuaState, row: Val) -> LuaResult<QuestObjective> {
    if !matches!(row, Val::Table(_)) {
        return Err(rilua::runtime_error(
            "SetQuestObjectives: each objective must be a table",
        ));
    }
    let Val::Bool(finished) = table_get(state, row, "finished") else {
        return Err(rilua::runtime_error(
            "SetQuestObjectives: finished must be a boolean",
        ));
    };
    Ok(QuestObjective {
        text: read_field(state, row, "text")?,
        objective_type: read_field(state, row, "type")?,
        finished,
        num_fulfilled: read_field(state, row, "numFulfilled")?,
        num_required: read_field(state, row, "numRequired")?,
        criteria_id: read_field(state, row, "criteriaID")?,
    })
}

fn read_field<T: FromStack>(state: &mut LuaState, row: Val, key: &str) -> LuaResult<T> {
    let value = table_get(state, row, key);
    let top = state.top;
    state.push(value);
    let result = T::from_stack(state, -1);
    state.top = top;
    result
}

fn dispatch_criteria(
    state: &mut LuaState,
    quest_id: i32,
    objective: &QuestObjective,
) -> LuaResult<()> {
    let Some(criteria_id) = objective.criteria_id else {
        return Ok(());
    };
    let top = state.top;
    let description = create_string(state, &objective.text);
    state.push(description);
    let result = dispatch_event_now(
        state,
        "QUEST_LOG_CRITERIA_UPDATE",
        &[
            Val::Num(f64::from(quest_id)),
            Val::Num(f64::from(criteria_id)),
            description,
            Val::Num(f64::from(objective.num_fulfilled)),
            Val::Num(f64::from(objective.num_required)),
        ],
    );
    state.top = top;
    result
}
