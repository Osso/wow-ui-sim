//! TotemDocumentation time-left, dismissal and destroy queries over host totem slots.
//! Retail/Forever TotemDocumentation blocks are byte-identical; the original consumer
//! is Blizzard_UnitFrame/Mainline/TotemFrame.lua (OnUpdate, OnClick).
//! INFERRED: DestroyTotem applies the server's slot removal locally (a server-response
//! stand-in) and notifies PLAYER_TOTEM_UPDATE(slot), the event TotemFrame refreshes on.

use super::{Totem, active_totem};
use crate::lua_api::globals::real::admin_fields as fields;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

pub(super) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetTotemTimeLeft", get_totem_time_left)?;
    LuaApiMut::register_function(lua, "GetTotemCannotDismiss", get_totem_cannot_dismiss)?;
    LuaApiMut::register_function(lua, "DestroyTotem", destroy_totem)
}

fn read_slot(state: &LuaState) -> LuaResult<usize> {
    Ok(u32::from_stack(state, 1)? as usize)
}

/// Seconds until the live totem expires; an empty slot reports zero (R-question R9-1).
fn get_totem_time_left(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let left = match active_totem(state, slot)? {
        Some(totem) => {
            let now = borrow_state(state)?.clock_elapsed().as_secs_f64();
            (totem.start_time + totem.duration - now).max(0.0)
        }
        None => 0.0,
    };
    state.push(Val::Num(left));
    Ok(1)
}

fn cannot_dismiss(state: &LuaState, slot: usize) -> LuaResult<bool> {
    Ok(borrow_state(state)?.totem_cannot_dismiss.contains(&slot))
}

/// Nilable result: only a live totem carries the supplied dismissal flag.
fn get_totem_cannot_dismiss(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let value = match active_totem(state, slot)? {
        Some(_) => Val::Bool(cannot_dismiss(state, slot)?),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

fn destroy_totem(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    if active_totem(state, slot)?.is_none() || cannot_dismiss(state, slot)? {
        return Ok(0);
    }
    store_totem(state, slot, None)?;
    Ok(0)
}

fn store_totem(state: &mut LuaState, slot: usize, totem: Option<(Totem, bool)>) -> LuaResult<()> {
    {
        let mut sim = borrow_state_mut(state)?;
        let cannot = totem.as_ref().is_some_and(|(_, cannot)| *cannot);
        sim.totem_slots[slot - 1] = totem.map(|(totem, _)| totem);
        if cannot {
            sim.totem_cannot_dismiss.insert(slot);
        } else {
            sim.totem_cannot_dismiss.remove(&slot);
        }
    }
    dispatch_event_now(state, "PLAYER_TOTEM_UPDATE", &[Val::Num(slot as f64)])
}

/// `A_Admin.SetTotem(slot, info|nil)`: supplies or clears one host totem slot.
pub(crate) fn admin_set_totem(state: &mut LuaState) -> LuaResult<u32> {
    let slot = admin_slot(state)?;
    let totem = match stack_val(state, 2) {
        Val::Nil => None,
        table @ Val::Table(_) => Some(read_totem_input(state, table)?),
        _ => {
            return Err(runtime_error(
                "A_Admin.SetTotem expects an info table or nil",
            ));
        }
    };
    store_totem(state, slot, totem)?;
    Ok(0)
}

fn admin_slot(state: &LuaState) -> LuaResult<usize> {
    let count = borrow_state(state)?.totem_slots.len();
    match stack_val(state, 1) {
        Val::Num(slot) if slot.fract() == 0.0 && slot >= 1.0 && slot <= count as f64 => {
            Ok(slot as usize)
        }
        _ => Err(runtime_error(format!(
            "A_Admin.SetTotem slot must be an integer in 1..{count}"
        ))),
    }
}

fn read_totem_input(state: &mut LuaState, table: Val) -> LuaResult<(Totem, bool)> {
    const API: &str = "A_Admin.SetTotem";
    let totem = Totem {
        name: fields::string(state, table, API, "name")?,
        start_time: fields::finite(state, table, API, "startTime")?,
        duration: fields::positive(state, table, API, "duration")?,
        icon: fields::unsigned(state, table, API, "icon")?,
        mod_rate: fields::positive(state, table, API, "modRate")?,
        spell_id: fields::unsigned(state, table, API, "spellID")?,
    };
    let cannot = fields::boolean(state, table, API, "cannotDismiss")?;
    Ok((totem, cannot))
}
