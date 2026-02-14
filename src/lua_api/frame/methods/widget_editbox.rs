//! EditBox widget methods: focus, cursor, text input, history, insets.

use super::widget_tooltip::{fire_tooltip_script, val_to_f32};
use crate::lua_api::frame::handle::{get_sim_state, lud_to_id};
use mlua::{LightUserData, Lua};

pub fn add_editbox_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_editbox_focus_methods(lua, methods)?;
    add_editbox_cursor_methods(lua, methods)?;
    add_editbox_number_methods(lua, methods)?;
    add_editbox_limit_methods(lua, methods)?;
    add_editbox_mode_flags(lua, methods)?;
    add_editbox_input_flags(lua, methods)?;
    add_editbox_history_methods(lua, methods)?;
    add_editbox_inset_methods(lua, methods)?;
    // Taint system stub: WoW's secure environment prevents addons from calling
    // SetText on protected EditBoxes. No taint system in the simulator.
    methods.set("SetSecurityDisableSetText", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    // GetInputLanguage returns the current input language for the editbox
    methods.set("GetInputLanguage", lua.create_function(|_, _ud: LightUserData| Ok("ROMAN"))?)?;
    Ok(())
}

fn add_editbox_focus_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetFocus", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let old_focus = {
            let state_rc = get_sim_state(lua);
            let mut s = state_rc.borrow_mut();
            let old = s.focused_frame_id;
            s.focused_frame_id = Some(id);
            old
        };
        // Already focused -- nothing to do (prevents infinite recursion when
        // OnEditFocusGained handlers call SetFocus again).
        if old_focus == Some(id) {
            return Ok(());
        }
        if let Some(old_id) = old_focus {
            fire_focus_handler(lua, old_id, "OnEditFocusLost")?;
        }
        fire_focus_handler(lua, id, "OnEditFocusGained")?;
        Ok(())
    })?)?;

    methods.set("ClearFocus", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let had_focus = {
            let state_rc = get_sim_state(lua);
            let mut s = state_rc.borrow_mut();
            if s.focused_frame_id == Some(id) {
                s.focused_frame_id = None;
                true
            } else {
                false
            }
        };
        if had_focus {
            fire_focus_handler(lua, id, "OnEditFocusLost")?;
        }
        Ok(())
    })?)?;

    methods.set("HasFocus", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        if let Ok(s) = state_rc.try_borrow() {
            return Ok(s.focused_frame_id == Some(id));
        }
        Ok(false)
    })?)?;

    Ok(())
}

fn add_editbox_cursor_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetCursorPosition", lua.create_function(|lua, (ud, pos): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_cursor_pos = pos;
        }
        Ok(())
    })?)?;

    methods.set("GetCursorPosition", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.editbox_cursor_pos).unwrap_or(0))
    })?)?;

    methods.set("HighlightText", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    methods.set("Insert", lua.create_function(|lua, (ud, text): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            let current = frame.text.get_or_insert_with(String::new);
            let pos = (frame.editbox_cursor_pos as usize).min(current.len());
            current.insert_str(pos, &text);
            frame.editbox_cursor_pos = (pos + text.len()) as i32;
        }
        Ok(())
    })?)?;

    methods.set("GetNumLetters", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let len = state.widgets.get(id)
            .and_then(|f| f.text.as_ref())
            .map(|t| t.chars().count())
            .unwrap_or(0);
        Ok(len as i32)
    })?)?;

    Ok(())
}

fn add_editbox_number_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetNumber", lua.create_function(|lua, (ud, n): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.text = Some(n.to_string());
        }
        Ok(())
    })?)?;

    methods.set("GetNumber", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id)
            && let Some(text) = &frame.text {
                return Ok(text.parse::<f64>().unwrap_or(0.0));
            }
        Ok(0.0)
    })?)?;

    Ok(())
}

fn add_editbox_limit_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetMaxLetters", lua.create_function(|lua, (ud, max): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_max_letters = max;
        }
        Ok(())
    })?)?;

    methods.set("GetMaxLetters", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.editbox_max_letters).unwrap_or(0))
    })?)?;

    methods.set("SetMaxBytes", lua.create_function(|lua, (ud, max): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_max_bytes = max;
        }
        Ok(())
    })?)?;

    methods.set("GetMaxBytes", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.editbox_max_bytes).unwrap_or(0))
    })?)?;

    Ok(())
}

fn add_editbox_mode_flags(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetMultiLine", lua.create_function(|lua, (ud, multi): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_multi_line = multi;
        }
        Ok(())
    })?)?;

    methods.set("IsMultiLine", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.editbox_multi_line).unwrap_or(false))
    })?)?;

    methods.set("SetAutoFocus", lua.create_function(|lua, (ud, auto): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_auto_focus = auto;
        }
        Ok(())
    })?)?;

    methods.set("IsAutoFocus", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.editbox_auto_focus).unwrap_or(false))
    })?)?;

    methods.set("SetNumeric", lua.create_function(|lua, (ud, numeric): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_numeric = numeric;
        }
        Ok(())
    })?)?;

    methods.set("IsNumeric", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.editbox_numeric).unwrap_or(false))
    })?)?;

    Ok(())
}

fn add_editbox_input_flags(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetPassword", lua.create_function(|lua, (ud, pw): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_password = pw;
        }
        Ok(())
    })?)?;

    methods.set("IsPassword", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.editbox_password).unwrap_or(false))
    })?)?;

    methods.set("SetBlinkSpeed", lua.create_function(|lua, (ud, speed): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_blink_speed = speed;
        }
        Ok(())
    })?)?;

    methods.set("GetBlinkSpeed", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.editbox_blink_speed).unwrap_or(0.5))
    })?)?;

    methods.set("SetCountInvisibleLetters", lua.create_function(|lua, (ud, count): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_count_invisible_letters = count;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn add_editbox_history_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("AddHistoryLine", lua.create_function(|lua, (ud, text): (LightUserData, String)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_history.push(text);
            let max = frame.editbox_history_max;
            if max > 0 && frame.editbox_history.len() > max as usize {
                frame.editbox_history.remove(0);
            }
        }
        Ok(())
    })?)?;

    methods.set("GetHistoryLines", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let count = state.widgets.get(id)
            .map(|f| f.editbox_history.len())
            .unwrap_or(0);
        Ok(count as i32)
    })?)?;

    methods.set("SetHistoryLines", lua.create_function(|lua, (ud, max): (LightUserData, i32)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_history_max = max;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn add_editbox_inset_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetTextInsets", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let mut it = args.into_iter();
        let l = val_to_f32(it.next(), 0.0);
        let r = val_to_f32(it.next(), 0.0);
        let t = val_to_f32(it.next(), 0.0);
        let b = val_to_f32(it.next(), 0.0);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.editbox_text_insets = (l, r, t, b);
        }
        Ok(())
    })?)?;

    methods.set("GetTextInsets", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            let (l, r, t, b) = frame.editbox_text_insets;
            return Ok((l, r, t, b));
        }
        Ok((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32))
    })?)?;

    Ok(())
}

/// Fire a focus-related script handler (OnEditFocusGained/OnEditFocusLost).
fn fire_focus_handler(lua: &mlua::Lua, frame_id: u64, handler: &str) -> mlua::Result<()> {
    fire_tooltip_script(lua, frame_id, handler)
}
