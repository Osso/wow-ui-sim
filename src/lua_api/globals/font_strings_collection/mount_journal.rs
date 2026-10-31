//! C_MountJournal namespace.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_string_static, create_table,
};
use crate::lua_bridge::{FromStack, IntoStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

use super::set_global_val;

fn mount_matches_search(mount: &crate::lua_api::state_types::MountData, search_text: &str) -> bool {
    search_text.is_empty() || mount.name.to_lowercase().contains(search_text)
}

fn displayed_mount_count(st: &crate::lua_api::state::SimState) -> i32 {
    st.world
        .mounts
        .iter()
        .filter(|mount| mount_matches(st, mount))
        .count() as i32
}

fn displayed_mount(
    st: &crate::lua_api::state::SimState,
    index: i32,
) -> Option<&crate::lua_api::state_types::MountData> {
    if index <= 0 {
        return None;
    }
    let i = (index - 1) as usize;
    st.world
        .mounts
        .iter()
        .filter(|mount| mount_matches(st, mount))
        .nth(i)
}

fn displayed_mount_snapshot(
    st: &crate::lua_api::state::SimState,
    index: i32,
) -> Option<MountInfoSnapshot> {
    displayed_mount(st, index).map(|mount| MountInfoSnapshot::from_state(st, mount))
}

fn push_mount_info(state: &mut LuaState, info: &MountInfoSnapshot) -> u32 {
    let name_val = create_string(state, &info.name);
    state.push(name_val);
    state.push(Val::Num(info.spell_id));
    state.push(Val::Num(info.icon));
    state.push(Val::Bool(info.is_active));
    state.push(Val::Bool(info.is_usable));
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    state.push(Val::Nil);
    state.push(Val::Bool(false));
    state.push(Val::Bool(info.is_collected));
    state.push(Val::Num(info.mount_id));
    12
}

fn register_mount_counts(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetNumMounts", |state| {
        let count = borrow_state(state)?.world.mounts.len() as i32;
        count.into_stack(state)
    })?
    .set_function("GetNumDisplayedMounts", |state| {
        let count = {
            let st = borrow_state(state)?;
            displayed_mount_count(&st)
        };
        count.into_stack(state)
    })?
    .set_function("GetDisplayedMountID", |state| {
        let index = i32::from_stack(state, 1)?;
        let mount_id = {
            let st = borrow_state(state)?;
            displayed_mount(&st, index).map(|mount| mount.mount_id as f64)
        };
        match mount_id {
            Some(id) => id.into_stack(state),
            None => Ok(0),
        }
    })
}

struct MountInfoSnapshot {
    name: String,
    spell_id: f64,
    icon: f64,
    is_active: bool,
    is_usable: bool,
    is_collected: bool,
    mount_id: f64,
}

impl MountInfoSnapshot {
    fn from_state(
        st: &crate::lua_api::state::SimState,
        m: &crate::lua_api::state_types::MountData,
    ) -> Self {
        Self {
            name: m.name.clone(),
            spell_id: m.spell_id as f64,
            icon: m.icon as f64,
            is_active: st.world.active_mount_id == Some(m.mount_id),
            is_usable: m.is_usable,
            is_collected: m.is_collected,
            mount_id: m.mount_id as f64,
        }
    }

    fn push(self, state: &mut LuaState) -> u32 {
        push_mount_info(state, &self)
    }
}

fn mount_get_displayed_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let snapshot = {
        let st = borrow_state(state)?;
        displayed_mount_snapshot(&st, index)
    };
    let Some(snapshot) = snapshot else {
        return Ok(0);
    };
    Ok(snapshot.push(state))
}

fn mount_get_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let mount_id = u32::from_stack(state, 1)?;
    let snapshot = {
        let st = borrow_state(state)?;
        st.world
            .mounts
            .iter()
            .find(|m| m.mount_id == mount_id)
            .map(|mount| MountInfoSnapshot::from_state(&st, mount))
    };
    let Some(snapshot) = snapshot else {
        return Ok(0);
    };
    Ok(snapshot.push(state))
}

fn mount_get_info_extra_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let mount_id = u32::from_stack(state, 1)?;
    let mount_type = {
        let st = borrow_state(state)?;
        st.world
            .mounts
            .iter()
            .find(|m| m.mount_id == mount_id)
            .map(|m| m.mount_type as f64)
    };
    let Some(mount_type) = mount_type else {
        return Ok(0);
    };
    let empty = create_string_static(state, "");
    let source = create_string(state, "Drop");
    state.push(Val::Num(0.0));
    state.push(empty);
    state.push(source);
    state.push(Val::Bool(false));
    state.push(Val::Num(mount_type));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
    Ok(9)
}

fn register_mount_info(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetDisplayedMountInfo", mount_get_displayed_info)?
        .set_function("GetMountInfoByID", mount_get_info_by_id)?
        .set_function("GetMountInfoExtraByID", mount_get_info_extra_by_id)
}

fn register_mount_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    let tb = register_mount_id_stubs(tb)?;
    let tb = register_mount_equipment_stubs(tb)?;
    let tb = register_mount_fanfare_stubs(tb)?;
    let tb = register_mount_filter_stubs(tb)?;
    let tb = register_mount_favorite_stubs(tb)?;
    register_mount_action_stubs(tb)
}

fn register_mount_id_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetMountIDs", mount_get_ids)?
        .set_function("GetMountAllCreatureDisplayInfoByID", |state| {
            create_table(state).into_stack(state)
        })
}

fn mount_get_ids(state: &mut LuaState) -> LuaResult<u32> {
    let ids = {
        let st = borrow_state(state)?;
        st.world
            .mounts
            .iter()
            .map(|mount| mount.mount_id as f64)
            .collect::<Vec<_>>()
    };
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return Ok(0);
    };
    for (index, mount_id) in ids.into_iter().enumerate() {
        if let Some(array) = state.gc.tables.get_mut(table_ref) {
            let _ = array.raw_set(
                Val::Num(index as f64 + 1.0),
                Val::Num(mount_id),
                &state.gc.string_arena,
            );
        }
        state.gc.barrier_back(table_ref);
    }
    table.into_stack(state)
}

fn register_mount_equipment_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("AreMountEquipmentEffectsSuppressed", |state| {
        false.into_stack(state)
    })?
    .set_function("GetAppliedMountEquipmentID", |_state| Ok(0))?
    .set_function("GetMountEquipmentUnlockLevel", |state| {
        (0i32).into_stack(state)
    })?
    .set_function("IsItemMountEquipment", |state| false.into_stack(state))?
    .set_function("IsMountEquipmentApplied", |state| false.into_stack(state))
}

fn register_mount_fanfare_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("ClearRecentFanfares", |_state| Ok(0))?
        .set_function("ClearFanfare", |_state| Ok(0))?
        .set_function("NeedsFanfare", |state| false.into_stack(state))?
        .set_function("GetNumMountsNeedingFanfare", |state| {
            (0i32).into_stack(state)
        })
}

fn register_mount_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    let tb = tb
        .set_function("IsDragonridingUnlocked", |state| false.into_stack(state))?
        .set_function("IsValidSourceFilter", |state| true.into_stack(state))?
        .set_function("IsValidTypeFilter", |state| true.into_stack(state))?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        return register_modeled_mount_filters(tb);
    }
    let tb = register_collected_filter_stubs(tb)?;
    let tb = register_default_filter_stubs(tb)?;
    let tb = register_source_filter_stubs(tb)?;
    register_type_filter_stubs(tb)
}

fn register_modeled_mount_filters(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("IsUsingDefaultFilters", |state| {
        let value = borrow_state(state)?.world.mount_filters == Default::default();
        value.into_stack(state)
    })?
    .set_function("SetDefaultFilters", |state| {
        update_mount_filters(state, |filters| *filters = Default::default())
    })?
    .set_function("GetCollectedFilterSetting", |state| {
        let index = i32::from_stack(state, 1)?;
        let value = borrow_state(state)?.world.mount_filters.collected(index);
        value.into_stack(state)
    })?
    .set_function("SetCollectedFilterSetting", |state| {
        let index = i32::from_stack(state, 1)?;
        let checked = bool::from_stack(state, 2)?;
        update_mount_filters(state, |f| f.set_collected(index, checked))
    })?
    .set_function("IsSourceChecked", |state| {
        let index = i32::from_stack(state, 1)?;
        let value = borrow_state(state)?.world.mount_filters.source(index);
        value.into_stack(state)
    })?
    .set_function("SetSourceFilter", |state| {
        let index = i32::from_stack(state, 1)?;
        let checked = bool::from_stack(state, 2)?;
        update_mount_filters(state, |f| f.set_source(index, checked))
    })?
    .set_function("SetAllSourceFilters", |state| {
        let checked = bool::from_stack(state, 1)?;
        update_mount_filters(state, |f| f.all_sources(checked))
    })?
    .set_function("IsTypeChecked", |state| {
        let index = i32::from_stack(state, 1)?;
        let value = borrow_state(state)?.world.mount_filters.mount_type(index);
        value.into_stack(state)
    })?
    .set_function("SetTypeFilter", |state| {
        let index = i32::from_stack(state, 1)?;
        let checked = bool::from_stack(state, 2)?;
        update_mount_filters(state, |f| f.set_type(index, checked))
    })?
    .set_function("SetAllTypeFilters", |state| {
        let checked = bool::from_stack(state, 1)?;
        update_mount_filters(state, |f| f.all_types(checked))
    })
}

fn update_mount_filters(
    state: &mut LuaState,
    change: impl FnOnce(&mut crate::c_api::mount_filters::MountFilters),
) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let before = displayed_mount_ids(&sim);
    change(&mut sim.world.mount_filters);
    if before != displayed_mount_ids(&sim) {
        sim.events.push_simple("MOUNT_JOURNAL_SEARCH_UPDATED");
    }
    Ok(0)
}

fn displayed_mount_ids(st: &crate::lua_api::state::SimState) -> Vec<u32> {
    st.world
        .mounts
        .iter()
        .filter(|m| mount_matches(st, m))
        .map(|m| m.mount_id)
        .collect()
}

fn mount_matches(
    st: &crate::lua_api::state::SimState,
    mount: &crate::lua_api::state_types::MountData,
) -> bool {
    if !mount_matches_search(mount, &st.world.mount_search_text) {
        return false;
    }
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return true;
    }
    let f = &st.world.mount_filters;
    let collected = if mount.is_collected { 1 } else { 2 };
    if !f.collected(collected) || (!mount.is_usable && !f.collected(3)) {
        return false;
    }
    // Current mount data publishes all sources as Drop.
    f.source(1) && f.mount_type(mount.filter_type)
}

fn register_collected_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetCollectedFilterSetting", |state| true.into_stack(state))?
        .set_function("SetCollectedFilterSetting", |_state| Ok(0))
}

fn register_default_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("IsUsingDefaultFilters", |state| true.into_stack(state))?
        .set_function("SetDefaultFilters", |_state| Ok(0))
}

fn register_source_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("IsSourceChecked", |state| false.into_stack(state))?
        .set_function("SetSourceFilter", |_state| Ok(0))?
        .set_function("SetAllSourceFilters", |_state| Ok(0))
}

fn register_type_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("IsTypeChecked", |state| false.into_stack(state))?
        .set_function("SetTypeFilter", |_state| Ok(0))
}

fn register_mount_favorite_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetIsFavorite", mount_get_is_favorite)?
        .set_function("SetIsFavorite", mount_set_is_favorite)?
        .set_function("GetMountLink", |state| {
            state.push(Val::Nil);
            Ok(1)
        })?
        .set_function("GetMountUsabilityByID", |state| {
            (false, false, false).into_stack(state)
        })
}

fn register_mount_action_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("Summon", |_state| Ok(0))?
        .set_function("SummonByID", mount_summon_by_id)?
        .set_function("Dismiss", mount_dismiss)?
        .set_function("SetSearch", mount_set_search)?
        .set_function("GetDynamicFlightModeSpellID", |state| {
            (0i32).into_stack(state)
        })?
        .set_function("PickupDynamicFlightMode", |_state| Ok(0))?
        .set_function("SwapDynamicFlightMode", |_state| Ok(0))
}

fn mount_id_from_display_index(st: &crate::lua_api::state::SimState, index: i32) -> Option<u32> {
    displayed_mount(st, index)
        .filter(|mount| mount.is_collected)
        .map(|mount| mount.mount_id)
}

fn mount_get_is_favorite(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let result = {
        let st = borrow_state(state)?;
        let mount_id = mount_id_from_display_index(&st, index);
        let is_favorite = mount_id
            .map(|id| st.world.favorite_mounts.contains(&id))
            .unwrap_or(false);
        let can_favorite = mount_id.is_some();
        (is_favorite, can_favorite)
    };
    result.into_stack(state)
}

fn mount_set_is_favorite(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let is_favorite = bool::from_stack(state, 2)?;
    let mut st = borrow_state_mut(state)?;
    let Some(mount_id) = mount_id_from_display_index(&st, index) else {
        return Ok(0);
    };

    if is_favorite {
        st.world.favorite_mounts.insert(mount_id);
    } else {
        st.world.favorite_mounts.remove(&mount_id);
    }
    Ok(0)
}

fn mount_summon_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let mount_id = u32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    let can_summon_random_favorite = mount_id == 0 && !st.world.favorite_mounts.is_empty();
    let target_mount_id = if can_summon_random_favorite {
        st.world.favorite_mounts.iter().copied().min()
    } else {
        Some(mount_id)
    };

    let Some(target_mount_id) = target_mount_id else {
        return Ok(0);
    };
    let is_usable =
        st.world.mounts.iter().any(|mount| {
            mount.mount_id == target_mount_id && mount.is_collected && mount.is_usable
        });
    if is_usable {
        st.world.active_mount_id = Some(target_mount_id);
    }
    Ok(0)
}

fn mount_dismiss(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.world.active_mount_id = None;
    Ok(0)
}

fn mount_set_search(state: &mut LuaState) -> LuaResult<u32> {
    let search_text = Option::<String>::from_stack(state, 1)?
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let mut sim = borrow_state_mut(state)?;
    let before = displayed_mount_ids(&sim);
    sim.world.mount_search_text = search_text;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail
        && before != displayed_mount_ids(&sim)
    {
        sim.events.push_simple("MOUNT_JOURNAL_SEARCH_UPDATED");
    }
    Ok(0)
}

pub fn register_rilua_mount_journal(lua: &mut rilua::Lua) -> LuaResult<()> {
    let tb = TableBuilder::new(lua.state_mut());
    let tb = register_mount_counts(tb)?;
    let tb = register_mount_info(tb)?;
    let tb = register_mount_stubs(tb)?;
    let t = tb.build();
    set_global_val(lua.state_mut(), "C_MountJournal", t);
    Ok(())
}
