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
    /// Console output from Lua print() calls.
    pub console_output: Vec<String>,
}

impl WowLuaEnv {
    /// Create a new WoW Lua environment with the API initialized.
    pub fn new() -> Result<Self> {
        // Use unsafe_new to get full standard library including debug
        // This is safe for our simulator since we control the Lua code
        let lua = unsafe { Lua::unsafe_new() };
        let state = Rc::new(RefCell::new(SimState::default()));

        // Create UIParent (the root frame)
        {
            let mut s = state.borrow_mut();
            let ui_parent = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("UIParent".to_string()),
                None,
            );
            let ui_parent_id = ui_parent.id;
            s.widgets.register(ui_parent);

            // Create Minimap (built-in UI element)
            let minimap = crate::widget::Frame::new(
                crate::widget::WidgetType::Frame,
                Some("Minimap".to_string()),
                Some(ui_parent_id),
            );
            s.widgets.register(minimap);
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

    /// Execute Lua code with a custom chunk name (for better error messages and debugstack).
    pub fn exec_named(&self, code: &str, name: &str) -> Result<()> {
        self.lua.load(code).set_name(name).exec()?;
        Ok(())
    }

    /// Execute Lua code with varargs (like WoW addon loading).
    /// In WoW, each addon file receives (addonName, addonTable) as varargs.
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
        Ok(table)
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
    pub fn fire_event_with_args(&self, event: &str, args: &[Value]) -> Result<()> {
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

                    // Build arguments: (self, event, ...args)
                    let mut call_args = vec![frame, Value::String(self.lua.create_string(event)?)];
                    call_args.extend(args.iter().cloned());

                    handler.call::<()>(MultiValue::from_vec(call_args))?;
                }
            }
        }

        Ok(())
    }

    /// Fire a script handler for a specific widget.
    /// handler_name is like "OnClick", "OnEnter", etc.
    /// extra_args are passed after the frame (self) argument.
    pub fn fire_script_handler(
        &self,
        widget_id: u64,
        handler_name: &str,
        extra_args: Vec<Value>,
    ) -> Result<()> {
        let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();

        if let Some(table) = scripts_table {
            let frame_key = format!("{}_{}", widget_id, handler_name);
            let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();

            if let Some(handler) = handler {
                // Get the frame userdata
                let frame_ref_key = format!("__frame_{}", widget_id);
                let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;

                // Build arguments: (self, ...extra_args)
                let mut call_args = vec![frame];
                call_args.extend(extra_args);

                handler.call::<()>(MultiValue::from_vec(call_args))?;
            }
        }

        Ok(())
    }

    /// Dispatch a slash command (e.g., "/wa options").
    /// Returns Ok(true) if a handler was found and called, Ok(false) if no handler matched.
    pub fn dispatch_slash_command(&self, input: &str) -> Result<bool> {
        let input = input.trim();
        if !input.starts_with('/') {
            return Ok(false);
        }

        // Parse command and message: "/wa options" -> cmd="/wa", msg="options"
        let (cmd, msg) = match input.find(' ') {
            Some(pos) => (&input[..pos], input[pos + 1..].trim()),
            None => (input, ""),
        };
        let cmd_lower = cmd.to_lowercase();

        // Scan globals for SLASH_* variables to find a matching command
        let globals = self.lua.globals();
        let slash_cmd_list: mlua::Table = globals.get("SlashCmdList")?;

        // Iterate through all globals looking for SLASH_* patterns
        for pair in globals.pairs::<String, Value>() {
            let (key, value) = pair?;

            // Look for SLASH_NAME1, SLASH_NAME2, etc.
            if !key.starts_with("SLASH_") {
                continue;
            }

            // Extract the command name (e.g., "SLASH_WEAKAURAS1" -> "WEAKAURAS")
            let suffix = &key[6..]; // Skip "SLASH_"
            let name = suffix.trim_end_matches(|c: char| c.is_ascii_digit());
            if name.is_empty() {
                continue;
            }

            // Check if this SLASH_ variable matches our command
            if let Value::String(slash_str) = value {
                if slash_str.to_str()?.to_lowercase() == cmd_lower {
                    // Found a match! Look up the handler in SlashCmdList
                    let handler: Option<mlua::Function> = slash_cmd_list.get(name).ok();
                    if let Some(handler) = handler {
                        let msg_value = self.lua.create_string(msg)?;
                        handler.call::<()>(msg_value)?;
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
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
