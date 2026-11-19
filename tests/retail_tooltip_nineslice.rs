#![cfg(feature = "client-retail")]

use std::path::Path;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn load_tooltip_screen(screen: ScreenKind) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Retail tooltip environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(screen);
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, screen) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

const TOOLTIP_NINESLICE_ASSERTIONS: &str = r#"
    local tooltip = assert(%s)
    local nineSlice = assert(tooltip.NineSlice)
    assert(nineSlice:GetParent() == tooltip)
    tooltip:SetOwner(UIParent, 'ANCHOR_NONE')
    tooltip:SetText('NineSlice preservation probe')
    tooltip:Show()
    assert(nineSlice:IsShown())
    local r, g, b = nineSlice:GetCenterColor()
    local er, eg, eb = TOOLTIP_DEFAULT_BACKGROUND_COLOR:GetRGB()
    assert(r == er and g == eg and b == eb)
    local br, bg, bb, ba = nineSlice:GetBorderColor()
    assert(br == 1 and bg == 1 and bb == 1 and ba == 1)
    for _, pieceName in ipairs({
        'TopLeftCorner', 'TopRightCorner', 'BottomLeftCorner', 'BottomRightCorner',
        'TopEdge', 'BottomEdge', 'LeftEdge', 'RightEdge', 'Center',
    }) do
        local piece = assert(nineSlice[pieceName], pieceName)
        assert(piece:GetNumPoints() > 0, pieceName .. ' must use the tooltip layout')
        assert(piece:GetTexelSnappingBias() == 0, pieceName .. ' texel bias')
        assert(not piece:GetSnapToPixelGrid(), pieceName .. ' pixel snapping')
    end
"#;

fn assert_tooltip_nineslice(tooltip: &str) {
    let code = TOOLTIP_NINESLICE_ASSERTIONS.replace("%s", tooltip);
    let env = load_tooltip_screen(if tooltip == "GlueTooltip" {
        ScreenKind::CharacterSelect
    } else {
        ScreenKind::Game
    });
    env.exec(&code)
        .unwrap_or_else(|error| panic!("original Retail {tooltip} provider should work: {error}"));
}

#[test]
fn retail_game_tooltip_uses_original_nineslice_provider() {
    assert_tooltip_nineslice("GameTooltip");
}

#[test]
fn retail_glue_tooltip_uses_original_nineslice_provider() {
    assert_tooltip_nineslice("GlueTooltip");
}
