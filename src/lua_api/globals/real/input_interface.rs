//! Input-interface probes backed by profile-specific simulator state.

#[cfg(feature = "client-wowforever")]
use crate::c_api::c_input_interface_style::InputInterfaceStyle;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::FromStack;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::TableBuilder;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn is_using_gamepad_state(sim: &crate::lua_api::state::SimState) -> LuaResult<bool> {
    #[cfg(feature = "client-wowforever")]
    return Ok(sim.input_interface_style == InputInterfaceStyle::Gamepad);
    #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
    return is_using_gamepad_with_overlap_window(sim);
}

fn is_using_gamepad(state: &mut LuaState) -> LuaResult<u32> {
    let is_using_gamepad = {
        let sim = borrow_state(state)?;
        is_using_gamepad_state(&sim)?
    };
    state.push(Val::Bool(is_using_gamepad));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
// INFERRED: GamePadOverlapMouseMs gates the native last-input timestamp.
fn is_using_gamepad_with_overlap_window(sim: &crate::lua_api::state::SimState) -> LuaResult<bool> {
    let Some(timestamp) = sim
        .last_gamepad_input_timestamp
        .filter(|timestamp| *timestamp != 0)
    else {
        return Ok(false);
    };
    let overlap_ms = sim
        .cvars
        .get("GamePadOverlapMouseMs")
        .ok_or_else(|| rilua::runtime_error("GamePadOverlapMouseMs CVar is unavailable"))?
        .parse::<u32>()
        .map_err(|_| {
            rilua::runtime_error("GamePadOverlapMouseMs CVar must be unsigned milliseconds")
        })?;
    let elapsed_ms = sim.cached_clock_ms().wrapping_sub(timestamp);
    Ok(elapsed_ms < overlap_ms)
}

fn can_auto_set_gamepad_cursor_control(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = bool::from_stack(state, 1)?;
    let can_auto_set = {
        let sim = borrow_state(state)?;
        let auto_enable = sim
            .cvars
            .get("GamePadCursorAutoEnable")
            .ok_or_else(|| rilua::runtime_error("GamePadCursorAutoEnable CVar is unavailable"))?
            .parse::<u32>()
            .map_err(|_| rilua::runtime_error("GamePadCursorAutoEnable CVar must be an integer"))?;
        is_using_gamepad_state(&sim)? && (!enabled || auto_enable != 0)
    };
    state.push(Val::Bool(can_auto_set));
    Ok(1)
}

fn set_gamepad_cursor_control(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?.gamepad_cursor_control_enabled = enabled;
    Ok(0)
}

fn get_gamepad_cursor_control_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.gamepad_cursor_control_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_using_gamepad_admin(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = bool::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
    {
        let timestamp = sim.cached_clock_ms().wrapping_sub(1);
        sim.last_gamepad_input_timestamp = Some(if timestamp == 0 { u32::MAX } else { timestamp });
        if !enabled {
            sim.last_gamepad_input_timestamp = None;
        }
    }
    #[cfg(feature = "client-wowforever")]
    {
        sim.set_input_interface_style(if enabled {
            InputInterfaceStyle::Gamepad
        } else {
            InputInterfaceStyle::Mkb
        });
    }
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(builder);
    }
    builder
        .set_function("SetUsingGamepad", set_using_gamepad_admin)?
        .set_function(
            "GetGamePadCursorControlEnabled",
            get_gamepad_cursor_control_enabled,
        )
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "IsUsingGamepad", is_using_gamepad)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(
            lua,
            "CanAutoSetGamePadCursorControl",
            can_auto_set_gamepad_cursor_control,
        )?;
        LuaApiMut::register_function(lua, "SetGamePadCursorControl", set_gamepad_cursor_control)?;
        LuaApiMut::register_function(
            lua,
            "IsGamePadCursorControlEnabled",
            get_gamepad_cursor_control_enabled,
        )?;
    }
    Ok(())
}

#[cfg(all(test, feature = "client-retail"))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn retail_is_using_gamepad_defaults_to_false() {
        let env = WowLuaEnv::new().expect("create Retail Lua environment");

        assert!(!env.eval::<bool>("return IsUsingGamepad()").unwrap());
    }
}
