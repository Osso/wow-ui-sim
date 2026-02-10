//! Targeting API: TargetUnit() and ClearTarget() globals.

use crate::lua_api::state::build_target_info;
use crate::lua_api::SimState;
use mlua::{Lua, Result};
use std::cell::RefCell;
use std::rc::Rc;

/// Register targeting globals: TargetUnit, ClearTarget, FocusUnit, ClearFocus,
/// SpellIsTargeting, SpellStopTargeting, SpellTargetUnit, CursorHasItem.
pub fn register_targeting_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let g = lua.globals();

    register_target_globals(&g, lua, state.clone())?;
    register_focus_globals(&g, lua, state)?;
    register_spell_targeting_stubs(&g, lua)?;

    Ok(())
}

fn register_target_globals(g: &mlua::Table, lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let st = state.clone();
    g.set("TargetUnit", lua.create_function(move |lua, unit_id: String| {
        let info = {
            let s = st.borrow();
            build_target_info(&unit_id, &s)
        };
        if let Some(info) = info {
            st.borrow_mut().current_target = Some(info);
            fire_event(lua, "PLAYER_TARGET_CHANGED")?;
        }
        Ok(())
    })?)?;

    g.set("ClearTarget", lua.create_function(move |lua, ()| {
        let had_target = state.borrow().current_target.is_some();
        if had_target {
            state.borrow_mut().current_target = None;
            fire_event(lua, "PLAYER_TARGET_CHANGED")?;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn register_focus_globals(g: &mlua::Table, lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let st = state.clone();
    g.set("FocusUnit", lua.create_function(move |lua, unit_id: String| {
        let info = {
            let s = st.borrow();
            build_target_info(&unit_id, &s)
        };
        if let Some(info) = info {
            st.borrow_mut().current_focus = Some(info);
            fire_event(lua, "PLAYER_FOCUS_CHANGED")?;
        }
        Ok(())
    })?)?;

    g.set("ClearFocus", lua.create_function(move |lua, ()| {
        let had_focus = state.borrow().current_focus.is_some();
        if had_focus {
            state.borrow_mut().current_focus = None;
            fire_event(lua, "PLAYER_FOCUS_CHANGED")?;
        }
        Ok(())
    })?)?;

    Ok(())
}

/// Spell-targeting and cursor stubs needed by SecureTemplates.lua SECURE_ACTIONS.
fn register_spell_targeting_stubs(g: &mlua::Table, lua: &Lua) -> Result<()> {
    g.set("SpellIsTargeting", lua.create_function(|_, ()| Ok(false))?)?;
    g.set("SpellStopTargeting", lua.create_function(|_, ()| Ok(()))?)?;
    g.set("SpellTargetUnit", lua.create_function(|_, _unit: String| Ok(()))?)?;
    g.set("CursorHasItem", lua.create_function(|_, ()| Ok(false))?)?;
    Ok(())
}

/// Fire a named event to all registered listeners.
fn fire_event(lua: &Lua, event_name: &str) -> Result<()> {
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call(lua.create_string(event_name)?)
}
