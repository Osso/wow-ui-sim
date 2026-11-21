//! Temporary `C_Map` map-group defaults.
//!
//! Sibling/floor map groups are not modeled yet. The world map probes these
//! optional APIs to decide whether to show the floor dropdown, so keep the
//! startup-compatible "no group" shape explicit until map-group state is
//! seeded.

const MAP_GROUP_DEFAULTS_LUA: &str = r#"
C_Map = C_Map or __wow_namespace()

if rawget(C_Map, "GetMapGroupID") == nil then
    function C_Map.GetMapGroupID(_mapID)
    end
end

if rawget(C_Map, "GetMapGroupMembersInfo") == nil then
    function C_Map.GetMapGroupMembersInfo(_mapGroupID)
        return {}
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(MAP_GROUP_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_map_group_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let (group_returns, members_type, members_count): (i32, String, i32) = env
            .eval(
                r##"
                local members = C_Map.GetMapGroupMembersInfo(12345)
                return select("#", C_Map.GetMapGroupID(2248)), type(members), #members
                "##,
            )
            .expect("map group defaults should be callable");

        assert_eq!(group_returns, 0);
        assert_eq!(members_type, "table");
        assert_eq!(members_count, 0);
    }

    #[test]
    fn preserves_existing_map_group_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Map = C_Map or __wow_namespace()

            function C_Map.GetMapGroupID(_mapID)
                return 55
            end

            function C_Map.GetMapGroupMembersInfo(_mapGroupID)
                return { { mapID = 1 }, { mapID = 2 } }
            end
            "#,
        )
        .expect("fixture should install existing map group provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32) = env
            .eval(
                r#"
                local members = C_Map.GetMapGroupMembersInfo(55)
                return C_Map.GetMapGroupID(2248), #members
                "#,
            )
            .expect("existing map group provider should remain callable");

        assert_eq!(result, (55, 2));
    }
}
