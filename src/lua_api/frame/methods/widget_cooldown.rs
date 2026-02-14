//! Cooldown widget methods: SetCooldown, swipe/edge/bling display, pause/resume.

use super::widget_tooltip::val_to_f32;
use crate::lua_api::frame::handle::{get_sim_state, lud_to_id};
use crate::widget::AttributeValue;
use mlua::{LightUserData, Lua, Value};

pub fn add_cooldown_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    add_cooldown_set_methods(lua, methods)?;
    add_cooldown_get_methods(lua, methods)?;
    add_cooldown_display_methods(lua, methods)?;
    add_cooldown_bool_display_methods(lua, methods)?;
    add_cooldown_texture_methods(lua, methods)?;
    add_cooldown_state_methods(lua, methods)?;
    Ok(())
}

fn add_cooldown_set_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetCooldown", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let mut it = args.into_iter();
        let start = match it.next() {
            Some(Value::Number(n)) => n,
            Some(Value::Integer(n)) => n as f64,
            _ => 0.0,
        };
        let duration = match it.next() {
            Some(Value::Number(n)) => n,
            Some(Value::Integer(n)) => n as f64,
            _ => 0.0,
        };
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_start = start;
            frame.cooldown_duration = duration;
        }
        Ok(())
    })?)?;

    methods.set("SetCooldownUNIX", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let mut it = args.into_iter();
        let start = match it.next() {
            Some(Value::Number(n)) => n,
            Some(Value::Integer(n)) => n as f64,
            _ => 0.0,
        };
        let end = match it.next() {
            Some(Value::Number(n)) => n,
            Some(Value::Integer(n)) => n as f64,
            _ => 0.0,
        };
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_start = start;
            frame.cooldown_duration = end - start;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn add_cooldown_get_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("GetCooldownTimes", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        if let Some(frame) = state.widgets.get(id) {
            return Ok((frame.cooldown_start, frame.cooldown_duration));
        }
        Ok((0.0_f64, 0.0_f64))
    })?)?;

    methods.set("GetCooldownDuration", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.cooldown_duration).unwrap_or(0.0))
    })?)?;

    Ok(())
}

fn add_cooldown_display_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetSwipeColor", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let mut it = args.into_iter();
        let r = val_to_f32(it.next(), 0.0);
        let g = val_to_f32(it.next(), 0.0);
        let b = val_to_f32(it.next(), 0.0);
        let a = val_to_f32(it.next(), 0.8);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.attributes.insert(
                "__swipe_color".to_string(),
                AttributeValue::String(format!("{},{},{},{}", r, g, b, a)),
            );
        }
        Ok(())
    })?)?;

    methods.set("SetHideCountdownNumbers", lua.create_function(|lua, (ud, hide): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_hide_countdown = hide;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn add_cooldown_bool_display_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetDrawSwipe", lua.create_function(|lua, (ud, draw): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_draw_swipe = draw;
        }
        Ok(())
    })?)?;

    methods.set("SetDrawEdge", lua.create_function(|lua, (ud, draw): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_draw_edge = draw;
        }
        Ok(())
    })?)?;

    methods.set("SetDrawBling", lua.create_function(|lua, (ud, draw): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_draw_bling = draw;
        }
        Ok(())
    })?)?;

    methods.set("SetReverse", lua.create_function(|lua, (ud, reverse): (LightUserData, bool)| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_reverse = reverse;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn add_cooldown_texture_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("SetEdgeTexture", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetSwipeTexture", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetBlingTexture", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetEdgeScale", lua.create_function(|_, (_ud, _scale): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetUseCircularEdge", lua.create_function(|_, (_ud, _use_circular): (LightUserData, bool)| Ok(()))?)?;
    methods.set("SetCountdownAbbrevThreshold", lua.create_function(|_, (_ud, _seconds): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetCountdownFont", lua.create_function(|_, (_ud, _font): (LightUserData, Value)| Ok(()))?)?;
    methods.set("SetUseAuraDisplayTime", lua.create_function(|_, (_ud, _use): (LightUserData, Value)| Ok(()))?)?;

    methods.set("GetReverse", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.cooldown_reverse).unwrap_or(false))
    })?)?;

    methods.set("SetCooldownDuration", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        let mut it = args.into_iter();
        let duration = match it.next() {
            Some(Value::Number(n)) => n,
            Some(Value::Integer(n)) => n as f64,
            _ => 0.0,
        };
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_duration = duration;
        }
        Ok(())
    })?)?;

    // Note: Clear() for Cooldown frames is handled in __index to avoid conflicts
    // with addons that use frame.Clear as a field
    Ok(())
}

fn add_cooldown_state_methods(lua: &Lua, methods: &mlua::Table) -> mlua::Result<()> {
    methods.set("Pause", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_paused = true;
        }
        Ok(())
    })?)?;

    methods.set("Resume", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.cooldown_paused = false;
        }
        Ok(())
    })?)?;

    methods.set("IsPaused", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        let state_rc = get_sim_state(lua);
        let state = state_rc.borrow();
        Ok(state.widgets.get(id).map(|f| f.cooldown_paused).unwrap_or(false))
    })?)?;

    Ok(())
}
