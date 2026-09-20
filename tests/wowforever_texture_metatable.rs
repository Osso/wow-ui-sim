//! Forever texture metatable access through the actual Blizzard consumer.

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_texture_metatable_loads_unit_frame_util() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let source = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_UnitFrame/Mainline/UnitFrameUtil.lua");
    env.exec(&std::fs::read_to_string(source).unwrap()).unwrap();
    env.exec(
        r#"
        local texture = CreateFrame("Frame"):CreateTexture()
        local meta = GetTextureMetatable()
        assert(meta == getmetatable(texture), "factory texture metatable identity")
        assert(meta == GetTextureMetatable(), "repeated metatable identity")
        assert(meta == GetTextureMetatable(texture), "explicit texture identity")
        assert(meta.__index.IsObjectType(texture, "Texture"), "real texture type")
        assert(not meta.__index.IsObjectType(texture, "FontString"), "non-font type")
        meta.__index.SetTexture(texture, "Interface/Icons/INV_Misc_QuestionMark")
        assert(texture:GetTexture() == "Interface/Icons/INV_Misc_QuestionMark", tostring(texture:GetTexture()))
        local copied = CopyTable(meta.__index)
        assert(copied ~= meta.__index, "independent copied index")
        assert(copied.IsObjectType(texture, "Texture"), "copied method dispatch")
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(not(feature = "client-wowforever"))]
fn wowforever_texture_metatable_is_not_published_elsewhere() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec("assert(GetTextureMetatable == nil)").unwrap();
}
