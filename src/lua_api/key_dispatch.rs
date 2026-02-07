//! Key press dispatch: Escape handling, OnKeyDown propagation, GameMenuFrame toggle.

use crate::Result;
use mlua::{MultiValue, Value};

use super::env::WowLuaEnv;

/// Check whether a Lua value is truthy (not nil and not false).
fn is_truthy(val: &Value) -> bool {
    !matches!(val, Value::Nil | Value::Boolean(false))
}

impl WowLuaEnv {
    /// Simulate a key press with WoW's full dispatch chain.
    pub fn send_key_press(&self, key: &str) -> Result<()> {
        if key == "ESCAPE" {
            self.dispatch_escape()
        } else {
            self.dispatch_key(key)
        }
    }

    /// Escape priority: focused EditBox → CloseSpecialWindows → toggle GameMenuFrame.
    fn dispatch_escape(&self) -> Result<()> {
        let focused = self.state.borrow().focused_frame_id;
        if let Some(fid) = focused {
            if self.fire_handler_returns_truthy(fid, "OnEscapePressed")? {
                return Ok(());
            }
        }
        if self.close_special_windows()? {
            return Ok(());
        }
        self.toggle_game_menu()
    }

    /// General key dispatch: special EditBox handler → OnKeyDown with propagation.
    fn dispatch_key(&self, key: &str) -> Result<()> {
        let focused = self.state.borrow().focused_frame_id;
        if let Some(fid) = focused {
            let special = match key {
                "ENTER" => Some("OnEnterPressed"),
                "TAB" => Some("OnTabPressed"),
                "SPACE" => Some("OnSpacePressed"),
                _ => None,
            };
            if let Some(handler) = special {
                if self.fire_handler_returns_truthy(fid, handler)? {
                    return Ok(());
                }
            }
        }
        self.dispatch_on_key_down(key)
    }

    /// Fire OnKeyDown on focused or keyboard-enabled frames, propagating up parents.
    fn dispatch_on_key_down(&self, key: &str) -> Result<()> {
        let start_id = {
            let state = self.state.borrow();
            state.focused_frame_id.or_else(|| {
                state
                    .widgets
                    .all_ids()
                    .into_iter()
                    .find(|&id| {
                        state
                            .widgets
                            .get(id)
                            .map(|f| f.keyboard_enabled && f.visible)
                            .unwrap_or(false)
                    })
            })
        };
        let Some(frame_id) = start_id else {
            return Ok(());
        };
        self.fire_on_key_down(frame_id, key)
    }

    /// Fire OnKeyDown on a frame; if propagate_keyboard_input, walk up parents.
    fn fire_on_key_down(&self, frame_id: u64, key: &str) -> Result<()> {
        let key_val = Value::String(self.lua.create_string(key)?);
        self.fire_script_handler(frame_id, "OnKeyDown", vec![key_val])?;
        let propagate = self
            .state
            .borrow()
            .widgets
            .get(frame_id)
            .map(|f| f.propagate_keyboard_input)
            .unwrap_or(false);
        if propagate {
            let parent = self
                .state
                .borrow()
                .widgets
                .get(frame_id)
                .and_then(|f| f.parent_id);
            if let Some(pid) = parent {
                return self.fire_on_key_down(pid, key);
            }
        }
        Ok(())
    }

    /// Fire a script handler and return whether it returned a truthy value.
    fn fire_handler_returns_truthy(&self, widget_id: u64, handler_name: &str) -> Result<bool> {
        let scripts_table: Option<mlua::Table> = self.lua.globals().get("__scripts").ok();
        let Some(table) = scripts_table else {
            return Ok(false);
        };
        let frame_key = format!("{}_{}", widget_id, handler_name);
        let handler: Option<mlua::Function> = table.get(frame_key.as_str()).ok();
        let Some(handler) = handler else {
            return Ok(false);
        };
        let frame_ref_key = format!("__frame_{}", widget_id);
        let frame: Value = self.lua.globals().get(frame_ref_key.as_str())?;
        let result: Value = handler.call(MultiValue::from_vec(vec![frame]))?;
        Ok(is_truthy(&result))
    }

    /// Iterate UISpecialFrames, hide visible ones. Returns true if any were closed.
    fn close_special_windows(&self) -> Result<bool> {
        let table: Option<mlua::Table> = self.lua.globals().get("UISpecialFrames").ok();
        let Some(table) = table else {
            return Ok(false);
        };
        let mut closed = false;
        for entry in table.sequence_values::<String>() {
            let name = entry?;
            let id = self.state.borrow().widgets.get_id_by_name(&name);
            if let Some(id) = id {
                let is_visible = self
                    .state
                    .borrow()
                    .widgets
                    .get(id)
                    .map(|f| f.visible)
                    .unwrap_or(false);
                if is_visible {
                    self.state
                        .borrow_mut()
                        .widgets
                        .get_mut(id)
                        .map(|f| f.visible = false);
                    closed = true;
                }
            }
        }
        Ok(closed)
    }

    /// Toggle GameMenuFrame visibility.
    fn toggle_game_menu(&self) -> Result<()> {
        let id = self.state.borrow().widgets.get_id_by_name("GameMenuFrame");
        let Some(id) = id else {
            return Ok(());
        };
        let is_visible = self
            .state
            .borrow()
            .widgets
            .get(id)
            .map(|f| f.visible)
            .unwrap_or(false);
        if is_visible {
            self.state
                .borrow_mut()
                .widgets
                .get_mut(id)
                .map(|f| f.visible = false);
        } else {
            self.state
                .borrow_mut()
                .widgets
                .get_mut(id)
                .map(|f| f.visible = true);
            super::frame::fire_on_show_recursive(&self.lua, &self.state, id)?;
        }
        Ok(())
    }
}
