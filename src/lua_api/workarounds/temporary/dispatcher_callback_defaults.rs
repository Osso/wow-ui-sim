//! Temporary Dispatcher callback global defaults.
//!
//! Global event callbacks are modeled from retail 12.0.0. Earlier profiles
//! retain legacy defaults; unit callbacks and DevTools handlers remain inert
//! until their independent registration and dispatch systems are modeled.

#[cfg(not(feature = "retail-12-0-0"))]
const LEGACY_EVENT_CALLBACK_DEFAULTS_LUA: &str = r#"
if RegisterEventCallback == nil then
    function RegisterEventCallback(_event, _callback)
    end
end

if UnregisterEventCallback == nil then
    function UnregisterEventCallback(_event, _callback)
    end
end

"#;

const DISPATCHER_CALLBACK_DEFAULTS_LUA: &str = r#"
if RegisterUnitEventCallback == nil then
    function RegisterUnitEventCallback(_event, _callback, _unit)
    end
end

if UnregisterUnitEventCallback == nil then
    function UnregisterUnitEventCallback(_event, _callback, _unit)
    end
end

if DevTools_AddMessageHandler == nil then
    function DevTools_AddMessageHandler(_handler)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(feature = "retail-12-0-0"))]
    lua.exec(LEGACY_EVENT_CALLBACK_DEFAULTS_LUA)?;
    lua.exec(DISPATCHER_CALLBACK_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_inert_dispatcher_callback_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if RegisterUnitEventCallback("UNIT_HEALTH", function() end, "player") ~= nil then return "register_unit" end
                if UnregisterUnitEventCallback("UNIT_HEALTH", function() end, "player") ~= nil then return "unregister_unit" end
                if DevTools_AddMessageHandler(function() end) ~= nil then return "devtools" end
                return "ok"
                "#,
            )
            .expect("dispatcher callback defaults probe should run");

        assert_eq!(result, "ok");
    }

    #[cfg(not(feature = "retail-12-0-0"))]
    #[test]
    fn installs_legacy_event_callback_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: bool = env
            .eval(
                r#"
                local callback = function() end
                return RegisterEventCallback("TEST", callback) == nil
                    and UnregisterEventCallback("TEST", callback) == nil
                "#,
            )
            .expect("legacy callback defaults probe should run");
        assert!(result);
    }

    #[test]
    fn preserves_existing_dispatcher_callback_globals() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            RegisterEventCallback = function() return "register" end
            UnregisterEventCallback = function() return "unregister" end
            RegisterUnitEventCallback = function() return "unit_register" end
            UnregisterUnitEventCallback = function() return "unit_unregister" end
            DevTools_AddMessageHandler = function() return "devtools" end
            "#,
        )
        .expect("fixture should install existing dispatcher callback globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("dispatcher callback defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                return table.concat({
                    RegisterEventCallback(),
                    UnregisterEventCallback(),
                    RegisterUnitEventCallback(),
                    UnregisterUnitEventCallback(),
                    DevTools_AddMessageHandler(),
                }, ",")
                "#,
            )
            .expect("dispatcher callback preservation probe should run");

        assert_eq!(
            result,
            "register,unregister,unit_register,unit_unregister,devtools"
        );
    }
}
