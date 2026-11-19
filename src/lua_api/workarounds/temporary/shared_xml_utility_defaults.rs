//! Temporary fallback for the later Settings definitions provider.
//!
//! Blizzard_SharedXMLBase publishes its own utility globals. The ping settings
//! initializer belongs to Blizzard_SettingsDefinitions_Frame, not SharedXMLBase.

const SHARED_XML_UTILITY_DEFAULTS_LUA: &str = r##"
if type(PingSystemInitializer) ~= "function" then
  function PingSystemInitializer(_category)
  end
end
"##;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SHARED_XML_UTILITY_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_ping_system_initializer_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("shared XML utility defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                PingSystemInitializer({})
                return "ok"
                "#,
            )
            .expect("shared XML utility defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_shared_xml_utility_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function CreateAnchor() return "existing_anchor" end
            function GetFinalNameFromTextureKit() return "existing_texture" end
            function CopyValuesAsKeys() return "existing_copy" end
            EnumUtil = { MakeEnum = function() return "existing_enum" end }
            function CreateCounter() return "existing_counter" end
            function GetOrCreateTableEntry() return "existing_entry" end
            function GenerateClosure() return "existing_closure" end
            function GetMicroIconForRole() return "existing_role_icon" end
            TextureKitConstants = { UseAtlasSize = "existing_texture_constant" }
            "#,
        )
        .expect("fixture should install existing utility globals");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("shared XML utility defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if CreateAnchor() ~= "existing_anchor" then return "overwrote_anchor" end
                if GetFinalNameFromTextureKit() ~= "existing_texture" then return "overwrote_texture" end
                if CopyValuesAsKeys() ~= "existing_copy" then return "overwrote_copy" end
                if EnumUtil.MakeEnum() ~= "existing_enum" then return "overwrote_enum" end
                if CreateCounter() ~= "existing_counter" then return "overwrote_counter" end
                if GetOrCreateTableEntry() ~= "existing_entry" then return "overwrote_table_entry" end
                if GenerateClosure() ~= "existing_closure" then return "overwrote_closure" end
                if TextureKitConstants.UseAtlasSize ~= "existing_texture_constant" then return "overwrote_texture_constants" end
                if GetMicroIconForRole() ~= "existing_role_icon" then return "overwrote_role_icon" end
                if type(PingSystemInitializer) ~= "function" then return "missing_ping" end
                return "ok"
                "#,
            )
            .expect("shared XML utility preservation probe should run");

        assert_eq!(result, "ok");
    }
}
