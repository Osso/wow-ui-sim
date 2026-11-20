#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn paper_doll_stagger_consumer_displays_light_moderate_and_heavy_amounts() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("profile Blizzard UI cache should exist");
    #[cfg(feature = "client-retail")]
    let consumer = ui.join("Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua");
    #[cfg(feature = "client-wowforever")]
    let consumer = ui.join("Blizzard_UIPanels_Game/Camelot/PaperDollFrame.lua");
    env.exec(&std::fs::read_to_string(consumer).expect("selected PaperDoll consumer should exist"))
        .expect("selected PaperDoll consumer should load");
    env.exec(
        r#"
        A_Admin.SetPlayerHealth(100000, 100000)
        local stat = CreateFrame("Frame")
        stat.Value = stat:CreateFontString()

        for _, sample in ipairs({
            {10000, 10},
            {40000, 40},
            {70000, 70},
        }) do
            A_Admin.SetUnitStagger("player", sample[1])
            PaperDollFrame_SetStagger(stat, "player")
            assert(stat.numericValue == sample[2], "PaperDoll stagger percentage")
            assert(stat.Value:GetText() == sample[2] .. "%", "PaperDoll stagger text")
        end
        "#,
    )
    .expect("PaperDoll consumer should display supplied stagger percentages");
}
