//! Aura instance enumeration backed by the existing public/private aura stores.

use crate::lua_api::game_data::AuraInfo;
use crate::lua_api::globals::auras::collect_filtered_unit_auras;
#[cfg(feature = "aura-containers")]
use crate::lua_api::methods::table_get;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_api::methods::table_set;
use crate::lua_api::methods::{borrow_state, create_table, table_set_num};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::cmp::Ordering;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let public = super::ensure_namespace(state, "C_UnitAuras")?;
    #[cfg(feature = "retail-12-1-5")]
    table_set_rust_fn_static(state, public, "GetAuraCasterGUID", get_aura_caster_guid)?;
    #[cfg(all(feature = "retail-12-1-0", not(feature = "retail-12-1-5")))]
    {
        let removed = create_table(state);
        table_set(state, removed, "GetAuraCasterGUID", Val::Bool(true));
        table_set(state, Val::Table(public), "__wow_removed_keys", removed);
    }
    table_set_rust_fn_static(
        state,
        public,
        "GetUnitAuraInstanceIDs",
        get_unit_aura_instance_ids,
    )?;
    #[cfg(feature = "aura-containers")]
    register_private_enumeration(state)?;
    Ok(())
}

#[cfg(feature = "aura-containers")]
fn register_private_enumeration(state: &mut LuaState) -> LuaResult<()> {
    let private = super::ensure_namespace(state, "C_UnitAurasPrivate")?;
    table_set_rust_fn_static(
        state,
        private,
        "GetAllPrivateAuraInstanceIDs",
        get_private_aura_instance_ids,
    )
}

#[cfg(feature = "retail-12-1-0")]
pub(crate) fn register_sound_trigger_enum(state: &mut LuaState, enums: Val) {
    let values = create_table(state);
    for (name, value) in [("Added", 0), ("ApplicationsIncreased", 1), ("Removed", 2)] {
        table_set(state, values, name, Val::Num(value as f64));
    }
    table_set(state, enums, "UnitAuraSoundTrigger", values);
    let metadata = create_table(state);
    for (name, value) in [("MinValue", 0), ("MaxValue", 2), ("NumValues", 3)] {
        table_set(state, metadata, name, Val::Num(value as f64));
    }
    table_set(state, enums, "UnitAuraSoundTriggerMeta", metadata);
}

#[cfg(feature = "retail-12-1-5")]
fn get_aura_caster_guid(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let instance_id = f64::from_stack(state, 2)? as i32;
    let exists =
        crate::lua_api::globals::group_queries::unit_exists_in_state(&*borrow_state(state)?, &unit);
    let aura = if exists {
        crate::lua_api::globals::auras::find_aura_by_instance_id(state, &unit, instance_id)
    } else {
        None
    };
    let sim = borrow_state(state)?;
    let guid = aura.and_then(|aura| {
        crate::lua_api::globals::unit_misc::existing_guid_for_unit(&sim, &aura.source_unit)
    });
    drop(sim);
    let result = guid
        .map(|guid| crate::lua_api::methods::create_string(state, &guid))
        .unwrap_or(Val::Nil);
    state.push(result);
    Ok(1)
}

fn get_unit_aura_instance_ids(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let filter = String::from_stack(state, 2)?;
    let limit = Option::<i32>::from_stack(state, 3)?;
    let sort_rule = Option::<i32>::from_stack(state, 4)?.unwrap_or(0);
    let direction = Option::<i32>::from_stack(state, 5)?.unwrap_or(0);
    validate_query_options(limit, sort_rule, direction)?;
    let mut auras = collect_filtered_unit_auras(state, &unit, &filter);
    sort_auras(state, &mut auras, sort_rule)?;
    if direction == 1 {
        auras.reverse();
    }
    if let Some(limit) = limit {
        auras.truncate(limit as usize);
    }
    push_instance_ids(state, auras.into_iter().map(|aura| aura.aura_instance_id));
    Ok(1)
}

fn validate_query_options(limit: Option<i32>, rule: i32, direction: i32) -> LuaResult<()> {
    if limit.is_some_and(|limit| limit < 0) {
        return Err(runtime_error(
            "GetUnitAuraInstanceIDs: maxCount must be non-negative",
        ));
    }
    if !(0..=6).contains(&rule) || !(0..=1).contains(&direction) {
        return Err(runtime_error(
            "GetUnitAuraInstanceIDs: invalid aura sort rule or direction",
        ));
    }
    Ok(())
}

fn sort_auras(state: &LuaState, auras: &mut [AuraInfo], rule: i32) -> LuaResult<()> {
    if rule == 0 {
        return Ok(());
    }
    let sim = borrow_state(state)?;
    auras.sort_by(|left, right| compare_auras(&sim, left, right, rule));
    Ok(())
}

// UnitAuraSortRule values from UnitAuraSharedDocumentation.lua.
fn compare_auras(sim: &SimState, left: &AuraInfo, right: &AuraInfo, rule: i32) -> Ordering {
    match rule {
        1 => compare_prioritized_auras(left, right, Ordering::Equal),
        2 => compare_defensive_auras(sim, left, right),
        3 => {
            let expiration = expiration_with_permanent_last(left)
                .total_cmp(&expiration_with_permanent_last(right));
            compare_prioritized_auras(left, right, expiration)
        }
        4 => left.expiration_time.total_cmp(&right.expiration_time),
        5 => compare_prioritized_auras(left, right, left.name.cmp(&right.name)),
        6 => left.name.cmp(&right.name),
        _ => unreachable!("sort options validated before comparison"),
    }
}

fn compare_prioritized_auras(left: &AuraInfo, right: &AuraInfo, secondary: Ordering) -> Ordering {
    player_priority(left)
        .cmp(&player_priority(right))
        .then(secondary)
        .then_with(|| left.aura_instance_id.cmp(&right.aura_instance_id))
}

fn compare_defensive_auras(sim: &SimState, left: &AuraInfo, right: &AuraInfo) -> Ordering {
    is_other_player_source(sim, right)
        .cmp(&is_other_player_source(sim, left))
        .then_with(|| right.expiration_time.total_cmp(&left.expiration_time))
        .then_with(|| left.aura_instance_id.cmp(&right.aura_instance_id))
}

fn player_priority(aura: &AuraInfo) -> (bool, bool) {
    (!aura.is_from_player_or_player_pet, !aura.can_apply_aura)
}

fn expiration_with_permanent_last(aura: &AuraInfo) -> f64 {
    if aura.expiration_time == 0.0 {
        f64::INFINITY
    } else {
        aura.expiration_time
    }
}

fn is_other_player_source(sim: &SimState, aura: &AuraInfo) -> bool {
    if aura.is_from_player_or_player_pet {
        return false;
    }
    if let Some(index) = crate::lua_api::globals::unit_api::parse_party_index(&aura.source_unit) {
        return sim.party_members.get(index).is_some();
    }
    aura.source_unit == "target"
        && sim
            .current_target
            .as_ref()
            .is_some_and(|target| target.is_player)
}

#[cfg(feature = "aura-containers")]
fn get_private_aura_instance_ids(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let namespace = super::global_val(state, "C_UnitAurasPrivate");
    let private_state = table_get(state, namespace, "_state");
    let by_unit = table_get(state, private_state, "privateAurasByUnit");
    let list = table_get(state, by_unit, &unit);
    let entries = match list {
        Val::Table(list) => state
            .gc
            .tables
            .get(list)
            .map(|list| {
                (1..=list.len(&state.gc.string_arena))
                    .map(|index| list.get_int(index as i64))
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let ids = entries
        .into_iter()
        .map(|aura| private_instance_id(state, aura))
        .collect::<LuaResult<Vec<_>>>()?;
    push_instance_ids(state, ids);
    Ok(1)
}

#[cfg(feature = "aura-containers")]
fn private_instance_id(state: &mut LuaState, aura: Val) -> LuaResult<i32> {
    match table_get(state, aura, "auraInstanceID") {
        Val::Num(id) => Ok(id as i32),
        _ => Err(runtime_error(
            "private aura state is missing auraInstanceID",
        )),
    }
}

fn push_instance_ids(state: &mut LuaState, ids: impl IntoIterator<Item = i32>) {
    let table = create_table(state);
    if let Val::Table(table_ref) = table {
        for (index, id) in ids.into_iter().enumerate() {
            table_set_num(state, table_ref, (index + 1) as f64, Val::Num(id as f64));
        }
    }
    state.push(table);
}
