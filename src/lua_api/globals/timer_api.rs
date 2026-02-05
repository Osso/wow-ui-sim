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
    let globals = lua.globals();

    let c_timer = lua.create_table()?;

    // C_Timer.After(seconds, callback) - one-shot timer, no handle returned
    let state_timer_after = Rc::clone(&state);
    let c_timer_after =
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
                handle_key: None, // After() doesn't pass handle to callback
            };

            state_timer_after.borrow_mut().timers.push_back(timer);
            Ok(())
        })?;
    c_timer.set("After", c_timer_after)?;

    // C_Timer.NewTicker(seconds, callback, iterations) - repeating timer with handle
    let state_timer_ticker = Rc::clone(&state);
    let c_timer_new_ticker = lua.create_function(
        move |lua, (seconds, callback, iterations): (f64, mlua::Function, Option<i32>)| {
            let id = next_timer_id();
            let callback_key = lua.create_registry_value(callback)?;
            let fire_at = Instant::now() + Duration::from_secs_f64(seconds);
            let interval = Duration::from_secs_f64(seconds);

            // Create the ticker handle table first so we can pass it to callbacks
            let ticker = lua.create_table()?;
            ticker.set("_id", id)?;
            ticker.set("_cancelled", false)?;

            let state_cancel = Rc::clone(&state_timer_ticker);
            let ticker_clone = ticker.clone();
            let cancel = lua.create_function(move |_, ()| {
                // Mark as cancelled in the handle table
                ticker_clone.set("_cancelled", true)?;
                // Also mark in the timer queue
                let mut state = state_cancel.borrow_mut();
                for timer in state.timers.iter_mut() {
                    if timer.id == id {
                        timer.cancelled = true;
                        break;
                    }
                }
                Ok(())
            })?;
            ticker.set("Cancel", cancel)?;

            // IsCancelled method checks the _cancelled field
            let ticker_for_is_cancelled = ticker.clone();
            let is_cancelled = lua.create_function(move |_, ()| {
                let cancelled: bool = ticker_for_is_cancelled.get("_cancelled").unwrap_or(false);
                Ok(cancelled)
            })?;
            ticker.set("IsCancelled", is_cancelled)?;

            // Store the handle in registry so we can pass it to callback
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

            state_timer_ticker.borrow_mut().timers.push_back(timer);

            Ok(ticker)
        },
    )?;
    c_timer.set("NewTicker", c_timer_new_ticker)?;

    // C_Timer.NewTimer(seconds, callback) - one-shot timer with handle
    let state_timer_new = Rc::clone(&state);
    let c_timer_new_timer =
        lua.create_function(move |lua, (seconds, callback): (f64, mlua::Function)| {
            let id = next_timer_id();
            let callback_key = lua.create_registry_value(callback)?;
            let fire_at = Instant::now() + Duration::from_secs_f64(seconds);

            // Create the timer handle table first so we can pass it to callback
            let timer_handle = lua.create_table()?;
            timer_handle.set("_id", id)?;
            timer_handle.set("_cancelled", false)?;

            let state_cancel = Rc::clone(&state_timer_new);
            let handle_clone = timer_handle.clone();
            let cancel = lua.create_function(move |_, ()| {
                // Mark as cancelled in the handle table
                handle_clone.set("_cancelled", true)?;
                // Also mark in the timer queue
                let mut state = state_cancel.borrow_mut();
                for timer in state.timers.iter_mut() {
                    if timer.id == id {
                        timer.cancelled = true;
                        break;
                    }
                }
                Ok(())
            })?;
            timer_handle.set("Cancel", cancel)?;

            // IsCancelled method checks the _cancelled field
            let handle_for_is_cancelled = timer_handle.clone();
            let is_cancelled = lua.create_function(move |_, ()| {
                let cancelled: bool = handle_for_is_cancelled.get("_cancelled").unwrap_or(false);
                Ok(cancelled)
            })?;
            timer_handle.set("IsCancelled", is_cancelled)?;

            // Store the handle in registry so we can pass it to callback
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

            state_timer_new.borrow_mut().timers.push_back(timer);

            Ok(timer_handle)
        })?;
    c_timer.set("NewTimer", c_timer_new_timer)?;

    globals.set("C_Timer", c_timer)?;

    Ok(())
}
