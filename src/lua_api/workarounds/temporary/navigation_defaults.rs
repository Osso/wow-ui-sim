//! Temporary `C_Navigation` defaults.
//!
//! Quest/navigation target state is not modeled yet. These inert defaults keep
//! Blizzard navigation UI callable until a real navigation backend owns them.

const NAVIGATION_DEFAULTS_LUA: &str = r#"
C_Navigation = C_Navigation or __wow_namespace()

local function installNavigationDefault(name, fn)
    if rawget(C_Navigation, name) == nil then
        C_Navigation[name] = fn
    end
end

installNavigationDefault("WasClampedToScreen", function()
    return false
end)

installNavigationDefault("GetTargetState", function()
    return 0
end)

installNavigationDefault("HasValidScreenPosition", function()
    return false
end)

installNavigationDefault("GetDistance", function()
    return 0
end)

installNavigationDefault("GetFrame", function()
    return nil
end)

if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
    installNavigationDefault("GetNextWaypointForMap", function()
        return nil
    end)
end
"#;

/// Epochs before the host-backed `c_navigation` provider keep the inert default.
const LEGACY_NEAREST_PARTY_TOKEN_LUA: &str = r#"
if rawget(C_Navigation, "GetNearestPartyMemberToken") == nil then
    C_Navigation.GetNearestPartyMemberToken = function()
        return nil
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(NAVIGATION_DEFAULTS_LUA)?;
    if !cfg!(feature = "retail-12-0-5") {
        lua.exec(LEGACY_NEAREST_PARTY_TOKEN_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_safe_empty_navigation_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (bool, i32, bool, i32, bool) = env
            .eval(
                r#"
                return
                    C_Navigation.WasClampedToScreen(),
                    C_Navigation.GetTargetState(),
                    C_Navigation.HasValidScreenPosition(),
                    C_Navigation.GetDistance(),
                    C_Navigation.GetFrame() == nil
                "#,
            )
            .expect("navigation defaults should be callable");

        assert_eq!(result, (false, 0, false, 0, true));
    }

    #[test]
    fn preserves_existing_navigation_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_Navigation.GetDistance()
                return 42
            end
            "#,
        )
        .expect("fixture should install existing navigation provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let distance: i32 = env
            .eval("return C_Navigation.GetDistance()")
            .expect("existing navigation provider should remain callable");

        assert_eq!(distance, 42);
    }
}
