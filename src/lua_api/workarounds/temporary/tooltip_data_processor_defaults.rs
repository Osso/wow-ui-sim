//! Temporary tooltip data processor defaults for non-Retail partial addon loads.
//!
//! Retail does not install these defaults; full game loads get the real
//! `TooltipDataProcessor` from Blizzard SharedXML.

#[cfg(any(not(feature = "client-retail"), test))]
const TOOLTIP_DATA_PROCESSOR_DEFAULTS_LUA: &str = r#"
local function tooltip_data_processor_noop()
end

if AddTooltipDataAccessor == nil then
    function AddTooltipDataAccessor()
    end
end

TooltipDataProcessor = TooltipDataProcessor or {
    AllTypes = 0,
    AddTooltipPostCall = tooltip_data_processor_noop,
    AddLinePostCall = tooltip_data_processor_noop,
}
"#;

#[cfg(any(not(feature = "client-retail"), test))]
pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(TOOLTIP_DATA_PROCESSOR_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(feature = "client-retail")]
    const RETAIL_TOOLTIP_DEFAULTS_PROBE: &str = r#"
                    return type(rawget(_G, "TooltipDataProcessor")),
                           type(rawget(_G, "AddTooltipDataAccessor")),
                           type(rawget(__secureenv, "TooltipDataProcessor")),
                           type(rawget(__secureenv, "AddTooltipDataAccessor"))
                    "#;

    #[cfg(not(feature = "client-retail"))]
    const NON_RETAIL_TOOLTIP_DEFAULTS_PROBE: &str = r#"
                    return type(TooltipDataProcessor),
                           TooltipDataProcessor.AllTypes,
                           type(TooltipDataProcessor.AddTooltipPostCall),
                           type(TooltipDataProcessor.AddLinePostCall),
                           AddTooltipDataAccessor() == nil
                    "#;

    fn apply_again(env: &WowLuaEnv) {
        let mut lua = env.lua.borrow_mut();
        super::apply_bootstrap(&mut lua).expect("tooltip data processor defaults should apply");
    }

    #[test]
    fn tooltip_data_processor_defaults_follow_profile_policy() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        #[cfg(feature = "client-retail")]
        assert_retail_tooltip_data_processor_defaults(&env);
        #[cfg(not(feature = "client-retail"))]
        assert_non_retail_tooltip_data_processor_defaults(&env);
    }

    #[cfg(feature = "client-retail")]
    fn assert_retail_tooltip_data_processor_defaults(env: &WowLuaEnv) {
        let result: (String, String, String, String) = env
            .eval(RETAIL_TOOLTIP_DEFAULTS_PROBE)
            .expect("Retail tooltip data processor profile probe should run");
        assert_eq!(
            result,
            ("nil".into(), "nil".into(), "nil".into(), "nil".into())
        );
    }

    #[cfg(not(feature = "client-retail"))]
    fn assert_non_retail_tooltip_data_processor_defaults(env: &WowLuaEnv) {
        let result: (String, i32, String, String, bool) = env
            .eval(NON_RETAIL_TOOLTIP_DEFAULTS_PROBE)
            .expect("non-Retail tooltip data processor defaults should run");
        assert_eq!(
            result,
            (
                "table".to_string(),
                0,
                "function".to_string(),
                "function".to_string(),
                true
            )
        );
    }

    #[test]
    fn preserves_existing_tooltip_data_processor_and_accessor() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            TooltipDataProcessor = {
                AllTypes = 99,
                AddTooltipPostCall = function() return "post" end,
            }
            function AddTooltipDataAccessor()
                return "accessor"
            end
            "#,
        )
        .expect("fixture should install existing tooltip data processor");

        apply_again(&env);

        let result: (i32, String, String) = env
            .eval(
                r#"
                return TooltipDataProcessor.AllTypes,
                       TooltipDataProcessor.AddTooltipPostCall(),
                       AddTooltipDataAccessor()
                "#,
            )
            .expect("tooltip data processor preservation probe should run");

        assert_eq!(result, (99, "post".to_string(), "accessor".to_string()));
    }
}
