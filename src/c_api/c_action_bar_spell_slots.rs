//! INFERRED direct-spell slot queries over existing assignments, not native parity.

use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, create_table};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "FindSpellActionButtons",
        find_spell_action_buttons,
    )?;
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
    let mut slots = match spell_id {
        Some(id) => {
            let sim = borrow_state(state)?;
            effective_spell_slots(&sim, id).collect::<Vec<_>>()
        }
        None => Vec::new(),
    };
    // Deterministic ascending order and one empty table on miss are inferred policies.
    slots.sort_unstable();
    let table = create_table(state);
    state.push(table);
    for (index, slot) in slots.into_iter().enumerate() {
        super::helpers::set_table_array(state, table, (index + 1) as i64, Val::Num(slot as f64));
    }
    Ok(1)
}

fn has_spell_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let has_slots =
        query_public_direct_spell_membership(state, "C_ActionBar.HasSpellActionButtons")?;
    state.push(Val::Bool(has_slots));
    Ok(1)
}

fn is_on_bar_or_special_bar(state: &mut LuaState) -> LuaResult<u32> {
    // Special-bar membership and native AllowedWhenTainted access remain unmodeled.
    let has_slots = query_public_direct_spell_membership(state, "C_ActionBar.IsOnBarOrSpecialBar")?;
    state.push(Val::Bool(has_slots));
    Ok(1)
}

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
