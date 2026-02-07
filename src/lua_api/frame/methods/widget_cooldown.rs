//! Cooldown widget methods: SetCooldown, swipe/edge/bling display, pause/resume.

use super::widget_tooltip::val_to_f32;
use super::FrameHandle;
use crate::widget::AttributeValue;
use mlua::{UserDataMethods, Value};

pub fn add_cooldown_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_cooldown_timing_methods(methods);
    add_cooldown_display_methods(methods);
    add_cooldown_state_methods(methods);
}

fn add_cooldown_timing_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetCooldown", |_, this, args: mlua::MultiValue| {
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
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_start = start;
            frame.cooldown_duration = duration;
        }
        Ok(())
    });
    methods.add_method("SetCooldownUNIX", |_, this, args: mlua::MultiValue| {
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
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_start = start;
            frame.cooldown_duration = end - start;
        }
        Ok(())
    });
    methods.add_method("GetCooldownTimes", |_, this, ()| {
        let state = this.state.borrow();
        if let Some(frame) = state.widgets.get(this.id) {
            return Ok((frame.cooldown_start, frame.cooldown_duration));
        }
        Ok((0.0_f64, 0.0_f64))
    });
    methods.add_method("GetCooldownDuration", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.cooldown_duration).unwrap_or(0.0))
    });
}

fn add_cooldown_display_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetSwipeColor", |_, this, args: mlua::MultiValue| {
        let mut it = args.into_iter();
        let r = val_to_f32(it.next(), 0.0);
        let g = val_to_f32(it.next(), 0.0);
        let b = val_to_f32(it.next(), 0.0);
        let a = val_to_f32(it.next(), 0.8);
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.attributes.insert(
                "__swipe_color".to_string(),
                AttributeValue::String(format!("{},{},{},{}", r, g, b, a)),
            );
        }
        Ok(())
    });
    methods.add_method("SetDrawSwipe", |_, this, draw: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_draw_swipe = draw;
        }
        Ok(())
    });
    methods.add_method("SetDrawEdge", |_, this, draw: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_draw_edge = draw;
        }
        Ok(())
    });
    methods.add_method("SetDrawBling", |_, this, draw: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_draw_bling = draw;
        }
        Ok(())
    });
    methods.add_method("SetReverse", |_, this, reverse: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_reverse = reverse;
        }
        Ok(())
    });
    methods.add_method("SetHideCountdownNumbers", |_, this, hide: bool| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_hide_countdown = hide;
        }
        Ok(())
    });
    // Note: Clear() for Cooldown frames is handled in __index to avoid conflicts
    // with addons that use frame.Clear as a field
}

fn add_cooldown_state_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("Pause", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_paused = true;
        }
        Ok(())
    });
    methods.add_method("Resume", |_, this, ()| {
        let mut state = this.state.borrow_mut();
        if let Some(frame) = state.widgets.get_mut(this.id) {
            frame.cooldown_paused = false;
        }
        Ok(())
    });
    methods.add_method("IsPaused", |_, this, ()| {
        let state = this.state.borrow();
        Ok(state.widgets.get(this.id).map(|f| f.cooldown_paused).unwrap_or(false))
    });
}
