#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::Path;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

const DEFAULT_LAYOUT_LUA: &str = r#"
    local tooltip = assert(%TOOLTIP%, "tooltip should load")
    local nineSlice = assert(tooltip.NineSlice, "XML template should create NineSlice")
    assert(tooltip.layoutType == "TooltipDefaultLayout", "template layout type")
    assert(nineSlice:GetParent() == tooltip, "NineSlice parent")
    tooltip:SetOwner(UIParent, "ANCHOR_NONE")
    tooltip:SetText("Default tooltip contents")
    tooltip:Show()
    assert(nineSlice.TopLeftCorner:GetAtlas() == "Tooltip-NineSlice-CornerTopLeft", "default left atlas")
    assert(nineSlice.TopRightCorner:GetAtlas() == "Tooltip-NineSlice-CornerTopRight", "default right atlas")
    assert(nineSlice:IsShown(), "default style shows NineSlice")
"#;

const TWO_STYLES_LUA: &str = r#"
    local tooltip = assert(GameTooltip, "GameTooltip should load")
    local nineSlice = assert(tooltip.NineSlice, "XML template should create NineSlice")
    assert(tooltip.layoutType == "TooltipDefaultLayout", "template layout type")
    assert(nineSlice:GetParent() == tooltip, "NineSlice parent")
    tooltip:SetOwner(UIParent, "ANCHOR_NONE")
    tooltip:SetText("Default tooltip contents")
    tooltip:Show()
    local defaultAtlas = nineSlice.TopLeftCorner:GetAtlas()
    assert(defaultAtlas == "Tooltip-NineSlice-CornerTopLeft", "default style atlas")
    tooltip:SetText("Azerite tooltip contents")
    SharedTooltip_SetBackdropStyle(tooltip, GAME_TOOLTIP_BACKDROP_STYLE_AZERITE_ITEM, false)
    tooltip:Show()
    local azeriteAtlas = nineSlice.TopLeftCorner:GetAtlas()
    assert(azeriteAtlas == "Tooltip-Azerite-NineSlice-CornerTopLeft", "Azerite style atlas")
    assert(defaultAtlas ~= azeriteAtlas, "styles use different NineSlice art")
    assert(nineSlice:IsShown(), "Azerite style shows NineSlice")
"#;

fn load_tooltip_screen(screen: ScreenKind) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create tooltip environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(screen);
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    load_tooltip_addons(&env, &ui, screen);
    settle_headless_startup(&env);
    env
}

fn load_tooltip_addons(env: &WowLuaEnv, ui: &Path, screen: ScreenKind) {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::WowForever {
        load_forever_tooltip_addons(env, ui, screen);
        return;
    }

    for (name, toc) in discover_blizzard_addons_for_screen(ui, screen) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
}

fn load_forever_tooltip_addons(env: &WowLuaEnv, ui: &Path, screen: ScreenKind) {
    use wow_ui_sim::loader::{discover_all_blizzard_addons, register_load_with_addons};

    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(ui));
    if screen == ScreenKind::Game {
        crate::common::supply_empty_forever_spellbook_outfits(env);
        crate::common::load_game_startup_inventory(env);
        return;
    }

    for (name, toc) in discover_blizzard_addons_for_screen(ui, screen) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
}

fn run_tooltip_assertions(screen: ScreenKind, tooltip: &str, assertions: &str) {
    let env = load_tooltip_screen(screen);
    let code = assertions.replace("%TOOLTIP%", tooltip);
    env.exec(&code)
        .unwrap_or_else(|error| panic!("original {tooltip} tooltip layout should work: {error}"));
}

#[test]
fn game_tooltip_uses_xml_nineslice_for_default_and_azerite_styles() {
    run_tooltip_assertions(ScreenKind::Game, "GameTooltip", TWO_STYLES_LUA);
}

#[test]
fn glue_tooltip_uses_xml_nineslice_for_default_style() {
    run_tooltip_assertions(
        ScreenKind::CharacterSelect,
        "GlueTooltip",
        DEFAULT_LAYOUT_LUA,
    );
}
