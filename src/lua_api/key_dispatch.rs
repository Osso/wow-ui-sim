//! Key press dispatch: Escape handling, OnKeyDown propagation, GameMenuFrame toggle.

use crate::Result;
use mlua::{MultiValue, Value};

use super::env::WowLuaEnv;

use std::ops::Range;

/// Check whether a Lua value is truthy (not nil and not false).
fn is_truthy(val: &Value) -> bool {
    !matches!(val, Value::Nil | Value::Boolean(false))
}

/// Get the byte range for the character at the given char index.
fn char_byte_range(s: &str, char_index: usize) -> Range<usize> {
    let mut chars = s.char_indices();
    let (start, ch) = chars.nth(char_index).unwrap();
    start..start + ch.len_utf8()
}

impl WowLuaEnv {
    /// Simulate a key press with WoW's full dispatch chain.
    /// `text` is the raw unicode character for typing into focused EditBoxes.
    pub fn send_key_press(&self, key: &str, text: Option<&str>) -> Result<()> {
        if key == "ESCAPE" {
            self.dispatch_escape()
        } else {
            self.dispatch_key(key, text)
        }
    }

    /// Escape priority: focused EditBox → clear target → CloseSpecialWindows → toggle GameMenuFrame.
    fn dispatch_escape(&self) -> Result<()> {
        let focused = self.state.borrow().focused_frame_id;
        if let Some(fid) = focused
            && self.fire_handler_returns_truthy(fid, "OnEscapePressed")? {
                return Ok(());
            }
        if self.clear_target_if_any()? {
            return Ok(());
        }
        if self.close_special_windows()? {
            return Ok(());
        }
        self.toggle_game_menu()
    }

    /// Clear current target if one exists, firing PLAYER_TARGET_CHANGED.
    /// Returns true if a target was cleared.
    fn clear_target_if_any(&self) -> Result<bool> {
        let has_target = self.state.borrow().current_target.is_some();
        if has_target {
            self.lua.load("ClearTarget()").exec()?;
            return Ok(true);
        }
        Ok(false)
    }

    /// General key dispatch: special EditBox handler → keybinding → OnKeyDown.
    fn dispatch_key(&self, key: &str, text: Option<&str>) -> Result<()> {
        let focused = self.state.borrow().focused_frame_id;
        if let Some(fid) = focused {
            let special = match key {
                "ENTER" => Some("OnEnterPressed"),
                "TAB" => Some("OnTabPressed"),
                "SPACE" => Some("OnSpacePressed"),
                _ => None,
            };
            if let Some(handler) = special
                && self.fire_handler_returns_truthy(fid, handler)? {
                    return Ok(());
                }
        }

        // Check keybindings (skip if an EditBox has focus — keys go to the EditBox).
        let is_editbox = focused.is_some_and(|fid| {
            self.state.borrow().widgets.get(fid)
                .map(|f| f.widget_type == crate::widget::WidgetType::EditBox)
                .unwrap_or(false)
        });
        if !is_editbox
            && super::keybindings::dispatch_key_binding(&self.lua, key)? {
                return Ok(());
            }

        self.dispatch_on_key_down(key)?;

        // EditBox text editing: handle backspace/delete/arrow keys and character input.
        if let Some(fid) = focused
            && is_editbox {
                match key {
                    "BACKSPACE" => self.editbox_backspace(fid)?,
                    "DELETE" => self.editbox_delete(fid)?,
                    "LEFT" => self.editbox_move_cursor(fid, -1)?,
                    "RIGHT" => self.editbox_move_cursor(fid, 1)?,
                    "HOME" => self.editbox_cursor_home(fid)?,
                    "END" => self.editbox_cursor_end(fid)?,
                    _ => {
                        // Insert printable characters (skip control chars like \n, \t)
                        if let Some(t) = text {
                            let printable: String = t.chars()
                                .filter(|c| !c.is_control())
                                .collect();
                            if !printable.is_empty() {
                                self.editbox_insert_text(fid, &printable)?;
                            }
                        }
                    }
                }
            }

        Ok(())
    }

    /// Fire OnKeyDown on focused or keyboard-enabled frames, propagating up parents.
    fn dispatch_on_key_down(&self, key: &str) -> Result<()> {
        let start_id = {
            let state = self.state.borrow();
            state.focused_frame_id.or_else(|| {
                state
                    .widgets
                    .iter_ids()
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
        use super::script_helpers::get_script;

        let Some(handler) = get_script(&self.lua, widget_id, handler_name) else {
            return Ok(false);
        };
        let frame = super::frame::frame_lud(widget_id);
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
                    self.state.borrow_mut().set_frame_visible(id, false);
                    closed = true;
                }
            }
        }
        Ok(closed)
    }

    // ── EditBox text editing helpers ─────────────────────────────────────

    /// Insert text at cursor position, fire OnChar and OnTextChanged.
    fn editbox_insert_text(&self, fid: u64, text: &str) -> Result<()> {
        // Check numeric restriction
        let numeric = self.state.borrow().widgets.get(fid)
            .map(|f| f.editbox_numeric).unwrap_or(false);
        if numeric && !text.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-') {
            return Ok(());
        }

        {
            let mut state = self.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(fid) {
                let current = frame.text.get_or_insert_with(String::new);
                let char_pos = frame.editbox_cursor_pos as usize;
                // Convert char position to byte position
                let byte_pos = current.char_indices()
                    .nth(char_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(current.len());
                current.insert_str(byte_pos, text);
                frame.editbox_cursor_pos += text.chars().count() as i32;
            }
        }

        // Fire OnChar with each character
        for ch in text.chars() {
            let char_val = Value::String(self.lua.create_string(ch.to_string())?);
            self.fire_script_handler(fid, "OnChar", vec![char_val])?;
        }

        // Fire OnTextChanged with userInput=true
        let user_input = Value::Boolean(true);
        self.fire_script_handler(fid, "OnTextChanged", vec![user_input])?;
        Ok(())
    }

    /// Delete the character before the cursor (Backspace).
    fn editbox_backspace(&self, fid: u64) -> Result<()> {
        let changed = {
            let mut state = self.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(fid) {
                let current = frame.text.get_or_insert_with(String::new);
                let char_pos = frame.editbox_cursor_pos as usize;
                if char_pos > 0 {
                    let byte_range = char_byte_range(current, char_pos - 1);
                    current.drain(byte_range);
                    frame.editbox_cursor_pos -= 1;
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if changed {
            let user_input = Value::Boolean(true);
            self.fire_script_handler(fid, "OnTextChanged", vec![user_input])?;
        }
        Ok(())
    }

    /// Delete the character after the cursor (Delete key).
    fn editbox_delete(&self, fid: u64) -> Result<()> {
        let changed = {
            let mut state = self.state.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(fid) {
                let current = frame.text.get_or_insert_with(String::new);
                let char_pos = frame.editbox_cursor_pos as usize;
                let char_count = current.chars().count();
                if char_pos < char_count {
                    let byte_range = char_byte_range(current, char_pos);
                    current.drain(byte_range);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if changed {
            let user_input = Value::Boolean(true);
            self.fire_script_handler(fid, "OnTextChanged", vec![user_input])?;
        }
        Ok(())
    }

    /// Move cursor by `delta` characters (negative = left, positive = right).
    fn editbox_move_cursor(&self, fid: u64, delta: i32) -> Result<()> {
        let mut state = self.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(fid) {
            let char_count = frame.text.as_ref()
                .map(|t| t.chars().count() as i32)
                .unwrap_or(0);
            let new_pos = (frame.editbox_cursor_pos + delta).clamp(0, char_count);
            frame.editbox_cursor_pos = new_pos;
        }
        Ok(())
    }

    /// Move cursor to the beginning of text (Home key).
    fn editbox_cursor_home(&self, fid: u64) -> Result<()> {
        let mut state = self.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(fid) {
            frame.editbox_cursor_pos = 0;
        }
        Ok(())
    }

    /// Move cursor to the end of text (End key).
    fn editbox_cursor_end(&self, fid: u64) -> Result<()> {
        let mut state = self.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(fid) {
            let char_count = frame.text.as_ref()
                .map(|t| t.chars().count() as i32)
                .unwrap_or(0);
            frame.editbox_cursor_pos = char_count;
        }
        Ok(())
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
            self.state.borrow_mut().set_frame_visible(id, false);
        } else {
            self.state.borrow_mut().set_frame_visible(id, true);
            super::frame::fire_on_show_recursive(&self.lua, id)?;
        }
        Ok(())
    }
}
