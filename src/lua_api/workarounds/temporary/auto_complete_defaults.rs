//! Temporary autocomplete realm defaults and legacy global forwarders.
//!
//! Realm completion data is not modeled yet, so realm probes return an empty
//! list. Character/name completion is modeled in `c_api::c_auto_complete`; this
//! bootstrap retains legacy globals before Retail 12.0.7. At 12.0.7+, only
//! Blizzard_DeprecatedAutoComplete owns those globals when its CVar permits them.

use rilua::LuaApiMut;

const AUTO_COMPLETE_DEFAULTS_LUA: &str = r#"
local publishLegacyNative = ...
C_AutoComplete = C_AutoComplete or __wow_namespace()
if rawget(C_AutoComplete, "GetAutoCompleteRealms") == nil then
    function C_AutoComplete.GetAutoCompleteRealms()
        return {}
    end
end
if publishLegacyNative and GetAutoCompleteRealms == nil then
    function GetAutoCompleteRealms()
        return C_AutoComplete.GetAutoCompleteRealms()
    end
end
if publishLegacyNative and GetAutoCompleteResults == nil then
    function GetAutoCompleteResults(name, numResults, cursorPosition, allowFullMatch, includeFlags, excludeFlags)
        return C_AutoComplete.GetAutoCompleteResults(name, numResults, cursorPosition, not not allowFullMatch, includeFlags, excludeFlags)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    let bootstrap = lua.load_bytes(
        AUTO_COMPLETE_DEFAULTS_LUA.as_bytes(),
        "@auto-complete-defaults",
    )?;
    lua.call_function(
        &bootstrap,
        &[rilua::Val::Bool(!cfg!(feature = "retail-12-0-7"))],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(feature = "retail-12-0-7"))]
    #[test]
    fn installs_empty_realm_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (namespace_count, global_count): (i32, i32) = env
            .eval(
                r#"
                local namespaceRealms = C_AutoComplete.GetAutoCompleteRealms()
                local globalRealms = GetAutoCompleteRealms()
                return #namespaceRealms, #globalRealms
                "#,
            )
            .expect("autocomplete realm defaults should be callable");

        assert_eq!(namespace_count, 0);
        assert_eq!(global_count, 0);
    }

    #[cfg(not(feature = "retail-12-0-7"))]
    #[test]
    fn installs_legacy_results_forwarder() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        env.exec(
            r#"
            GetAutoCompleteResults = nil
            function C_AutoComplete.GetAutoCompleteResults(_name, _numResults, _cursorPosition, allowFullMatch)
                return { allowFullMatch and "coerced" or "not-coerced" }
            end
            "#,
        )
        .expect("fixture should install modeled autocomplete result function");
        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                return GetAutoCompleteResults("name", 1, 0, 1, nil, nil)[1]
                "#,
            )
            .expect("legacy autocomplete result function should forward to C_AutoComplete");

        assert_eq!(result, "coerced");
    }

    #[cfg(feature = "retail-12-0-7")]
    #[test]
    fn bootstrap_keeps_retired_globals_absent_and_namespace_callable() {
        let env = WowLuaEnv::new().unwrap();
        super::apply_bootstrap(&mut env.rilua_mut()).unwrap();
        env.exec(
            r#"
            assert(rawget(_G, "GetAutoCompleteRealms") == nil)
            assert(rawget(_G, "GetAutoCompleteResults") == nil)
            assert(#C_AutoComplete.GetAutoCompleteRealms() == 0)
        "#,
        )
        .unwrap();
    }

    #[test]
    fn preserves_existing_legacy_results_forwarder() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        env.exec(
            r#"
            function GetAutoCompleteResults()
                return { "legacy-modeled-result" }
            end
            "#,
        )
        .expect("fixture should install legacy autocomplete result function");
        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                r#"
                return GetAutoCompleteResults("name", 1, 0, true, nil, nil)[1]
                "#,
            )
            .expect("existing legacy autocomplete result function should remain callable");

        assert_eq!(result, "legacy-modeled-result");
    }

    #[test]
    fn preserves_existing_autocomplete_realms() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_AutoComplete.GetAutoCompleteRealms()
                return { "modeled-realm" }
            end
            function GetAutoCompleteRealms()
                return { "legacy-modeled-realm" }
            end
            "#,
        )
        .expect("fixture should install existing functions");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let (namespace_realm, global_realm): (String, String) = env
            .eval(
                r#"
                return C_AutoComplete.GetAutoCompleteRealms()[1],
                    GetAutoCompleteRealms()[1]
                "#,
            )
            .expect("existing autocomplete realm functions should remain callable");

        assert_eq!(namespace_realm, "modeled-realm");
        assert_eq!(global_realm, "legacy-modeled-realm");
    }
}
