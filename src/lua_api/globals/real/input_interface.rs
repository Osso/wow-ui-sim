//! Input-interface probes backed by profile-specific simulator state.

#[cfg(feature = "client-wowforever")]
use crate::c_api::c_input_interface_style::InputInterfaceStyle;
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn is_using_gamepad(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "client-wowforever")]
    let is_using_gamepad = {
        let sim = borrow_state(state)?;
        sim.input_interface_style == InputInterfaceStyle::Gamepad
    };
    #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
    let is_using_gamepad = {
        let sim = borrow_state(state)?;
        is_using_gamepad_with_overlap_window(&sim)?
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

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "IsUsingGamepad", is_using_gamepad)
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn retail_and_ptr_is_using_gamepad_defaults_to_false() {
        let env = WowLuaEnv::new().expect("create Retail Lua environment");

        assert!(!env.eval::<bool>("return IsUsingGamepad()").unwrap());
    }

    #[test]
    fn retail_and_ptr_use_the_inferred_overlap_window_after_clock_wrap() {
        let env = WowLuaEnv::new().expect("create Retail Lua environment");
        env.exec("A_Admin.SetManualClock(4294968); SetCVar('GamePadOverlapMouseMs', '1500')")
            .expect("supply clock and overlap window");
        env.state().borrow_mut().last_gamepad_input_timestamp = Some(u32::MAX - 500);

        assert!(env.eval::<bool>("return IsUsingGamepad()").unwrap());
        env.exec("SetCVar('GamePadOverlapMouseMs', '1200')")
            .expect("shorten overlap window");
        assert!(!env.eval::<bool>("return IsUsingGamepad()").unwrap());
    }
}
