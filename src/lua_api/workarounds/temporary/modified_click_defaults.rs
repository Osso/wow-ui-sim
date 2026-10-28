//! Temporary modified-click setting defaults.
//!
//! Real WoW stores these in client CVar/input settings. The simulator only
//! needs enough state for startup UI paths until input settings are modeled.

const MODIFIED_CLICK_DEFAULTS_LUA: &str = r#"
if type(rawget(_G, "__wow_modified_clicks")) ~= "table" then
    rawset(_G, "__wow_modified_clicks", {})
end
if GetModifiedClick == nil then
    function GetModifiedClick(action)
        return rawget(_G, "__wow_modified_clicks")[action] or "NONE"
    end
end
if SetModifiedClick == nil then
    function SetModifiedClick(action, modifier)
        rawget(_G, "__wow_modified_clicks")[action] = modifier or "NONE"
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        lua.exec(
            r#"
            if type(rawget(_G, "__wow_modified_clicks")) ~= "table" then
                rawset(_G, "__wow_modified_clicks", {
                    AUTOLOOTTOGGLE = "SHIFT", CHATLINK = "SHIFT", COMPAREITEMS = "SHIFT",
                    DRESSUP = "CTRL", FOCUSCAST = "NONE", SELFCAST = "ALT",
                })
            end
        "#,
        )?;
    }
    lua.exec(MODIFIED_CLICK_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_modified_click_state_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("GetModifiedClick = nil; SetModifiedClick = nil; __wow_modified_clicks = { SELFCAST = 'ALT' }")
            .expect("fixture should clear modified-click globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("modified-click defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if GetModifiedClick("SELFCAST") ~= "ALT" then return "default" end
                SetModifiedClick("SELFCAST", "CTRL")
                if GetModifiedClick("SELFCAST") ~= "CTRL" then return "updated" end
                SetModifiedClick("SELFCAST", nil)
                if GetModifiedClick("SELFCAST") ~= "NONE" then return "nil_default" end
                return "ok"
                "#,
            )
            .expect("modified-click state probe should run");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_modified_click_globals() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            GetModifiedClick = function(action) return "custom:" .. action end
            SetModifiedClick = function(action, modifier)
                modifiedClickSet = action .. "=" .. tostring(modifier)
            end
            "#,
        )
        .expect("fixture should install existing modified-click globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("modified-click defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if GetModifiedClick("SELFCAST") ~= "custom:SELFCAST" then return "get" end
                SetModifiedClick("SELFCAST", "SHIFT")
                if modifiedClickSet ~= "SELFCAST=SHIFT" then return "set" end
                return "ok"
                "#,
            )
            .expect("modified-click preservation probe should run");

        assert_eq!(result, "ok");
    }
}
