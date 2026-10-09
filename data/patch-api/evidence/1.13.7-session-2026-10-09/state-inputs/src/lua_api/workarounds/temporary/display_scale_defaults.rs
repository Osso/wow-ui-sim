//! Temporary display scale globals.
//!
//! The simulator does not have a full display/render-scale settings model yet.
//! Keep these startup defaults explicit until display configuration is modeled.

const DISPLAY_SCALE_DEFAULTS_LUA: &str = r#"
if GetDefaultScale == nil then
  function GetDefaultScale()
    return 1
  end
end

if GetMinRenderScale == nil then
  function GetMinRenderScale()
    return 0.5
  end
end

if GetMaxRenderScale == nil then
  function GetMaxRenderScale()
    return 1.0
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(DISPLAY_SCALE_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_display_scale_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if GetDefaultScale() ~= 1 then return "default" end
                if GetMinRenderScale() ~= 0.5 then return "min" end
                if GetMaxRenderScale() ~= 1.0 then return "max" end
                return "ok"
                "#,
            )
            .expect("display scale defaults probe should run");

        assert_eq!(result, "ok");
    }
}
