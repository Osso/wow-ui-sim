//! Frame metatable and builtin-frame initialisation.

use crate::lua_api::frame::methods::{
    button_anchor_hierarchy, core_state, map_frames, misc, text_attribute_event, widgets,
};
use crate::lua_api::methods::{
    borrow_state_mut, extract_frame_id, get_frame_env_for_debug, get_or_create_frame_fields,
    registry_set, table_set_static, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::{LuaApiMut, Val};
use std::cell::RefCell;
use std::rc::Rc;

use super::super::builtin_frames::create_builtin_frames;
use super::super::state::SimState;

/// Create built-in frames in the widget registry before Lua loads.
/// Registers a `__BuiltIn` pseudo-addon as their owner.
pub(crate) fn init_builtin_frames(state: &Rc<RefCell<SimState>>) {
    let mut s = state.borrow_mut();
    let owner = s.addons.len() as u16;
    s.addons.push(super::super::AddonInfo {
        folder_name: "__BuiltIn".to_string(),
        title: "Built-in Frames".to_string(),
        enabled: true,
        loaded: true,
        ..Default::default()
    });
    let (w, h) = (s.screen_width, s.screen_height);
    create_builtin_frames(&mut s.widgets, w, h, owner);
}

/// Only simulator bootstrap has run here; addon Lua/XML definitions start later.
/// Include Lua-created workaround frames as well as the Rust builtin stand-ins.
pub(super) fn mark_bootstrap_frame_placeholders(state: &Rc<RefCell<SimState>>) {
    let mut sim = state.borrow_mut();
    let ids: Vec<_> = sim.widgets.iter_ids().collect();
    for id in ids {
        if let Some(frame) = sim.widgets.get_mut(id) {
            frame.is_simulator_placeholder = true;
        }
    }
}

pub(super) fn init_frame_metatable(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    let frame_mt = Val::Table(state.gc.alloc_table(rilua::vm::table::Table::new()));
    table_set_static(state, frame_mt, "__index", frame_mt);
    registry_set(state, "__rilua_frame_mt", frame_mt);

    let Val::Table(frame_mt_ref) = frame_mt else {
        unreachable!("frame metatable must be a table");
    };
    super::super::timer_layout::register_layout_fns_on_table(state, frame_mt_ref)?;
    core_state::register_all(state, frame_mt_ref)?;
    misc::register_all(state, frame_mt_ref)?;
    map_frames::register_all(state, frame_mt_ref)?;
    text_attribute_event::register_all(state, frame_mt_ref)?;
    button_anchor_hierarchy::register_all(state, frame_mt_ref)?;
    widgets::register_all(state, frame_mt_ref)?;
    #[cfg(any(
        feature = "client-wrath",
        feature = "client-mists",
        feature = "client-era",
        feature = "client-anniversary"
    ))]
    crate::wrath::frame_methods::register_all(state, frame_mt_ref)?;

    // Replace the self-referencing `__index` with a shallow clone that omits
    // metamethod keys. Blizzard's restricted code does
    // `CopyTable(GetFrameMetatable().__index)` (RestrictedExecution.lua), which
    // would infinitely recurse if `__index` pointed back at the metatable
    // itself. The clone captures the methods registered above and stays
    // stable for the lifetime of the VM (method registration only happens
    // here at init).
    let frame_index = build_frame_index_table(state, frame_mt_ref);
    table_set_static(state, frame_mt, "__index", Val::Table(frame_index));
    table_set_rust_fn_static(state, frame_mt_ref, "__newindex", frame_newindex)?;
    table_set_rust_fn_static(
        state,
        state.global,
        "__wow_get_frame_env",
        get_frame_env_for_debug,
    )?;

    // Pin the shared frame metatable + its __index clone for the lifetime
    // of the VM. Method registration only happens here at init, so the
    // pair is effectively immutable — GC never needs to walk it.
    state.gc.pin_object(Val::Table(frame_mt_ref));
    state.gc.pin_object(Val::Table(frame_index));
    Ok(())
}

fn frame_newindex(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let frame_val = stack_val(state, 1);
    let Some(parent_id) = extract_frame_id(state, frame_val) else {
        return Ok(0);
    };
    let key_val = stack_val(state, 2);
    let value = stack_val(state, 3);
    let Val::Table(table_ref) = frame_val else {
        return Ok(0);
    };

    assign_frame_table_field(state, table_ref, key_val, value);
    assign_frame_fields_value(state, parent_id, key_val, value);

    let Some(key) = string_key(state, key_val) else {
        return Ok(0);
    };
    if let Some(child_id) = extract_frame_id(state, value) {
        sync_child_key(state, parent_id, child_id, key)?;
    } else {
        remove_child_key(state, parent_id, &key)?;
    }

    Ok(0)
}

fn assign_frame_table_field(
    state: &mut rilua::vm::state::LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    key_val: Val,
    value: Val,
) {
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(key_val, value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}

#[cfg_attr(feature = "rehash-stats", track_caller)]
fn assign_frame_fields_value(
    state: &mut rilua::vm::state::LuaState,
    frame_id: u64,
    key_val: Val,
    value: Val,
) {
    let fields = get_or_create_frame_fields(state, frame_id);
    let Val::Table(fields_ref) = fields else {
        return;
    };
    if let Some(table) = state.gc.tables.get_mut(fields_ref) {
        let _ = table.raw_set(key_val, value, &state.gc.string_arena);
    }
    state.gc.barrier_back(fields_ref);
}

fn string_key(state: &mut rilua::vm::state::LuaState, key_val: Val) -> Option<String> {
    matches!(key_val, Val::Str(_))
        .then(|| val_to_string(state, key_val))
        .flatten()
}

fn sync_child_key(
    state: &mut rilua::vm::state::LuaState,
    parent_id: u64,
    child_id: u64,
    key: String,
) -> rilua::LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    if let Some(parent) = sim.widgets.get_mut(parent_id) {
        parent.children_keys.insert(key.clone(), child_id);
    }
    if let Some(child) = sim.widgets.get_mut(child_id) {
        child.parent_key.get_or_insert(key);
    }

    Ok(())
}

fn remove_child_key(
    state: &mut rilua::vm::state::LuaState,
    parent_id: u64,
    key: &str,
) -> rilua::LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    if let Some(parent) = sim.widgets.get_mut(parent_id) {
        parent.children_keys.remove(key);
    }
    Ok(())
}

/// Build a shallow, non-cyclic clone of the frame metatable's method entries.
///
/// Skips keys that start with `__` (metamethods) so the resulting table only
/// exposes frame methods — matching what Blizzard's restricted loader expects
/// from `GetFrameMetatable().__index`.
fn build_frame_index_table(
    state: &mut rilua::vm::state::LuaState,
    frame_mt_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> rilua::vm::gc::arena::GcRef<rilua::vm::table::Table> {
    let new_ref = state.gc.alloc_table(rilua::vm::table::Table::new());
    let entries = state
        .gc
        .tables
        .get(frame_mt_ref)
        .map(|table| table.hash_entries())
        .unwrap_or_default();
    for (key, value) in entries {
        if let Val::Str(str_ref) = key
            && let Some(name) = state.gc.string_arena.get(str_ref)
            && name.data().starts_with(b"__")
        {
            continue;
        }
        if let Some(t) = state.gc.tables.get_mut(new_ref) {
            let _ = t.raw_set(key, value, &state.gc.string_arena);
        }
    }
    new_ref
}
