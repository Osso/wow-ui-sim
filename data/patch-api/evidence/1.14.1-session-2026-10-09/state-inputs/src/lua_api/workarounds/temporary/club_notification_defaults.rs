//! Temporary `C_Club` notification-settings defaults.
//!
//! Guild and community club data has a state-backed owner, but per-stream
//! notification preferences are not modeled yet. Keep that empty preference
//! shape explicit in the workaround layer until notification state exists.

const CLUB_NOTIFICATION_DEFAULTS_LUA: &str = r#"
C_Club = C_Club or __wow_namespace()

if rawget(C_Club, "GetClubStreamNotificationSettings") == nil then
    function C_Club.GetClubStreamNotificationSettings()
        return {}
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CLUB_NOTIFICATION_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_empty_club_stream_notification_settings() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let count: i32 = env
            .eval("return #C_Club.GetClubStreamNotificationSettings('guild-0')")
            .expect("club notification settings should be queryable");

        assert_eq!(count, 0);
    }

    #[test]
    fn preserves_existing_club_notification_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_Club.GetClubStreamNotificationSettings()
                return { "existing" }
            end
            "#,
        )
        .expect("fixture should install existing club notification provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let first_setting: String = env
            .eval("return C_Club.GetClubStreamNotificationSettings('guild-0')[1]")
            .expect("existing club notification provider should remain callable");

        assert_eq!(first_setting, "existing");
    }
}
