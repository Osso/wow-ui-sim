//! C_Timer namespace for WoW timer functionality.
//!
//! Provides timer creation and management functions used by addons.

use super::super::{next_timer_id, PendingTimer, SimState};
use mlua::{Lua, Result};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Register C_Timer namespace and timer-related functions.
pub fn register_timer_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let c_timer = lua.create_table()?;

    c_timer.set("After", create_timer_after(lua, Rc::clone(&state))?)?;
    c_timer.set("NewTicker", create_new_ticker(lua, Rc::clone(&state))?)?;
    c_timer.set("NewTimer", create_new_timer(lua, Rc::clone(&state))?)?;

    lua.globals().set("C_Timer", c_timer)?;
    Ok(())
}

/// C_Timer.After(seconds, callback) - one-shot timer, no handle returned.
fn create_timer_after(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    lua.create_function(move |lua, (seconds, callback): (f64, mlua::Function)| {
        let id = next_timer_id();
        let callback_key = lua.create_registry_value(callback)?;
        let fire_at = Instant::now() + Duration::from_secs_f64(seconds);

        let timer = PendingTimer {
            id,
            fire_at,
            callback_key,
            interval: None,
            remaining: None,
            cancelled: false,
            handle_key: None,
        };

        state.borrow_mut().timers.push_back(timer);
        Ok(())
    })
}

/// C_Timer.NewTicker(seconds, callback, iterations) - repeating timer with handle.
fn create_new_ticker(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    lua.create_function(
        move |lua, (seconds, callback, iterations): (f64, mlua::Function, Option<i32>)| {
            let id = next_timer_id();
            let callback_key = lua.create_registry_value(callback)?;
            let fire_at = Instant::now() + Duration::from_secs_f64(seconds);
            let interval = Duration::from_secs_f64(seconds);

            let ticker = create_timer_handle(lua, id, &state)?;
            let handle_key = lua.create_registry_value(ticker.clone())?;

            let timer = PendingTimer {
                id,
                fire_at,
                callback_key,
                interval: Some(interval),
                remaining: iterations,
                cancelled: false,
                handle_key: Some(handle_key),
            };

            state.borrow_mut().timers.push_back(timer);
            Ok(ticker)
        },
    )
}

/// C_Timer.NewTimer(seconds, callback) - one-shot timer with handle.
fn create_new_timer(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    lua.create_function(move |lua, (seconds, callback): (f64, mlua::Function)| {
        let id = next_timer_id();
        let callback_key = lua.create_registry_value(callback)?;
        let fire_at = Instant::now() + Duration::from_secs_f64(seconds);

        let timer_handle = create_timer_handle(lua, id, &state)?;
        let handle_key = lua.create_registry_value(timer_handle.clone())?;

        let timer = PendingTimer {
            id,
            fire_at,
            callback_key,
            interval: None,
            remaining: None,
            cancelled: false,
            handle_key: Some(handle_key),
        };

        state.borrow_mut().timers.push_back(timer);
        Ok(timer_handle)
    })
}

/// Create a timer handle table with Cancel and IsCancelled methods.
fn create_timer_handle(
    lua: &Lua,
    id: u64,
    state: &Rc<RefCell<SimState>>,
) -> Result<mlua::Table> {
    let handle = lua.create_table()?;
    handle.set("_id", id)?;
    handle.set("_cancelled", false)?;

    let state_cancel = Rc::clone(state);
    let handle_clone = handle.clone();
    let cancel = lua.create_function(move |_, ()| {
        handle_clone.set("_cancelled", true)?;
        let mut state = state_cancel.borrow_mut();
        for timer in state.timers.iter_mut() {
            if timer.id == id {
                timer.cancelled = true;
                break;
            }
        }
        Ok(())
    })?;
    handle.set("Cancel", cancel)?;

    let handle_for_check = handle.clone();
    let is_cancelled = lua.create_function(move |_, ()| {
        let cancelled: bool = handle_for_check.get("_cancelled").unwrap_or(false);
        Ok(cancelled)
    })?;
    handle.set("IsCancelled", is_cancelled)?;

    Ok(handle)
}
