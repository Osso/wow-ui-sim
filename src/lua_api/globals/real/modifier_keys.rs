//! Modifier-key probes backed by `SimState::modifier_keys`.
//!
//! WoW exposes five globals: `IsShiftKeyDown` / `IsControlKeyDown` /
//! `IsAltKeyDown` / `IsMetaKeyDown` read individual keys; `IsModifierKeyDown`
//! returns true iff any of shift/control/alt is held (meta is *not* included
//! — Blizzard keeps that on its own probe).
//!
//! All keys default to `false` in the sim because no physical keyboard state
//! is captured in headless tests. Admin API
//! `A_Admin.SetShiftKeyDown(b)` / `SetControlKeyDown` / `SetAltKeyDown` /
//! `SetMetaKeyDown` toggles them for tests that want to exercise
//! modifier-aware UI paths (e.g. `IsModifiedClick("CHATLINK")`).

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn make_modifiers(state: &mut LuaState) -> LuaResult<u32> {
    let modifiers = {
        let sim = borrow_state(state)?;
        u8::from(sim.modifier_keys.shift)
            | (u8::from(sim.modifier_keys.control) << 1)
            | (u8::from(sim.modifier_keys.alt) << 2)
    };
    state.push(Val::Num(f64::from(modifiers)));
    Ok(1)
}

pub fn is_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let key = String::from_stack(state, 1)?;
    let key = crate::lua_api::key_dispatch::normalized_key_name(&key);
    let down = {
        let sim = borrow_state(state)?;
        match key.as_str() {
            "SHIFT" | "LSHIFT" | "RSHIFT" => sim.modifier_keys.shift,
            "CTRL" | "CONTROL" | "LCTRL" | "RCTRL" => sim.modifier_keys.control,
            "ALT" | "LALT" | "RALT" => sim.modifier_keys.alt,
            "META" | "LMETA" | "RMETA" => sim.modifier_keys.meta,
            "LEFTBUTTON" | "RIGHTBUTTON" | "MIDDLEBUTTON" | "BUTTON4" | "BUTTON5" => {
                sim.mouse_buttons.is_down(Some(&key))
            }
            _ => sim.pressed_keys.contains(&key),
        }
    };
    state.push(if key.is_empty() {
        Val::Nil
    } else {
        Val::Bool(down)
    });
    Ok(1)
}

pub fn is_shift_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = borrow_state(state)?.modifier_keys.shift;
    state.push(Val::Bool(down));
    Ok(1)
}

pub fn is_control_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = borrow_state(state)?.modifier_keys.control;
    state.push(Val::Bool(down));
    Ok(1)
}

pub fn is_alt_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = borrow_state(state)?.modifier_keys.alt;
    state.push(Val::Bool(down));
    Ok(1)
}

fn push_modifier_side(
    state: &mut LuaState,
    accessor: impl FnOnce(&crate::lua_api::SimState) -> bool,
) -> LuaResult<u32> {
    let down = {
        let sim = borrow_state(state)?;
        accessor(&sim)
    };
    state.push(Val::Bool(down));
    Ok(1)
}

pub fn is_left_shift_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.shift)
}

pub fn is_right_shift_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.shift)
}

pub fn is_left_control_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.control)
}

pub fn is_right_control_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.control)
}

pub fn is_left_alt_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.alt)
}

pub fn is_right_alt_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.alt)
}

/// Like the other sided probes, both meta sides read the single modeled meta key.
fn is_left_meta_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.meta)
}

fn is_right_meta_key_down(state: &mut LuaState) -> LuaResult<u32> {
    push_modifier_side(state, |sim| sim.modifier_keys.meta)
}

pub fn is_meta_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let down = borrow_state(state)?.modifier_keys.meta;
    state.push(Val::Bool(down));
    Ok(1)
}

pub fn is_modifier_key_down(state: &mut LuaState) -> LuaResult<u32> {
    let any = borrow_state(state)?.modifier_keys.any_modifier();
    state.push(Val::Bool(any));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_modified_click(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::methods::create_string;
    let action = match crate::lua_bridge::stack_val(state, 1) {
        Val::Num(value) => value.to_string(),
        Val::Str(_) => String::from_stack(state, 1)?,
        _ => return Err(rilua::runtime_error("Usage: GetModifiedClick(\"action\")")),
    };
    let binding = modified_click_binding(state, &action)?;
    let value = match binding {
        Some(binding) => create_string(state, &binding),
        None if cfg!(feature = "client-wowforever") => create_string(state, "NONE"),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_modified_click(state: &mut LuaState) -> LuaResult<u32> {
    let action = String::from_stack(state, 1)?;
    let binding = Option::<String>::from_stack(state, 2)?.unwrap_or_else(|| "NONE".to_owned());
    crate::lua_api::methods::borrow_state_mut(state)?
        .modified_clicks
        .insert(action, binding);
    Ok(0)
}

fn modified_click_binding(state: &mut LuaState, action: &str) -> LuaResult<Option<String>> {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    return Ok(borrow_state(state)?.modified_clicks.get(action).cloned());
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    {
        use crate::lua_api::methods::{table_get, table_get_static, val_to_string};
        let table = table_get_static(state, Val::Table(state.global), "__wow_modified_clicks");
        let binding = table_get(state, table, action);
        Ok(val_to_string(state, binding))
    }
}

pub fn is_modified_click(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::methods::val_to_string;
    let action = val_to_string(state, crate::lua_bridge::stack_val(state, 1));
    let keys = borrow_state(state)?.modifier_keys;
    let active = if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        keys.any_modifier()
    } else if let Some(action) = action {
        matches_modified_click(state, &action, keys)?
    } else {
        keys.any_modifier()
    };
    state.push(Val::Bool(active));
    Ok(1)
}

fn matches_modified_click(
    state: &mut LuaState,
    action: &str,
    keys: crate::lua_api::sim_substates::ModifierKeys,
) -> LuaResult<bool> {
    let modifier_down = |part: &str| match part {
        "SHIFT" => Some(keys.shift),
        "CTRL" => Some(keys.control),
        "ALT" => Some(keys.alt),
        _ => None,
    };
    if action.split('-').all(|part| modifier_down(part).is_some()) {
        return Ok(action
            .split('-')
            .all(|part| modifier_down(part) == Some(true)));
    }
    // The current mouse-script button is not tracked; an unset button passes.
    Ok(
        modified_click_binding(state, action)?.is_some_and(|binding| {
            binding
                .split('-')
                .any(|part| modifier_down(part) == Some(true))
        }),
    )
}

fn register_modern_modifiers(lua: &mut rilua::Lua) -> LuaResult<()> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "MakeModifiers", make_modifiers)?;
        #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
        {
            LuaApiMut::register_function(lua, "GetModifiedClick", get_modified_click)?;
            LuaApiMut::register_function(lua, "SetModifiedClick", set_modified_click)?;
        }
        // Retail/Forever InputDocumentation declare byte-identical meta-side blocks.
        LuaApiMut::register_function(lua, "IsLeftMetaKeyDown", is_left_meta_key_down)?;
        LuaApiMut::register_function(lua, "IsRightMetaKeyDown", is_right_meta_key_down)?;
    }
    Ok(())
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_modern_modifiers(lua)?;
    let state = lua.state_mut();
    let g = state.global;
    table_set_rust_fn_static(state, g, "IsKeyDown", is_key_down)?;
    table_set_rust_fn_static(state, g, "IsShiftKeyDown", is_shift_key_down)?;
    table_set_rust_fn_static(state, g, "IsControlKeyDown", is_control_key_down)?;
    table_set_rust_fn_static(state, g, "IsAltKeyDown", is_alt_key_down)?;
    table_set_rust_fn_static(state, g, "IsLeftShiftKeyDown", is_left_shift_key_down)?;
    table_set_rust_fn_static(state, g, "IsRightShiftKeyDown", is_right_shift_key_down)?;
    table_set_rust_fn_static(state, g, "IsLeftControlKeyDown", is_left_control_key_down)?;
    table_set_rust_fn_static(state, g, "IsRightControlKeyDown", is_right_control_key_down)?;
    table_set_rust_fn_static(state, g, "IsLeftAltKeyDown", is_left_alt_key_down)?;
    table_set_rust_fn_static(state, g, "IsRightAltKeyDown", is_right_alt_key_down)?;
    table_set_rust_fn_static(state, g, "IsMetaKeyDown", is_meta_key_down)?;
    table_set_rust_fn_static(state, g, "IsModifierKeyDown", is_modifier_key_down)?;
    table_set_rust_fn_static(state, g, "IsModifiedClick", is_modified_click)?;
    Ok(())
}
