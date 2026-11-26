//! Retail tooltip geometry updates before glyph-backed rendering.
#![cfg(all(feature = "gui", feature = "client-retail"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_tooltip_height_grows_for_each_unrendered_row() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.exec(
        r#"
        GameTooltip:AddLine("first supplied row")
        local first_height = GameTooltip:GetHeight()
        GameTooltip:AddLine("second supplied row")
        local second_height = GameTooltip:GetHeight()
        assert(second_height > first_height,
            "each supplied row must add height before glyph-backed rendering")
        "#,
    )
    .expect("public tooltip row sizing succeeds before glyph-backed rendering");
}
