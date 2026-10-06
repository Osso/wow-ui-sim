//! Temporary C_TransmogSets empty/default surface.
//!
//! Available-set inventory, eligibility and filters are modeled in C API state.
//! Variant/base/primary-appearance details remain empty defaults; retire this
//! module when those catalog relations and their queries are modeled.

const TRANSMOG_SETS_DEFAULTS_LUA: &str = r#"
C_TransmogSets = C_TransmogSets or __wow_namespace()

local function emptyTable()
    return {}
end

if rawget(C_TransmogSets, "GetBaseSetID") == nil then
    function C_TransmogSets.GetBaseSetID(_setID)
        return 0
    end
end

if rawget(C_TransmogSets, "GetSetPrimaryAppearances") == nil then
    C_TransmogSets.GetSetPrimaryAppearances = emptyTable
end

if rawget(C_TransmogSets, "GetBaseSets") == nil then
    C_TransmogSets.GetBaseSets = emptyTable
end

if rawget(C_TransmogSets, "GetAllSets") == nil then
    C_TransmogSets.GetAllSets = emptyTable
end

if rawget(C_TransmogSets, "GetUsableSets") == nil then
    C_TransmogSets.GetUsableSets = emptyTable
end

if rawget(C_TransmogSets, "IsBaseSetCollected") == nil then
    function C_TransmogSets.IsBaseSetCollected(_setID)
        return false
    end
end

if rawget(C_TransmogSets, "GetSourcesForSlot") == nil then
    C_TransmogSets.GetSourcesForSlot = emptyTable
end

if rawget(C_TransmogSets, "GetAllSetAppearancesByID") == nil then
    C_TransmogSets.GetAllSetAppearancesByID = emptyTable
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(TRANSMOG_SETS_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_transmog_sets_empty_inventory_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let result: String = env
            .eval(
                r#"
                if C_TransmogSets.GetBaseSetID(1) ~= 0 then return "base_id" end
                if #C_TransmogSets.GetBaseSets() ~= 0 then return "base_sets" end
                if #C_TransmogSets.GetAllSetAppearancesByID(1) ~= 0 then return "appearances" end
                if C_TransmogSets.HasAvailableSets() ~= false then return "available" end
                local info = C_TransmogSets.GetSetInfo(1)
                if info ~= nil then
                    return "set_info"
                end
                return "ok"
                "#,
            )
            .expect("transmog sets defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_transmog_sets_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_TransmogSets = C_TransmogSets or __wow_namespace()

            function C_TransmogSets.GetBaseSetID(_setID)
                return 42
            end

            function C_TransmogSets.GetBaseSets()
                return { { setID = 42 } }
            end
            "#,
        )
        .expect("fixture should install existing transmog sets provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32) = env
            .eval(
                r#"
                return C_TransmogSets.GetBaseSetID(1),
                       C_TransmogSets.GetBaseSets()[1].setID
                "#,
            )
            .expect("existing transmog sets provider should remain callable");

        assert_eq!(result, (42, 42));
    }
}
