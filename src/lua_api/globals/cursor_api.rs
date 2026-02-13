//! Cursor/drag-and-drop API: GetCursorInfo, ClearCursor, PickupAction,
//! PlaceAction, C_Spell.PickupSpell, C_ActionBar.PutActionInSlot.

use crate::lua_api::SimState;
use crate::lua_api::state::CursorInfo;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all cursor-related global functions.
pub fn register_cursor_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let g = lua.globals();
    register_get_cursor_info(lua, &g, &state)?;
    register_clear_cursor(lua, &g, &state)?;
    register_pickup_action(lua, &g, &state)?;
    register_place_action(lua, &g, &state)?;
    register_pickup_globals(lua, &g)?;
    Ok(())
}

/// Register C_Spell.PickupSpell on an existing C_Spell table.
pub fn register_c_spell_pickup(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let g = lua.globals();
    let c_spell: mlua::Table = g.get("C_Spell")?;
    let st = Rc::clone(state);
    c_spell.set("PickupSpell", lua.create_function(move |_, spell_id: i32| {
        let spell_id = spell_id as u32;
        if crate::spells::get_spell(spell_id).is_some() {
            eprintln!("[cursor] PickupSpell({})", spell_id);
            st.borrow_mut().cursor_item = Some(CursorInfo::Spell { spell_id });
        }
        Ok(())
    })?)?;
    Ok(())
}

/// Register C_ActionBar.PutActionInSlot on an existing C_ActionBar table.
pub fn register_c_action_bar_put(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let g = lua.globals();
    let c_ab: mlua::Table = g.get("C_ActionBar")?;
    let st = Rc::clone(state);
    c_ab.set("PutActionInSlot", lua.create_function(move |lua, (action, slot): (i32, i32)| {
        put_action_in_slot(&st, lua, action as u32, slot as u32)
    })?)?;
    Ok(())
}

/// GetCursorInfo() -> type, spellID, ...
fn register_get_cursor_info(
    lua: &Lua, g: &mlua::Table, state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(state);
    g.set("GetCursorInfo", lua.create_function(move |lua, ()| {
        let s = st.borrow();
        match &s.cursor_item {
            None => Ok(mlua::MultiValue::new()),
            Some(CursorInfo::Action { spell_id, .. }) | Some(CursorInfo::Spell { spell_id }) => {
                let kind = lua.create_string("spell")?;
                Ok(mlua::MultiValue::from_vec(vec![
                    Value::String(kind),
                    Value::Integer(*spell_id as i64),
                ]))
            }
        }
    })?)?;
    Ok(())
}

/// ClearCursor() — drop whatever is on the cursor.
fn register_clear_cursor(
    lua: &Lua, g: &mlua::Table, state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(state);
    g.set("ClearCursor", lua.create_function(move |_, ()| {
        let mut s = st.borrow_mut();
        if s.cursor_item.is_some() {
            eprintln!("[cursor] ClearCursor");
            s.cursor_item = None;
        }
        Ok(())
    })?)?;
    Ok(())
}

/// PickupAction(slot) — pick up the action in the given slot.
/// If cursor already holds an action/spell, swap it into the slot.
fn register_pickup_action(
    lua: &Lua, g: &mlua::Table, state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(state);
    g.set("PickupAction", lua.create_function(move |lua, slot: i32| {
        pickup_action(&st, lua, slot as u32)
    })?)?;
    Ok(())
}

/// PlaceAction(slot) — place cursor item into the given action bar slot.
fn register_place_action(
    lua: &Lua, g: &mlua::Table, state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(state);
    g.set("PlaceAction", lua.create_function(move |lua, slot: i32| {
        place_action(&st, lua, slot as u32)
    })?)?;
    Ok(())
}

/// Stub pickup functions for types we don't handle yet.
fn register_pickup_globals(lua: &Lua, g: &mlua::Table) -> Result<()> {
    let noop = lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?;
    for name in [
        "PickupBagFromSlot", "PickupInventoryItem", "PickupMacro",
        "PickupMerchantItem", "PickupPetAction", "PickupPlayerMoney",
        "PickupCompanion",
    ] {
        g.set(name, noop.clone())?;
    }
    Ok(())
}

/// Core logic for PickupAction — handles swap if cursor already has something.
fn pickup_action(state: &Rc<RefCell<SimState>>, lua: &Lua, slot: u32) -> Result<()> {
    let old_cursor = { state.borrow_mut().cursor_item.take() };
    let old_spell = { state.borrow().action_bars.get(&slot).copied() };

    // Place whatever was on cursor into this slot
    if let Some(cursor_info) = old_cursor {
        let new_spell_id = match cursor_info {
            CursorInfo::Action { spell_id, .. } => spell_id,
            CursorInfo::Spell { spell_id } => spell_id,
        };
        state.borrow_mut().action_bars.insert(slot, new_spell_id);
        eprintln!("[cursor] PickupAction({}) — placed spell {} into slot", slot, new_spell_id);
    } else {
        // Remove spell from slot
        state.borrow_mut().action_bars.remove(&slot);
        eprintln!("[cursor] PickupAction({}) — removed from slot", slot);
    }

    // Put old slot contents on cursor
    if let Some(spell_id) = old_spell {
        state.borrow_mut().cursor_item = Some(CursorInfo::Action { slot, spell_id });
    }

    fire_action_bar_updates(state, lua)?;
    Ok(())
}

/// Core logic for PlaceAction — drop cursor item into slot.
fn place_action(state: &Rc<RefCell<SimState>>, lua: &Lua, slot: u32) -> Result<()> {
    let cursor = { state.borrow_mut().cursor_item.take() };
    let Some(cursor_info) = cursor else { return Ok(()) };

    let spell_id = match cursor_info {
        CursorInfo::Action { spell_id, .. } => spell_id,
        CursorInfo::Spell { spell_id } => spell_id,
    };

    // If something is already in the target slot, put it on cursor
    let old = { state.borrow().action_bars.get(&slot).copied() };
    state.borrow_mut().action_bars.insert(slot, spell_id);
    eprintln!("[cursor] PlaceAction({}) — spell {}", slot, spell_id);

    if let Some(old_spell) = old {
        state.borrow_mut().cursor_item = Some(CursorInfo::Action { slot, spell_id: old_spell });
    }

    fire_action_bar_updates(state, lua)?;
    Ok(())
}

/// Core logic for PutActionInSlot.
fn put_action_in_slot(state: &Rc<RefCell<SimState>>, lua: &Lua, action: u32, slot: u32) -> Result<()> {
    // The "action" here is a spell_id — place it directly into the slot.
    state.borrow_mut().action_bars.insert(slot, action);
    eprintln!("[cursor] PutActionInSlot({}, {})", action, slot);
    fire_action_bar_updates(state, lua)?;
    Ok(())
}

/// Fire ACTIONBAR_SLOT_CHANGED and push button state updates.
fn fire_action_bar_updates(state: &Rc<RefCell<SimState>>, lua: &Lua) -> Result<()> {
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    // arg1=0 means "all slots changed" — without it, button handlers skip the update.
    fire.call::<()>((lua.create_string("ACTIONBAR_SLOT_CHANGED")?, 0))?;
    fire.call::<()>(lua.create_string("ACTIONBAR_UPDATE_STATE")?)?;
    super::action_bar_api::push_action_button_state_update(state, lua)?;
    Ok(())
}
