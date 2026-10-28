//! Original side-panel consumer and build 12.1.5.70448 atlas geometry.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

#[test]
fn original_sidepanel_tab_uses_common_sidetab_atlas() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let root = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let source = root.join("Blizzard_SharedXML/Shared/Tabs/SidePanelTabButton.lua");
    env.exec(&std::fs::read_to_string(source).unwrap()).unwrap();
    env.exec(
        r#"
        local width, height = SidePanelTabButtonMixin.CalculateTabSizeBasedOnTabArt()
        assert(width == 45 and height == 55)
        local info = assert(C_Texture.GetAtlasInfo("common-sidetab"))
        assert(info.width == 45 and info.height == 60)
        assert(info.filename == "Interface\\common\\commonsidetab.blp")
        assert(math.abs(info.leftTexCoord - 48/128) < 0.000001)
        assert(math.abs(info.rightTexCoord - 93/128) < 0.000001)
        assert(math.abs(info.topTexCoord - 1/128) < 0.000001)
        assert(math.abs(info.bottomTexCoord - 61/128) < 0.000001)
        "#,
    )
    .unwrap();
}
