use crate::c_api::trainer_service::{TrainerService, TrainerServiceType};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

type TrainerGlobal = (&'static str, fn(&mut LuaState) -> LuaResult<u32>);

const TRAINER_GLOBALS: &[TrainerGlobal] = &[
    ("BuyTrainerService", buy_trainer_service),
    ("CloseTrainer", close_trainer),
    ("GetNumTrainerServices", get_num_trainer_services),
    ("GetTrainerSelectionIndex", get_trainer_selection_index),
    (
        "GetTrainerServiceAbilityReq",
        get_trainer_service_ability_req,
    ),
    ("GetTrainerServiceCost", get_trainer_service_cost),
    (
        "GetTrainerServiceDescription",
        get_trainer_service_description,
    ),
    ("GetTrainerServiceIcon", get_trainer_service_icon),
    ("GetTrainerServiceInfo", get_trainer_service_info),
    ("GetTrainerServiceLevelReq", get_trainer_service_level_req),
    (
        "GetTrainerServiceNumAbilityReq",
        get_trainer_service_num_ability_req,
    ),
    ("GetTrainerServiceSkillLine", get_trainer_service_skill_line),
    ("GetTrainerServiceSkillReq", get_trainer_service_skill_req),
    (
        "GetTrainerServiceTypeFilter",
        get_trainer_service_type_filter,
    ),
    ("GetTrainerServiceStepIndex", get_trainer_service_step_index),
    (
        "GetTrainerTradeskillRankValues",
        get_trainer_tradeskill_rank_values,
    ),
    ("IsTradeskillTrainer", is_tradeskill_trainer),
    ("SelectTrainerService", select_trainer_service),
    (
        "SetTrainerServiceTypeFilter",
        set_trainer_service_type_filter,
    ),
];

pub(crate) fn register_legacy_globals(state: &mut LuaState) -> LuaResult<()> {
    for (name, function) in TRAINER_GLOBALS {
        table_set_rust_fn_static(state, state.global, name, *function)?;
    }
    Ok(())
}

fn get_num_trainer_services(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.trainer.visible_service_count();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_trainer_service_info(state: &mut LuaState) -> LuaResult<u32> {
    let service = service_from_stack(state)?;
    push_service_info(state, service.as_ref());
    Ok(6)
}

fn push_service_info(state: &mut LuaState, service: Option<&TrainerService>) {
    let Some(service) = service else {
        for _ in 0..6 {
            state.push(Val::Nil);
        }
        return;
    };
    let name = create_string(state, &service.name);
    let service_type = create_string(state, service.service_type.as_lua_value());
    state.push(name);
    state.push(service_type);
    state.push(Val::Num(service.icon as f64));
    push_optional_number(state, service.required_level.map(f64::from));
    let sub_text = create_string(state, &service.sub_text);
    state.push(sub_text);
    state.push(Val::Nil);
}

fn get_trainer_service_cost(state: &mut LuaState) -> LuaResult<u32> {
    let service = service_from_stack(state)?;
    state.push(Val::Num(
        service.as_ref().map_or(0.0, |service| service.cost as f64),
    ));
    state.push(Val::Bool(
        service.is_some_and(|service| service.is_profession),
    ));
    Ok(2)
}

fn get_trainer_service_icon(state: &mut LuaState) -> LuaResult<u32> {
    let icon = service_from_stack(state)?.map(|service| service.icon);
    push_optional_number(state, icon.map(f64::from));
    Ok(1)
}

fn get_trainer_service_level_req(state: &mut LuaState) -> LuaResult<u32> {
    let level = service_from_stack(state)?.and_then(|service| service.required_level);
    push_optional_number(state, level.map(f64::from));
    Ok(1)
}

fn get_trainer_service_skill_req(state: &mut LuaState) -> LuaResult<u32> {
    let requirement = service_from_stack(state)?.and_then(|service| service.required_skill);
    let Some(requirement) = requirement else {
        state.push(Val::Nil);
        state.push(Val::Nil);
        state.push(Val::Nil);
        return Ok(3);
    };
    let skill_name = create_string(state, &requirement.name);
    state.push(skill_name);
    state.push(Val::Num(requirement.rank as f64));
    state.push(Val::Bool(requirement.met));
    Ok(3)
}

fn get_trainer_service_skill_line(state: &mut LuaState) -> LuaResult<u32> {
    let requirement = service_from_stack(state)?.and_then(|service| service.required_skill);
    match requirement {
        Some(requirement) => {
            let skill_name = create_string(state, &requirement.name);
            state.push(skill_name);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_trainer_service_num_ability_req(state: &mut LuaState) -> LuaResult<u32> {
    let count = service_from_stack(state)?.map_or(0, |service| service.ability_requirements.len());
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_trainer_service_ability_req(state: &mut LuaState) -> LuaResult<u32> {
    let service_index = i32::from_stack(state, 1)?;
    let requirement_index =
        usize::try_from(i32::from_stack(state, 2)?.saturating_sub(1)).unwrap_or(usize::MAX);
    let requirement = service_from_index(state, service_index)?
        .and_then(|service| service.ability_requirements.get(requirement_index).cloned());
    match requirement {
        Some(requirement) => {
            let name = create_string(state, &requirement.name);
            state.push(name);
            state.push(Val::Bool(requirement.met));
        }
        None => {
            state.push(Val::Nil);
            state.push(Val::Bool(false));
        }
    }
    Ok(2)
}

fn get_trainer_service_type_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = String::from_stack(state, 1)?;
    let visible = match filter.as_str() {
        "available" => borrow_state(state)?.trainer.show_available,
        "unavailable" => borrow_state(state)?.trainer.show_unavailable,
        "used" => borrow_state(state)?.trainer.show_used,
        _ => false,
    };
    state.push(Val::Bool(visible));
    Ok(1)
}

fn set_trainer_service_type_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = String::from_stack(state, 1)?;
    let visible = bool::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    match filter.as_str() {
        "available" => sim.trainer.show_available = visible,
        "unavailable" => sim.trainer.show_unavailable = visible,
        "used" => sim.trainer.show_used = visible,
        _ => {}
    }
    Ok(0)
}

fn select_trainer_service(state: &mut LuaState) -> LuaResult<u32> {
    let service_index = i32::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.trainer.selected_service_index = if sim.trainer.service_at(service_index).is_some() {
        Some(service_index)
    } else {
        None
    };
    Ok(0)
}

fn get_trainer_selection_index(state: &mut LuaState) -> LuaResult<u32> {
    let selected = borrow_state(state)?
        .trainer
        .selected_service_index
        .unwrap_or(0);
    state.push(Val::Num(selected as f64));
    Ok(1)
}

fn get_trainer_service_step_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = borrow_state(state)?.trainer.step_service_index;
    push_optional_number(state, index.map(f64::from));
    Ok(1)
}

fn get_trainer_tradeskill_rank_values(state: &mut LuaState) -> LuaResult<u32> {
    let values = borrow_state(state)?.trainer.tradeskill_rank;
    push_rank_values(state, values);
    Ok(3)
}

fn push_rank_values(state: &mut LuaState, values: Option<(u32, u32, u32)>) {
    let values =
        values.map(|(rank, maximum, modifier)| [rank as f64, maximum as f64, modifier as f64]);
    for value in values.map_or([None; 3], |values| values.map(Some)) {
        push_optional_number(state, value);
    }
}

fn is_tradeskill_trainer(state: &mut LuaState) -> LuaResult<u32> {
    let has_services = borrow_state(state)?.trainer.visible_service_count() > 0;
    state.push(Val::Bool(has_services));
    Ok(1)
}

fn buy_trainer_service(state: &mut LuaState) -> LuaResult<u32> {
    let service_index = i32::from_stack(state, 1)?;
    let changed = mark_service_used(&mut borrow_state_mut(state)?, service_index);
    if changed {
        fire_named_event_state(state, "TRAINER_UPDATE", &[]);
    }
    Ok(0)
}

fn mark_service_used(sim: &mut crate::lua_api::state::SimState, service_index: i32) -> bool {
    let Some(service) = sim.trainer.service_at(service_index) else {
        return false;
    };
    if service.service_type != TrainerServiceType::Available {
        return false;
    }
    let Some(stored) = sim
        .trainer
        .services
        .iter_mut()
        .find(|stored| stored.spell_id == service.spell_id)
    else {
        return false;
    };
    stored.service_type = TrainerServiceType::Used;
    true
}

fn close_trainer(state: &mut LuaState) -> LuaResult<u32> {
    fire_named_event_state(state, "TRAINER_CLOSED", &[]);
    Ok(0)
}

fn get_trainer_service_description(state: &mut LuaState) -> LuaResult<u32> {
    let description = service_from_stack(state)?
        .and_then(|service| u32::try_from(service.spell_id).ok())
        .and_then(crate::spell_descriptions::get_spell_description)
        .filter(|description| !description.is_empty());
    match description {
        Some(description) => {
            let description = create_string(state, description);
            state.push(description);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn service_from_stack(state: &mut LuaState) -> LuaResult<Option<TrainerService>> {
    let index = i32::from_stack(state, 1)?;
    service_from_index(state, index)
}

fn service_from_index(state: &LuaState, index: i32) -> LuaResult<Option<TrainerService>> {
    Ok(borrow_state(state)?.trainer.service_at(index).cloned())
}

fn push_optional_number(state: &mut LuaState, number: Option<f64>) {
    state.push(number.map_or(Val::Nil, Val::Num));
}
