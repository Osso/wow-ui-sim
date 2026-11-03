//! Temporary `C_Minimap` tracking defaults.
//!
//! Supply defaults for tracking APIs that have no profile-specific provider.
//! Widget-specific minimap methods remain on Minimap frames.

/// Retail build 70688 initialized tracking-default mask.
pub(crate) const RETAIL_TRACKING_DEFAULT_MASK: u32 = 0x2f5aef;

const MINIMAP_TRACKING_DEFAULTS_LUA: &str = r#"
C_Minimap = C_Minimap or __wow_namespace()

local function installMinimapDefault(name, fn)
    if rawget(C_Minimap, name) == nil then
        C_Minimap[name] = fn
    end
end

installMinimapDefault("GetNumTrackingTypes", function()
    return 0
end)

installMinimapDefault("GetTrackingInfo", function(_index)
    return nil
end)

installMinimapDefault("GetTrackingFilter", function()
    return {
        filterID = 0,
        spellID = 0,
    }
end)

installMinimapDefault("SetTracking", function(_index, _enabled)
end)

installMinimapDefault("ClearAllTracking", function()
end)

installMinimapDefault("GetViewRadius", function()
    return 200
end)
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(MINIMAP_TRACKING_DEFAULTS_LUA)?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        // Replace this default when minimap tracking state is modeled.
        lua.exec(
            r#"
            if rawget(C_Minimap, "IsFilteredOut") == nil then
                function C_Minimap.IsFilteredOut(filterType)
                    assert(type(filterType) == "number", "filterType must be a number")
                    return false
                end
            end
        "#,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    #[cfg(not(feature = "client-wowforever"))]
    fn installs_minimap_tracking_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                local filter = C_Minimap.GetTrackingFilter()
                if C_Minimap.GetNumTrackingTypes() ~= 0 then return "tracking-count" end
                if C_Minimap.GetTrackingInfo(1) ~= nil then return "tracking-info" end
                if filter.filterID ~= 0 or filter.spellID ~= 0 then return "filter" end
                if C_Minimap.SetTracking(1, true) ~= nil then return "set" end
                if C_Minimap.ClearAllTracking() ~= nil then return "clear" end
                if C_Minimap.GetViewRadius() ~= 200 then return "radius" end
                return "ok"
                "#,
            )
            .expect("minimap tracking defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_minimap_tracking_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Minimap = C_Minimap or __wow_namespace()

            function C_Minimap.GetNumTrackingTypes()
                return 2
            end
            function C_Minimap.GetViewRadius()
                return 333
            end
            "#,
        )
        .expect("fixture should install existing C_Minimap providers");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32) = env
            .eval(
                r#"
                return C_Minimap.GetNumTrackingTypes(),
                    C_Minimap.GetViewRadius()
                "#,
            )
            .expect("existing C_Minimap providers should remain callable");

        assert_eq!(result, (2, 333));
    }
}
