//! Temporary gamepad cursor-control defaults.
//!
//! Preserve inert gamepad cursor-control globals on profiles without a model.

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const GAMEPAD_CURSOR_CONTROL_DEFAULTS_LUA: &str = r#"
if CanAutoSetGamePadCursorControl == nil then
  function CanAutoSetGamePadCursorControl(_enabled)
    return false
  end
end

if SetGamePadCursorControl == nil then
  function SetGamePadCursorControl(_enabled)
  end
end
"#;

pub(crate) fn apply_bootstrap(_lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    _lua.exec(GAMEPAD_CURSOR_CONTROL_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    any(
        feature = "client-ptr",
        feature = "client-mists",
        feature = "client-wrath",
        feature = "client-era",
        feature = "client-anniversary"
    )
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_gamepad_cursor_control_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if CanAutoSetGamePadCursorControl(true) ~= false then return "auto" end
                if pcall(SetGamePadCursorControl, true) ~= true then return "set" end
                return "ok"
                "#,
            )
            .expect("gamepad cursor-control defaults probe should run");

        assert_eq!(result, "ok");
    }
}
