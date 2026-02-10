//! Action bar functions: queries, cooldowns, UseAction, and stubs.

use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register all action bar functions (stateful + stubs).
pub fn register_action_bar_functions(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    register_action_bar_queries(lua, &state)?;
    register_action_cooldown(lua, &state)?;
    register_use_action(lua, &state)?;
    register_action_bar_stubs(lua, &state)?;
    Ok(())
}

/// Extract a slot number from a Lua Value (integer or number).
fn slot_from_value(v: &Value) -> Option<u32> {
    match v {
        Value::Integer(n) => Some(*n as u32),
        Value::Number(n) => Some(*n as u32),
        _ => None,
    }
}

/// Look up the texture path for an action bar slot.
fn action_texture_path(state: &SimState, slot: u32) -> Option<String> {
    let spell_id = state.action_bars.get(&slot)?;
    let spell = crate::spells::get_spell(*spell_id)?;
    let path = crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id)?;
    Some(format!("Interface\\{}", path.replace('/', "\\")))
}

/// HasAction, GetActionInfo, GetActionTexture, IsUsableAction.
fn register_action_bar_queries(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    let st = Rc::clone(state);
    globals.set(
        "HasAction",
        lua.create_function(move |_, slot: Value| {
            let s = st.borrow();
            Ok(slot_from_value(&slot).is_some_and(|n| s.action_bars.contains_key(&n)))
        })?,
    )?;

    let st = Rc::clone(state);
    globals.set(
        "GetActionInfo",
        lua.create_function(move |lua, slot: Value| {
            let s = st.borrow();
            let Some(n) = slot_from_value(&slot) else {
                return Ok(mlua::MultiValue::new());
            };
            let Some(&spell_id) = s.action_bars.get(&n) else {
                return Ok(mlua::MultiValue::new());
            };
            Ok(mlua::MultiValue::from_vec(vec![
                Value::String(lua.create_string("spell")?),
                Value::Integer(spell_id as i64),
                Value::String(lua.create_string("spell")?),
            ]))
        })?,
    )?;

    let st = Rc::clone(state);
    globals.set(
        "GetActionTexture",
        lua.create_function(move |lua, slot: Value| {
            let s = st.borrow();
            let Some(n) = slot_from_value(&slot) else {
                return Ok(Value::Nil);
            };
            match action_texture_path(&s, n) {
                Some(path) => Ok(Value::String(lua.create_string(&path)?)),
                None => Ok(Value::Nil),
            }
        })?,
    )?;

    let st = Rc::clone(state);
    globals.set(
        "IsUsableAction",
        lua.create_function(move |_, slot: Value| {
            let s = st.borrow();
            let has = slot_from_value(&slot).is_some_and(|n| s.action_bars.contains_key(&n));
            Ok((has, false))
        })?,
    )?;

    Ok(())
}

/// GetActionCooldown(slot) — returns (start, duration, enable, modRate).
fn register_action_cooldown(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let st = Rc::clone(state);
    lua.globals().set(
        "GetActionCooldown",
        lua.create_function(move |_, slot: Value| {
            let s = st.borrow();
            let Some(n) = slot_from_value(&slot) else {
                return Ok((0.0_f64, 0.0_f64, 1, 1.0_f64));
            };
            let (start, duration) = action_cooldown_times(&s, n);
            Ok((start, duration, 1, 1.0_f64))
        })?,
    )?;
    Ok(())
}

/// Look up the active cooldown for an action bar slot.
fn action_cooldown_times(state: &SimState, slot: u32) -> (f64, f64) {
    let now = state.start_time.elapsed().as_secs_f64();
    let spell_id = match state.action_bars.get(&slot) {
        Some(&id) => id,
        None => return (0.0, 0.0),
    };
    spell_cooldown_times(state, spell_id, now)
}

/// Look up the active cooldown for a spell.
/// Returns (start, duration) — the GCD or spell CD, whichever ends later.
pub fn spell_cooldown_times(state: &SimState, spell_id: u32, now: f64) -> (f64, f64) {
    let mut best_start = 0.0_f64;
    let mut best_end = 0.0_f64;

    // Check GCD
    if let Some((gcd_start, gcd_dur)) = state.gcd {
        let gcd_end = gcd_start + gcd_dur;
        if gcd_end > now {
            best_start = gcd_start;
            best_end = gcd_end;
        }
    }

    // Check per-spell cooldown
    if let Some(cd) = state.spell_cooldowns.get(&spell_id) {
        let cd_end = cd.start + cd.duration;
        if cd_end > now && cd_end > best_end {
            best_start = cd.start;
            best_end = cd_end;
        }
    }

    if best_end > now {
        (best_start, best_end - best_start)
    } else {
        (0.0, 0.0)
    }
}

/// UseAction(slot) — look up spell, start cast or apply instant effect.
fn register_use_action(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let st = Rc::clone(state);
    lua.globals().set(
        "UseAction",
        lua.create_function(move |lua, (slot, _unit, _button): (i32, Value, Value)| {
            let spell_id = {
                let s = st.borrow();
                s.action_bars.get(&(slot as u32)).copied()
            };
            let Some(spell_id) = spell_id else {
                return Ok(());
            };
            if st.borrow().casting.is_some() {
                return Ok(());
            }
            let cast_time_ms = super::spell_api::spell_cast_time(spell_id as i32);
            if cast_time_ms > 0 {
                start_cast(&st, lua, spell_id, cast_time_ms)?;
            } else {
                apply_instant_spell(&st, lua, spell_id)?;
            }
            start_cooldowns(&st, lua, spell_id)?;
            Ok(())
        })?,
    )
}

/// Start GCD and per-spell cooldown after using an action.
pub fn start_cooldowns(
    state: &Rc<RefCell<SimState>>,
    lua: &Lua,
    spell_id: u32,
) -> Result<()> {
    use crate::lua_api::game_data::{
        GCD_DURATION, SpellCooldownState,
        spell_cooldown_duration, spell_triggers_gcd,
    };

    let now = state.borrow().start_time.elapsed().as_secs_f64();
    {
        let mut s = state.borrow_mut();
        if spell_triggers_gcd(spell_id) {
            s.gcd = Some((now, GCD_DURATION));
        }
        let cd = spell_cooldown_duration(spell_id);
        if cd > 0.0 {
            s.spell_cooldowns.insert(spell_id, SpellCooldownState {
                start: now,
                duration: cd,
            });
        }
    }

    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call::<()>(lua.create_string("SPELL_UPDATE_COOLDOWN")?)?;
    fire.call::<()>(lua.create_string("ACTIONBAR_UPDATE_COOLDOWN")?)?;
    Ok(())
}

/// Start a cast-time spell: store CastingState, fire UNIT_SPELLCAST_START.
pub fn start_cast(
    state: &Rc<RefCell<SimState>>,
    lua: &Lua,
    spell_id: u32,
    cast_time_ms: i32,
) -> Result<()> {
    use crate::lua_api::state::CastingState;

    let spell = crate::spells::get_spell(spell_id);
    let spell_name = spell.map(|s| s.name).unwrap_or("Unknown").to_string();
    let icon_id = spell.map(|s| s.icon_file_data_id).unwrap_or(136243);
    let icon_path = crate::manifest_interface_data::get_texture_path(icon_id)
        .map(|p| format!("Interface\\{}", p.replace('/', "\\")))
        .unwrap_or_default();

    let cast_id = {
        let mut s = state.borrow_mut();
        let cast_id = s.next_cast_id;
        s.next_cast_id += 1;
        let now = s.start_time.elapsed().as_secs_f64();
        s.casting = Some(CastingState {
            spell_id,
            spell_name: spell_name.clone(),
            icon_path,
            start_time: now,
            end_time: now + cast_time_ms as f64 / 1000.0,
            cast_id,
        });
        cast_id
    };

    eprintln!("[cast] Starting {} (id={}, {:.1}s)", spell_name, cast_id, cast_time_ms as f64 / 1000.0);
    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call::<()>((
        lua.create_string("UNIT_SPELLCAST_START")?,
        lua.create_string("player")?,
        cast_id as i64,
        spell_id as i64,
    ))?;
    // Tell action buttons to re-check IsCurrentAction() for checked state.
    fire.call::<()>(lua.create_string("ACTIONBAR_UPDATE_STATE")?)?;
    Ok(())
}

/// Apply an instant spell effect and fire UNIT_SPELLCAST_SUCCEEDED.
pub fn apply_instant_spell(
    state: &Rc<RefCell<SimState>>,
    lua: &Lua,
    spell_id: u32,
) -> Result<()> {
    let cast_id = {
        let mut s = state.borrow_mut();
        let id = s.next_cast_id;
        s.next_cast_id += 1;
        id
    };

    let spell_name = crate::spells::get_spell(spell_id)
        .map(|s| s.name).unwrap_or("Unknown");
    eprintln!("[cast] Instant {} (id={})", spell_name, cast_id);

    let fire: mlua::Function = lua.globals().get("FireEvent")?;
    fire.call::<()>((
        lua.create_string("UNIT_SPELLCAST_SUCCEEDED")?,
        lua.create_string("player")?,
        cast_id as i64,
        spell_id as i64,
    ))
}

/// Action bar stub functions (mostly stateless, some need state).
fn register_action_bar_stubs(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    register_action_slot_query_stubs(lua, state)?;
    register_action_bar_indices(lua)?;
    register_release_and_highlight_functions(lua)?;
    Ok(())
}

fn register_release_and_highlight_functions(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set(
        "ReleaseAction",
        lua.create_function(|_, (slot, _unit, _button): (i32, Value, Value)| {
            eprintln!("[action] ReleaseAction({})", slot);
            Ok(())
        })?,
    )?;
    globals.set(
        "GetNewActionHighlightMark",
        lua.create_function(|_, _slot: Value| Ok(false))?,
    )?;
    globals.set(
        "ClearNewActionHighlight",
        lua.create_function(|_, _slot: Value| Ok(()))?,
    )?;
    Ok(())
}

/// Action slot query stubs (GetActionCooldown is overridden by stateful version).
fn register_action_slot_query_stubs(lua: &Lua, state: &Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();
    globals.set("GetActionText", lua.create_function(|_, _slot: Value| Ok(Value::Nil))?)?;
    globals.set("GetActionCount", lua.create_function(|_, _slot: Value| Ok(0))?)?;
    globals.set("IsConsumableAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsStackableAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsAttackAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    globals.set("IsAutoRepeatAction", lua.create_function(|_, _slot: Value| Ok(false))?)?;
    let st = Rc::clone(state);
    globals.set("IsCurrentAction", lua.create_function(move |_, slot: Value| {
        let slot = slot_from_value(&slot).unwrap_or(0);
        let state = st.borrow();
        let casting = match &state.casting {
            Some(c) => c.spell_id,
            None => return Ok(false),
        };
        Ok(state.action_bars.get(&slot).copied() == Some(casting))
    })?)?;
    globals.set(
        "GetActionCharges",
        lua.create_function(|_, _slot: Value| Ok((0, 0, 0.0_f64, 0.0_f64, 1.0_f64)))?,
    )?;
    globals.set("GetPossessInfo", lua.create_function(|_, _index: Value| Ok(Value::Nil))?)?;
    globals.set("SetActionUIButton", lua.create_function(|_, _args: mlua::MultiValue| Ok(()))?)?;
    Ok(())
}

/// Action bar page/index stubs.
fn register_action_bar_indices(lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    globals.set("GetActionBarPage", lua.create_function(|_, ()| Ok(1))?)?;
    globals.set("GetBonusBarOffset", lua.create_function(|_, ()| Ok(0))?)?;
    globals.set("GetOverrideBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetVehicleBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetTempShapeshiftBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    globals.set("GetMultiCastBarIndex", lua.create_function(|_, ()| Ok(7i32))?)?;
    globals.set("GetExtraBarIndex", lua.create_function(|_, ()| Ok(13i32))?)?;
    globals.set("IsPossessBarVisible", lua.create_function(|_, ()| Ok(false))?)?;
    Ok(())
}
