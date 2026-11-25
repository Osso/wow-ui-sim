#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn game_ui_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_FrameXML",
            "Blizzard_ItemButton",
            "Blizzard_UIPanels_Game",
            "Blizzard_WorldMap",
        ],
        &[],
    );
    for name in [
        "Blizzard_FrameXML",
        "Blizzard_ItemButton",
        "Blizzard_UIPanels_Game",
        "Blizzard_WorldMap",
    ] {
        assert!(loaded.iter().any(|loaded| loaded == name), "{name} must load");
    }
    env
}

fn addon_createframe_probe() -> tempfile::TempDir {
    let addon = tempfile::tempdir().expect("create tainted addon fixture");
    std::fs::write(
        addon.path().join("CreateFrameDefaultsProbe.toc"),
        "## Title: CreateFrameDefaultsProbe\nProbe.lua\n",
    )
    .expect("write probe TOC");
    std::fs::write(
        addon.path().join("Probe.lua"),
        r#"
        EquipmentFlyout_Show(CharacterFinger0Slot)
        local tooltip = CreateFrame("GameTooltip", nil, UIParent)
        local ordinary = CreateFrame("Frame", nil, UIParent)
        CreateFrameDefaultsProbe = { tooltip = tooltip, ordinary = ordinary }
        "#,
    )
    .expect("write tainted addon probe");
    addon
}

fn assert_item_button_and_tooltip_contract(env: &WowLuaEnv) {
    let result: (bool, bool, bool, bool, String, String) = env
        .eval(
            r#"
            local probe = CreateFrameDefaultsProbe
            local seen, count = {}, 0
            local expectedA = select(10, GetItemInfo(49812))
            local expectedB = select(10, GetItemInfo(49803))
            for _, button in ipairs(EquipmentFlyoutFrame.buttons) do
                if button:IsShown() then
                    count = count + 1
                    seen[button.icon:GetTexture()] = button
                    if button.icon:GetParent() ~= button
                        or button.IconBorder:GetParent() ~= button then
                        return false, false, false, false,
                            probe.tooltip:GetFrameStrata(), probe.ordinary:GetFrameStrata()
                    end
                end
            end
            return count >= 2, expectedA ~= expectedB,
                seen[expectedA] ~= nil, seen[expectedB] ~= nil,
                probe.tooltip:GetFrameStrata(), probe.ordinary:GetFrameStrata()
            "#,
        )
        .expect("query original equipment flyout and constructor behavior");
    assert_eq!(result, (true, true, true, true, "TOOLTIP".into(), "MEDIUM".into()));
}

fn assert_map_canvas_contract(env: &WowLuaEnv) {
    let contract: (bool, bool) = env
        .eval(
            r#"
            local scroll = WorldMapFrame.ScrollContainer
            return scroll:GetParent() == WorldMapFrame,
                WorldMapFrame:GetCanvasContainer() == scroll
                    and WorldMapFrame:GetCanvas() == scroll.Child
            "#,
        )
        .expect("query XML-published MapCanvas child and methods");
    assert_eq!(contract, (true, true));
}

#[test]
fn original_consumers_get_constructor_defaults_and_xml_children() {
    let env = game_ui_env();
    env.exec(
        "A_Admin.EquipItem(11, 49812); A_Admin.EquipItem(12, 49803); \
         A_Admin.AddBagItem(0, 1, 49812); A_Admin.AddBagItem(0, 2, 49803)",
    )
    .expect("supply two distinct ring items to the equipment flyout");
    let addon = addon_createframe_probe();
    load_addon(
        &env.loader_env(),
        &addon.path().join("CreateFrameDefaultsProbe.toc"),
    )
    .expect("load addon-tainted CreateFrame caller");
    assert_item_button_and_tooltip_contract(&env);
    assert_map_canvas_contract(&env);
}
