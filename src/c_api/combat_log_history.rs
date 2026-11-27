//! INFERRED Retail/Forever combat-log history model.
//!
//! It preserves the temporary fixture's Lua-visible traversal, settings state and return shapes.
//! Host code supplies history explicitly; the simulator invents no combat events.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, val_to_string};
use crate::lua_bridge::TableBuilder;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::api::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Debug, PartialEq)]
pub enum CombatLogValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatLogEntry {
    pub values: Vec<CombatLogValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatLogHistory {
    pub entries: Vec<CombatLogEntry>,
    pub current_index: Option<usize>,
    pub current_entry: i64,
    pub retention_time: f64,
    pub filtered_events_enabled: bool,
    pub filter_settings: Option<Val>,
    pub message_limit: i64,
}

impl Default for CombatLogHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current_index: None,
            current_entry: 0,
            retention_time: 300.0,
            filtered_events_enabled: false,
            filter_settings: None,
            message_limit: 300,
        }
    }
}

impl CombatLogHistory {
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    fn current_entry_values(&self) -> Option<&[CombatLogValue]> {
        let newest = self.entries.len().checked_sub(1)?;
        let index = self
            .current_index
            .filter(|index| *index <= newest)
            .unwrap_or(newest);
        Some(&self.entries[index].values)
    }

    fn seek_newest(&mut self) -> bool {
        let Some(index) = self.entries.len().checked_sub(1) else {
            self.current_index = None;
            return false;
        };
        self.current_index = Some(index);
        true
    }

    fn seek_previous(&mut self) -> bool {
        let Some(newest) = self.entries.len().checked_sub(1) else {
            return false;
        };
        let index = self
            .current_index
            .filter(|index| *index <= newest)
            .unwrap_or(newest);
        let Some(previous) = index.checked_sub(1) else {
            return false;
        };
        self.current_index = Some(previous);
        true
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.current_index = None;
        self.current_entry = 0;
    }
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }
    register_combat_log_namespace(state)?;
    register_secure_namespace(state)?;
    register_internal_namespace(state)?;
    register_lua_helpers(state)
}

fn register_combat_log_namespace(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_CombatLog")?;
    table_set_rust_fn_static(state, namespace, "GetEntryCount", entry_count)?;
    table_set_rust_fn_static(state, namespace, "GetCurrentEntryInfo", current_entry_index)?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        table_set_rust_fn_static(state, namespace, "GetCurrentEventInfo", current_event_info)?;
    }
    table_set_rust_fn_static(
        state,
        namespace,
        "ShouldShowCurrentEntry",
        should_show_current_entry,
    )?;
    table_set_rust_fn_static(state, namespace, "GetEntryRetentionTime", retention_time)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "AreFilteredEventsEnabled",
        filtered_events_enabled,
    )?;
    table_set_rust_fn_static(state, namespace, "GetMessageLimit", message_limit)
}

fn register_secure_namespace(state: &mut LuaState) -> LuaResult<()> {
    // Retail publishes SecureOnly methods inside __secureenv; Forever uses its global namespace.
    let namespace = if crate::client_profile::is_active_retail_12_1_5() {
        super::ensure_secure_namespace(state, "C_CombatLogSecure")?
    } else {
        super::ensure_namespace(state, "C_CombatLogSecure")?
    };
    table_set_rust_fn_static(state, namespace, "GetEntryCount", entry_count)?;
    table_set_rust_fn_static(state, namespace, "GetCurrentEntryInfo", current_event_info)?;
    table_set_rust_fn_static(state, namespace, "GetCurrentEventInfo", current_event_info)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "ShouldShowCurrentEntry",
        should_show_current_entry,
    )?;
    table_set_rust_fn_static(state, namespace, "SeekToNewestEntry", seek_newest)?;
    table_set_rust_fn_static(state, namespace, "SeekToPreviousEntry", seek_previous)
}

fn register_internal_namespace(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_CombatLogInternal")?;
    table_set_rust_fn_static(state, namespace, "GetCurrentEventInfo", current_event_info)
}

fn register_lua_helpers(state: &mut LuaState) -> LuaResult<()> {
    LuaApiMut::register_function(state, "__wow_combat_log_clear_entries", clear_entries)?;
    LuaApiMut::register_function(
        state,
        "__wow_combat_log_set_current_entry",
        set_current_entry,
    )?;
    LuaApiMut::register_function(state, "__wow_combat_log_advance_entry", advance_entry)?;
    LuaApiMut::register_function(
        state,
        "__wow_combat_log_set_retention_time",
        set_retention_time,
    )?;
    LuaApiMut::register_function(
        state,
        "__wow_combat_log_set_filtered_events_enabled",
        set_filtered_events_enabled,
    )?;
    LuaApiMut::register_function(
        state,
        "__wow_combat_log_set_message_limit",
        set_message_limit,
    )?;
    LuaApiMut::register_function(
        state,
        "__wow_combat_log_set_filter_settings",
        set_filter_settings,
    )?;
    Ok(())
}

fn entry_count(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.combat_log_history.entry_count();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn current_entry_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = borrow_state(state)?.combat_log_history.current_entry;
    state.push(Val::Num(index as f64));
    Ok(1)
}

fn current_event_info(state: &mut LuaState) -> LuaResult<u32> {
    let values = borrow_state(state)?
        .combat_log_history
        .current_entry_values()
        .map(<[_]>::to_vec);
    let Some(values) = values else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let count = values.len() as u32;
    for value in values {
        push_value(state, value);
    }
    Ok(count)
}

fn push_value(state: &mut LuaState, value: CombatLogValue) {
    let value = match value {
        CombatLogValue::Nil => Val::Nil,
        CombatLogValue::Boolean(value) => Val::Bool(value),
        CombatLogValue::Number(value) => Val::Num(value),
        CombatLogValue::String(value) => create_string(state, &value),
    };
    state.push(value);
}

fn should_show_current_entry(state: &mut LuaState) -> LuaResult<u32> {
    let visible = borrow_state(state)?.combat_log_history.entry_count() > 0;
    state.push(Val::Bool(visible));
    Ok(1)
}

fn seek_newest(state: &mut LuaState) -> LuaResult<u32> {
    let success = borrow_state_mut(state)?.combat_log_history.seek_newest();
    state.push(Val::Bool(success));
    Ok(1)
}

fn seek_previous(state: &mut LuaState) -> LuaResult<u32> {
    let success = borrow_state_mut(state)?.combat_log_history.seek_previous();
    state.push(Val::Bool(success));
    Ok(1)
}

fn retention_time(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.combat_log_history.retention_time;
    state.push(Val::Num(value));
    Ok(1)
}

fn filtered_events_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .combat_log_history
        .filtered_events_enabled;
    state.push(Val::Bool(value));
    Ok(1)
}

fn message_limit(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.combat_log_history.message_limit;
    state.push(Val::Num(value as f64));
    Ok(1)
}

fn clear_entries(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.combat_log_history.clear();
    Ok(0)
}

fn set_current_entry(state: &mut LuaState) -> LuaResult<u32> {
    let value = i64::from_stack(state, 1)?.max(0);
    borrow_state_mut(state)?.combat_log_history.current_entry = value;
    Ok(0)
}

fn advance_entry(state: &mut LuaState) -> LuaResult<u32> {
    let step = i64::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.combat_log_history.current_entry = sim
        .combat_log_history
        .current_entry
        .saturating_add(step)
        .max(0);
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}

fn set_retention_time(state: &mut LuaState) -> LuaResult<u32> {
    let value = f64::from_stack(state, 1)?;
    borrow_state_mut(state)?.combat_log_history.retention_time = value;
    Ok(0)
}

fn set_filtered_events_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let value = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .combat_log_history
        .filtered_events_enabled = value;
    Ok(0)
}

fn set_message_limit(state: &mut LuaState) -> LuaResult<u32> {
    let value = i64::from_stack(state, 1)?;
    borrow_state_mut(state)?.combat_log_history.message_limit = value;
    Ok(0)
}

fn set_filter_settings(state: &mut LuaState) -> LuaResult<u32> {
    let value = Val::from_stack(state, 1)?;
    borrow_state_mut(state)?.combat_log_history.filter_settings = Some(value);
    Ok(0)
}

/// `A_Admin.SetCombatLogEntries(entries)`: replace the host-supplied history.
/// Each entry is an array of the values `GetCurrentEventInfo` returns, oldest
/// first. Only boolean, number and string values are accepted.
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetCombatLogEntries", set_combat_log_entries)
}

fn set_combat_log_entries(state: &mut LuaState) -> LuaResult<u32> {
    let rows = read_admin_array(state, crate::lua_bridge::stack_val(state, 1), "entries")?;
    let mut entries = Vec::with_capacity(rows.len());
    for (index, row) in rows.into_iter().enumerate() {
        let label = format!("entries[{}]", index + 1);
        let values = read_admin_array(state, row, &label)?
            .into_iter()
            .map(|value| admin_value(state, value, &label))
            .collect::<LuaResult<Vec<_>>>()?;
        entries.push(CombatLogEntry { values });
    }
    let mut sim = borrow_state_mut(state)?;
    sim.combat_log_history.entries = entries;
    sim.combat_log_history.current_index = None;
    Ok(0)
}

fn read_admin_array(state: &LuaState, value: Val, label: &str) -> LuaResult<Vec<Val>> {
    let error = || {
        runtime_error(format!(
            "A_Admin.SetCombatLogEntries: {label} must be an array"
        ))
    };
    let Val::Table(reference) = value else {
        return Err(error());
    };
    let table = state.gc.tables.get(reference).ok_or_else(error)?;
    let len = table.len(&state.gc.string_arena);
    Ok((1..=len).map(|index| table.get_int(index as i64)).collect())
}

fn admin_value(state: &LuaState, value: Val, label: &str) -> LuaResult<CombatLogValue> {
    match value {
        Val::Bool(value) => Ok(CombatLogValue::Boolean(value)),
        Val::Num(value) if value.is_finite() => Ok(CombatLogValue::Number(value)),
        Val::Str(_) => Ok(CombatLogValue::String(
            val_to_string(state, value).unwrap_or_default(),
        )),
        _ => Err(runtime_error(format!(
            "A_Admin.SetCombatLogEntries: {label} values must be booleans, finite numbers or strings"
        ))),
    }
}
