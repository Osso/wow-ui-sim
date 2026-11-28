//! `C_LFGInfo` probe surface backed by `SimState.lfg_category_info` and
//! `SimState.lfg_active_categories`.
//!
//! Migrates 4 entries off the namespace stub tables:
//!
//! - `C_LFGInfo.CanPlayerUseLFD()` — returns `(true, nil)` matching the
//!   existing `c_lfg_info_can_player_use` behaviour in `c_model_info.rs`.
//! - `C_LFGInfo.CanPlayerUseGroupFinder()` — returns `(true, nil)` so the
//!   LFG micro menu button is available in the default simulator state.
//! - `C_LFGInfo.GetLFGCategoryInfo(categoryID)` — returns a table with
//!   `name` and `order` fields from `lfg_category_info`, or nil for
//!   unknown categories.
//! - `C_LFGInfo.GetSystemPanelData()` — returns a minimal table with
//!   `isAvailable = true` and `isAvailableAndEnabled = true`.
//! - `C_LFGInfo.IsLFGModeActiveForCategory(categoryID)` — returns true
//!   when the category id is in `lfg_active_categories`, false otherwise.

use crate::c_api::ensure_namespace;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::c_api::helpers::set_table_array;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::table_get;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_api::state_types::LfgCategoryInfo;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

pub(crate) fn register_c_lfg_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_LFGInfo")?;
    register_access_methods(state, table_ref)?;
    register_category_methods(state, table_ref)?;
    register_dungeon_methods(state, table_ref)?;
    #[cfg(feature = "client-retail")]
    register_queue_state_methods(state, table_ref)?;
    #[cfg(feature = "client-wowforever")]
    register_forever_queue_title_methods(state, table_ref)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        // LFGInfoDocumentation.lua:85-91 is byte-identical on Retail and Forever.
        table_set_rust_fn_static(
            state,
            table_ref,
            "ConfirmLfgExpandSearch",
            confirm_expand_search,
        )?;
    }
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Mists {
        super::lfr_availability::register(state, table_ref)?;
        super::scenario_finder_availability::register(state, table_ref)?;
    }
    register_active_dungeon_name(state, table_ref)
}

fn confirm_expand_search(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    if sim
        .lfg_expand_prompt
        .as_ref()
        .is_some_and(|(key, _)| *key != 0)
    {
        sim.lfg_expand_prompt = None;
    }
    Ok(0)
}

#[cfg(feature = "retail-12-1-5")]
fn register_active_dungeon_name(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetActiveLFGDungeonName",
        get_active_lfg_dungeon_name,
    )
}

#[cfg(not(feature = "retail-12-1-5"))]
fn register_active_dungeon_name(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    // Keep the namespace fallback from fabricating this PTR-only method.
    let removed = create_table(state);
    table_set(state, removed, "GetActiveLFGDungeonName", Val::Bool(true));
    table_set(state, Val::Table(table_ref), "__wow_removed_keys", removed);
    Ok(())
}

#[cfg(feature = "retail-12-1-5")]
fn get_active_lfg_dungeon_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = {
        let sim = borrow_state(state)?;
        match sim.world.instance_lfg_dungeon_id {
            None => String::new(),
            Some(id) => sim
                .lfd_dungeons
                .iter()
                .find(|dungeon| dungeon.dungeon_id == id)
                .map(|dungeon| dungeon.name.clone())
                .ok_or_else(|| {
                    rilua::runtime_error(format!(
                        "C_LFGInfo.GetActiveLFGDungeonName: instance LFG dungeon ID {id} is absent from lfd_dungeons"
                    ))
                })?,
        }
    };
    let result = create_string(state, &name);
    state.push(result);
    Ok(1)
}

fn register_access_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "CanPlayerUseGroupFinder",
        can_player_use_group_finder,
    )?;
    table_set_rust_fn_static(state, table_ref, "CanPlayerUseLFD", can_player_use_lfd)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "CanPlayerUsePremadeGroup",
        can_player_use_premade_group,
    )?;
    Ok(())
}

fn register_category_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetLFGCategoryInfo",
        get_lfg_category_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetSystemPanelData",
        get_system_panel_data,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsLFGModeActiveForCategory",
        is_lfg_mode_active_for_category,
    )?;
    Ok(())
}

#[cfg(feature = "client-wowforever")]
fn register_forever_queue_title_methods(
    state: &mut LuaState,
    table: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "GetAllEntriesForCategory",
        get_all_entries_for_category,
    )?;
    table_set_rust_fn_static(state, table, "HideNameFromUI", hide_name_from_ui)
}

#[cfg(feature = "client-retail")]
fn register_queue_state_methods(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "GetAllEntriesForCategory",
        get_all_entries_for_category,
    )?;
    table_set_rust_fn_static(state, table, "GetLFDLockStates", get_lfd_lock_states)?;
    table_set_rust_fn_static(state, table, "HideNameFromUI", hide_name_from_ui)
}

/// INFERRED from the Retail model (R88da) and Forever docs: return supplied IDs by category.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_all_entries_for_category(state: &mut LuaState) -> LuaResult<u32> {
    let category = i32::from_stack(state, 1)?;
    let entries = borrow_state(state)?
        .lfg_queue_entries
        .iter()
        .filter(|entry| entry.category_id == category)
        .map(|entry| entry.entry_id)
        .collect::<Vec<_>>();
    let result = create_table(state);
    for (index, entry_id) in entries.iter().enumerate() {
        set_table_array(
            state,
            result,
            index as i64 + 1,
            Val::Num(f64::from(*entry_id)),
        );
    }
    state.push(result);
    Ok(1)
}

/// INFERRED from Retail Blizzard_GroupFinder (R88da): return supplied lock rows.
#[cfg(feature = "client-retail")]
fn get_lfd_lock_states(state: &mut LuaState) -> LuaResult<u32> {
    let locks = borrow_state(state)?.lfg_lock_states.clone();
    let result = create_table(state);
    for (index, lock) in locks.iter().enumerate() {
        let row = create_table(state);
        table_set(
            state,
            row.clone(),
            "lfgID",
            Val::Num(f64::from(lock.lfg_id)),
        );
        table_set(
            state,
            row.clone(),
            "reason",
            Val::Num(f64::from(lock.reason)),
        );
        table_set(state, row.clone(), "hideEntry", Val::Bool(lock.hide_entry));
        set_table_array(state, result, index as i64 + 1, row);
    }
    state.push(result);
    Ok(1)
}

/// INFERRED from the Retail model (R88da) and Forever docs: unsupplied IDs are visible; supplied rows set this flag.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn hide_name_from_ui(state: &mut LuaState) -> LuaResult<u32> {
    let entry_id = i32::from_stack(state, 1)?;
    let hide_name = borrow_state(state)?
        .lfg_queue_entries
        .iter()
        .find(|entry| entry.entry_id == entry_id)
        .is_some_and(|entry| entry.hide_name);
    state.push(Val::Bool(hide_name));
    Ok(1)
}

/// Supplies queue entries and lock rows for selected Retail and Forever LFG consumers.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(
    builder: crate::lua_bridge::TableBuilder,
) -> LuaResult<crate::lua_bridge::TableBuilder> {
    builder
        .set_function("SetLfgQueueData", set_lfg_queue_data)?
        .set_function("ShowLfgExpandSearchPrompt", show_expand_search_prompt)
}

/// `A_Admin.ShowLfgExpandSearchPrompt(key, data)`: stores the server prompt
/// (a non-negative integer key and an opaque string) and fires
/// `SHOW_LFG_EXPAND_SEARCH_PROMPT` (LFGInfoDocumentation.lua:488), which the
/// original `LFGEventFrame_OnEvent` turns into the `LFG_QUEUE_EXPAND` popup.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn show_expand_search_prompt(state: &mut LuaState) -> LuaResult<u32> {
    let key = f64::from_stack(state, 1)?;
    let data = String::from_stack(state, 2)?;
    if !(key.is_finite() && key >= 0.0 && key.fract() == 0.0 && key <= u64::MAX as f64) {
        return Err(rilua::runtime_error(
            "ShowLfgExpandSearchPrompt: key must be a non-negative integer",
        ));
    }
    borrow_state_mut(state)?.lfg_expand_prompt = Some((key as u64, data.into_bytes()));
    crate::lua_api::script_helpers::fire_named_event_state(
        state,
        "SHOW_LFG_EXPAND_SEARCH_PROMPT",
        &[],
    );
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_lfg_queue_data(state: &mut LuaState) -> LuaResult<u32> {
    let entry_rows = stack_val(state, 1);
    let lock_rows = stack_val(state, 2);
    let entries = read_lfg_queue_entries(state, entry_rows)?;
    let locks = read_lfg_lock_states(state, lock_rows)?;
    let mut sim = borrow_state_mut(state)?;
    sim.lfg_queue_entries = entries;
    sim.lfg_lock_states = locks;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_lfg_queue_entries(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<Vec<crate::lua_api::state_types::collections::LfgQueueEntry>> {
    read_array(state, value)?
        .into_iter()
        .map(|row| {
            let entry_id = read_lfg_integer(table_get(state, row.clone(), "entryID"), "entryID")?;
            let hide_name = match table_get(state, row.clone(), "hideName") {
                Val::Bool(value) => value,
                _ => return Err(rilua::runtime_error("LFG entry hideName must be boolean")),
            };
            Ok(crate::lua_api::state_types::collections::LfgQueueEntry {
                category_id: read_lfg_integer(table_get(state, row, "categoryID"), "categoryID")?,
                entry_id,
                hide_name,
            })
        })
        .collect()
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_lfg_lock_states(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<Vec<crate::lua_api::state_types::collections::LfgLockState>> {
    read_array(state, value)?
        .into_iter()
        .map(|row| {
            let hide_entry = match table_get(state, row.clone(), "hideEntry") {
                Val::Bool(value) => value,
                _ => return Err(rilua::runtime_error("LFG lock hideEntry must be boolean")),
            };
            Ok(crate::lua_api::state_types::collections::LfgLockState {
                lfg_id: read_lfg_integer(table_get(state, row.clone(), "lfgID"), "lfgID")?,
                reason: read_lfg_integer(table_get(state, row, "reason"), "reason")?,
                hide_entry,
            })
        })
        .collect()
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_array(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let Val::Table(reference) = value else {
        return Err(rilua::runtime_error("LFG supplied data must be an array"));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| rilua::runtime_error("LFG supplied array is unavailable"))?;
    Ok((1..=table.len(&state.gc.string_arena))
        .map(|index| table.get_int(index as i64))
        .collect())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_lfg_integer(value: Val, field: &str) -> LuaResult<i32> {
    match value {
        Val::Num(number) if number.is_finite() && number.fract() == 0.0 => {
            i32::try_from(number as i64)
                .map_err(|_| rilua::runtime_error(format!("LFG {field} is outside i32 range")))
        }
        _ => Err(rilua::runtime_error(format!(
            "LFG {field} must be an integer"
        ))),
    }
}

fn register_dungeon_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsInLFGFollowerDungeon",
        is_in_lfg_follower_dungeon,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsLFGFollowerDungeon",
        is_lfg_follower_dungeon,
    )?;
    #[cfg(feature = "lair-state-queries")]
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsInMatchmadeRaidWithoutRoleRequirements",
        is_in_matchmade_raid_without_role_requirements,
    )?;
    Ok(())
}

/// INFERRED: LFGUtil's `OverrideLFGSetRoleRestriction` lifts role restrictions
/// exactly for LFG-matched lairs, so a raid-sized group in one is the matchmade
/// raid without role requirements.
#[cfg(feature = "lair-state-queries")]
fn is_in_matchmade_raid_without_role_requirements(state: &mut LuaState) -> LuaResult<u32> {
    let in_raid = crate::lua_api::globals::group_queries::active_party_count(state)? >= 6;
    let lfg_lair = {
        let sim = borrow_state(state)?;
        sim.has_active_lair && sim.active_lair_is_lfg
    };
    state.push(Val::Bool(in_raid && lfg_lair));
    Ok(1)
}

fn can_player_use_group_finder(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    state.push(Val::Nil);
    Ok(2)
}

fn can_player_use_lfd(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    state.push(Val::Nil);
    Ok(2)
}

pub(crate) fn can_player_use_premade_group(state: &mut LuaState) -> LuaResult<u32> {
    let v = borrow_state(state)?.can_use_premade_group;
    state.push(Val::Bool(v));
    Ok(1)
}

fn get_lfg_category_info(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let info = lookup_lfg_category_info(state, category_id)?;
    push_lfg_category_info(state, info);
    Ok(1)
}

fn lookup_lfg_category_info(
    state: &LuaState,
    category_id: i32,
) -> LuaResult<Option<LfgCategoryInfo>> {
    let sim = borrow_state(state)?;
    Ok(sim.lfg_category_info.get(&category_id).cloned())
}

fn push_lfg_category_info(state: &mut LuaState, info: Option<LfgCategoryInfo>) {
    let Some(info) = info else {
        state.push(Val::Nil);
        return;
    };
    let t = create_table(state);
    populate_lfg_category_table(state, t, &info);
    state.push(t);
}

fn populate_lfg_category_table(state: &mut LuaState, table: Val, info: &LfgCategoryInfo) {
    populate_lfg_category_identity_fields(state, table.clone(), info);
    populate_lfg_category_flag_fields(state, table, info);
}

fn populate_lfg_category_identity_fields(state: &mut LuaState, table: Val, info: &LfgCategoryInfo) {
    let name_val = create_string(state, &info.name);
    table_set(state, table.clone(), "name", name_val);
    table_set(state, table.clone(), "order", Val::Num(info.order as f64));
}

fn populate_lfg_category_flag_fields(state: &mut LuaState, table: Val, info: &LfgCategoryInfo) {
    table_set(
        state,
        table.clone(),
        "separateRecommended",
        Val::Bool(info.separate_recommended),
    );
    table_set(
        state,
        table.clone(),
        "preferCurrentArea",
        Val::Bool(info.prefer_current_area),
    );
    table_set(
        state,
        table.clone(),
        "allowCrossFaction",
        Val::Bool(info.allow_cross_faction),
    );
    table_set(
        state,
        table.clone(),
        "autoChooseActivity",
        Val::Bool(info.auto_choose_activity),
    );
    table_set(
        state,
        table,
        "showPlaystyleDropdown",
        Val::Bool(info.show_playstyle_dropdown),
    );
}

fn get_system_panel_data(state: &mut LuaState) -> LuaResult<u32> {
    let t = create_table(state);
    table_set(state, t, "isAvailable", Val::Bool(true));
    table_set(state, t, "isAvailableAndEnabled", Val::Bool(true));
    state.push(t);
    Ok(1)
}

fn is_lfg_mode_active_for_category(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let active = {
        let sim = borrow_state(state)?;
        is_lfg_category_active(&sim.lfg_active_categories, category_id)
    };
    state.push(Val::Bool(active));
    Ok(1)
}

fn is_lfg_category_active(active_categories: &HashSet<i32>, category_id: i32) -> bool {
    active_categories.contains(&category_id)
}

/// `C_LFGInfo.IsInLFGFollowerDungeon()` -> bool.
fn is_in_lfg_follower_dungeon(state: &mut LuaState) -> LuaResult<u32> {
    let is_follower = {
        let sim = borrow_state(state)?;
        sim.world.in_instance
            && sim.world.instance_lfg_dungeon_id.is_some_and(|id| {
                sim.lfd_dungeons
                    .iter()
                    .find(|dungeon| dungeon.dungeon_id == id)
                    .is_some_and(|dungeon| dungeon.is_follower_dungeon)
            })
    };
    state.push(Val::Bool(is_follower));
    Ok(1)
}

/// `C_LFGInfo.IsLFGFollowerDungeon(dungeonID)` -> bool.
fn is_lfg_follower_dungeon(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon_id = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => {
            state.push(Val::Bool(false));
            return Ok(1);
        }
    };
    let is_follower = borrow_state(state)?
        .lfd_dungeons
        .iter()
        .find(|d| d.dungeon_id == dungeon_id)
        .map(|d| d.is_follower_dungeon)
        .unwrap_or(false);
    state.push(Val::Bool(is_follower));
    Ok(1)
}

/// `A_Admin.SetCanUsePremadeGroup(b?)` — no-arg defaults to true.
pub fn admin_set_can_use_premade_group(state: &mut LuaState) -> LuaResult<u32> {
    let v = Option::<bool>::from_stack(state, 1)?.unwrap_or(true);
    borrow_state_mut(state)?.can_use_premade_group = v;
    Ok(0)
}
