//! ScrollFrame and ScrollBox widget methods.

use super::widget_tooltip::fire_tooltip_script;
use crate::lua_api::frame::handle::{extract_frame_id, frame_lud, get_sim_state, lud_to_id};
use mlua::{LightUserData, Lua, Value};

pub fn add_scrollframe_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_scrollframe_child_methods(lua, methods)?;
    add_scrollframe_offset_methods(lua, methods)?;
    add_scrollframe_range_methods(lua, methods)?;
    Ok(())
}

pub fn add_scrollbox_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("RegisterCallback", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("ForEachFrame", lua.create_function(|_, (_ud, _cb): (LightUserData, mlua::Function)| Ok(()))?)?;
    methods.set("UnregisterCallback", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("CanInterpolateScroll", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    methods.set("SetInterpolateScroll", lua.create_function(|_, (_ud, _enabled): (LightUserData, bool)| Ok(()))?)?;
    Ok(())
}

fn add_scrollframe_child_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetScrollChild", lua.create_function(|lua, (ud, child): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        let child_id = extract_frame_id(&child);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.scroll_child_id = child_id;
        }
        Ok(())
    })?)?;

    methods.set("GetScrollChild", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let child_id = {
            let state_rc = get_sim_state(lua);
            let state = state_rc.borrow();
            state.widgets.get(id).and_then(|f| f.scroll_child_id)
        };
        match child_id {
            Some(cid) => Ok(frame_lud(cid)),
            None => Ok(Value::Nil),
        }
    })?)?;

    methods.set("UpdateScrollChildRect", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    Ok(())
}

fn add_scrollframe_offset_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetHorizontalScroll", lua.create_function(|lua, (ud, offset): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.scroll_horizontal = offset;
        }
        Ok(())
    })?)?;

    methods.set("GetHorizontalScroll", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.scroll_horizontal).unwrap_or(0.0))
    })?)?;

    methods.set("SetVerticalScroll", lua.create_function(|lua, (ud, offset): (LightUserData, f64)| {
        let id = lud_to_id(ud);
        {
            let state_rc = get_sim_state(lua);
            let mut state = state_rc.borrow_mut();
            if let Some(frame) = state.widgets.get_mut_visual(id) {
                frame.scroll_vertical = offset;
            }
        }
        fire_tooltip_script(lua, id, "OnScrollRangeChanged")?;
        Ok(())
    })?)?;

    methods.set("GetVerticalScroll", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.scroll_vertical).unwrap_or(0.0))
    })?)?;

    Ok(())
}

fn add_scrollframe_range_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetHorizontalScrollRange", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let frame = match state.widgets.get(id) {
            Some(f) => f,
            None => return Ok(0.0_f64),
        };
        let child_width = frame.scroll_child_id
            .and_then(|cid| state.widgets.get(cid))
            .map(|c| c.width as f64)
            .unwrap_or(0.0);
        Ok((child_width - frame.width as f64).max(0.0))
    })?)?;

    methods.set("GetVerticalScrollRange", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        let frame = match state.widgets.get(id) {
            Some(f) => f,
            None => return Ok(0.0_f64),
        };
        let child_height = frame.scroll_child_id
            .and_then(|cid| state.widgets.get(cid))
            .map(|c| c.height as f64)
            .unwrap_or(0.0);
        Ok((child_height - frame.height as f64).max(0.0))
    })?)?;

    Ok(())
}
