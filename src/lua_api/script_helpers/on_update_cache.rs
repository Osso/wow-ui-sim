use std::collections::HashSet;

use rilua::vm::state::LuaState;
use rilua::{LuaApi, LuaApiMut, Val};

use super::{ON_POST_UPDATE_SCRIPTS_KEY, ON_UPDATE_SCRIPTS_KEY, registry_table};

pub(super) fn sync_on_update_runtime_cache(state: &mut LuaState, widget_id: u64) {
    use crate::lua_api::env::WowLuaAppData;

    let has_on_update = cached_handler_present(state, ON_UPDATE_SCRIPTS_KEY, widget_id);
    let has_on_post_update = cached_handler_present(state, ON_POST_UPDATE_SCRIPTS_KEY, widget_id);
    let should_track = has_on_update || has_on_post_update;

    let Some(app) = state.app_data_mut::<WowLuaAppData>() else {
        return;
    };
    let Ok(mut sim) = app.sim_state.try_borrow_mut() else {
        app.on_update_cache_dirty = true;
        return;
    };

    if should_track {
        sim.on_update_frames.insert(widget_id);
    } else {
        sim.on_update_frames.remove(&widget_id);
    }
    sim.visible_on_update_cache = None;
}

/// Rebuild `SimState::on_update_frames` from registry handler caches if
/// incremental sync previously missed updates due borrow contention.
pub fn reconcile_on_update_runtime_cache_if_dirty(lua: &mut rilua::Lua) {
    use crate::lua_api::env::WowLuaAppData;

    let is_dirty = lua
        .state()
        .app_data::<WowLuaAppData>()
        .is_some_and(|app| app.on_update_cache_dirty);
    if !is_dirty {
        return;
    }

    let state = lua.state_mut();
    let tracked = collect_on_update_handler_ids(state);

    let Some(app) = state.app_data_mut::<WowLuaAppData>() else {
        return;
    };
    let Ok(mut sim) = app.sim_state.try_borrow_mut() else {
        app.on_update_cache_dirty = true;
        return;
    };
    app.on_update_cache_dirty = false;

    if sim.on_update_frames != tracked {
        sim.on_update_frames = tracked;
        sim.visible_on_update_cache = None;
    }
}

fn collect_on_update_handler_ids(state: &mut LuaState) -> HashSet<u64> {
    let mut tracked = HashSet::new();
    collect_handler_ids_from_cache_table(state, ON_UPDATE_SCRIPTS_KEY, &mut tracked);
    collect_handler_ids_from_cache_table(state, ON_POST_UPDATE_SCRIPTS_KEY, &mut tracked);
    tracked
}

fn collect_handler_ids_from_cache_table(
    state: &mut LuaState,
    cache_key: &'static str,
    tracked: &mut HashSet<u64>,
) {
    let Some(table_ref) = registry_table(state, cache_key) else {
        return;
    };
    let Some(table) = state.gc.tables.get(table_ref) else {
        return;
    };

    for (index, value) in table.array_slice().iter().enumerate() {
        if !matches!(value, Val::Nil) {
            tracked.insert(index as u64 + 1);
        }
    }

    for (key, value) in table.hash_entries() {
        if matches!(value, Val::Nil) {
            continue;
        }
        let Some(widget_id) = numeric_key_to_widget_id(key) else {
            continue;
        };
        tracked.insert(widget_id);
    }
}

fn numeric_key_to_widget_id(key: Val) -> Option<u64> {
    let Val::Num(raw_id) = key else {
        return None;
    };
    if !raw_id.is_finite() || raw_id <= 0.0 {
        return None;
    }
    let widget_id = raw_id as u64;
    ((widget_id as f64 - raw_id).abs() <= f64::EPSILON).then_some(widget_id)
}

/// The subset of `frame_ids` that have an OnPostUpdate handler, in order.
pub fn frames_with_on_post_update(state: &mut LuaState, frame_ids: &[u64]) -> Vec<u64> {
    let Some(table_ref) = registry_table(state, ON_POST_UPDATE_SCRIPTS_KEY) else {
        return Vec::new();
    };
    let Some(table) = state.gc.tables.get(table_ref) else {
        return Vec::new();
    };
    frame_ids
        .iter()
        .copied()
        .filter(|&id| !matches!(table.get_int(id as i64), Val::Nil))
        .collect()
}

fn cached_handler_present(state: &mut LuaState, cache_key: &'static str, widget_id: u64) -> bool {
    let Some(table_ref) = registry_table(state, cache_key) else {
        return false;
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| !matches!(table.get_int(widget_id as i64), Val::Nil))
        .unwrap_or(false)
}
