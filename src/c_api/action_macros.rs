//! Macro action associations; directive recognition is a simulator assumption.
#[cfg(feature = "retail-12-1-5")]
use crate::lua_api::methods::borrow_state;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::SimState;
use crate::lua_bridge::FromStack;
#[cfg(feature = "retail-12-1-5")]
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

pub fn clear_slot(sim: &mut SimState, slot: u32) {
    sim.action_bars.remove(&slot);
    sim.action_outfits.remove(&slot);
    sim.action_macros.remove(&slot);
    sim.equipped_gear_outfit_action_slots.remove(&slot);
}

pub fn assign_macro(sim: &mut SimState, slot: u32, id: u32) -> LuaResult<()> {
    let exists = id
        .checked_sub(1)
        .and_then(|index| sim.macros.get(index as usize))
        .is_some_and(|entry| !entry.name.is_empty());
    if slot == 0 || !exists {
        return Err(runtime_error(
            "macro action requires a positive slot and existing macro",
        ));
    }
    clear_slot(sim, slot);
    sim.action_macros.insert(slot, id);
    Ok(())
}

pub fn set_macro_action_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let id = u32::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    assign_macro(&mut sim, slot, id)?;
    Ok(0)
}

#[cfg(feature = "retail-12-1-5")]
fn has_showtooltip(body: &str) -> bool {
    body.lines().any(|line| {
        let line = line.trim_start_matches([' ', '\t']);
        line.strip_prefix("#showtooltip")
            .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
    })
}

#[cfg(feature = "retail-12-1-5")]
fn is_macro_action_with_showtooltip(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    if slot == 0 {
        return Err(runtime_error("action slot must be positive"));
    }
    let result = {
        let sim = borrow_state(state)?;
        sim.action_macros
            .get(&slot)
            .and_then(|id| id.checked_sub(1))
            .and_then(|index| sim.macros.get(index as usize))
            .is_some_and(|entry| has_showtooltip(&entry.body))
    };
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    #[cfg(feature = "retail-12-1-5")]
    table_set_rust_fn_static(
        state,
        namespace,
        "IsMacroActionWithShowTooltip",
        is_macro_action_with_showtooltip,
    )?;
    #[cfg(not(feature = "retail-12-1-5"))]
    {
        use crate::lua_api::methods::{create_table, table_get, table_set};
        let table = Val::Table(namespace);
        let existing = table_get(state, table, "__wow_removed_keys");
        let removed = if matches!(existing, Val::Table(_)) {
            existing
        } else {
            create_table(state)
        };
        table_set(
            state,
            removed,
            "IsMacroActionWithShowTooltip",
            Val::Bool(true),
        );
        table_set(state, table, "__wow_removed_keys", removed);
    }
    Ok(())
}
