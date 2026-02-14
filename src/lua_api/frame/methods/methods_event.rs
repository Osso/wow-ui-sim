//! Event registration methods: RegisterEvent, UnregisterEvent, etc.

use crate::lua_api::frame::handle::{get_sim_state, lud_to_id};
use mlua::{LightUserData, Lua, Value};

/// Add event registration methods to the frame methods table.
pub fn add_event_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_event_register_methods(lua, methods)?;
    add_event_query_methods(lua, methods)?;
    add_keyboard_propagation_methods(lua, methods)?;
    Ok(())
}

/// RegisterEvent, RegisterUnitEvent, UnregisterEvent, UnregisterAllEvents, RegisterAllEvents
fn add_event_register_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("RegisterEvent", lua.create_function(|lua, (ud, event): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(id) {
            frame.register_event(&event);
        }
        Ok(())
    })?)?;

    // Some addons pass a callback function as the last argument (non-standard)
    methods.set("RegisterUnitEvent", lua.create_function(
        |lua, (ud, event, _args): (LightUserData, String, mlua::Variadic<Value>)| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(id) {
                frame.register_event(&event);
            }
            Ok(())
        },
    )?)?;

    methods.set("UnregisterEvent", lua.create_function(|lua, (ud, event): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(id) {
            frame.unregister_event(&event);
        }
        Ok(())
    })?)?;

    methods.set("UnregisterAllEvents", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(id) {
            frame.registered_events.clear();
        }
        Ok(())
    })?)?;

    methods.set("RegisterAllEvents", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(id) {
            frame.register_all_events = true;
        }
        Ok(())
    })?)?;

    Ok(())
}

/// IsEventRegistered, RegisterEventCallback
fn add_event_query_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("IsEventRegistered", lua.create_function(|lua, (ud, event): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            return Ok(frame.register_all_events || frame.registered_events.contains(&event));
        }
        Ok(false)
    })?)?;

    // RegisterEventCallback(event, callbackContainer) - callback-based event registration
    methods.set("RegisterEventCallback", lua.create_function(
        |_lua, (_ud, _event, _cb): (LightUserData, Value, Value)| Ok(()),
    )?)?;

    Ok(())
}

/// SetPropagateKeyboardInput, GetPropagateKeyboardInput
fn add_keyboard_propagation_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetPropagateKeyboardInput", lua.create_function(
        |lua, (ud, propagate): (LightUserData, bool)| {
            let id = lud_to_id(ud);
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(f) = state.widgets.get_mut(id) {
                f.propagate_keyboard_input = propagate;
            }
            Ok(())
        },
    )?)?;

    methods.set("GetPropagateKeyboardInput", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let propagate = state
            .widgets
            .get(id)
            .map(|f| f.propagate_keyboard_input)
            .unwrap_or(false);
        Ok(propagate)
    })?)?;

    Ok(())
}
