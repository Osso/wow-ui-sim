//! Input-interface probes backed by profile-specific simulator state.

#[cfg(feature = "client-wowforever")]
use crate::c_api::c_input_interface_style::InputInterfaceStyle;
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn is_using_gamepad(state: &mut LuaState) -> LuaResult<u32> {
    let is_using_gamepad = {
        let sim = borrow_state(state)?;
        #[cfg(feature = "client-wowforever")]
        {
            sim.input_interface_style == InputInterfaceStyle::Gamepad
        }
        #[cfg(any(feature = "client-retail", feature = "client-ptr"))]
        {
            sim.last_gamepad_input_timestamp
                .is_some_and(|timestamp| timestamp != 0)
        }
    };
    state.push(Val::Bool(is_using_gamepad));
    Ok(1)
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
}
