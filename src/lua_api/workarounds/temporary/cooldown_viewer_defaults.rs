//! Temporary `C_CooldownViewer` defaults.
//!
//! Category sets, group buff items and cooldown info are modeled in
//! `c_api::c_cooldown_viewer`. The empty cooldown-ID default keeps the Blizzard
//! cooldown-viewer UI loadable until that lookup has a real backend.

const COOLDOWN_VIEWER_DEFAULTS_LUA: &str = r#"
C_CooldownViewer = C_CooldownViewer or __wow_namespace()

local function installCooldownViewerDefault(name, fn)
    if rawget(C_CooldownViewer, name) == nil then
        C_CooldownViewer[name] = fn
    end
end

installCooldownViewerDefault("GetCooldownID", function()
    return nil
end)
"#;

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const PTR_GROUP_BUFF_DEFAULT_LUA: &str = r#"
if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
    if rawget(C_CooldownViewer, "GetGroupBuffItems") == nil then
        C_CooldownViewer.GetGroupBuffItems = function()
            return {}
        end
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(COOLDOWN_VIEWER_DEFAULTS_LUA)?;
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr {
        lua.exec(PTR_GROUP_BUFF_DEFAULT_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    retail_only_namespace_member_fixture! {
    #[test]
    fn installs_empty_cooldown_viewer_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, bool) = env
            .eval(
                r#"
                return #C_CooldownViewer.GetGroupBuffItems(),
                    C_CooldownViewer.GetCooldownID() == nil
                "#,
            )
            .expect("cooldown viewer defaults should be callable");

        assert_eq!(result, (0, true));
    }
    }

    #[test]
    fn preserves_existing_cooldown_viewer_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_CooldownViewer.GetCooldownID()
                return "existing"
            end
            "#,
        )
        .expect("fixture should install existing cooldown viewer provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let first_category: String = env
            .eval("return C_CooldownViewer.GetCooldownID()")
            .expect("existing cooldown viewer provider should remain callable");

        assert_eq!(first_category, "existing");
    }
}
