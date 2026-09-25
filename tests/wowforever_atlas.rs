//! Build 1.60.1.69913 atlas geometry from UiTextureAtlas/Member CSVs.
#![cfg(feature = "client-wowforever")]

#[test]
fn forever_backpack_metal_aliases_use_logical_geometry_without_resizing_explicit_members() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local entries = {
            {"ui-frame-portraitmetal-cornertopleftsmall", "ui-frame-portraitmetal-cornertopleftsmall-c60-2x", 95, 95, 190, 190},
            {"ui-frame-metal-cornerbottomleft", "ui-frame-metal-cornerbottomleft-c60-2x", 95, 100, 190, 200},
            {"_ui-frame-metal-edgetop", "_ui-frame-metal-edgetop-c60-2x", 128, 95, 256, 190},
        }
        for _, entry in ipairs(entries) do
            local alias, explicit, width, height, physicalWidth, physicalHeight = unpack(entry)
            local info = C_Texture.GetAtlasInfo(alias)
            local physical = C_Texture.GetAtlasInfo(explicit)
            assert(info and info.width == width and info.height == height, alias)
            assert(physical and physical.width == physicalWidth and physical.height == physicalHeight, explicit)
            assert(info.leftTexCoord == physical.leftTexCoord and info.rightTexCoord == physical.rightTexCoord
                and info.topTexCoord == physical.topTexCoord and info.bottomTexCoord == physical.bottomTexCoord, alias .. " UV")
            local texture = UIParent:CreateTexture(nil, "ARTWORK")
            texture:SetAtlas(alias, true)
            assert(texture:GetWidth() == width and texture:GetHeight() == height, alias .. " texture")
            texture:SetAtlas(explicit, true)
            assert(texture:GetWidth() == physicalWidth and texture:GetHeight() == physicalHeight, explicit .. " texture")
        end
    "#,
    )
    .unwrap();
}

#[test]
fn forever_minimap_cycle_resolves_element_geometry() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local info = C_Texture.GetAtlasInfo("UI-HUD-Minimap-Frame-Cycle")
        assert(info, "Forever cycle atlas missing")
        assert(info.width == 42 and info.height == 42)
        assert(math.abs(info.leftTexCoord - 256/512) < 0.000001)
        assert(math.abs(info.rightTexCoord - 298/512) < 0.000001)
        assert(math.abs(info.topTexCoord - 256/512) < 0.000001)
        assert(math.abs(info.bottomTexCoord - 298/512) < 0.000001)
    "#,
    )
    .unwrap();
    assert_eq!(
        wow_ui_sim::atlas::get_atlas_name_by_element_id(35224),
        Some("ui-hud-minimap-frame-cycle")
    );
}
