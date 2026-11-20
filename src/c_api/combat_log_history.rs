//! INFERRED Retail/Forever combat-log history model.
//!
//! It preserves the temporary fixture's Lua-visible traversal and return shapes.
//! Host code supplies history explicitly; the simulator invents no combat events.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::api::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

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
    let namespace = super::ensure_namespace(state, "C_CombatLogSecure")?;
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
    )
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
