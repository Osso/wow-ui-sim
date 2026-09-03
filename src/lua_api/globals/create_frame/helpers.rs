//! Low-level helpers shared by the dropdown, template, and top-level modules.

use crate::lua_api::hot_literals::{hot_metatable_key, metatable_idx};
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, create_table, extract_frame_id, frame_ref,
    get_or_create_frame_fields, registry_get, table_get, table_set,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ---------------------------------------------------------------------------
// Global table helpers
// ---------------------------------------------------------------------------

/// Get a named global value from the Lua global table.
///
/// Content-hashes `name` on every call. When the caller holds a
/// pre-interned `GcRef<LuaString>` (e.g. via `HotLiteralHandles`),
/// prefer [`get_global_by_key`] to skip the hash.
pub(super) fn get_global(state: &mut LuaState, name: &str) -> Val {
    let key = state.gc.intern_string(name.as_bytes());
    get_global_by_key(state, key)
}

/// Handle-aware variant of [`get_global`]: looks up `key` directly on
/// the global table without re-interning. Intended for callers that
/// cache the handle (e.g. consumers of the Track 1 hot-literal
/// registry).
pub(super) fn get_global_by_key(
    state: &LuaState,
    key: rilua::vm::gc::arena::GcRef<rilua::vm::string::LuaString>,
) -> Val {
    state
        .gc
        .tables
        .get(state.global)
        .map(|g| g.get_str(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

/// Set a named global to a numeric value.
pub(super) fn set_global_num(state: &mut LuaState, name: &str, value: f64) {
    set_global_raw(state, name, Val::Num(value));
}

/// Set a named global to any Val.
pub(super) fn set_global_raw(state: &mut LuaState, name: &str, value: Val) {
    let key = state.gc.intern_string(name.as_bytes());
    set_global_raw_by_key(state, key, value);
    crate::lua_api::global_slots::refresh_installed_slots_for_name(state, name);
}

/// Handle-aware variant of [`set_global_raw`]: writes `value` under
/// `key` on the global table without re-interning.
pub(super) fn set_global_raw_by_key(
    state: &mut LuaState,
    key: rilua::vm::gc::arena::GcRef<rilua::vm::string::LuaString>,
    value: Val,
) {
    let global = state.global;
    if let Some(g) = state.gc.tables.get_mut(global) {
        let _ = g.raw_set(Val::Str(key), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
}

// ---------------------------------------------------------------------------
// Frame field helpers
// ---------------------------------------------------------------------------

/// Set a named field (arg 2) on the frame (arg 1)'s fields table.
pub(super) fn set_frame_field(state: &mut LuaState, field_name: &str) -> LuaResult<u32> {
    let frame: Val = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let value: Val = crate::lua_bridge::FromStack::from_stack(state, 2)?;
    if let Some(id) = extract_frame_id(state, frame) {
        let fields = get_or_create_frame_fields(state, id);
        let key = create_string(state, field_name);
        if let Val::Table(fields_ref) = fields {
            if let Some(t) = state.gc.tables.get_mut(fields_ref) {
                let _ = t.raw_set(key, value, &state.gc.string_arena);
            }
            state.gc.barrier_back(fields_ref);
        }
        let frame_val = frame_ref(state, id)?;
        table_set(state, frame_val, field_name, value);
    }
    Ok(0)
}

/// Get a named field from the frame table in arg 1; push result, return 1.
pub(super) fn get_frame_field(state: &mut LuaState, field_name: &str) -> LuaResult<u32> {
    let frame: Val = crate::lua_bridge::FromStack::from_stack(state, 1)?;
    let value = table_get(state, frame, field_name);
    state.push(value);
    Ok(1)
}

/// Get a string-keyed value from a `Val::Table`.
pub(super) fn table_get_str(state: &mut LuaState, table: Val, key: &str) -> Val {
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    let key_ref = state.gc.intern_string(key.as_bytes());
    state
        .gc
        .tables
        .get(table_ref)
        .map(|t| t.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

// ---------------------------------------------------------------------------
// Frame strata parsing
// ---------------------------------------------------------------------------

pub(super) fn parse_frame_strata(strata: &str) -> Option<crate::widget::FrameStrata> {
    Some(
        if strata.eq_ignore_ascii_case("WORLD") || strata.eq_ignore_ascii_case("BACKGROUND") {
            crate::widget::FrameStrata::Background
        } else if strata.eq_ignore_ascii_case("LOW") {
            crate::widget::FrameStrata::Low
        } else if strata.eq_ignore_ascii_case("MEDIUM") {
            crate::widget::FrameStrata::Medium
        } else if strata.eq_ignore_ascii_case("HIGH") {
            crate::widget::FrameStrata::High
        } else if strata.eq_ignore_ascii_case("DIALOG") {
            crate::widget::FrameStrata::Dialog
        } else if strata.eq_ignore_ascii_case("FULLSCREEN") {
            crate::widget::FrameStrata::Fullscreen
        } else if strata.eq_ignore_ascii_case("FULLSCREEN_DIALOG") {
            crate::widget::FrameStrata::FullscreenDialog
        } else if strata.eq_ignore_ascii_case("TOOLTIP") {
            crate::widget::FrameStrata::Tooltip
        } else {
            return None;
        },
    )
}

// ---------------------------------------------------------------------------
// Mixin helpers
// ---------------------------------------------------------------------------

pub(crate) fn apply_frame_mixins(
    state: &mut LuaState,
    frame_id: u64,
    mixins: Option<&str>,
) -> LuaResult<()> {
    let Some(mixins) = mixins else {
        return Ok(());
    };

    for mixin_name in mixins
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        apply_frame_mixin(state, frame_id, mixin_name, None)?;
    }
    Ok(())
}

pub(crate) fn apply_frame_mixin(
    state: &mut LuaState,
    frame_id: u64,
    mixin_name: &str,
    source: Option<&str>,
) -> LuaResult<()> {
    apply_frame_mixin_with_partitions(state, frame_id, mixin_name, source, None, None, false)
}

pub(crate) fn apply_frame_mixin_with_partitions(
    state: &mut LuaState,
    frame_id: u64,
    mixin_name: &str,
    source: Option<&str>,
    target_partition: Option<&str>,
    inbound_partition: Option<&str>,
    secure_delegates: bool,
) -> LuaResult<()> {
    let mixin_val = match source {
        Some("secure") => resolve_secure_or_scoped_global_path(state, mixin_name),
        Some("local") => resolve_local_template_path(state, mixin_name),
        _ => resolve_default_mixin_path(state, mixin_name),
    };
    if source == Some("secure")
        || target_partition.is_some()
        || inbound_partition.is_some()
        || secure_delegates
    {
        apply_xml_mixin_helper(
            state,
            frame_id,
            mixin_val,
            target_partition,
            inbound_partition,
            secure_delegates,
        );
        return Ok(());
    }

    let mixin_exists = matches!(mixin_val, Val::Table(_));
    copy_table_into_frame(state, frame_id, mixin_val);
    if mixin_exists && mixin_name == "EditModeSystemMixin" {
        seed_edit_mode_base_method_aliases(state, frame_id)?;
    }
    Ok(())
}

fn seed_edit_mode_base_method_aliases(state: &mut LuaState, frame_id: u64) -> LuaResult<()> {
    let frame = frame_ref(state, frame_id)?;
    for (method_name, alias_name) in [
        ("SetScale", "SetScaleBase"),
        ("SetPoint", "SetPointBase"),
        ("ClearAllPoints", "ClearAllPointsBase"),
        ("SetShown", "SetShownBase"),
        ("Show", "ShowBase"),
        ("Hide", "HideBase"),
        ("IsShown", "IsShownBase"),
    ] {
        let method_key = create_string(state, method_name);
        let method = state.gettable(frame, method_key)?;
        if !matches!(method, Val::Function(_)) {
            return Err(rilua::runtime_error(format!(
                "EditModeSystemMixin requires callable frame method {method_name}"
            )));
        }
        table_set(state, frame, alias_name, method);
    }
    Ok(())
}

fn apply_xml_mixin_helper(
    state: &mut LuaState,
    frame_id: u64,
    mixin_val: Val,
    target_partition: Option<&str>,
    inbound_partition: Option<&str>,
    secure_delegates: bool,
) {
    let helper = resolve_global_path(state, "__wow_apply_xml_mixin");
    let Ok(frame) = frame_ref(state, frame_id) else {
        return;
    };
    let args = [
        frame,
        mixin_val,
        optional_string_val(state, target_partition),
        optional_string_val(state, inbound_partition),
        Val::Bool(secure_delegates),
    ];
    let _ = call_function_state(state, helper, &args);
}

fn optional_string_val(state: &mut LuaState, value: Option<&str>) -> Val {
    value
        .map(|value| create_string(state, value))
        .unwrap_or(Val::Nil)
}

fn resolve_default_mixin_path(state: &mut LuaState, path: &str) -> Val {
    let addon_local = resolve_local_template_path(state, path);
    if addon_local != Val::Nil {
        return addon_local;
    }

    let scoped_or_global = resolve_scoped_or_global_path(state, path);
    if scoped_or_global != Val::Nil {
        return scoped_or_global;
    }

    let secureenv = registry_get(state, "__secureenv");
    resolve_table_path(state, secureenv, path)
}

fn resolve_scoped_or_global_path(state: &mut LuaState, path: &str) -> Val {
    let scoped_env = {
        borrow_state(state)
            .ok()
            .and_then(|sim| sim.loading_scoped_script_env)
    };
    if let Some(scoped_env) = scoped_env {
        let current = resolve_table_path(state, scoped_env, path);
        if current != Val::Nil {
            return current;
        }
    }
    resolve_global_path(state, path)
}

pub(super) fn resolve_global_path(state: &mut LuaState, path: &str) -> Val {
    let current = resolve_table_path(state, Val::Table(state.global), path);
    if current != Val::Nil {
        return current;
    }
    let secureenv = registry_get(state, "__secureenv");
    resolve_table_path(state, secureenv, path)
}

fn resolve_local_template_path(state: &mut LuaState, path: &str) -> Val {
    let globals = Val::Table(state.global);
    let local_source =
        crate::lua_api::methods::table_get_static(state, globals, "__wow_loading_addon_table");
    resolve_table_path(state, local_source, path)
}

fn resolve_secure_or_scoped_global_path(state: &mut LuaState, path: &str) -> Val {
    let secureenv = registry_get(state, "__secureenv");
    let secure_value = resolve_table_path(state, secureenv, path);
    if secure_value != Val::Nil {
        return secure_value;
    }
    resolve_scoped_or_global_path(state, path)
}

fn resolve_table_path(state: &mut LuaState, root: Val, path: &str) -> Val {
    let mut current = root;
    for segment in path
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
    {
        let Val::Table(table_ref) = current else {
            return Val::Nil;
        };
        let key = state.gc.intern_string(segment.as_bytes());
        current = state
            .gc
            .tables
            .get(table_ref)
            .map(|table| table.get_str(key, &state.gc.string_arena))
            .unwrap_or(Val::Nil);
    }
    current
}

fn copy_table_into_frame(state: &mut LuaState, frame_id: u64, source: Val) {
    let Val::Table(source_ref) = source else {
        return;
    };
    let frame = frame_ref(state, frame_id).ok();
    let Some(Val::Table(frame_ref_val)) = frame else {
        return;
    };

    copy_table_entries_into_frame(state, frame_ref_val, source_ref);
    let index_key = hot_metatable_key(state, metatable_idx::INDEX);
    let index_table = state
        .gc
        .tables
        .get(source_ref)
        .and_then(|table| table.metatable())
        .and_then(|mt_ref| state.gc.tables.get(mt_ref))
        .map(|mt| mt.get_str(index_key, &state.gc.string_arena))
        .and_then(|value| match value {
            Val::Table(table_ref) => Some(table_ref),
            _ => None,
        });
    if let Some(index_ref) = index_table {
        copy_table_entries_into_frame(state, frame_ref_val, index_ref);
    }
}

fn copy_table_entries_into_frame(
    state: &mut LuaState,
    frame_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    source_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) {
    let array_values = state
        .gc
        .tables
        .get(source_ref)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();
    let hash_entries = collect_filtered_hash_entries(state, source_ref);
    if let Some(fields_table) = state.gc.tables.get_mut(frame_ref) {
        for (index, value) in array_values.into_iter().enumerate() {
            let _ =
                fields_table.raw_set(Val::Num((index + 1) as f64), value, &state.gc.string_arena);
        }
        for (key, value) in hash_entries {
            let _ = fields_table.raw_set(key, value, &state.gc.string_arena);
        }
    }
    state.gc.barrier_back(frame_ref);
}

fn collect_filtered_hash_entries(
    state: &LuaState,
    source_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> Vec<(Val, Val)> {
    state
        .gc
        .tables
        .get(source_ref)
        .map(|table| table.hash_entries())
        .unwrap_or_default()
        .into_iter()
        .filter(|(key, _)| {
            let Val::Str(str_ref) = key else { return true };
            !state
                .gc
                .string_arena
                .get(*str_ref)
                .and_then(|name| name.as_str())
                .is_some_and(|s| {
                    matches!(
                        s,
                        "RegisterCallback" | "UnregisterCallback" | "TriggerEvent"
                    )
                })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Parent array helpers
// ---------------------------------------------------------------------------

pub(crate) fn append_parent_array_entry(
    state: &mut LuaState,
    parent_id: u64,
    key: &str,
    child_id: u64,
) {
    let Ok(parent) = frame_ref(state, parent_id) else {
        return;
    };
    let Ok(child) = frame_ref(state, child_id) else {
        return;
    };
    let array = match table_get(state, parent, key) {
        Val::Table(existing) => Val::Table(existing),
        _ => {
            let created = create_table(state);
            table_set(state, parent, key, created);
            created
        }
    };
    let Val::Table(array_ref) = array else {
        return;
    };
    normalize_parent_array_entries(state, array_ref);
    if parent_array_contains_child(state, array_ref, child, child_id) {
        return;
    }
    if replace_stale_named_parent_array_entries(state, array_ref, child_id, child) {
        return;
    }
    append_parent_array_value(state, array_ref, child);
}

fn parent_array_contains_child(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    child: Val,
    child_id: u64,
) -> bool {
    let entries = state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();
    entries
        .into_iter()
        .any(|entry| entries_conflict(state, entry, child, Some(child_id)))
}

fn normalize_parent_array_entries(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) {
    let existing_entries = state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();
    let existing_len = existing_entries.len();
    let mut normalized = Vec::with_capacity(existing_entries.len());

    for entry in existing_entries {
        if normalized
            .iter()
            .copied()
            .any(|existing| entries_conflict(state, existing, entry, None))
        {
            continue;
        }
        normalized.push(entry);
    }

    if normalized.len() != existing_len {
        write_parent_array_entries(state, table_ref, &normalized);
    }
}

fn replace_stale_named_parent_array_entries(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    child_id: u64,
    child: Val,
) -> bool {
    let Some(child_name) = frame_name_for_id(state, child_id) else {
        return false;
    };
    let existing_entries = state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();
    let mut rewritten_entries = Vec::with_capacity(existing_entries.len());
    let mut replaced = false;

    for entry in existing_entries {
        if parent_array_entry_name(state, entry).as_deref() == Some(child_name.as_str()) {
            if !replaced {
                rewritten_entries.push(child);
                replaced = true;
            }
            continue;
        }
        rewritten_entries.push(entry);
    }

    if !replaced {
        return false;
    }

    write_parent_array_entries(state, table_ref, &rewritten_entries);
    true
}

fn frame_name_for_id(state: &LuaState, frame_id: u64) -> Option<String> {
    borrow_state(state).ok().and_then(|sim| {
        sim.widgets
            .get(frame_id)
            .and_then(|frame| frame.name.clone())
    })
}

fn parent_array_entry_name(state: &mut LuaState, entry: Val) -> Option<String> {
    let frame_id = extract_frame_id(state, entry)?;
    frame_name_for_id(state, frame_id)
}

fn entries_conflict(
    state: &mut LuaState,
    existing: Val,
    candidate: Val,
    candidate_id: Option<u64>,
) -> bool {
    if existing == candidate {
        return true;
    }
    let existing_id = extract_frame_id(state, existing);
    let candidate_id = candidate_id.or_else(|| extract_frame_id(state, candidate));
    if existing_id.is_some() && existing_id == candidate_id {
        return true;
    }
    let existing_name = parent_array_entry_name(state, existing);
    let candidate_name = parent_array_entry_name(state, candidate);
    existing_name.is_some() && existing_name == candidate_name
}

fn write_parent_array_entries(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    entries: &[Val],
) {
    let existing_len = state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.array_slice().len())
        .unwrap_or(0);
    let cleared_len = existing_len.max(entries.len());

    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        for (index, entry) in entries.iter().copied().enumerate() {
            let _ = table.raw_set(Val::Num((index + 1) as f64), entry, &state.gc.string_arena);
        }
        for index in entries.len()..cleared_len {
            let _ = table.raw_set(
                Val::Num((index + 1) as f64),
                Val::Nil,
                &state.gc.string_arena,
            );
        }
    }
    state.gc.barrier_back(table_ref);
}

fn append_parent_array_value(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    child: Val,
) {
    let next_index = next_table_array_index(state, table_ref);
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Num(next_index as f64), child, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}

fn next_table_array_index(
    state: &LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> i64 {
    let mut index = 1_i64;
    while state
        .gc
        .tables
        .get(table_ref)
        .map(|table| !matches!(table.get_int(index), Val::Nil))
        .unwrap_or(false)
    {
        index += 1;
    }
    index
}
