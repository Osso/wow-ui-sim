use super::*;

fn config_ids_for_spec_id(spec_id: u32) -> Vec<i32> {
    talent_state::seeded_class_talent_configs(spec_id)
        .iter()
        .map(|config| config.id)
        .collect()
}

#[cfg(not(feature = "retail-12-0-5"))]
fn current_config_ids(state: &LuaState) -> Vec<i32> {
    current_spec_id(state)
        .map(config_ids_for_spec_id)
        .unwrap_or_default()
}

fn register_c_class_talents_hero_fns(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetHeroTalentSpecsForClassSpec",
        c_class_talents_get_hero_talent_specs_for_class_spec,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetActiveHeroTalentSpec",
        c_class_talents_get_active_hero_talent_spec,
    )?;
    Ok(())
}

const CLASS_TALENTS_CONFIG_METHODS: &[(&str, rilua::RustFn)] = &[
    (
        "GetConfigIDsBySpecID",
        c_class_talents_get_config_ids_by_spec_id,
    ),
    ("GetActiveConfigID", c_class_talents_get_active_config_id),
    (
        "GetLastSelectedSavedConfigID",
        c_class_talents_get_last_selected_saved_config_id,
    ),
    (
        "GetTraitTreeForSpec",
        c_class_talents_get_trait_tree_for_spec,
    ),
    (
        "UpdateLastSelectedSavedConfigID",
        c_class_talents_update_last_selected_saved_config_id,
    ),
    ("CanEditTalents", c_class_talents_can_edit_talents),
    ("CanChangeTalents", c_class_talents_can_change_talents),
    ("GetHasStarterBuild", c_class_talents_get_has_starter_build),
    (
        "IsStarterBuildActive",
        c_class_talents_is_starter_build_active,
    ),
    (
        "GetStarterBuildActive",
        c_class_talents_is_starter_build_active,
    ),
    (
        "SetStarterBuildActive",
        c_class_talents_set_starter_build_active,
    ),
    (
        "GetNextStarterBuildPurchase",
        c_class_talents_get_next_starter_build_purchase,
    ),
    (
        "HasUnspentTalentPoints",
        c_class_talents_has_unspent_talent_points,
    ),
    (
        "HasUnspentHeroTalentPoints",
        c_class_talents_has_unspent_hero_talent_points,
    ),
    (
        "InitializeViewLoadout",
        c_class_talents_initialize_view_loadout,
    ),
    ("LoadConfig", c_class_talents_load_config),
];

fn register_c_class_talents_config_fns(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    for &(name, func) in CLASS_TALENTS_CONFIG_METHODS {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

fn register_c_class_talents_query_fns(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    register_c_class_talents_hero_fns(state, table_ref)?;
    register_c_class_talents_config_fns(state, table_ref)?;
    Ok(())
}

#[cfg(feature = "retail-12-0-5")]
fn register_c_class_talents_action_fns(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    crate::c_api::class_talent_commands::register(state, table_ref)
}

#[cfg(not(feature = "retail-12-0-5"))]
fn register_c_class_talents_action_fns(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "SwitchToLoadoutByName",
        c_class_talents_switch_to_loadout_by_name,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SwitchToLoadoutByIndex",
        c_class_talents_switch_to_loadout_by_index,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SwitchToSpecializationByName",
        c_class_talents_switch_to_specialization_by_name,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SwitchToSpecializationByIndex",
        c_class_talents_switch_to_specialization_by_index,
    )?;
    Ok(())
}

pub(super) fn register_c_class_talents(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_ClassTalents")?;
    register_c_class_talents_query_fns(state, table_ref)?;
    register_c_class_talents_action_fns(state, table_ref)?;
    Ok(())
}
fn hero_specs_for_spec(spec_id: u32) -> &'static [u32] {
    match spec_id {
        65 => &[49, 50],
        66 => &[48, 49],
        70 => &[48, 50],
        _ => &[],
    }
}

fn c_class_talents_get_hero_talent_specs_for_class_spec(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = match (stack_val(state, 1), stack_val(state, 2)) {
        (_, Val::Num(value)) => value as u32,
        (Val::Num(value), Val::Nil) => config_spec_id(value as i32)
            .or_else(|| current_spec_id(state))
            .unwrap_or(0),
        _ => current_spec_id(state).unwrap_or(0),
    };
    let hero_specs = push_u32_array(state, hero_specs_for_spec(spec_id).iter().copied());
    state.push(hero_specs);
    state.push(Val::Num(71.0));
    Ok(2)
}

fn c_class_talents_get_active_hero_talent_spec(state: &mut LuaState) -> LuaResult<u32> {
    match borrow_state(state)
        .ok()
        .and_then(|sim| hero_talents::get_active_hero_subtree(&sim))
    {
        Some(subtree_id) => state.push(Val::Num(subtree_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_class_talents_initialize_view_loadout(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = u32::from_stack(state, 1)?;
    let _level = u32::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .talents
        .initialize_view_loadout(spec_id);
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_class_talents_get_config_ids_by_spec_id(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = u32::from_stack(state, 1)?;
    let config_ids = push_i32_array(state, config_ids_for_spec_id(spec_id));
    state.push(config_ids);
    Ok(1)
}

fn c_class_talents_get_active_config_id(state: &mut LuaState) -> LuaResult<u32> {
    let active_config_id = borrow_state(state)?.talents.active_config_id as f64;
    state.push(Val::Num(active_config_id));
    Ok(1)
}

fn c_class_talents_get_last_selected_saved_config_id(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = u32::from_stack(state, 1)?;
    let config_id = borrow_state(state)?
        .talents
        .last_selected_config_id_by_spec_id
        .get(&spec_id)
        .copied()
        .or_else(|| talent_state::default_class_talent_config_id(spec_id))
        .unwrap_or(0);
    state.push(Val::Num(config_id as f64));
    Ok(1)
}

fn c_class_talents_update_last_selected_saved_config_id(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = u32::from_stack(state, 1)?;
    let config_id = match stack_val(state, 2) {
        Val::Nil => None,
        Val::Num(value) => Some(value as i32),
        _ => None,
    };
    let mut sim = borrow_state_mut(state)?;
    match config_id {
        Some(config_id) => {
            sim.talents
                .last_selected_config_id_by_spec_id
                .insert(spec_id, config_id);
        }
        None => {
            sim.talents
                .last_selected_config_id_by_spec_id
                .remove(&spec_id);
        }
    }
    Ok(0)
}

#[cfg(not(feature = "retail-12-0-5"))]
fn c_class_talents_switch_to_loadout_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let config_id = current_config_ids(state)
        .into_iter()
        .find(|config_id| config_name(*config_id) == name)
        .unwrap_or_else(|| {
            borrow_state(state)
                .map(|sim| sim.talents.active_config_id)
                .unwrap_or(0)
        });
    if let Some(spec_id) = current_spec_id(state) {
        borrow_state_mut(state)?
            .talents
            .switch_to_loadout(spec_id, config_id);
    }
    Ok(0)
}

#[cfg(not(feature = "retail-12-0-5"))]
fn c_class_talents_switch_to_loadout_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?.max(1) as usize - 1;
    let configs = current_config_ids(state);
    if let Some(config_id) = configs.get(index).copied()
        && let Some(spec_id) = current_spec_id(state)
    {
        borrow_state_mut(state)?
            .talents
            .switch_to_loadout(spec_id, config_id);
    }
    Ok(0)
}

#[cfg(not(feature = "retail-12-0-5"))]
fn c_class_talents_switch_to_specialization_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let spec_name = String::from_stack(state, 1)?;
    let class_id = borrow_state(state)?.player.class_index as u32;
    let Some((index, spec)) = specializations::specs_for_class(class_id)
        .enumerate()
        .find(|(_, spec)| spec.name == spec_name)
    else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    sim.player.active_spec_index = index as i32 + 1;
    sim.talents.switch_to_spec(spec.id);
    Ok(0)
}

#[cfg(not(feature = "retail-12-0-5"))]
fn c_class_talents_switch_to_specialization_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let spec_index = i32::from_stack(state, 1)?.max(1) as usize - 1;
    let class_id = borrow_state(state)?.player.class_index as u32;
    let Some(spec) = specializations::specs_for_class(class_id).nth(spec_index) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    sim.player.active_spec_index = spec_index as i32 + 1;
    sim.talents.switch_to_spec(spec.id);
    Ok(0)
}

fn c_class_talents_get_trait_tree_for_spec(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = u32::from_stack(state, 1)?;
    match c_class_talents_trait_tree_for_spec(spec_id) {
        Some(tree_id) => state.push(Val::Num(tree_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_class_talents_load_config(state: &mut LuaState) -> LuaResult<u32> {
    const RESULT_ERROR: f64 = 0.0;
    const RESULT_NO_CHANGES: f64 = 1.0;
    const RESULT_READY: f64 = 3.0;

    let config_id = i32::from_stack(state, 1)?;
    let Some(spec_id) = config_spec_id(config_id) else {
        state.push(Val::Num(RESULT_ERROR));
        state.push(Val::Nil);
        state.push(Val::Nil);
        return Ok(3);
    };

    let active_config_id = borrow_state(state)?.talents.active_config_id;
    let load_result = if config_id == active_config_id {
        RESULT_NO_CHANGES
    } else {
        RESULT_READY
    };

    borrow_state_mut(state)?
        .talents
        .switch_to_loadout(spec_id, config_id);

    fire_named_event_with_arg(
        state,
        "ACTIVE_COMBAT_CONFIG_CHANGED",
        Val::Num(config_id as f64),
    );

    state.push(Val::Num(load_result));
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(3)
}

fn c_class_talents_can_change_talents(state: &mut LuaState) -> LuaResult<u32> {
    let can_change = borrow_state(state)?.talents.can_change_talents;
    state.push(Val::Bool(can_change));
    state.push(Val::Bool(can_change));
    if can_change {
        state.push(Val::Nil);
    } else {
        let reason = create_string(state, "You can't do that right now.");
        state.push(reason);
    }
    Ok(3)
}

fn c_class_talents_can_edit_talents(state: &mut LuaState) -> LuaResult<u32> {
    let can_edit = borrow_state(state)?.talents.can_change_talents;
    state.push(Val::Bool(can_edit));
    if can_edit {
        state.push(Val::Nil);
    } else {
        let reason = create_string(state, "You can't do that right now.");
        state.push(reason);
    }
    Ok(2)
}

fn c_class_talents_get_has_starter_build(state: &mut LuaState) -> LuaResult<u32> {
    let has_starter = borrow_state(state)?.talents.has_starter_build;
    state.push(Val::Bool(has_starter));
    Ok(1)
}

fn c_class_talents_is_starter_build_active(state: &mut LuaState) -> LuaResult<u32> {
    let is_active = borrow_state(state)?.talents.is_starter_build_active;
    state.push(Val::Bool(is_active));
    Ok(1)
}

fn c_class_talents_set_starter_build_active(state: &mut LuaState) -> LuaResult<u32> {
    let is_active = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.talents.is_starter_build_active = is_active;
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_class_talents_get_next_starter_build_purchase(state: &mut LuaState) -> LuaResult<u32> {
    let next_purchase = borrow_state(state)
        .ok()
        .filter(|sim| sim.talents.has_starter_build && sim.talents.is_starter_build_active)
        .and_then(|sim| starter_build_purchase_for_state(&sim));

    match next_purchase {
        Some((node_id, entry_id)) => {
            state.push(Val::Num(node_id as f64));
            state.push(Val::Num(entry_id as f64));
        }
        None => {
            state.push(Val::Nil);
            state.push(Val::Nil);
        }
    }
    Ok(2)
}

fn c_class_talents_has_unspent_talent_points(state: &mut LuaState) -> LuaResult<u32> {
    let has_unspent = {
        let sim = borrow_state(state)?;
        has_unspent_talent_points(&sim)
    };
    state.push(Val::Bool(has_unspent));
    Ok(1)
}

fn c_class_talents_has_unspent_hero_talent_points(state: &mut LuaState) -> LuaResult<u32> {
    let (has_unspent, num_points) = borrow_state(state)
        .ok()
        .and_then(|sim| {
            let currency_id = sim
                .talents
                .active_hero_subtree()
                .and_then(c_traits::subtree_trait_currency_id)?;
            let max_points = HERO_TALENT_POINT_BUDGET;
            let spent = sim.talents.spent_for_currency(currency_id);
            Some((spent < max_points, max_points.saturating_sub(spent)))
        })
        .unwrap_or((false, 0));

    state.push(Val::Bool(has_unspent));
    state.push(Val::Num(num_points as f64));
    Ok(2)
}

fn c_class_talents_trait_tree_for_spec(spec_id: u32) -> Option<u32> {
    class_talent_tree_for_spec(spec_id)
}
