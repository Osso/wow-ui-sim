//! WM92 PTR 70668 SpellRangeVisualization inputs and local stores.
//! The pinned PTR UI is 70493; this is not same-build native evidence.

mod unknown_policy;

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state_mut, table_get, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

/// A supplied action record and its optional successful visualization lookup.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionVisualizationRecord {
    pub identity: String,
    pub visualization_value: Option<u32>,
}

/// Empty inputs mean unavailable records/installed object, not native defaults.
/// Events, tooltip ownership, combat and taint policy remain UNKNOWN; no extra
/// effects are modeled beyond the recovered local stores.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpellRangeVisualizationState {
    pub installed_object_identity: Option<String>,
    pub records: HashMap<u32, ActionVisualizationRecord>,
    /// Native converted slot, zero-based. Leave preserves it.
    pub stored_slot: Option<u32>,
    pub value: u32,
    pub enabled: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Ptr {
        return Ok(());
    }
    // Pinned PTR ActionBarFrameDocumentation.lua:35–44,960–969.
    let namespace = super::helpers::ensure_namespace(state, "C_ActionBar")?;
    table_set_rust_fn_static(state, namespace, "EnterActionButton", enter)?;
    table_set_rust_fn_static(state, namespace, "LeaveActionButton", leave)?;
    // Native-only enum names/values from WM92 PTR 70668; the pinned original
    // ActionButton.lua:941,951 uses ActionCombatSpellHitIndicationEnabled.
    let enums = super::helpers::ensure_global_table(state, "Enum");
    let rules = table_get(state, enums, "GameRule");
    table_set(
        state,
        rules,
        "ActionCombatSpellHitIndicationEnabled",
        Val::Num(204.0),
    );
    table_set(state, rules, "CraftFromBankRestricted", Val::Num(211.0));
    Ok(())
}

fn argument_error(function: &str, reason: &str) -> rilua::LuaError {
    rilua::runtime_error(format!("bad argument #1 to '{function}' ({reason})"))
}

/// Subtract before truncation: 0 is unusable, but 0.25 selects converted slot 0.
fn read_slot(state: &LuaState, function: &str) -> LuaResult<Option<u32>> {
    let usage = format!("Usage: C_ActionBar.{function}(actionID)");
    if matches!(stack_val(state, 1), Val::Nil) {
        return Err(argument_error(function, &usage));
    }
    let value = unknown_policy::read_accessible_argument(state, &usage)?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| super::helpers::parse_numeric_string(string.data())),
        _ => None,
    };
    let number = number
        .filter(|n| n.is_finite() && (0.0..=f64::from(u32::MAX)).contains(n))
        .ok_or_else(|| {
            argument_error(
                function,
                &format!("outside of expected range 0 to 4294967295 - {usage}"),
            )
        })?;
    let converted = (number - 1.0).trunc();
    // Values outside the signed conversion domain cannot select a usable slot.
    Ok((0.0..408.0)
        .contains(&converted)
        .then_some(converted as u32))
}

fn select_visualization(sim: &crate::lua_api::SimState, slot: u32) -> Option<u32> {
    if !sim
        .game_rules
        .rules
        .contains_key("ActionCombatSpellHitIndicationEnabled")
    {
        return None;
    }
    let spell = sim.action_bars.get(&(slot + 1))?;
    let inputs = &sim.spell_range_visualization;
    let record = inputs.records.get(spell)?;
    let installed_identity = inputs.installed_object_identity.as_ref()?;
    if installed_identity == &record.identity {
        return None;
    }
    record.visualization_value
}

fn enter(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = read_slot(state, "EnterActionButton")? else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(value) = select_visualization(&sim, slot) {
        let visualization = &mut sim.spell_range_visualization;
        visualization.stored_slot = Some(slot);
        visualization.value = value;
        visualization.enabled = true;
    }
    Ok(0)
}

fn leave(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = read_slot(state, "LeaveActionButton")? else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    let visualization = &mut sim.spell_range_visualization;
    if visualization.stored_slot == Some(slot) {
        visualization.value = 0;
        visualization.enabled = false;
    }
    Ok(0)
}
