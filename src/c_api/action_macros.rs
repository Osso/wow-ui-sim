//! Macro action associations; directive recognition is a simulator assumption.
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
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

// Inferred label rule: only an occupied macro action supplies text, regardless
// of body/directives. Empty names denote unused macro entries in this model.
fn read_action_text(state: &LuaState) -> LuaResult<Option<String>> {
    let slot = match stack_val(state, 1) {
        Val::Num(number) if number >= 0.0 => number as u32,
        _ => return Ok(None),
    };
    let sim = borrow_state(state)?;
    let name = sim
        .action_macros
        .get(&slot)
        .and_then(|id| id.checked_sub(1))
        .and_then(|index| sim.macros.get(index as usize))
        .map(|entry| &entry.name);
    Ok(name.filter(|name| !name.is_empty()).cloned())
}

fn uses_action_text(state: &mut LuaState) -> LuaResult<u32> {
    let uses_text = read_action_text(state)?.is_some();
    state.push(Val::Bool(uses_text));
    Ok(1)
}

fn get_action_text(state: &mut LuaState) -> LuaResult<u32> {
    let text = read_action_text(state)?;
    let value = match text {
        Some(name) => create_string(state, &name),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

pub fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "UsesActionText", uses_action_text)?;
    table_set_rust_fn_static(state, namespace, "GetActionText", get_action_text)?;
    #[cfg(feature = "retail-12-1-5")]
    table_set_rust_fn_static(
        state,
        namespace,
        "IsMacroActionWithShowTooltip",
        is_macro_action_with_showtooltip,
    )?;
    #[cfg(not(feature = "retail-12-1-5"))]
    super::mark_namespace_keys_removed(state, namespace, &["IsMacroActionWithShowTooltip"]);
    Ok(())
}
