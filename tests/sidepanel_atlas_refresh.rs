//! Original side-panel consumer and build 12.1.5.70448 atlas geometry.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

#[test]
fn original_sidepanel_tab_uses_common_sidetab_atlas() {
    let env = crate::common::panel_fixtures::setup_env();
    env.exec(
        r#"
        local tab = CreateFrame("Frame", nil, UIParent, "LargeSideTabButtonTemplate")
        assert(tab.Background:GetAtlas() == "common-sidetab", "template background atlas")
        assert(tab.Background:GetWidth() == 45 and tab.Background:GetHeight() == 60, "template background geometry")
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
