//! `C_UnitAuras` probe surface backed by `SimState.player.buffs`.
//!
//! Every method walks the live aura list rather than a hard-coded
//! fixture, so admin-added buffs pushed via `admin_buffs::add_buff`
//! are findable by all three keys the Blizzard UI uses:
//!
//! - **index** (`GetAuraDataByIndex`, `GetBuffDataByIndex`,
//!   `GetDebuffDataByIndex`)
//! - **aura instance id** (`GetAuraDataByAuraInstanceID`)
//! - **spell name** (`GetAuraDataBySpellName`)
//!
//! Slot ids round-trip through `aura_instance_id` so the
//! `GetAuraSlots` → `GetAuraDataBySlot` handshake stays consistent
//! with the per-aura data returned by index / name queries.
//!
//! Registered after `stubs::register_all` so the `stub_nil`
//! registrations for the same methods are overwritten.
//!
//! NOTE: `GetAuraSlots` MUST NOT return an empty table as its first
//! value — `AuraUtil.ForEachAura` drives a `repeat ... until
//! token == nil` loop and treats any truthy first return as "more
//! slots available", spinning forever (see
//! docs/wiki/investigations/partyframe-tree.md).

use crate::lua_api::game_data::AuraInfo;
use crate::lua_api::globals::font_strings_collection::colors::{
    debuff_type_color, make_rilua_color_table,
};
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, create_table, create_table_with_capacity,
    table_get, table_set, table_set_num,
};
use crate::lua_bridge::FromStack;
use crate::lua_bridge::stack_val;
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, RustFn, Val};

const AURA_DATA_HASH_FIELDS: usize = 23;
const AURA_FILTER_COUNT: usize = 4;

pub fn register_all(state: &mut LuaState) {
    let ns = ensure_c_unit_auras(state);
    ensure_runtime_aura_state(state, ns);
    install_c_unit_auras_methods(state, ns);
    install_legacy_aura_globals(state);
    install_aura_util_methods(state);
}

fn ensure_c_unit_auras(state: &mut LuaState) -> Val {
    ensure_global_table(state, "C_UnitAuras")
}

fn ensure_global_table(state: &mut LuaState, name: &str) -> Val {
    let global = Val::Table(state.global);
    match table_get(state, global, name) {
        ns @ Val::Table(_) => ns,
        _ => {
            let ns = create_table(state);
            table_set(state, global, name, ns);
            ns
        }
    }
}

fn install(state: &mut LuaState, ns: Val, name: &'static str, func: RustFn) {
    let closure = Closure::Rust(RustClosure::new(func, name));
    let closure_ref = state.gc.alloc_closure(closure);
    table_set(state, ns, name, Val::Function(closure_ref));
}

fn install_methods(state: &mut LuaState, ns: Val, methods: &[(&'static str, RustFn)]) {
    for (name, func) in methods {
        install(state, ns, name, *func);
    }
}

fn install_c_unit_auras_methods(state: &mut LuaState, ns: Val) {
    install_methods(
        state,
        ns,
        &[
            ("GetAuraSlots", get_aura_slots),
            ("GetUnitAuraInstanceIDs", get_unit_aura_instance_ids),
            ("GetAuraDataBySlot", get_aura_data_by_slot),
            ("GetAuraDataByIndex", get_aura_data_by_index),
            (
                "GetAuraDataByAuraInstanceID",
                get_aura_data_by_aura_instance_id,
            ),
            ("GetAuraDataBySpellName", get_aura_data_by_spell_name),
            ("GetUnitAuras", get_unit_auras),
            ("GetBuffDataByIndex", get_buff_data_by_index),
            ("GetDebuffDataByIndex", get_debuff_data_by_index),
            ("GetAuraDispelTypeColor", get_aura_dispel_type_color),
            ("GetPlayerAuraBySpellID", get_player_aura_by_spell_id),
            ("AddBlockedAura", add_blocked_aura),
            ("SwitchAuraDataProvider", switch_aura_data_provider),
            ("ResetAuraDataProvider", reset_aura_data_provider),
        ],
    );
    #[cfg(feature = "retail-12-1-0")]
    install(
        state,
        ns,
        "IsAuraFilteredOutByInstanceID",
        is_aura_filtered_out_by_instance_id,
    );
}

fn install_legacy_aura_globals(state: &mut LuaState) {
    let legacy_globals: &[(&'static str, RustFn)] = &[
        ("GetPlayerAuraBySpellID", get_player_aura_by_spell_id),
        ("UnitBuff", unit_buff),
        ("UnitDebuff", unit_debuff),
        ("UnitAura", unit_aura),
    ];
    install_methods(state, Val::Table(state.global), legacy_globals);
}

fn install_aura_util_methods(state: &mut LuaState) {
    let aura_util = ensure_global_table(state, "AuraUtil");
    ensure_aura_filters(state, aura_util);
    install_methods(
        state,
        aura_util,
        &[
            ("CreateFilterString", aura_util_create_filter_string),
            ("IsValidFilterString", aura_util_is_valid_filter_string),
            ("UnpackAuraData", aura_util_unpack_aura_data),
            ("ForEachAura", aura_util_for_each_aura),
            ("FindAura", aura_util_find_aura),
            ("FindAuraByName", aura_util_find_aura_by_name),
            (
                "GetAuraDataByAuraInstanceID",
                aura_util_get_aura_data_by_aura_instance_id,
            ),
            ("GetUnitAuras", get_unit_auras),
        ],
    );
}

fn aura_util_is_valid_filter_string(state: &mut LuaState) -> LuaResult<u32> {
    let filter = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let valid = !filter.is_empty()
        && filter.split('|').all(|token| {
            matches!(
                token.to_ascii_uppercase().as_str(),
                "HELPFUL" | "HARMFUL" | "RAID" | "INCLUDE_NAME_PLATE_ONLY" | "PLAYER"
            )
        });
    state.push(Val::Bool(valid));
    Ok(1)
}

fn ensure_aura_filters(state: &mut LuaState, aura_util: Val) {
    if matches!(table_get(state, aura_util, "AuraFilters"), Val::Table(_)) {
        return;
    }

    let filters = create_table_with_capacity(state, AURA_FILTER_COUNT);
    let helpful = create_string(state, "HELPFUL");
    let harmful = create_string(state, "HARMFUL");
    let raid = create_string(state, "RAID");
    let include_nameplate_only = create_string(state, "INCLUDE_NAME_PLATE_ONLY");
    table_set(state, filters, "Helpful", helpful);
    table_set(state, filters, "Harmful", harmful);
    table_set(state, filters, "Raid", raid);
    table_set(
        state,
        filters,
        "IncludeNameplateOnly",
        include_nameplate_only,
    );
    table_set(state, aura_util, "AuraFilters", filters);
}

fn ensure_runtime_aura_state(state: &mut LuaState, ns: Val) {
    if !matches!(table_get(state, ns, "_blockedAuras"), Val::Table(_)) {
        let blocked = create_table(state);
        table_set(state, ns, "_blockedAuras", blocked);
    }
    if matches!(table_get(state, ns, "_providerSwitched"), Val::Nil) {
        table_set(state, ns, "_providerSwitched", Val::Bool(false));
    }
}

// ── Filter handling ──────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum AuraFilter {
    Helpful,
    Harmful,
    ExternalDefensive,
    /// Torghast anima powers. The sim has no MAW auras, so this variant
    /// matches nothing — preventing Blizzard_MawBuffs from showing outside
    /// Torghast when the player has any ordinary buff.
    Maw,
}

fn filter_from_str(filter: &str) -> AuraFilter {
    let upper = filter.to_uppercase();
    if upper.contains("MAW") {
        AuraFilter::Maw
    } else if upper.contains("EXTERNAL_DEFENSIVE") {
        AuraFilter::ExternalDefensive
    } else if upper.contains("HARMFUL") {
        AuraFilter::Harmful
    } else {
        AuraFilter::Helpful
    }
}

fn aura_matches_filter(aura: &AuraInfo, filter: AuraFilter) -> bool {
    match filter {
        AuraFilter::Helpful => aura.is_helpful,
        AuraFilter::Harmful => !aura.is_helpful,
        AuraFilter::ExternalDefensive => false,
        AuraFilter::Maw => false,
    }
}

#[cfg(feature = "retail-12-1-0")]
fn aura_matches_filter_string(aura: &AuraInfo, filter: &str) -> bool {
    let matches_polarity = aura_matches_filter(aura, filter_from_str(filter));
    let requires_player_source = filter
        .split('|')
        .any(|token| token.eq_ignore_ascii_case("PLAYER"));
    let matches_source = !requires_player_source || aura.is_from_player_or_player_pet;
    matches_polarity && matches_source
}

fn target_fixture_auras() -> Vec<AuraInfo> {
    vec![
        AuraInfo {
            name: "Weakened Armor".to_string(),
            spell_id: 113746,
            icon: 136039,
            duration: 15.0,
            expiration_time: 15.0,
            applications: 1,
            source_unit: "target".to_string(),
            is_helpful: false,
            is_stealable: false,
            can_apply_aura: false,
            is_from_player_or_player_pet: false,
            dispel_type: None,
            aura_instance_id: 1,
        },
        AuraInfo {
            name: "Expose Weakness".to_string(),
            spell_id: 53338,
            icon: 132090,
            duration: 12.0,
            expiration_time: 12.0,
            applications: 2,
            source_unit: "target".to_string(),
            is_helpful: false,
            is_stealable: false,
            can_apply_aura: false,
            is_from_player_or_player_pet: false,
            dispel_type: Some("Magic".to_string()),
            aura_instance_id: 2,
        },
    ]
}

fn collect_unit_auras(state: &mut LuaState, unit: &str, filter: AuraFilter) -> Vec<AuraInfo> {
    let Ok(sim) = borrow_state(state) else {
        return Vec::new();
    };
    use crate::lua_api::globals::unit_api::parse_party_index;
    let auras: Vec<&AuraInfo> = if let Some(idx) = parse_party_index(unit) {
        if let Some(member) = sim.party_members.get(idx) {
            match filter {
                AuraFilter::Helpful => member.buffs.iter().collect(),
                AuraFilter::Harmful => member.debuffs.iter().collect(),
                AuraFilter::ExternalDefensive => return Vec::new(),
                AuraFilter::Maw => return Vec::new(),
            }
        } else {
            return Vec::new();
        }
    } else if unit == "player" {
        sim.player
            .buffs
            .iter()
            .filter(|a| aura_matches_filter(a, filter))
            .collect()
    } else if unit == "target" {
        let target_auras = target_fixture_auras();
        return target_auras
            .into_iter()
            .filter(|a| aura_matches_filter(a, filter))
            .collect();
    } else {
        return Vec::new();
    };
    auras.into_iter().cloned().collect()
}

fn blocked_aura_key(unit: &str, aura_instance_id: i32) -> String {
    format!("{unit}:{aura_instance_id}")
}

fn is_blocked_aura(state: &mut LuaState, unit: &str, aura_instance_id: i32) -> bool {
    let ns = ensure_c_unit_auras(state);
    let blocked = table_get(state, ns, "_blockedAuras");
    matches!(
        table_get(state, blocked, &blocked_aura_key(unit, aura_instance_id)),
        Val::Bool(true)
    )
}

fn collect_visible_unit_auras(
    state: &mut LuaState,
    unit: &str,
    filter: AuraFilter,
) -> Vec<AuraInfo> {
    collect_unit_auras(state, unit, filter)
        .into_iter()
        .filter(|a| !is_blocked_aura(state, unit, a.aura_instance_id))
        .collect()
}

fn provider_switched(state: &mut LuaState) -> bool {
    let ns = ensure_c_unit_auras(state);
    matches!(table_get(state, ns, "_providerSwitched"), Val::Bool(true))
}

// ── GetAuraSlots ─────────────────────────────────────────────────────────────
//
// Signature:
//   (continuationToken, slot1, slot2, ...) = GetAuraSlots(unit, filter, batchSize, token)
//
// Slot IDs map 1:1 to `aura_instance_id` so `GetAuraDataBySlot(slot)`
// is equivalent to `GetAuraDataByAuraInstanceID(slot)`.
fn get_aura_slots(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let filter_str: String = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let token = Option::<f64>::from_stack(state, 4)?;

    // Second call after we already produced our slots — terminate.
    if token.is_some() {
        return Ok(0);
    }

    let auras = collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str));

    // continuationToken = nil (done after this batch)
    state.push(Val::Nil);
    for aura in &auras {
        state.push(Val::Num(aura.aura_instance_id as f64));
    }
    Ok(auras.len() as u32 + 1)
}

// ── GetUnitAuraInstanceIDs ───────────────────────────────────────────────────
//
// `(unit, filter, maxCount, sortRule, sortDirection)` -> table of aura
// instance IDs (UnitAuraDocumentation.lua). The 12.1 aura containers
// (`AuraContainerPublicAuraSource:GetAllAuraInstanceIDs`) rebuild their
// groups from it and iterate the result with ipairs, so a unit with no
// auras gets an empty table, never nil. The sort arguments are accepted and
// ignored: the containers re-sort with their own comparator.
fn get_unit_aura_instance_ids(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let filter_str: String = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let max_count = Option::<f64>::from_stack(state, 3)?
        .filter(|count| *count > 0.0)
        .map(|count| count as usize)
        .unwrap_or(usize::MAX);
    let auras = collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str));
    let table = create_table(state);
    if let Val::Table(table_ref) = table {
        for (index, aura) in auras.iter().take(max_count).enumerate() {
            table_set_num(
                state,
                table_ref,
                (index + 1) as f64,
                Val::Num(aura.aura_instance_id as f64),
            );
        }
    }
    state.push(table);
    Ok(1)
}

// ── GetAuraDataBySlot ────────────────────────────────────────────────────────
fn get_aura_data_by_slot(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let slot = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    push_aura_by_instance_id(state, &unit, slot);
    Ok(1)
}

// ── GetAuraDataByIndex ───────────────────────────────────────────────────────
//
// `(unit, index, filter)` — filter decides whether the index addresses a buff
// or a debuff list. 1-based index into the filtered list.
fn get_aura_data_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let index = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    let filter_str = Option::<String>::from_stack(state, 3)?.unwrap_or_default();
    push_aura_at_filtered_index(state, &unit, filter_from_str(&filter_str), index);
    Ok(1)
}

fn get_unit_auras(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let filter_str = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let table = create_table(state);

    if let Val::Table(table_ref) = table {
        for (index, aura) in collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str))
            .into_iter()
            .enumerate()
        {
            let aura_table = build_aura_table(state, &aura, &unit);
            table_set_num(state, table_ref, (index + 1) as f64, aura_table);
        }
    }

    state.push(table);
    Ok(1)
}

fn get_aura_data_by_aura_instance_id(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let aura_id = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    push_aura_by_instance_id(state, &unit, aura_id);
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_aura_filtered_out_by_instance_id(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let aura_instance_id = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    let filter = Option::<String>::from_stack(state, 3)?.unwrap_or_default();
    let is_filtered = find_aura_by_instance_id(state, &unit, aura_instance_id)
        .map(|aura| !aura_matches_filter_string(&aura, &filter))
        .unwrap_or(true);
    state.push(Val::Bool(is_filtered));
    Ok(1)
}

fn get_aura_data_by_spell_name(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let name = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let found = collect_unit_auras(state, &unit, AuraFilter::Helpful)
        .into_iter()
        .chain(collect_unit_auras(state, &unit, AuraFilter::Harmful))
        .find(|a| a.name == name);
    match found {
        Some(aura) => {
            let table = build_aura_table(state, &aura, &unit);
            state.push(table);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_buff_data_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let index = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    push_aura_at_filtered_index(state, &unit, AuraFilter::Helpful, index);
    Ok(1)
}

fn get_debuff_data_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let index = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    push_aura_at_filtered_index(state, &unit, AuraFilter::Harmful, index);
    Ok(1)
}

fn get_aura_dispel_type_color(state: &mut LuaState) -> LuaResult<u32> {
    let unit: String = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let aura_id = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    let color = find_aura_by_instance_id(state, &unit, aura_id)
        .and_then(|aura| aura.dispel_type)
        .and_then(|dispel_type| debuff_type_color(&dispel_type))
        .unwrap_or((1.0, 1.0, 1.0, 0.0));
    let color = make_rilua_color_table(state, color.0, color.1, color.2, color.3)?;
    state.push(color);
    Ok(1)
}

fn get_player_aura_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = Option::<f64>::from_stack(state, 1)?.unwrap_or_default() as i32;
    let found = collect_unit_auras(state, "player", AuraFilter::Helpful)
        .into_iter()
        .find(|a| a.spell_id == spell_id);
    match found {
        Some(aura) => {
            let table = build_aura_table(state, &aura, "player");
            state.push(table);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

// ── Aura lookup helpers ──────────────────────────────────────────────────────

fn push_aura_by_instance_id(state: &mut LuaState, unit: &str, aura_instance_id: i32) {
    let found = find_aura_by_instance_id(state, unit, aura_instance_id);
    match found {
        Some(aura) => {
            let table = build_aura_table(state, &aura, unit);
            state.push(table);
        }
        None => state.push(Val::Nil),
    }
}

fn find_aura_by_instance_id(
    state: &mut LuaState,
    unit: &str,
    aura_instance_id: i32,
) -> Option<AuraInfo> {
    collect_unit_auras(state, unit, AuraFilter::Helpful)
        .into_iter()
        .chain(collect_unit_auras(state, unit, AuraFilter::Harmful))
        .find(|a| a.aura_instance_id == aura_instance_id)
}

fn push_aura_at_filtered_index(state: &mut LuaState, unit: &str, filter: AuraFilter, index: i32) {
    if index < 1 {
        state.push(Val::Nil);
        return;
    }
    let auras = collect_visible_unit_auras(state, unit, filter);
    match auras.get((index - 1) as usize) {
        Some(aura) => {
            let table = build_aura_table(state, aura, unit);
            state.push(table);
        }
        None => state.push(Val::Nil),
    }
}

fn add_blocked_aura(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let aura_instance_id = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    let ns = ensure_c_unit_auras(state);
    ensure_runtime_aura_state(state, ns);
    let blocked = table_get(state, ns, "_blockedAuras");
    table_set(
        state,
        blocked,
        &blocked_aura_key(&unit, aura_instance_id),
        Val::Bool(true),
    );
    Ok(0)
}

fn switch_aura_data_provider(state: &mut LuaState) -> LuaResult<u32> {
    let ns = ensure_c_unit_auras(state);
    ensure_runtime_aura_state(state, ns);
    table_set(state, ns, "_providerSwitched", Val::Bool(true));
    Ok(0)
}

fn reset_aura_data_provider(state: &mut LuaState) -> LuaResult<u32> {
    let ns = ensure_c_unit_auras(state);
    ensure_runtime_aura_state(state, ns);
    table_set(state, ns, "_providerSwitched", Val::Bool(false));
    Ok(0)
}

fn unit_buff(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let index = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    let aura = collect_visible_unit_auras(state, &unit, AuraFilter::Helpful)
        .into_iter()
        .nth(index.saturating_sub(1) as usize);
    Ok(push_legacy_aura_result(state, aura.as_ref(), &unit))
}

fn unit_debuff(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let index = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    let aura = collect_visible_unit_auras(state, &unit, AuraFilter::Harmful)
        .into_iter()
        .nth(index.saturating_sub(1) as usize);
    Ok(push_legacy_aura_result(state, aura.as_ref(), &unit))
}

fn unit_aura(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let index = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    let filter_str = Option::<String>::from_stack(state, 3)?.unwrap_or_default();
    let aura = collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str))
        .into_iter()
        .nth(index.saturating_sub(1) as usize);
    Ok(push_legacy_aura_result(state, aura.as_ref(), &unit))
}

fn aura_util_create_filter_string(state: &mut LuaState) -> LuaResult<u32> {
    let args = state.top.saturating_sub(state.base);
    let mut filters = Vec::new();

    for index in 1..=args {
        let Val::Str(value_ref) = stack_val(state, index as i32) else {
            continue;
        };
        let Some(value) = state.gc.string_arena.get(value_ref) else {
            continue;
        };
        if !value.data().is_empty() {
            filters.push(value.data().to_vec());
        }
    }

    let joined = filters.join(b"|".as_slice());
    let lua_string = state.gc.intern_string(&joined);
    state.push(Val::Str(lua_string));
    Ok(1)
}

fn aura_util_unpack_aura_data(state: &mut LuaState) -> LuaResult<u32> {
    Ok(push_legacy_aura_tuple(state, stack_val(state, 1)))
}

fn aura_util_for_each_aura(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let filter_str = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let max_count = Option::<f64>::from_stack(state, 3)?.map(|n| n.max(0.0) as usize);
    let callback = Val::from_stack(state, 4)?;
    let Val::Function(_) = callback else {
        return Ok(0);
    };

    for (seen, aura) in collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str))
        .into_iter()
        .enumerate()
    {
        if max_count.is_some_and(|limit| seen >= limit) {
            break;
        }
        let aura_table = build_aura_table(state, &aura, &unit);
        let result = call_function_state(state, callback, &[aura_table])?;
        if is_truthy(result) {
            break;
        }
    }

    Ok(0)
}

fn aura_util_find_aura(state: &mut LuaState) -> LuaResult<u32> {
    let predicate = Val::from_stack(state, 1)?;
    let unit = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let filter_str = Option::<String>::from_stack(state, 3)?.unwrap_or_default();
    let max_count = Option::<f64>::from_stack(state, 4)?.map(|n| n.max(0.0) as usize);
    let Val::Function(_) = predicate else {
        state.push(Val::Nil);
        return Ok(1);
    };

    for (seen, aura) in collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str))
        .into_iter()
        .enumerate()
    {
        if max_count.is_some_and(|limit| seen >= limit) {
            break;
        }
        let aura_table = build_aura_table(state, &aura, &unit);
        let matched = is_truthy(call_function_state(state, predicate, &[aura_table])?);
        if matched {
            state.push(aura_table);
            return Ok(1);
        }
    }

    state.push(Val::Nil);
    Ok(1)
}

fn aura_util_find_aura_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let unit = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let filter_str = Option::<String>::from_stack(state, 3)?.unwrap_or_default();
    let found = collect_visible_unit_auras(state, &unit, filter_from_str(&filter_str))
        .into_iter()
        .find(|aura| aura.name == name);
    match found {
        Some(aura) => {
            let table = build_aura_table(state, &aura, &unit);
            state.push(table);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn aura_util_get_aura_data_by_aura_instance_id(state: &mut LuaState) -> LuaResult<u32> {
    if provider_switched(state) {
        state.push(Val::Nil);
        return Ok(1);
    }
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let aura_id = Option::<f64>::from_stack(state, 2)?.unwrap_or_default() as i32;
    push_aura_by_instance_id(state, &unit, aura_id);
    Ok(1)
}

fn push_legacy_aura_result(state: &mut LuaState, aura: Option<&AuraInfo>, unit: &str) -> u32 {
    match aura {
        Some(aura) => {
            let aura_table = build_aura_table(state, aura, unit);
            push_legacy_aura_tuple(state, aura_table)
        }
        None => {
            state.push(Val::Nil);
            1
        }
    }
}

fn push_legacy_aura_tuple(state: &mut LuaState, aura: Val) -> u32 {
    let Val::Table(_) = aura else {
        state.push(Val::Nil);
        return 1;
    };

    let name = table_get(state, aura, "name");
    let icon = table_get(state, aura, "icon");
    let applications = table_get(state, aura, "applications");
    let dispel_name = table_get(state, aura, "dispelName");
    let duration = table_get(state, aura, "duration");
    let expiration_time = table_get(state, aura, "expirationTime");
    let source_unit = table_get(state, aura, "sourceUnit");
    let is_stealable = table_get(state, aura, "isStealable");
    let spell_id = table_get(state, aura, "spellId");

    state.push(name);
    state.push(icon);
    state.push(applications);
    state.push(dispel_name);
    state.push(duration);
    state.push(expiration_time);
    state.push(source_unit);
    state.push(is_stealable);
    state.push(Val::Nil);
    state.push(spell_id);
    10
}

fn is_truthy(value: Val) -> bool {
    !matches!(value, Val::Nil | Val::Bool(false))
}

// ── Aura table builder ───────────────────────────────────────────────────────

fn build_aura_table(state: &mut LuaState, aura: &AuraInfo, unit: &str) -> Val {
    let t = create_table_with_capacity(state, AURA_DATA_HASH_FIELDS);
    write_aura_identity(state, t, aura, unit);
    write_aura_flags(state, t, aura, unit);
    t
}

fn write_aura_identity(state: &mut LuaState, t: Val, aura: &AuraInfo, unit: &str) {
    let name = create_string(state, &aura.name);
    let source_unit = if unit == "player" {
        "player"
    } else {
        aura.source_unit.as_str()
    };
    let source = create_string(state, source_unit);
    let empty_points = create_table(state);
    // Retail contract: dispelName is nilable — absent for non-dispellable auras.
    let dispel = aura.dispel_type.as_deref().map(|d| create_string(state, d));

    table_set(state, t, "name", name);
    table_set(state, t, "icon", Val::Num(aura.icon as f64));
    table_set(state, t, "applications", Val::Num(aura.applications as f64));
    table_set(state, t, "charges", Val::Num(aura.applications as f64));
    table_set(state, t, "stackCount", Val::Num(aura.applications as f64));
    if let Some(dispel) = dispel {
        table_set(state, t, "dispelName", dispel);
    }
    table_set(state, t, "duration", Val::Num(aura.duration));
    table_set(state, t, "expirationTime", Val::Num(aura.expiration_time));
    table_set(state, t, "sourceUnit", source);
    table_set(state, t, "spellId", Val::Num(aura.spell_id as f64));
    table_set(state, t, "timeMod", Val::Num(1.0));
    table_set(state, t, "points", empty_points);
    table_set(
        state,
        t,
        "auraInstanceID",
        Val::Num(aura.aura_instance_id as f64),
    );
}

fn write_aura_flags(state: &mut LuaState, t: Val, aura: &AuraInfo, unit: &str) {
    table_set(state, t, "isStealable", Val::Bool(aura.is_stealable));
    table_set(state, t, "nameplateShowPersonal", Val::Bool(false));
    table_set(state, t, "canApplyAura", Val::Bool(aura.can_apply_aura));
    table_set(state, t, "isBossAura", Val::Bool(false));
    let from_player = if unit == "player" {
        true
    } else {
        aura.is_from_player_or_player_pet
    };
    table_set(state, t, "isFromPlayerOrPlayerPet", Val::Bool(from_player));
    table_set(state, t, "nameplateShowAll", Val::Bool(false));
    table_set(state, t, "isHelpful", Val::Bool(aura.is_helpful));
    table_set(state, t, "isHarmful", Val::Bool(!aura.is_helpful));
    table_set(state, t, "isNameplateOnly", Val::Bool(false));
    table_set(state, t, "isRaid", Val::Bool(aura.is_helpful));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_helpful_aura() -> AuraInfo {
        AuraInfo {
            name: "Plain Buff".to_string(),
            spell_id: 99021,
            icon: 134973,
            duration: 30.0,
            expiration_time: 30.0,
            applications: 1,
            source_unit: "player".to_string(),
            is_helpful: true,
            is_stealable: false,
            can_apply_aura: false,
            is_from_player_or_player_pet: true,
            dispel_type: None,
            aura_instance_id: 1,
        }
    }

    #[test]
    fn external_defensive_filter_does_not_match_plain_buffs() {
        let aura = plain_helpful_aura();

        assert!(aura_matches_filter(&aura, filter_from_str("HELPFUL")));
        assert!(!aura_matches_filter(
            &aura,
            filter_from_str("HELPFUL|EXTERNAL_DEFENSIVE")
        ));
    }
}
