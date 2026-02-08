//! EditBox widget methods: focus, cursor, text input, history, insets.

use super::widget_tooltip::{fire_tooltip_script, val_to_f32};
use super::FrameHandle;
use mlua::UserDataMethods;

pub fn add_editbox_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_editbox_focus_methods(methods);
    add_editbox_cursor_methods(methods);
    add_editbox_number_methods(methods);
    add_editbox_limit_methods(methods);
    add_editbox_flag_methods(methods);
    add_editbox_history_methods(methods);
    add_editbox_inset_methods(methods);
    // GetInputLanguage returns the current input language for the editbox
    methods.add_method("GetInputLanguage", |_, _this, ()| Ok("ROMAN"));
}

fn add_editbox_focus_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetFocus", |lua, this, ()| {
        let old_focus = {
            let mut s = this.state.borrow_mut();
            let old = s.focused_frame_id;
            s.focused_frame_id = Some(this.id);
            old
        };
        if let Some(old_id) = old_focus
            && old_id != this.id {
                fire_focus_handler(lua, old_id, "OnEditFocusLost")?;
            }
        fire_focus_handler(lua, this.id, "OnEditFocusGained")?;
        Ok(())
    });
    methods.add_method("ClearFocus", |lua, this, ()| {
        let had_focus = {
            let mut s = this.state.borrow_mut();
            if s.focused_frame_id == Some(this.id) {
                s.focused_frame_id = None;
                true
            } else {
                false
            }
        };
        if had_focus {
            fire_focus_handler(lua, this.id, "OnEditFocusLost")?;
        }
        Ok(())
    });
    methods.add_method("HasFocus", |_, this, ()| {
        if let Ok(s) = this.state.try_borrow() {
            return Ok(s.focused_frame_id == Some(this.id));
        }
        Ok(false)
    });
}

fn add_editbox_cursor_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetCursorPosition", |_, this, pos: i32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_cursor_pos = pos;
        }
        Ok(())
    });
    methods.add_method("GetCursorPosition", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.editbox_cursor_pos).unwrap_or(0))
    });
    methods.add_method("HighlightText", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("Insert", |_, this, text: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            let current = frame.text.get_or_insert_with(String::new);
            let pos = (frame.editbox_cursor_pos as usize).min(current.len());
            current.insert_str(pos, &text);
            frame.editbox_cursor_pos = (pos + text.len()) as i32;
        }
        Ok(())
    });
    methods.add_method("GetNumLetters", |_, this, ()| {
        let state = this.state.borrow();
        let len = state.widgets.get(this.id)
            .and_then(|f| f.text.as_ref())
            .map(|t| t.chars().count())
            .unwrap_or(0);
        Ok(len as i32)
    });
}

fn add_editbox_number_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetNumber", |_, this, n: f64| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.text = Some(n.to_string());
        }
        Ok(())
    });
    methods.add_method("GetNumber", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id)
            && let Some(text) = &frame.text {
                return Ok(text.parse::<f64>().unwrap_or(0.0));
            }
        Ok(0.0)
    });
}

fn add_editbox_limit_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetMaxLetters", |_, this, max: i32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_max_letters = max;
        }
        Ok(())
    });
    methods.add_method("GetMaxLetters", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.editbox_max_letters).unwrap_or(0))
    });
    methods.add_method("SetMaxBytes", |_, this, max: i32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_max_bytes = max;
        }
        Ok(())
    });
    methods.add_method("GetMaxBytes", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.editbox_max_bytes).unwrap_or(0))
    });
}

fn add_editbox_flag_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_editbox_mode_flags(methods);
    add_editbox_input_flags(methods);
}

fn add_editbox_mode_flags<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetMultiLine", |_, this, multi: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_multi_line = multi;
        }
        Ok(())
    });
    methods.add_method("IsMultiLine", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.editbox_multi_line).unwrap_or(false))
    });
    methods.add_method("SetAutoFocus", |_, this, auto: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_auto_focus = auto;
        }
        Ok(())
    });
    methods.add_method("IsAutoFocus", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.editbox_auto_focus).unwrap_or(false))
    });
    methods.add_method("SetNumeric", |_, this, numeric: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_numeric = numeric;
        }
        Ok(())
    });
    methods.add_method("IsNumeric", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.editbox_numeric).unwrap_or(false))
    });
}

fn add_editbox_input_flags<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetPassword", |_, this, pw: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_password = pw;
        }
        Ok(())
    });
    methods.add_method("IsPassword", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.editbox_password).unwrap_or(false))
    });
    methods.add_method("SetBlinkSpeed", |_, this, speed: f64| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_blink_speed = speed;
        }
        Ok(())
    });
    methods.add_method("GetBlinkSpeed", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.editbox_blink_speed).unwrap_or(0.5))
    });
    methods.add_method("SetCountInvisibleLetters", |_, this, count: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_count_invisible_letters = count;
        }
        Ok(())
    });
}

fn add_editbox_history_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("AddHistoryLine", |_, this, text: String| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_history.push(text);
            let max = frame.editbox_history_max;
            if max > 0 && frame.editbox_history.len() > max as usize {
                frame.editbox_history.remove(0);
            }
        }
        Ok(())
    });
    methods.add_method("GetHistoryLines", |_, this, ()| {
        let state = this.state.borrow();
        let count = state.widgets.get(this.id)
            .map(|f| f.editbox_history.len())
            .unwrap_or(0);
        Ok(count as i32)
    });
    methods.add_method("SetHistoryLines", |_, this, max: i32| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_history_max = max;
        }
        Ok(())
    });
}

fn add_editbox_inset_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetTextInsets", |_, this, args: mlua::MultiValue| {
        let mut it = args.into_iter();
        let l = val_to_f32(it.next(), 0.0);
        let r = val_to_f32(it.next(), 0.0);
        let t = val_to_f32(it.next(), 0.0);
        let b = val_to_f32(it.next(), 0.0);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.editbox_text_insets = (l, r, t, b);
        }
        Ok(())
    });
    methods.add_method("GetTextInsets", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            let (l, r, t, b) = frame.editbox_text_insets;
            return Ok((l, r, t, b));
        }
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
    });
}

/// Fire a focus-related script handler (OnEditFocusGained/OnEditFocusLost).
fn fire_focus_handler(lua: &mlua::Lua, frame_id: u64, handler: &str) -> mlua::Result<()> {
    fire_tooltip_script(lua, frame_id, handler)
}
