//! Temporary possess-slot info defaults.
//!
//! Vehicle and possess action-bar visibility is state-backed, but individual
//! possess slots are not modeled yet. Keep the legacy `GetPossessInfo` probe
//! explicit here until possession buttons have real slot data.

const POSSESS_INFO_DEFAULTS_LUA: &str = r#"
if GetPossessInfo == nil then
  function GetPossessInfo(_index)
    return nil, nil, false
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(POSSESS_INFO_DEFAULTS_LUA)?;
    Ok(())
}

// Retail and Forever publish the supplied-slot Rust GetPossessInfo (c58a87bd4b).
#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_possess_info_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local texture, count, enabled = GetPossessInfo(1)
                if texture ~= nil or count ~= nil or enabled ~= false then return "shape" end
                return "ok"
                "#,
            )
            .expect("GetPossessInfo probe should run");

        assert_eq!(result, "ok");
    }
}
