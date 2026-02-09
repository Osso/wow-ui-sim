//! Lightweight loader environment for addon loading.
//!
//! Borrows the Lua instance instead of owning it, allowing both startup loading
//! (via WowLuaEnv) and runtime on-demand loading (from Lua callbacks).

use super::state::SimState;
use crate::Result;
use mlua::Lua;
use std::cell::RefCell;
use std::rc::Rc;

/// Lightweight loader environment that borrows the Lua instance.
pub struct LoaderEnv<'a> {
    pub(crate) lua: &'a Lua,
    pub(crate) state: Rc<RefCell<SimState>>,
}

impl<'a> LoaderEnv<'a> {
    /// Create from a Lua reference and shared state (for runtime loading).
    pub fn new(lua: &'a Lua, state: Rc<RefCell<SimState>>) -> Self {
        Self { lua, state }
    }

    /// Execute Lua code.
    pub fn exec(&self, code: &str) -> Result<()> {
        self.lua.load(code).exec()?;
        Ok(())
    }

    /// Execute Lua code with varargs (addon loading pattern).
    pub fn exec_with_varargs(
        &self,
        code: &str,
        name: &str,
        addon_name: &str,
        addon_table: mlua::Table,
    ) -> Result<()> {
        let chunk = self.lua.load(code).set_name(name);
        let func: mlua::Function = chunk.into_function()?;
        func.call::<()>((addon_name.to_string(), addon_table))?;
        Ok(())
    }

    /// Create a new empty table for addon private storage.
    pub fn create_addon_table(&self) -> Result<mlua::Table> {
        let table = self.lua.create_table()?;
        let unpack_fn = self.lua.create_function(|_, this: mlua::Table| {
            let v1: mlua::Value = this.get(1).unwrap_or(mlua::Value::Nil);
            let v2: mlua::Value = this.get(2).unwrap_or(mlua::Value::Nil);
            let v3: mlua::Value = this.get(3).unwrap_or(mlua::Value::Nil);
            let v4: mlua::Value = this.get(4).unwrap_or(mlua::Value::Nil);
            Ok((v1, v2, v3, v4))
        })?;
        table.set("unpack", unpack_fn)?;
        Ok(table)
    }

    /// Get access to the Lua state.
    pub fn lua(&self) -> &Lua {
        self.lua
    }

    /// Get access to the simulator state.
    pub fn state(&self) -> &Rc<RefCell<SimState>> {
        &self.state
    }

    /// Fire an event with arguments to all registered frames.
    pub fn fire_event_with_args(&self, event: &str, args: &[mlua::Value]) -> Result<()> {
        use super::script_helpers::{call_error_handler, get_frame_ref, get_script};

        let listeners = {
            let state = self.state.borrow();
            state.widgets.get_event_listeners(event)
        };

        for widget_id in listeners {
            if let Some(handler) = get_script(self.lua, widget_id, "OnEvent")
                && let Some(frame) = get_frame_ref(self.lua, widget_id) {
                    let mut call_args =
                        vec![frame, mlua::Value::String(self.lua.create_string(event)?)];
                    call_args.extend(args.iter().cloned());
                    if let Err(e) = handler.call::<()>(mlua::MultiValue::from_vec(call_args)) {
                        call_error_handler(self.lua, &e.to_string());
                    }
                }
        }

        Ok(())
    }
}
