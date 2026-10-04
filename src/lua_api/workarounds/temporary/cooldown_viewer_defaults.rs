//! Temporary `C_CooldownViewer` defaults.
//!
//! Category sets and cooldown info are modeled in `c_api::c_cooldown_viewer`.
//! Group buff items and cooldown-ID lookup are not modeled yet; these empty
//! defaults keep the Blizzard cooldown-viewer UI loadable until a real backend
//! owns them.

const COOLDOWN_VIEWER_DEFAULTS_LUA: &str = r#"
C_CooldownViewer = C_CooldownViewer or __wow_namespace()

local function installCooldownViewerDefault(name, fn)
    if rawget(C_CooldownViewer, name) == nil then
        C_CooldownViewer[name] = fn
    end
end

if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
    installCooldownViewerDefault("GetGroupBuffItems", function()
        return {}
    end)
end

installCooldownViewerDefault("GetCooldownID", function()
    return nil
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(COOLDOWN_VIEWER_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

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
