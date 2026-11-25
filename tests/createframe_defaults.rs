#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn game_ui_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("initialize game UI environment");
    env.set_screen_size(1024.0, 768.0);
    common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
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
        EquipmentFlyout_Show(CharacterHandsSlot)
        local tooltip = CreateFrame("GameTooltip", nil, UIParent)
        local ordinary = CreateFrame("Frame", nil, UIParent)
        CreateFrameDefaultsProbe = { tooltip = tooltip, ordinary = ordinary }
        "#,
    )
    .expect("write tainted addon probe");
    addon
}

fn assert_item_button_and_tooltip_contract(env: &WowLuaEnv) {
    let result: (bool, bool, bool, bool, bool, String, String) = env
        .eval(
            r#"
            local probe = CreateFrameDefaultsProbe
            local expectedA = select(10, GetItemInfo(49812))
            local expectedB = select(10, GetItemInfo(49803))
            local count, seenA, seenB, parents = 0, false, false, true
            for _, button in ipairs(EquipmentFlyoutFrame.buttons) do
                if button:IsShown() then
                    count = count + 1
                    local texture = button.icon:GetTexture()
                    seenA = seenA or texture == expectedA
                    seenB = seenB or texture == expectedB
                    parents = parents and button.icon:GetParent() == button
                        and button.IconBorder:GetParent() == button
                end
            end
            return EquipmentFlyoutFrame:IsShown(), count >= 2,
                expectedA ~= expectedB, seenA and seenB, parents,
                probe.tooltip:GetFrameStrata(), probe.ordinary:GetFrameStrata()
            "#,
        )
        .expect("query original equipment flyout and constructor behavior");
    assert_eq!(
        result,
        (true, true, true, true, true, "TOOLTIP".into(), "MEDIUM".into())
    );
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
        r#"
        A_Admin.EquipItem(11, 49812)
        A_Admin.EquipItem(12, 49803)
        A_Admin.SetInventorySlotCandidates(10, false, {
            { equipmentSlot = 11 },
            { equipmentSlot = 12 },
        }, {
            slotNames = { HandsSlot = 10 },
            carriedEnabled = true,
            carriedCount = 2,
            interactionGuidNonzero = false,
            transmogReady = false,
        })
        "#,
    )
    .expect("supply two candidates to the original equipment flyout");
    let addon = addon_createframe_probe();
    load_addon(
        &env.loader_env(),
        &addon.path().join("CreateFrameDefaultsProbe.toc"),
    )
    .expect("load addon-tainted CreateFrame caller");
    assert_item_button_and_tooltip_contract(&env);
    assert_map_canvas_contract(&env);
}
