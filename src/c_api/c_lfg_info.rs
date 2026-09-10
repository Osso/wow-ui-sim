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
    register_active_dungeon_name(state, table_ref)
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

fn register_dungeon_methods(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsLFGFollowerDungeon",
        is_lfg_follower_dungeon,
    )?;
    Ok(())
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
