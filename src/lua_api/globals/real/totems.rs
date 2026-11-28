//! Host-supplied totem slots share one snapshot across info and duration queries.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

#[derive(Debug, Clone)]
pub struct Totem {
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
    pub icon: u32,
    pub mod_rate: f64,
    pub spell_id: u32,
    /// Server flag read by `GetTotemCannotDismiss`; dismissal requests are refused.
    pub cannot_dismiss: bool,
}

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetTotemInfo", get_totem_info)?;
    if cfg!(feature = "retail-12-0-5") || ACTIVE == ClientProfile::WowForever {
        LuaApiMut::register_function(lua, "GetNumTotemSlots", get_num_totem_slots)?;
    }
    #[cfg(feature = "retail-12-0-5")]
    LuaApiMut::register_function(lua, "GetTotemDuration", get_totem_duration)?;
    #[cfg(feature = "totem-dismissal")]
    dismissal::register(lua)?;
    Ok(())
}

fn read_totem(state: &LuaState) -> LuaResult<Option<Totem>> {
    let slot = u32::from_stack(state, 1)? as usize;
    let sim = borrow_state(state)?;
    let totem = slot
        .checked_sub(1)
        .and_then(|slot| sim.totem_slots.get(slot))
        .and_then(|slot| slot.as_ref());
    let now = sim.clock_elapsed().as_secs_f64();
    Ok(totem
        .filter(|totem| totem.duration > 0.0 && now < totem.start_time + totem.duration)
        .cloned())
}

fn get_totem_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(totem) = read_totem(state)? else {
        state.push(Val::Bool(false));
        state.push(Val::Nil);
        state.push(Val::Num(0.0));
        state.push(Val::Num(0.0));
        state.push(Val::Nil);
        return Ok(5);
    };

    state.push(Val::Bool(true));
    let name = create_string(state, &totem.name);
    state.push(name);
    state.push(Val::Num(totem.start_time));
    state.push(Val::Num(totem.duration));
    state.push(Val::Num(f64::from(totem.icon)));
    if ACTIVE == ClientProfile::WowForever {
        state.push(Val::Num(totem.mod_rate));
        state.push(Val::Num(f64::from(totem.spell_id)));
        return Ok(7);
    }
    Ok(5)
}

fn get_num_totem_slots(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.totem_slots.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn get_totem_duration(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: inactive/out-of-range slots have an empty duration object.
    let (start, seconds) =
        read_totem(state)?.map_or((0.0, 0.0), |totem| (totem.start_time, totem.duration));
    crate::lua_api::globals::lua_duration_object::push_timed_duration_object(state, start, seconds)
}

/// Remaining time, dismissal flag and `DestroyTotem` for the supplied slots.
///
/// Declarations: Retail and Forever `TotemDocumentation.lua` (byte-identical).
/// Consumer: `Blizzard_UnitFrame/Mainline/TotemFrame.lua` (`OnUpdate` shows
/// `math.ceil(GetTotemTimeLeft(slot))`; a right click calls `DestroyTotem(slot)`
/// unless `GetTotemCannotDismiss(slot)`), refreshed by `PLAYER_TOTEM_UPDATE(slot)`.
/// INFERRED (R-ag10-6): an empty or expired slot has 0 seconds left and a nil
/// dismissal flag; destroying an empty or undismissable slot changes nothing.
#[cfg(feature = "totem-dismissal")]
pub(crate) mod dismissal {
    use super::{Totem, read_totem};
    use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
    use crate::lua_api::methods::{borrow_state, borrow_state_mut};
    use crate::lua_bridge::{FromStack, TableBuilder};
    use rilua::vm::state::LuaState;
    use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

    const SET_USAGE: &str =
        "Usage: A_Admin.SetTotem(slot, name|nil, duration, icon, spellID, cannotDismiss)";

    fn slot_index(state: &LuaState, slot: i64) -> LuaResult<usize> {
        let count = borrow_state(state)?.totem_slots.len() as i64;
        if !(1..=count).contains(&slot) {
            return Err(runtime_error(SET_USAGE));
        }
        Ok((slot - 1) as usize)
    }

    fn read_supplied_totem(state: &LuaState) -> LuaResult<Option<Totem>> {
        let Some(name) = Option::<String>::from_stack(state, 2)? else {
            return Ok(None);
        };
        let duration = f64::from_stack(state, 3)?;
        if name.is_empty() || !duration.is_finite() || duration <= 0.0 {
            return Err(runtime_error(SET_USAGE));
        }
        Ok(Some(Totem {
            name,
            start_time: borrow_state(state)?.clock_elapsed().as_secs_f64(),
            duration,
            icon: u32::from_stack(state, 4)?,
            mod_rate: 1.0,
            spell_id: u32::from_stack(state, 5)?,
            cannot_dismiss: Option::<bool>::from_stack(state, 6)?.unwrap_or(false),
        }))
    }

    /// Summons (or with a nil name, removes) the totem in `slot`, starting now.
    fn set_totem(state: &mut LuaState) -> LuaResult<u32> {
        let slot = i64::from_stack(state, 1)?;
        let index = slot_index(state, slot)?;
        let totem = read_supplied_totem(state)?;
        borrow_state_mut(state)?.totem_slots[index] = totem;
        dispatch_event_now(state, "PLAYER_TOTEM_UPDATE", &[Val::Num(slot as f64)])?;
        Ok(0)
    }

    fn destroy_totem(state: &mut LuaState) -> LuaResult<u32> {
        let slot = i64::from_stack(state, 1)?;
        let Some(totem) = read_totem(state)? else {
            return Ok(0);
        };
        if totem.cannot_dismiss {
            return Ok(0);
        }
        borrow_state_mut(state)?.totem_slots[(slot - 1) as usize] = None;
        dispatch_event_now(state, "PLAYER_TOTEM_UPDATE", &[Val::Num(slot as f64)])?;
        Ok(0)
    }

    fn get_totem_time_left(state: &mut LuaState) -> LuaResult<u32> {
        let now = borrow_state(state)?.clock_elapsed().as_secs_f64();
        let left = read_totem(state)?.map_or(0.0, |totem| {
            (totem.start_time + totem.duration - now).max(0.0)
        });
        state.push(Val::Num(left));
        Ok(1)
    }

    fn get_totem_cannot_dismiss(state: &mut LuaState) -> LuaResult<u32> {
        let flag = read_totem(state)?.map_or(Val::Nil, |totem| Val::Bool(totem.cannot_dismiss));
        state.push(flag);
        Ok(1)
    }

    pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
        LuaApiMut::register_function(lua, "DestroyTotem", destroy_totem)?;
        LuaApiMut::register_function(lua, "GetTotemTimeLeft", get_totem_time_left)?;
        LuaApiMut::register_function(lua, "GetTotemCannotDismiss", get_totem_cannot_dismiss)
    }

    pub(crate) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
        b.set_function("SetTotem", set_totem)
    }
}
