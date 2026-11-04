//! WM93 PTR 70668 counted action-combat channel; pinned UI is 70493.

mod unknown_policy;

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;
use std::collections::HashMap;

/// Explicit simulator input, not a computed geometric or target-selection result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SuppliedCombatRangeResult {
    pub in_range: bool,
    pub available: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatRangeRegistration {
    pub references: i32,
    pub cached_result: Option<SuppliedCombatRangeResult>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionCombatRangeState {
    /// Native converted keys, including -1 and 408 for ordinary slots 0 and 409.
    pub registrations: HashMap<i32, CombatRangeRegistration>,
    /// No snapshot means no supplied evaluation. Never invent a range Boolean.
    pub supplied_results: HashMap<i32, SuppliedCombatRangeResult>,
}

const FUNCTION: &str = "EnableActionCombatActionRangeCheck";
const USAGE: &str = "Usage: C_ActionBar.EnableActionCombatActionRangeCheck(actionID, enable)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Ptr {
        return Ok(());
    }
    // Pinned PTR ActionBarFrameDocumentation.lua:11–20,1088–1100.
    let namespace = super::helpers::ensure_namespace(state, "C_ActionBar")?;
    table_set_rust_fn_static(state, namespace, FUNCTION, update_registration)?;
    // Both observer/monitor adapters are installed here. Native installation and
    // lifetime are UNKNOWN; this explicit seam supplies no automatic schedule.
    let admin = super::helpers::ensure_namespace(state, "A_Admin")?;
    table_set_rust_fn_static(
        state,
        admin,
        "SweepActionCombatRangeChecks",
        unknown_policy::sweep_supplied_results,
    )?;
    Ok(())
}

fn update_registration(state: &mut LuaState) -> LuaResult<u32> {
    let key = super::action_button_hover::read_converted_slot(state, FUNCTION, USAGE)?;
    let enable = unknown_policy::read_enable(state, FUNCTION, USAGE)?;
    let mut sim = borrow_state_mut(state)?;
    if !sim
        .game_rules
        .rules
        .contains_key("ActionCombatSpellHitIndicationEnabled")
    {
        unknown_policy::report_native_diagnostic(
            "Enabling Action Combat range checking requires the ActionCombatSpellHitIndicationEnabled game rule to be enabled",
        );
        return Ok(0);
    }
    let registrations = &mut sim.action_combat_range.registrations;
    let previous = registrations.get(&key).map_or(0, |entry| entry.references);
    let references = previous.wrapping_add(if enable { 1 } else { -1 });
    if references < 0 {
        unknown_policy::report_native_diagnostic("EnableRangeCheck underflow.");
        return Ok(0);
    }
    if references == 0 {
        registrations.remove(&key);
    } else {
        registrations
            .entry(key)
            .or_insert(CombatRangeRegistration {
                references: 0,
                cached_result: None,
            })
            .references = references;
    }
    Ok(0)
}

fn collect_updates(
    channel: &mut ActionCombatRangeState,
    force: bool,
) -> Vec<(i32, SuppliedCombatRangeResult)> {
    let mut updates = Vec::new();
    for (&key, registration) in &mut channel.registrations {
        let Some(mut result) =
            unknown_policy::select_supplied_result(&channel.supplied_results, key)
        else {
            continue;
        };
        if !result.available {
            if let Some(previous) = registration.cached_result {
                result.in_range = previous.in_range;
            }
        }
        if force || registration.cached_result != Some(result) {
            updates.push((key, result));
        }
        registration.cached_result = Some(result);
    }
    updates
}
