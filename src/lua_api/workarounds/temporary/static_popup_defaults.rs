//! Temporary StaticPopup compatibility defaults.
//!
//! Blizzard's Retail and Forever StaticPopup providers load before their consumers.
//! Keep the inert fallbacks only for Mists and Wrath, whose provider order is not covered.

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const STATIC_POPUP_DEFAULTS_LUA: &str = r#"
if StaticPopup_Hide == nil then
    function StaticPopup_Hide(_which, _data)
    end
end

if StaticPopup_Show == nil then
    function StaticPopup_Show(_which, _text_arg1, _text_arg2, _data)
        return nil
    end
end

if StaticPopup_AddShowCondition == nil then
    function StaticPopup_AddShowCondition()
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    lua.exec(STATIC_POPUP_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    fn installs_inert_static_popup_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(StaticPopup_Show) ~= "function" then return "show_type" end
                if type(StaticPopup_Hide) ~= "function" then return "hide_type" end
                if StaticPopup_Show("TEST") ~= nil then return "show_value" end
                if StaticPopup_Hide("TEST") ~= nil then return "hide_value" end
                return "ok"
                "#,
            )
            .expect("StaticPopup defaults probe should run");

        assert_eq!(result, "ok");
        #[cfg(not(feature = "client-retail"))]
        env.exec("assert(type(StaticPopup_AddShowCondition) == 'function'); assert(StaticPopup_AddShowCondition() == nil)")
            .expect("non-Retail condition fallback remains callable");
    }

    #[test]
    fn preserves_existing_static_popup_globals() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            StaticPopup_Show = function() return "shown" end
            StaticPopup_Hide = function() return "hidden" end
            StaticPopup_AddShowCondition = function() return "condition" end
            "#,
        )
        .expect("fixture should install existing StaticPopup globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("StaticPopup defaults should apply");
        }

        let (show, hide, condition): (String, String, String) = env
            .eval("return StaticPopup_Show(), StaticPopup_Hide(), StaticPopup_AddShowCondition()")
            .expect("StaticPopup preservation probe should run");

        assert_eq!(show, "shown");
        assert_eq!(hide, "hidden");
        assert_eq!(condition, "condition");
    }
}
