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
            // Validate target compatibility (borrow must be dropped before
            // fire_ui_error because AddMessage borrows state again).
            let blocked_msg = {
                let s = st.borrow();
                crate::lua_api::game_data::validate_spell_target(
                    spell_id, s.current_target.as_ref(),
                ).err()
            };
            if let Some(msg) = blocked_msg {
                eprintln!("[action] Blocked: {} (spell {})", msg, spell_id);
                super::spell_api::fire_ui_error(lua, msg)?;
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
    // Push state update to registered action UI buttons (replaces ACTIONBAR_UPDATE_STATE).
    push_action_button_state_update(state, lua)?;
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
    ))?;
    // Push state update to registered action buttons (instant spells don't cast).
    push_action_button_state_update(state, lua)?;
    Ok(())
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
    let st = Rc::clone(state);
    globals.set("SetActionUIButton", lua.create_function(move |_, args: mlua::MultiValue| {
        let mut iter = args.iter();
        let frame_id = iter.next().and_then(|v| {
            if let Value::UserData(ud) = v {
                ud.borrow::<super::super::frame::FrameHandle>().ok().map(|h| h.id)
            } else {
                None
            }
        });
        let action = iter.next().and_then(|v| slot_from_value(v));
        if let (Some(fid), Some(slot)) = (frame_id, action) {
            let mut s = st.borrow_mut();
            // Remove any previous registration for this frame.
            s.action_ui_buttons.retain(|(id, _)| *id != fid);
            s.action_ui_buttons.push((fid, slot));
        }
        Ok(())
    })?)?;
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

/// C_ActionBar namespace — called from c_stubs_api after globals are set.
pub fn register_c_action_bar_namespace(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let t = lua.create_table()?;
    register_c_action_bar_stub_methods(lua, &t)?;
    register_c_action_bar_slot_stubs(lua, &t)?;
    register_c_action_bar_stateful(lua, &t, &state)?;
    lua.globals().set("C_ActionBar", t)?;
    Ok(())
}

fn register_c_action_bar_stub_methods(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetBonusBarIndexForSlot", lua.create_function(|_, _s: i32| Ok(0i32))?)?;
    t.set("IsOnBarOrSpecialBar", lua.create_function(|_, _s: i32| Ok(false))?)?;
    t.set("FindSpellActionButtons", lua.create_function(|lua, _: i32| lua.create_table())?)?;
    t.set("GetCurrentActionBarByClass", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("HasFlyoutActionButtons", lua.create_function(|_, _: i32| Ok(false))?)?;
    t.set("EnableActionRangeCheck", lua.create_function(|_, (_, _): (Value, bool)| Ok(()))?)?;
    t.set("IsAssistedCombatAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("HasAssistedCombatActionButtons", lua.create_function(|_, ()| Ok(false))?)?;
    // Page/index methods
    t.set("GetActionBarPage", lua.create_function(|_, ()| Ok(1i32))?)?;
    t.set("SetActionBarPage", lua.create_function(|_, _: Value| Ok(()))?)?;
    t.set("GetExtraBarIndex", lua.create_function(|_, ()| Ok(13i32))?)?;
    t.set("GetMultiCastBarIndex", lua.create_function(|_, ()| Ok(7i32))?)?;
    t.set("GetVehicleBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetOverrideBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetTempShapeshiftBarIndex", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    t.set("GetBonusBarIndex", lua.create_function(|_, ()| Ok(0i32))?)?;
    t.set("GetOverrideBarSkin", lua.create_function(|_, ()| Ok(Value::Nil))?)?;
    // Boolean state queries
    t.set("HasVehicleActionBar", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("HasOverrideActionBar", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("HasBonusActionBar", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("HasTempShapeshiftActionBar", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("HasExtraActionBar", lua.create_function(|_, ()| Ok(false))?)?;
    t.set("IsPossessBarVisible", lua.create_function(|_, ()| Ok(false))?)?;
    Ok(())
}

fn register_c_action_bar_slot_stubs(lua: &Lua, t: &mlua::Table) -> Result<()> {
    t.set("GetActionText", lua.create_function(|_, _: Value| Ok(Value::Nil))?)?;
    t.set("GetActionCount", lua.create_function(|_, _: Value| Ok(0i32))?)?;
    t.set("GetActionDisplayCount", lua.create_function(|_, (_, _): (Value, Value)| Ok(Value::Nil))?)?;
    t.set("GetActionUseCount", lua.create_function(|_, _: Value| Ok(0i32))?)?;
    t.set("IsConsumableAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("IsStackableAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("IsItemAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("IsAttackAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("IsAutoRepeatAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("IsEquippedAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("IsEquippedGearOutfitAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("IsHelpfulAction", lua.create_function(|_, (_, _): (Value, Value)| Ok(false))?)?;
    t.set("IsHarmfulAction", lua.create_function(|_, (_, _): (Value, Value)| Ok(false))?)?;
    t.set("GetActionLossOfControlCooldown", lua.create_function(|_, _: Value| Ok((0.0_f64, 0.0_f64)))?)?;
    t.set("GetSpell", lua.create_function(|_, _: Value| Ok(Value::Nil))?)?;
    t.set("GetItemActionOnEquipSpellID", lua.create_function(|_, _: Value| Ok(Value::Nil))?)?;
    t.set("FindFlyoutActionButtons", lua.create_function(|lua, _: i32| lua.create_table())?)?;
    t.set("FindPetActionButtons", lua.create_function(|lua, _: Value| lua.create_table())?)?;
    t.set("GetPetActionPetBarIndices", lua.create_function(|lua, _: Value| lua.create_table())?)?;
    t.set("RegisterActionUIButton", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    // PutActionInSlot is registered by cursor_api with real implementation.
    t.set("IsAutoCastPetAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("IsEnabledAutoCastPetAction", lua.create_function(|_, _: Value| Ok(false))?)?;
    t.set("ToggleAutoCastPetAction", lua.create_function(|_, _: Value| Ok(()))?)?;
    Ok(())
}

fn register_c_action_bar_stateful(
    lua: &Lua,
    t: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(state);
    t.set("HasAction", lua.create_function(move |_, slot: Value| {
        let s = st.borrow();
        Ok(slot_from_value(&slot).is_some_and(|n| s.action_bars.contains_key(&n)))
    })?)?;
    let st = Rc::clone(state);
    t.set("GetActionTexture", lua.create_function(move |lua, slot: Value| {
        let s = st.borrow();
        let n = match slot_from_value(&slot) { Some(n) => n, None => return Ok(Value::Nil) };
        match action_texture_path(&s, n) {
            Some(path) => Ok(Value::String(lua.create_string(&path)?)),
            None => Ok(Value::Nil),
        }
    })?)?;
    let st = Rc::clone(state);
    t.set("IsUsableAction", lua.create_function(move |_, slot: Value| {
        let s = st.borrow();
        Ok((slot_from_value(&slot).is_some_and(|n| s.action_bars.contains_key(&n)), false))
    })?)?;
    let st = Rc::clone(state);
    t.set("IsCurrentAction", lua.create_function(move |_, slot: Value| {
        let n = slot_from_value(&slot).unwrap_or(0);
        let s = st.borrow();
        let casting = match &s.casting { Some(c) => c.spell_id, None => return Ok(false) };
        Ok(s.action_bars.get(&n).copied() == Some(casting))
    })?)?;
    register_c_action_bar_cooldowns(lua, t, state)?;
    Ok(())
}

fn register_c_action_bar_cooldowns(
    lua: &Lua,
    t: &mlua::Table,
    state: &Rc<RefCell<SimState>>,
) -> Result<()> {
    let st = Rc::clone(state);
    t.set("GetActionCooldown", lua.create_function(move |lua, slot: Value| {
        let s = st.borrow();
        let (start, dur) = slot_from_value(&slot)
            .and_then(|n| s.action_bars.get(&n).map(|&id| {
                spell_cooldown_times(&s, id, s.start_time.elapsed().as_secs_f64())
            }))
            .unwrap_or((0.0, 0.0));
        let info = lua.create_table()?;
        info.set("startTime", start)?;
        info.set("duration", dur)?;
        info.set("isEnabled", true)?;
        info.set("modRate", 1.0_f64)?;
        Ok(info)
    })?)?;
    t.set("GetActionCharges", lua.create_function(|lua, _: Value| {
        let info = lua.create_table()?;
        info.set("currentCharges", 0)?;
        info.set("maxCharges", 0)?;
        info.set("cooldownStartTime", 0.0_f64)?;
        info.set("cooldownDuration", 0.0_f64)?;
        info.set("chargeModRate", 1.0_f64)?;
        Ok(info)
    })?)?;
    Ok(())
}

/// Push state updates to all buttons registered via SetActionUIButton.
/// This replaces the ACTIONBAR_UPDATE_STATE event which WoW's C++ engine
/// pushes directly to registered buttons (the event is commented out in Lua).
pub fn push_action_button_state_update(
    state: &Rc<RefCell<SimState>>,
    lua: &Lua,
) -> Result<()> {
    let buttons: Vec<u64> = state.borrow().action_ui_buttons
        .iter().map(|(id, _)| *id).collect();
    for frame_id in buttons {
        // Use Lua code so __index resolves mixin methods correctly.
        let code = format!(
            "do local f = __frame_{} if f and f.UpdateState then f:UpdateState() end end",
            frame_id
        );
        if let Err(e) = lua.load(&code).exec() {
            eprintln!("[action] UpdateState error for frame {}: {}", frame_id, e);
        }
    }
    Ok(())
}
