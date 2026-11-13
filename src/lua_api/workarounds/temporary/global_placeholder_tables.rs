//! Temporary global placeholder tables expected by Blizzard UI files.
//!
//! These tables are mutable Lua registries populated by Blizzard addons at
//! load time. They are compatibility scaffolding, not item/spell/tooltip API
//! surface, so keep them out of `globals::missing_surface`.

const GLOBAL_PLACEHOLDER_TABLES_LUA: &str = r#"
StaticPopupDialogs = StaticPopupDialogs or {}
SOUNDKIT = SOUNDKIT or {}

UISpecialFrames = UISpecialFrames or {}
UI_SPECIAL_FRAMES = UISpecialFrames
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(GLOBAL_PLACEHOLDER_TABLES_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_global_placeholder_tables() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (String, String, String, bool) = env
            .eval(
                r#"
                return type(StaticPopupDialogs),
                       type(UIPanelWindows),
                       type(SOUNDKIT),
                       UI_SPECIAL_FRAMES == UISpecialFrames
                "#,
            )
            .expect("global placeholder table probe should run");

        assert_eq!(
            result,
            (
                "table".to_string(),
                "nil".to_string(),
                "table".to_string(),
                true
            )
        );
    }
}
