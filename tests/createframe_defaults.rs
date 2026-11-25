#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "common/mod.rs"]
mod common;

use common::blizzard_addon_harness::{
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
        &["Blizzard_FrameXML", "Blizzard_ItemButton", "Blizzard_WorldMap"],
        &[],
    );
    for name in ["Blizzard_FrameXML", "Blizzard_ItemButton", "Blizzard_WorldMap"] {
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
        local first = EquipmentFlyout_CreateButton()
        local second = EquipmentFlyout_CreateButton()
        SetItemButtonTexture(first, GetInventoryItemTexture("player", 1))
        SetItemButtonTexture(second, GetInventoryItemTexture("player", 16))
        local tooltip = CreateFrame("GameTooltip", nil, UIParent)
        local ordinary = CreateFrame("Frame", nil, UIParent)
        CreateFrameDefaultsProbe = {
            first = first, second = second, tooltip = tooltip,
            ordinary = ordinary,
        }
        "#,
    )
    .expect("write tainted addon probe");
    addon
}

fn assert_item_button_and_tooltip_contract(env: &WowLuaEnv) {
    let result: (bool, bool, bool, bool, bool, bool, String, String) = env
        .eval(
            r#"
            local probe = CreateFrameDefaultsProbe
            return probe.first.icon:GetParent() == probe.first,
                probe.first.IconBorder:GetParent() == probe.first,
                probe.second.icon:GetParent() == probe.second,
                probe.second.IconBorder:GetParent() == probe.second,
                probe.first.icon:GetTexture() == GetInventoryItemTexture("player", 1),
                probe.second.icon:GetTexture() == GetInventoryItemTexture("player", 16),
                probe.tooltip:GetFrameStrata(), probe.ordinary:GetFrameStrata()
            "#,
        )
        .expect("query original ItemButton and constructor behavior");
    assert_eq!(result, (true, true, true, true, true, true, "TOOLTIP".into(), "MEDIUM".into()));
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
    env.exec("A_Admin.EquipItem(1, 211993); A_Admin.EquipItem(16, 231268)")
        .expect("supply two distinct equipped items");
    let addon = addon_createframe_probe();
    load_addon(
        &env.loader_env(),
        &addon.path().join("CreateFrameDefaultsProbe.toc"),
    )
    .expect("load addon-tainted CreateFrame caller");
    assert_item_button_and_tooltip_contract(&env);
    assert_map_canvas_contract(&env);
}
