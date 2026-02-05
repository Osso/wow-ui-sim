//! Event registration methods: RegisterEvent, UnregisterEvent, etc.

use super::FrameHandle;
use mlua::{UserDataMethods, Value};

/// Add event registration methods to FrameHandle UserData.
pub fn add_event_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    // RegisterEvent(event)
    methods.add_method("RegisterEvent", |_, this, event: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.register_event(&event);
        }
        Ok(())
    });

    // RegisterUnitEvent(event, unit1, unit2, ...) - register for unit-specific events
    // Some addons pass a callback function as the last argument (non-standard)
    methods.add_method(
        "RegisterUnitEvent",
        |_, this, (event, _args): (String, mlua::Variadic<Value>)| {
            let mut state = this.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut(this.id) {
                frame.register_event(&event);
            }
            Ok(())
        },
    );

    // UnregisterEvent(event)
    methods.add_method("UnregisterEvent", |_, this, event: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.unregister_event(&event);
        }
        Ok(())
    });

    // UnregisterAllEvents()
    methods.add_method("UnregisterAllEvents", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.registered_events.clear();
        }
        Ok(())
    });

    // IsEventRegistered(event) -> bool
    methods.add_method("IsEventRegistered", |_, this, event: String| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            return Ok(frame.registered_events.contains(&event));
        }
        Ok(false)
    });

    // SetPropagateKeyboardInput(propagate) - keyboard input propagation
    methods.add_method(
        "SetPropagateKeyboardInput",
        |_, _this, _propagate: bool| {
            // In the simulator, this is a no-op
            Ok(())
        },
    );

    // GetPropagateKeyboardInput() -> bool
    methods.add_method("GetPropagateKeyboardInput", |_, _this, ()| {
        // Default to false in the simulator
        Ok(false)
    });
}
