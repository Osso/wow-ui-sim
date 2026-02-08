//! Event registration methods: RegisterEvent, UnregisterEvent, etc.

use super::FrameHandle;
use mlua::{UserDataMethods, Value};

/// Add event registration methods to FrameHandle UserData.
pub fn add_event_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_event_register_methods(methods);
    add_keyboard_propagation_methods(methods);
}

/// RegisterEvent, RegisterUnitEvent, UnregisterEvent, UnregisterAllEvents, IsEventRegistered
fn add_event_register_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("RegisterEvent", |_, this, event: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_silent(this.id) {
            frame.register_event(&event);
        }
        Ok(())
    });

    // Some addons pass a callback function as the last argument (non-standard)
    methods.add_method(
        "RegisterUnitEvent",
        |_, this, (event, _args): (String, mlua::Variadic<Value>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_silent(this.id) {
                frame.register_event(&event);
            }
            Ok(())
        },
    );

    methods.add_method("UnregisterEvent", |_, this, event: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_silent(this.id) {
            frame.unregister_event(&event);
        }
        Ok(())
    });

    methods.add_method("UnregisterAllEvents", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_silent(this.id) {
            frame.registered_events.clear();
        }
        Ok(())
    });

    methods.add_method("IsEventRegistered", |_, this, event: String| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            return Ok(frame.register_all_events || frame.registered_events.contains(&event));
        }
        Ok(false)
    });

    // RegisterAllEvents() - register for all events
    methods.add_method("RegisterAllEvents", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_silent(this.id) {
            frame.register_all_events = true;
        }
        Ok(())
    });
}

/// SetPropagateKeyboardInput, GetPropagateKeyboardInput
fn add_keyboard_propagation_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method(
        "SetPropagateKeyboardInput",
        |_, this, propagate: bool| {
            let mut state = this.state.borrow_mut();
            if let Some(f) = state.widgets.get_mut_silent(this.id) {
                f.propagate_keyboard_input = propagate;
            }
            Ok(())
        },
    );

    methods.add_method("GetPropagateKeyboardInput", |_, this, ()| {
        let state = this.state.borrow();
        let propagate = state
            .widgets
            .get(this.id)
            .map(|f| f.propagate_keyboard_input)
            .unwrap_or(false);
        Ok(propagate)
    });
}
