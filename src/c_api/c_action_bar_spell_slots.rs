//! INFERRED direct-spell slot queries over existing assignments, not native parity.

use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

#[cfg(feature = "retail-12-0-5")]
pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "FindAssistedCombatActionButtons",
        find_assisted_combat_action_buttons,
    )?;
    register_find_spell_action_buttons(state, namespace)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "HasSpellActionButtons",
        has_spell_action_buttons,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsOnBarOrSpecialBar",
        is_on_bar_or_special_bar,
    )
}

pub(crate) fn register_find_spell_action_buttons(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    // Forever declares this query; its action-bar-set filtering remains unmodeled.
    table_set_rust_fn_static(
        state,
        namespace,
        "FindSpellActionButtons",
        find_spell_action_buttons,
    )
}

fn effective_spell_slots(sim: &SimState, spell_id: u32) -> impl Iterator<Item = u32> + '_ {
    sim.action_bars
        .iter()
        .filter_map(move |(&slot, &assigned)| {
            // GetActionInfo gives outfits and macros priority over direct spells.
            (slot > 0
                && assigned == spell_id
                && !sim.action_outfits.contains_key(&slot)
                && !sim.action_macros.contains_key(&slot))
            .then_some(slot)
        })
}

fn find_spell_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = super::c_spell::read_public_spell_identifier_at(
        state,
        1,
        "C_ActionBar.FindSpellActionButtons",
    )?;
    let slots = match spell_id {
        Some(id) => {
            let sim = borrow_state(state)?;
            collect_sorted_spell_slots(&sim, id)
        }
        None => Vec::new(),
    };
    // One empty table on miss is an inferred direct-query policy.
    let table = create_table(state);
    state.push(table);
    for (index, slot) in slots.into_iter().enumerate() {
        super::helpers::set_table_array(state, table, (index + 1) as i64, Val::Num(slot as f64));
    }
    Ok(1)
}

fn collect_sorted_spell_slots(sim: &SimState, spell_id: u32) -> Vec<u32> {
    let mut slots: Vec<_> = effective_spell_slots(sim, spell_id).collect();
    // Deterministic ascending order is simulator policy, not native ordering.
    slots.sort_unstable();
    slots
}

#[cfg(feature = "retail-12-0-5")]
fn find_assisted_combat_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let slots = {
        let sim = borrow_state(state)?;
        sim.assisted_combat
            .action_spell_id
            .map(|id| collect_sorted_spell_slots(&sim, id))
    };
    let Some(slots) = slots.filter(|slots| !slots.is_empty()) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = create_table(state);
    state.push(table);
    for (index, slot) in slots.into_iter().enumerate() {
        super::helpers::set_table_array(
            state,
            table,
            (index + 1) as i64,
            Val::Num(f64::from(slot)),
        );
    }
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn has_spell_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let has_slots =
        query_public_direct_spell_membership(state, "C_ActionBar.HasSpellActionButtons")?;
    state.push(Val::Bool(has_slots));
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn is_on_bar_or_special_bar(state: &mut LuaState) -> LuaResult<u32> {
    // AllowedWhenTainted is NOT AllowedWhenUntainted: retain the bounded public reader.
    // Native secret-input permission/result secrecy remain unmodeled.
    let spell_id = super::c_spell::read_public_spell_identifier_at(
        state,
        1,
        "C_ActionBar.IsOnBarOrSpecialBar",
    )?;
    let is_on_bar = match spell_id {
        Some(id) => {
            let sim = borrow_state(state)?;
            // INFERRED union of effective direct slots and explicit special membership.
            effective_spell_slots(&sim, id).next().is_some() || sim.special_bar_spells.contains(&id)
        }
        None => false,
    };
    state.push(Val::Bool(is_on_bar));
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn query_public_direct_spell_membership(state: &LuaState, api_name: &str) -> LuaResult<bool> {
    let spell_id = super::c_spell::read_public_spell_identifier_at(state, 1, api_name)?;
    match spell_id {
        Some(id) => {
            let sim = borrow_state(state)?;
            let has_slots = effective_spell_slots(&sim, id).next().is_some();
            Ok(has_slots)
        }
        None => Ok(false),
    }
}
