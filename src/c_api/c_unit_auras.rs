//! Aura instance enumeration backed by the existing public/private aura stores.

use crate::lua_api::game_data::AuraInfo;
use crate::lua_api::globals::auras::collect_filtered_unit_auras;
use crate::lua_api::methods::{borrow_state, create_table, table_get, table_set, table_set_num};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::cmp::Ordering;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let public = super::ensure_namespace(state, "C_UnitAuras")?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        table_set_rust_fn_static(state, public, "ClearBlockedAuras", clear_blocked_auras)?;
    }
    // Forever 1.60.1.70707 declares this query and its selected aura source
    // calls it. Share the existing aura/GUID model without enabling a Retail epoch.
    if cfg!(feature = "retail-12-1-5") || cfg!(feature = "client-wowforever") {
        table_set_rust_fn_static(state, public, "GetAuraCasterGUID", get_aura_caster_guid)?;
    } else if cfg!(feature = "retail-12-1-0") {
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
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(
        state,
        public,
        "CancelAuraByInstanceID",
        cancel_aura_by_instance_id,
    )?;
    #[cfg(feature = "aura-containers")]
    register_private_enumeration(state)?;
    Ok(())
}

/// Clear the selected unit's entries in the live AddBlockedAura/enumeration model.
fn clear_blocked_auras(state: &mut LuaState) -> LuaResult<u32> {
    // BEST GUESS, non-native: reject nil/omitted unit before mutation.
    // Pending Retail native evidence; the nil-policy selector is independent.
    let unit = String::from_stack(state, 1)?;
    let namespace = super::global_val(state, "C_UnitAuras");
    let blocked = table_get(state, namespace, "_blockedAuras");
    let Val::Table(blocked_table) = blocked else {
        return Ok(0);
    };
    let prefix = format!("{unit}:");
    let entries = state
        .gc
        .tables
        .get(blocked_table)
        .ok_or_else(|| runtime_error("C_UnitAuras blocked-aura table is unavailable"))?
        .hash_entries();
    let keys: Vec<String> = entries
        .into_iter()
        .filter_map(|(key, _)| crate::lua_api::methods::val_to_string(state, key))
        .filter(|key| key.starts_with(&prefix))
        .collect();
    for key in keys {
        table_set(state, blocked, &key, Val::Nil);
    }
    Ok(0)
}

#[cfg(feature = "aura-containers")]
fn register_private_enumeration(state: &mut LuaState) -> LuaResult<()> {
    let private = super::ensure_namespace(state, "C_UnitAurasPrivate")?;
    table_set_rust_fn_static(
        state,
        private,
        "GetAllPrivateAuraInstanceIDs",
        get_private_aura_instance_ids,
    )?;
    table_set_rust_fn_static(
        state,
        private,
        "IsPrivateAuraFilteredOutByInstanceID",
        is_private_aura_filtered_out_by_instance_id,
    )
}

/// Private-aura counterpart of `C_UnitAuras.IsAuraFilteredOutByInstanceID`,
/// evaluated against the private aura store. INFERRED: absent instances are
/// filtered, matching the public query.
#[cfg(feature = "aura-containers")]
fn is_private_aura_filtered_out_by_instance_id(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let instance_id = i32::from_stack(state, 2)?;
    let filter = String::from_stack(state, 3)?;
    let context = crate::c_api::aura_filter::AuraFilterContext::for_unit(state, &unit);
    let is_filtered = private_aura_info(state, &unit, instance_id).is_none_or(|aura| {
        !crate::lua_api::globals::auras::aura_matches_filter_string(&aura, &filter, &context)
    });
    state.push(Val::Bool(is_filtered));
    Ok(1)
}

/// Read one private AuraData record as the public aura model.
#[cfg(feature = "aura-containers")]
fn private_aura_info(state: &mut LuaState, unit: &str, instance_id: i32) -> Option<AuraInfo> {
    let namespace = super::global_val(state, "C_UnitAurasPrivate");
    let private_state = table_get(state, namespace, "_state");
    let by_unit = table_get(state, private_state, "auraDataByUnit");
    let Val::Table(records) = table_get(state, by_unit, unit) else {
        return None;
    };
    let record = state
        .gc
        .tables
        .get(records)?
        .get_int(i64::from(instance_id));
    if !matches!(record, Val::Table(_)) {
        return None;
    }
    Some(AuraInfo {
        name: record_string(state, record, "name").unwrap_or_default(),
        spell_id: record_number(state, record, "spellId") as i32,
        icon: record_number(state, record, "icon") as i32,
        duration: record_number(state, record, "duration"),
        expiration_time: record_number(state, record, "expirationTime"),
        applications: record_number(state, record, "applications") as i32,
        source_unit: record_string(state, record, "sourceUnit").unwrap_or_default(),
        is_helpful: record_bool(state, record, "isHelpful"),
        is_raid: record_bool(state, record, "isRaid"),
        is_nameplate_only: record_bool(state, record, "isNameplateOnly"),
        is_stealable: record_bool(state, record, "isStealable"),
        can_apply_aura: record_bool(state, record, "canApplyAura"),
        is_from_player_or_player_pet: record_bool(state, record, "isFromPlayerOrPlayerPet"),
        dispel_type: record_string(state, record, "dispelName"),
        aura_instance_id: instance_id,
    })
}

#[cfg(feature = "aura-containers")]
fn record_number(state: &mut LuaState, record: Val, key: &str) -> f64 {
    match table_get(state, record, key) {
        Val::Num(number) => number,
        _ => 0.0,
    }
}

#[cfg(feature = "aura-containers")]
fn record_bool(state: &mut LuaState, record: Val, key: &str) -> bool {
    matches!(table_get(state, record, key), Val::Bool(true))
}

#[cfg(feature = "aura-containers")]
fn record_string(state: &mut LuaState, record: Val, key: &str) -> Option<String> {
    let value = table_get(state, record, key);
    crate::lua_api::methods::val_to_string(state, value)
}

pub(crate) fn register_sound_trigger_enum(state: &mut LuaState, enums: Val) {
    if !cfg!(any(
        feature = "retail-12-1-0",
        feature = "client-wowforever"
    )) {
        return;
    }
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

fn get_aura_caster_guid(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let (instance_id, secret_selector) = read_caster_instance(state)?;
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
    // Approved selector policy: identity follows a numeric-secret selector.
    // The separate unit-aura output-restriction layer remains held.
    let result = match guid {
        Some(guid) if secret_selector => {
            rilua::table_security::wrap_host_secret_string(state, &guid)
        }
        Some(guid) => crate::lua_api::methods::create_string(state, &guid),
        None => Val::Nil,
    };
    state.push(result);
    Ok(1)
}

fn read_caster_instance(state: &LuaState) -> LuaResult<(i32, bool)> {
    let value = crate::lua_bridge::stack_val(state, 2);
    let secret_selector = cfg!(any(feature = "profile-retail", feature = "client-ptr"))
        && rilua::table_security::secret_payload_kind(state, value)
            == Some(rilua::table_security::SecretPayloadKind::Number);
    let instance_id = if secret_selector {
        rilua::table_security::inspect_host_secret_number(state, value, |number| Ok(number as i32))?
    } else {
        f64::from_stack(state, 2)? as i32
    };
    Ok((instance_id, secret_selector))
}

/// Removes one of the player's own helpful auras and reports it through an
/// incremental `UNIT_AURA` payload. INFERRED: like `CancelUnitBuff`, debuffs
/// and other units' auras cannot be cancelled.
#[cfg(feature = "retail-12-1-0")]
fn cancel_aura_by_instance_id(state: &mut LuaState) -> LuaResult<u32> {
    super::unit_aura_access::require_unit_aura_access(state, "C_UnitAuras.CancelAuraByInstanceID")?;
    let unit = String::from_stack(state, 1)?;
    let instance_id = f64::from_stack(state, 2)? as i32;
    if unit != "player" {
        return Ok(0);
    }
    let removed = {
        let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
        let before = sim.player.buffs.len();
        sim.player
            .buffs
            .retain(|aura| !(aura.is_helpful && aura.aura_instance_id == instance_id));
        sim.player.buffs.len() != before
    };
    if removed {
        let update_info = super::unit_aura_access::removal_payload(state, &[instance_id])?;
        let unit = crate::lua_api::methods::create_string(state, "player");
        crate::lua_api::script_helpers::fire_named_event_state(
            state,
            "UNIT_AURA",
            &[unit, update_info],
        );
    }
    Ok(0)
}

fn get_unit_aura_instance_ids(state: &mut LuaState) -> LuaResult<u32> {
    super::unit_aura_access::require_unit_aura_access(state, "C_UnitAuras.GetUnitAuraInstanceIDs")?;
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
