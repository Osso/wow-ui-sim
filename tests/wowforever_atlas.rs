//! Build 1.60.1.69913 atlas geometry from UiTextureAtlas/Member CSVs.
#![cfg(feature = "client-wowforever")]

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
