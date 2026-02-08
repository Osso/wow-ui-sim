//! Targeting API: TargetUnit() and ClearTarget() globals.

use crate::lua_api::state::build_target_info;
use crate::lua_api::SimState;
use mlua::{Lua, Result};
use std::cell::RefCell;
use std::rc::Rc;

/// Register TargetUnit(unitId) and ClearTarget() globals.
pub fn register_targeting_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let g = lua.globals();

    let st = state.clone();
    g.set("TargetUnit", lua.create_function(move |lua, unit_id: String| {
        let info = {
            let s = st.borrow();
            build_target_info(&unit_id, &s)
        };
        if let Some(info) = info {
            st.borrow_mut().current_target = Some(info);
            fire_target_changed(lua)?;
        }
        Ok(())
    })?)?;

    g.set("ClearTarget", lua.create_function(move |lua, ()| {
        let had_target = state.borrow().current_target.is_some();
        if had_target {
            state.borrow_mut().current_target = None;
            fire_target_changed(lua)?;
        }
        Ok(())
    })?)?;

    Ok(())
}

/// Fire PLAYER_TARGET_CHANGED event to all registered listeners.
fn fire_target_changed(lua: &Lua) -> Result<()> {
    // FireEvent is registered by system_api and dispatches to all event listeners.
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call(lua.create_string("PLAYER_TARGET_CHANGED")?)
}
