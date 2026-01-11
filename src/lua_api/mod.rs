//! Lua API bindings implementing WoW's addon API.

mod frame_methods;
mod globals;

use crate::event::{EventQueue, ScriptRegistry};
use crate::widget::WidgetRegistry;
use crate::Result;
use mlua::{Lua, MultiValue, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// The WoW Lua environment.
pub struct WowLuaEnv {
    lua: Lua,
    state: Rc<RefCell<SimState>>,
}

/// Shared simulator state accessible from Lua.
#[derive(Debug, Default)]
pub struct SimState {
    pub widgets: WidgetRegistry,
    pub events: EventQueue,
    pub scripts: ScriptRegistry,
}

impl WowLuaEnv {
    /// Create a new WoW Lua environment with the API initialized.
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        let state = Rc::new(RefCell::new(SimState::default()));

        // Create UIParent (the root frame)
        {
            let mut s = state.borrow_mut();
            let ui_parent = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("UIParent".to_string()),
                None,
            );
            s.widgets.register(ui_parent);
        }

        // Register global functions
        globals::register_globals(&lua, Rc::clone(&state))?;

        Ok(Self { lua, state })
    }

    /// Execute Lua code.
    pub fn exec(&self, code: &str) -> Result<()> {
        self.lua.load(code).exec()?;
        Ok(())
    }

    /// Execute Lua code and return the result.
    pub fn eval<T: mlua::FromLuaMulti>(&self, code: &str) -> Result<T> {
        let result = self.lua.load(code).eval()?;
        Ok(result)
    }

    /// Fire an event to all registered frames.
    pub fn fire_event(&self, event: &str) -> Result<()> {
        self.fire_event_with_args(event, &[])
    }

    /// Fire an event with arguments to all registered frames.
    pub fn fire_event_with_args(&self, event: &str, _args: &[Value]) -> Result<()> {
        let listeners = {
            let state = self.state.borrow();
            state.widgets.get_event_listeners(event)
        };

        for widget_id in listeners {
            // Get the handler function from our scripts table
            let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();

            if let Some(table) = scripts_table {
                let frame_key = format!("{}_OnEvent", widget_id);
                let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

                if let Some(handler) = handler {
                    // Get the frame userdata
                    let frame_ref_key = format!("__frame_{}", widget_id);
                    let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                    // Build arguments: (self, event, ...)
                    let call_args = vec![frame, Value::String(self.lua.create_string(event)?)];

                    handler.call::<()>(MultiValue::from_vec(call_args))?;
                }
            }
        }

        Ok(())
    }

    /// Get access to the Lua state.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Get access to the simulator state.
    pub fn state(&self) -> &Rc<RefCell<SimState>> {
        &self.state
    }
}
